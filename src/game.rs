use player::Player;
use std::collections::HashMap;
use rand::{thread_rng, Rng};
use rs_poker::core::{Deck, Card, Flattenable, FlatDeck, Rank};
use uuid::Uuid;

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
    PostBlind(usize),
}

#[derive(Debug)]
/// Contains the state of the poker game, including players, cards, action, etc.
pub struct Game {
    // TODO:
    //  - Push game moves to a database
    // Possible Extensions (unnecessarily advanced)
    //  - Game consisting of multiple tables w/ appropriate table breaks
    //  - Optional ante
    
    // Public fields
    pub board : Vec<Card>,                // Community cards
    pub players : HashMap<usize, Player>, // Players in the game (see player.rs)
    pub seat_order : Vec<usize>,          // Positions of players around the table
                                          // N.B. seat_order[0] is button
    pub street : Street,                  // Street we're currently on
    pub to_act : usize,                   // Which player has action

    game_id : Uuid, // TODO: use this

    // Private Fields
    deck : FlatDeck,                      // A deck of cards

    num_players : usize,                  // Player count
    num_in_play : usize,                  // Number of players able to act (not folded or all-in)
    num_folded  : usize,                  // Number of players who have folded
    num_eliminated : usize,               // Number of players who have gone to 0 chips

    starting_stack : usize,               // Number of chips we start with, with 1/2 blinds
    button : usize,                       // Position of the dealer button

    current_bet : usize,
    min_raise   : usize,
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
            num_in_play : 0,
            num_folded : 0,
            num_eliminated : 0,
            button : 0,
            street : Street::River,
            to_act : 0,
            game_id : Uuid::new_v4(),
            current_bet : 0,
            min_raise : 2,
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

    /// Takes a player action and applies it to the game
    pub fn player_action(&mut self, recv_action: Action) -> () {
        // TODO: 
        // Verify the correct player posted action
        // Verify the action is valid in context
        // And actually take the action...

        let real_action;

        { // For mutable borrow of self through plyr
            let plyr = self.players.get_mut(&self.to_act).unwrap();
            // Interpret the recieved action into a legal action
            match recv_action {
                Action::Check => {
                    if self.current_bet > plyr.street_contrib {
                        // Player didn't contribute enough for a check to be valid
                        real_action = Action::Fold;
                    } else {
                        real_action = Action::Check;
                    }
                },
                Action::Fold => {
                    // We can always fold
                    real_action = Action::Fold;
                },
                Action::Bet(bet) => {
                    if bet == 0 {
                        if self.current_bet == 0 || (plyr.has_option 
                                                     && self.current_bet == plyr.street_contrib) {
                            real_action = Action::Check;
                        } else {
                            real_action = Action::Fold;
                        }
                    } else if bet + plyr.street_contrib == self.current_bet {
                        // This is a call
                        real_action = Action::Bet(plyr.chips.min(bet));
                    } else if bet + plyr.street_contrib < self.current_bet {
                        // Sub-call is a call or all-in
                        real_action = Action::Bet(plyr.chips.min(self.current_bet 
                                                                 - plyr.street_contrib))
                    } else {
                        // Player is trying to raise
                        if bet + plyr.street_contrib - self.current_bet < self.min_raise {
                            // Under-raise is a min-raise or all-in
                            real_action = Action::Bet(plyr.chips.min(self.current_bet 
                                                      + self.min_raise - plyr.street_contrib));
                        } else {
                            // Valid raise. Use it or all-in
                            real_action = Action::Bet(plyr.chips.min(bet))
                        }
                    }
                },
                Action::PostBlind(blind) => {
                    // Post as much of the blind as possible
                    real_action = Action::PostBlind(plyr.chips.min(blind))
                },
            }

            match real_action {
                Action::Check => {
                    println!("GAME - Player {} checks",plyr.display_name);
                },
                Action::Fold => {
                    plyr.folded = true;
                    self.num_in_play -= 1;
                    self.num_folded += 1;
                    println!("GAME - Player {} folds",plyr.display_name);
                },
                Action::Bet(bet) => {
                    plyr.street_contrib += bet;
                    plyr.hand_contrib += bet;
                    plyr.chips -= bet;
                    println!("GAME - Player {} bets {}",plyr.display_name, bet);
                },
                Action::PostBlind(blind) => {
                    plyr.street_contrib += blind;
                    plyr.hand_contrib += blind;
                    plyr.chips -= blind;
                    println!("GAME - Player {} posts blind {}",plyr.display_name, blind);
                },
            }

            if plyr.chips == 0 {
                plyr.all_in = true;
                self.num_in_play -= 1;
                println!("GAME - Player {} has gone all-in!",plyr.display_name);
            }
        } // End of mutable borrow block

        // Now see if the hand is over

        // TODO: check hand is over and go to victory checker & chip redistributer
        // If not done, get ready for the next player's action
        self.to_act = self.next_player(self.to_act);
    }

    pub fn next_street(&mut self) -> () {
        // TODO: Post stuff to web
        self.to_act = self.next_player(self.seat_order[0]);
        println!("GAME - To act: {}:{}",self.to_act,self.players.get(&self.to_act).unwrap().display_name);

        match self.street {
            Street::PreFlop => {
                println!("GAME - Dealing Flop");
                println!("GAME - Flop: {:?}",&self.board[0..3]);
                self.street = Street::Flop;
            },
            Street::Flop    => {
                println!("GAME - Dealing turn");
                println!("GAME - Turn: {:?}",&self.board[3]);
                self.street = Street::Turn;
            },
            Street::Turn    => {
                println!("GAME - Dealing River");
                println!("GAME - Turn: {:?}",&self.board[4]);
                self.street = Street::River;
            },
            Street::River    => {
                println!("GAME - New Hand!");
                &self.new_hand();
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

        self.num_in_play = self.num_players - self.num_eliminated;
        self.num_folded = 0;

        self.street = Street::PreFlop;

        self.player_action(Action::PostBlind(1));
        self.player_action(Action::PostBlind(2));

    } // pub fn new_hand

    pub fn start(&mut self) -> () {
        println!("GAME - Starting");
        self.next_street();
    }

    /// Return the index of the next unfolded player in the move order
    pub fn next_player(&self, current_player: usize) -> usize {
        let mut found_current = false;
        
        for &i in &self.seat_order {
            if i == current_player {
                found_current = true;
            } else if found_current {
                let plyr = self.players.get(&i).unwrap();
                if !plyr.folded && !plyr.all_in && !plyr.eliminated {
                    return i;
                }
            }
        }

        // We got to the end and didn't find anyone... So return the first unfolded player
        for &i in &self.seat_order {
            if i == current_player {
                panic!("Only one unfolded player!");
            } else {
                if !self.players.get(&i).unwrap().folded {
                    return i;
                }
            }
        }

        panic!("Something wrong in Game::next_player");
    }

    /// Of the players still in the hand, return a `Vec<usize>` of 
    /// the ID(s) of the player(s) with the best hand
    pub fn get_winners(&self, ids: Vec<usize>) -> Vec<usize> {
        let mut best_hands = Vec::<usize>::new();

        let best_rank = ids.iter()
                           .fold(Rank::HighCard(0), |best, id| {
                                let player = self.players.get(&id).unwrap();
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