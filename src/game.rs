use player::Player;
use std::collections::HashMap;

use rs_poker::core::{Deck, Card, Flattenable, FlatDeck, Rank};

#[derive(Debug)]
pub struct Game {
    deck : FlatDeck,
    pub board : Vec<Card>,
    pub players : HashMap<usize, Player>,
    pub starting_stack : usize,
    pub seat_order : Vec<usize>,
    pub num_players : usize,
}

impl Game {
    pub fn new(stack : usize) -> Game {
        Game{
            deck : create_deck(),
            board : Vec::new(),
            players : HashMap::new(),
            starting_stack : stack,
            seat_order : Vec::new(),
            num_players : 0,
        }
    } // pub fn new

    pub fn add_player(&mut self, name : &str) -> () {
        let id = self.num_players;
        self.players.insert(
            id,
            Player::new(String::from(name), self.starting_stack)
        );
        self.seat_order.push(id);
        
        // TODO shuffle seat order
        //self.seat_order.shuffle();

        self.num_players += 1;
    } // pub fn add_player

    pub fn deal_hand(&mut self) -> () {
        self.deck = create_deck();

        self.board = deal_community(&mut self.deck);

        for (_, mut plyr) in self.players.iter(){
            let cards = deal_hole(&mut self.deck);
            plyr.give_hand(&cards);
        }
    } // pub fn deal_hand

    pub fn get_winners(&self) -> Vec<usize> {
        let mut best_hands = Vec::<usize>::new();

        let best_rank = self.players.iter()
                                    .fold(Rank::HighCard(0), |best, (_, player)| {
                                        if !player.folded {
                                            let new_rank = (&player).get_rank(&self.board);
                                            if new_rank > best {
                                                new_rank.to_owned()
                                            } else {
                                                best
                                            }
                                        } else {
                                            best
                                        }
                                    });

        for (id, player) in self.players.iter() {
            if player.get_rank(&self.board) == best_rank {
                best_hands.push(id.clone());
            }
        }

        return best_hands;
    } // pub fn get_winners
} // impl Game

/// Returns a vec of 2 cards as hole cards
fn deal_hole(deck: &mut FlatDeck) -> Vec<Card> {
    deal_cards(deck, 2)
}

/// Returns a vec of 5 cards as community cards
fn deal_community(deck: &mut FlatDeck) -> Vec<Card> {
    deal_cards(deck, 5)
}

/// Returns a vec with capacity `num` filled with cards
fn deal_cards(deck: &mut FlatDeck, num: usize) -> Vec<Card> {
    let mut cards = Vec::<Card>::with_capacity(2);

    for _ in 0..num {
        let tmp_card = deck.deal().unwrap();
        cards.push(Card{
                    value : tmp_card.value,
                    suit  : tmp_card.suit,
                }
        );
    }

    return cards;
}

/// Returns a shuffled and dealable deck
fn create_deck() -> FlatDeck {
    let mut deck = Deck::default().flatten();
    deck.shuffle();
    return deck;
}