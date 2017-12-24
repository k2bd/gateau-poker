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
    
    println!("Play Order: {:?}",&game.seat_order.iter()
                                                .map(|id| &game.players.get(&id).unwrap().display_name)
                                                .collect::<Vec<_>>());

    &game.start();

    &game.player_action(Action::Bet(20));
    &game.player_action(Action::Fold);
    &game.player_action(Action::Bet(200));
    &game.player_action(Action::Bet(10));
    &game.player_action(Action::Bet(20));
    &game.player_action(Action::Bet(20));

    &game.next_street();

    let winners = &game.get_winners();
    let winning_players = winners.iter()
                                 .map(|id| &game.players.get(&id).unwrap().display_name)
                                 .collect::<Vec<_>>();

    println!("Board: {:?}",game.board);
    for (id, plyr) in &game.players {
        println!("Player {}:{} hand: {:?} with {:?}", id,plyr.display_name, plyr.hole_cards, plyr.get_rank(&game.board));
    }
    println!("Winners: {:?} with {:?}",winning_players,&game.players.get(&winners[0]).unwrap().get_rank(&game.board));
}