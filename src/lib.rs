extern crate rs_poker;

mod game;
mod player;

//use std::collections::HashMap;
//use rs_poker::core::{Deck,Card,FlatDeck,Flattenable,Rank,Hand,Rankable};
//use player::Player;
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

    &game.deal_hand();

    let winners = &game.get_winners();

    println!("{:?}",game);
}