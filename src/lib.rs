extern crate rs_poker;
extern crate rand;
extern crate uuid;

mod game;
mod player;

use game::{Game,Action};

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

    &game.player_action(Action::Bet(2));
    &game.player_action(Action::Bet(2));
    &game.player_action(Action::Bet(2));
    &game.player_action(Action::Bet(1));
    &game.player_action(Action::Bet(5));
    &game.player_action(Action::Bet(5));
    &game.player_action(Action::Bet(5));
    &game.player_action(Action::Bet(5));
    &game.player_action(Action::Bet(5));
    &game.player_action(Action::Bet(5));
    &game.player_action(Action::Fold);
    &game.player_action(Action::Fold);
    &game.player_action(Action::Fold);
    &game.player_action(Action::Fold);

    println!("Play Order: {:?}",&game.seat_order.iter()
                                                .map(|id| &game.players.get(&id).unwrap().display_name)
                                                .collect::<Vec<_>>());

    &game.player_action(Action::Bet(18));
}