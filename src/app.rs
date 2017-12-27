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

#[derive(Serialize,Deserialize)]
struct GameConfig {
    Config : String, // Field to modify. At the moment just 'starting_stack'
    Value  : usize,
}

#[derive(Serialize,Deserialize)]
struct PlayerMessage {
    PlayerID : usize,  // Player must confirm its ID when it makes a move. TODO: Make this a serializable uuid
    Action   : String, // Bet, Call, Fold, Check, AllIn
    Value    : usize,  // In the case of bet, the amount to bet, otherwise unused
}

#[derive(Serialize,Deserialize)]
struct JoinData {
    Name    : String, // Display name for the player
    Address : String, // IP address of the player
}

#[post("/config", format="application/json", data="<game_config>")]
fn configure_game(game_config: Json<GameConfig>, game: State<Game>) -> () {
    //
    match game_config.Config.to_lowercase().as_ref() {
        "starting_stack" => {
            &game.set_starting_stack(game_config.Value);
        }
    }
}

#[post("/reg", format="application/json", data="<reg_data>")]
fn join_game(reg_data: Json<JoinData>, game: State<Game>) -> () {
    let id = &game.add_player(reg_data.Name.as_ref(),reg_data.Address.as_ref());

    // TODO: POST this ID to the new player's address so they can make moves
}

#[post("/game", format="application/json", data="<action>")]
fn make_move(action: Json<PlayerMessage>, game: State<Game>) -> Json<Value> {
    // TODO: Check against the current player's uuid
    if action.PlayerID != game.to_act {
        return Json(json!({
            "status" : "error",
            "reason" : "Not your turn!"
        }));
    }

    match action.Action.to_lowercase().as_ref() {
        "check" => &game.player_action(Action::Check),
        "call"  => &game.player_action(Action::Call),
        "fold"  => &game.player_action(Action::Fold),
        "allin" => &game.player_action(Action::AllIn),
        "bet"   => &game.player_action(Action::Bet(action.Value)),
        other   => println!("Invalid action recieved {}",other), // do nothing
    }

    Json(json!({
        "status" : "ok",
    }))
}

fn rocket() -> rocket::Rocket {
    // TODO:
    // We can make this client managed several Games in a Vec... manage(Vec::<Game>::new())
    rocket::ignite()
        .mount("/kev-poker",routes![make_move])
        .manage(Game::new(0))
}

fn main() {
    rocket().launch();
}