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

mod game;
mod player;

use rocket_contrib::{Json, Value};
use rocket::{State};
use game::{Game,Action};
use std::sync::RwLock;
use uuid::Uuid;

#[derive(Serialize,Deserialize)]
struct GameConfig {
    Config : String, // Field to modify. At the moment just 'starting_stack'
    Value  : usize,
}

#[derive(Serialize,Deserialize)]
struct PlayerMessage {
    PlayerID : Uuid,  // Player must confirm its ID when it makes a move. TODO: Make this a serializable uuid
    Action   : String, // Bet, Call, Fold, Check, AllIn
    Value    : usize,  // In the case of bet, the amount to bet, otherwise unused
}

#[derive(Serialize,Deserialize)]
struct JoinData {
    Name    : String, // Display name for the player
    Address : String, // IP address of the player
}

#[post("/config", format="application/json", data="<game_config>")]
fn configure_game(game_config: Json<GameConfig>, game_lock: State<RwLock<Game>>) -> () {
    //
    let mut game = game_lock.write().unwrap();

    match game_config.Config.to_lowercase().as_ref() {
        // TODO: 
        // - Make move timers configurable
        "starting_stack" => {
            (*game).set_starting_stack(game_config.Value);
        },
        "start" => {
            (*game).start();
        },
        other => {println!("DEBUG - Bad config option: {}",other)},
    }
}

#[post("/reg", format="application/json", data="<reg_data>")]
fn join_game(reg_data: Json<JoinData>, game_lock: State<RwLock<Game>>) -> () {
    let mut game = game_lock.write().unwrap();

    let id = (*game).add_player(reg_data.Name.as_ref(),reg_data.Address.as_ref());

    // TODO: POST this ID to the new player's address so they can make moves
}

#[post("/game", format="application/json", data="<action>")]
fn make_move(action: Json<PlayerMessage>, game_lock: State<RwLock<Game>>) -> Json<Value> {
    let mut game = game_lock.write().unwrap();

    // TODO: Check against the current player's uuid
    if action.PlayerID != game.to_act {
        return Json(json!({
            "status" : "error",
            "reason" : "Not your turn!"
        }));
    }

    match action.Action.to_lowercase().as_ref() {
        "check" => (*game).player_action(Action::Check),
        "call"  => (*game).player_action(Action::Call),
        "fold"  => (*game).player_action(Action::Fold),
        "allin" => (*game).player_action(Action::AllIn),
        "bet"   => (*game).player_action(Action::Bet(action.Value)),
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
        .mount("/kev-poker",routes![configure_game, join_game, make_move])
        .manage(RwLock::new(Game::new(0)))
}

fn main() {
    rocket().launch();
}