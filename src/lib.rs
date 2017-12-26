extern crate rs_poker;
extern crate rand;
extern crate uuid;

mod game;
mod player;

use game::{Game,Action};
// Temp
use rand::{thread_rng, Rng};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub fn test() {
    let mut game = Game::new(100);

    &game.add_player("Kevin");
    &game.add_player("Bevin");
    &game.add_player("Sevin");
    &game.add_player("Levin");
    &game.add_player("Aevin");

    &game.start();

    println!("Play Order: {:?}",&game.seat_order.iter()
                                                .map(|id| &game.players.get(&id).unwrap().display_name)
                                                .collect::<Vec<_>>());

    loop {
        &game.player_action(Action::Call);
    }
}