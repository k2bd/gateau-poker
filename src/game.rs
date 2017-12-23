use player::Player;
use std::collections::HashMap;
use rand::{thread_rng, Rng};
use rs_poker::core::{Deck, Card, Flattenable, FlatDeck, Rank};

#[derive(Debug)]
pub enum Street {
    PreFlop,
    Flop,
    Turn,
    River,
}

pub enum Action {
    Fold,
    Check,
    Bet(usize),
}

#[derive(Debug)]
/// Contains the state of the poker game, including players, cards, action, etc.
pub struct Game {
    // Possible Extensions (unnecessarily advanced)
    //  - Game consisting of multiple tables w/ appropriate table breaks
    //  - Optional ante
    
    // Public fields
    pub board : Vec<Card>,                // Community cards
    pub players : HashMap<usize, Player>, // Players in the game (see player.rs)
    pub seat_order : Vec<usize>,          // Positions of players around the table
                                          // N.B. 0 us UTG
    pub street : Street,                  // Street we're currently on
    pub to_act : usize,                   // Which player has action

    // Private Fields
    deck : FlatDeck,                      // A deck of cards
    num_players : usize,                  // Player count
    starting_stack : usize,               // Number of chips we start with, with 1/2 blinds
    button : usize,                       // Position of the dealer button
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
            button : 0,
            street : Street::River,
            to_act : 0,
        }
    } // pub fn new

    pub fn add_player(&mut self, name : &str) -> () {
        let id = self.num_players;
        self.players.insert(
            id,
            Player::new(String::from(name), self.starting_stack)
        );
        self.seat_order.push(id);
        
        thread_rng().shuffle(&mut self.seat_order);

        self.num_players += 1;
    } // pub fn add_player

    pub fn next_street(&mut self) -> () {
        self.action = 0; // Reset action to UTG

        match self.street {
            Street::PreFlop => {
                println!("Moving to Flop!");
                self.street = Street::Flop;
            },
            Street::Flop    => {
                println!("Moving to Turn!");
                self.street = Street::Turn;
            },
            Street::Turn    => {
                println!("Moving to River!");
                self.street = Street::River;
            },
            Street::River    => {
                println!("New Turn!");
                &self.new_hand();
                self.street = Street::PreFlop;
            },
        }
    }

    pub fn player_action(&mut self, action: Action) -> {
        // TODO verify the correct player posted action

        let player = &game.players.get(&game.to_act).unwrap();

        match action {
            Action::Check => {
                println!("Player {} checks",player.display_name);
            },
            Action::Fold => {
                println!("Player {} folds",player.display_name);
            },
            Action::Bet(e) => {
                println!("Player {} bets {}",player.display_name, e);
            },
        }
    }

    /// Sets up a new hand: shuffles a new deck, deals, etc.
    pub fn new_hand(&mut self) -> () {
        // Create a new deck
        self.deck = create_deck();

        self.board = deal_community(&mut self.deck);

        for (_, plyr) in &mut self.players {
            let cards = deal_hole(&mut self.deck);
            plyr.give_hand(&cards);
        }

        // TODO post blinds from nonfolded players and move action accordingly

    } // pub fn new_hand

    /// Of the players still in the hand, return a `Vec<usize>` of 
    /// the ID(s) of the player(s) with the best hand
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

/// Returns a vec of 2 cards to be used as a player's hole cards
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