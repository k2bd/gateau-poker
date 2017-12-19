extern crate rs_poker;

use rs_poker::core::{Hand,Card,Rankable,Rank};

pub struct Player {
    pub hole_cards : Vec<Card>,
    pub folded: bool,
}

impl Player {
    /// Get a rank based on the player's hand and the community cards
    pub fn get_rank(self, community: &Vec<Card>) -> Rank {
        let mut my_comm = community.to_owned();

        for card in self.hole_cards {
            my_comm.push(card.clone());
        }

        return Hand::new_with_cards(my_comm).rank();
    }
}