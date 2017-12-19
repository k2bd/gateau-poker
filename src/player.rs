extern crate rs_poker;

use rs_poker::core::{Card, Hand, Rank, Rankable};

#[derive(Debug)]
pub struct Player {
    pub hole_cards : Vec<Card>,
    pub folded : bool,
    pub chips : usize,
    pub display_name : String,
}

impl Player {
    pub fn new(name : String, starting_stack : usize) -> Player {
        Player {
            chips : starting_stack,
            display_name : name,
            hole_cards : Vec::new(),
            folded : false,
        }
    }

    pub fn give_hand(&mut self, hand : &Vec<Card>) {
        self.hole_cards = hand.clone();
        self.folded = false;
    }
    
    pub fn get_rank(&self, community: &Vec<Card>) -> Rank {
        let mut my_hand = community.to_owned();

        for card in &self.hole_cards {
            my_hand.push(card.clone());
        }

        return Hand::new_with_cards(my_hand).rank();
    }
}