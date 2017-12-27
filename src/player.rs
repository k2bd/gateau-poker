extern crate rs_poker;

use rs_poker::core::{Card, Hand, Rank, Rankable};

#[derive(Debug)]
pub struct Player {
    pub hole_cards : Vec<Card>,
    pub folded : bool,
    pub chips : usize,
    pub display_name : String,
    pub address : String,
    pub street_contrib : usize,
    pub hand_contrib : usize,
    pub has_option : bool,
    pub all_in : bool,
    pub eliminated : bool,
}

impl Player {
    pub fn new(name : String, address: String, starting_stack : usize) -> Player {
        Player {
            chips : starting_stack,
            display_name : name,
            address : address,
            hole_cards : Vec::new(),
            folded : false,
            hand_contrib : 0,
            street_contrib : 0,
            has_option : false,
            all_in : false,
            eliminated : false,
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