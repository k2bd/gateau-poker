#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rs_poker;
extern crate rand;
extern crate uuid;
extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate reqwest;
extern crate hyper;

mod game;
mod player;

use rocket_contrib::{Json, Value};
use rocket::{State};
use game::{Game,Action};
use std::sync::RwLock;
use uuid::Uuid;

#[derive(Serialize,Deserialize)]
struct GameConfig {
    config : String, // Field to modify. At the moment just 'starting_stack'
    value  : usize,
}

#[derive(Serialize,Deserialize)]
struct PlayerMessage {
    secret_id : Uuid,  // Player must confirm its ID when it makes a move.
    action   : String, // Bet, Call, Fold, Check, AllIn
    value    : usize,  // In the case of bet, the amount to bet, otherwise unused
}

#[derive(Serialize,Deserialize)]
struct JoinData {
    name    : String, // Display name for the player
    address : String, // IP address of the player
}

#[post("/config", format="application/json", data="<game_config>")]
fn configure_game(game_config: Json<GameConfig>, game_lock: State<RwLock<Game>>) -> Json<Value> {
    //
    let mut game = game_lock.write().unwrap();

    match game_config.config.to_lowercase().as_ref() {
        // TODO: 
        // - Make move timers configurable
        "starting_stack" => {
            let success = (*game).set_starting_stack(game_config.value);
            if !success {
                return Json(json!({
                    "status" : "error",
                    "reason" : "Game already started!",
                }));
            }
        },
        "max_players" => {
            let success = (*game).set_player_limit(game_config.value);
            if !success {
                return Json(json!({
                    "status" : "error",
                    "reason" : "Game already started!",
                }));
            }
        },
        "start" => {
            let success = (*game).start();
            if !success {
                return Json(json!({
                    "status" : "error",
                    "reason" : "Game already started!",
                }));
            }
        },
        other => {
            println!("DEBUG - Bad config option: {}",other);
            return Json(json!({
                "status" : "error",
                "reason" : "Bad config option!"
            }));
        },
    }

    Json(json!({
        "status" : "ok",
    }))
}

#[post("/reg", format="application/json", data="<reg_data>")]
fn join_game(reg_data: Json<JoinData>, game_lock: State<RwLock<Game>>) -> Json<Value> {
    let mut game = game_lock.write().unwrap();

    // Could change this to Option<PlayerInfo> or Result<PlayerInfo> and return stuff here
    let space_to_join = (*game).add_player(reg_data.name.as_ref(),reg_data.address.as_ref());

    // TODO: POST this ID to the new player's address so they can make moves
    // ^ put this is the add_player method...?

    if space_to_join {
        return Json(json!({
            "status" : "ok",
        }));
    } else {
        return Json(json!({
            "status" : "error",
            "reason" : "No space to join this game"
        }));
    }
}

#[post("/game", format="application/json", data="<action>")]
fn make_move(action: Json<PlayerMessage>, game_lock: State<RwLock<Game>>) -> Json<Value> {
    let mut game = game_lock.write().unwrap();

    // TODO: Check against the current player's uuid
    if action.secret_id != game.players.get(&game.to_act).unwrap().secret_id {
        println!("DEBUG - Recieved secret ID {} does not match expected {}",action.secret_id,game.players.get(&game.to_act).unwrap().secret_id);
        return Json(json!({
            "status" : "error",
            "reason" : "Not your turn!"
        }));
    }

    match action.action.to_lowercase().as_ref() {
        "check" => (*game).player_action(Action::Check),
        "call"  => (*game).player_action(Action::Call),
        "fold"  => (*game).player_action(Action::Fold),
        "allin" => (*game).player_action(Action::AllIn),
        "bet"   => (*game).player_action(Action::Bet(action.value)),
        other   => println!("DEBUG - Invalid action recieved {}",other), // do nothing
    }

    Json(json!({
        "status" : "ok",
    }))
}

fn rocket() -> rocket::Rocket {
    // TODO:
    // We can make this client managed several Games in a Vec... manage a Vec of RwLocked games
    rocket::ignite()
        .mount("/",routes![configure_game, join_game, make_move])
        .manage(RwLock::new(Game::new(200))) // Default game is 100 big blinds
}

fn main() {
    rocket().launch();
}