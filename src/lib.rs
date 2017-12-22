extern crate rs_poker;
extern crate rand;

mod game;
mod player;

use game::Game;

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

    &game.next_street();

    let winners = &game.get_winners();

    println!("Board: {:?}",game.board);
    for (id, plyr) in &game.players {
        println!("Player {} hand: {:?} with {:?}", id, plyr.hole_cards, plyr.get_rank(&game.board));
    }
    println!("Winner is player {:?} with {:?}",winners,&game.players.get(&winners[0]).unwrap().get_rank(&game.board));
}