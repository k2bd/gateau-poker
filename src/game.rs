use player::Player;
use std::collections::HashMap;
use rand::{thread_rng, Rng};
use rs_poker::core::{Deck, Card, Flattenable, FlatDeck, Rank};
use uuid::Uuid;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Street {
    PreFlop,
    Flop,
    Turn,
    River,
}

// TODO:
// - Call
// - All In
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

        let real_action;

        { // For mutable borrow of self through plyr
            let plyr = self.players.get_mut(&self.to_act).unwrap();
            // Interpret the recieved action into a legal action
            match recv_action {
                Action::Check => {
                    if self.current_bet > plyr.street_contrib {
                        // Player didn't contribute enough for a check to be valid
                        println!("DEBUG - Player {} cannot check! Folding hand...",plyr.display_name);
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
                            println!("DEBUG - Player {} bet(0) interpreted as check.",plyr.display_name);
                            real_action = Action::Check;
                        } else {
                            println!("DEBUG - Player {} bet(0) interpreted as fold.",plyr.display_name);
                            real_action = Action::Fold;
                        }
                    } else if bet + plyr.street_contrib == self.current_bet {
                        // This is a call
                        real_action = Action::Bet(plyr.chips.min(bet));
                    } else if bet + plyr.street_contrib < self.current_bet {
                        // Sub-call is a call or all-in
                        println!("DEBUG - Player {} tried to bet {}, not enough for a call!",plyr.display_name,bet);
                        real_action = Action::Bet(plyr.chips.min(self.current_bet 
                                                                 - plyr.street_contrib))
                    } else {
                        // Player is trying to raise
                        if bet + plyr.street_contrib - self.current_bet < self.min_raise {
                            // Under-raise is a min-raise or all-in
                            println!("DEBUG - Player {} tried to under-raise by {}!",plyr.display_name,bet + plyr.street_contrib - self.current_bet);
                            real_action = Action::Bet(plyr.chips.min(self.current_bet 
                                                      + self.min_raise - plyr.street_contrib));
                        } else {
                            // Valid raise. Use it or all-in
                            real_action = Action::Bet(plyr.chips.min(bet));
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
                    plyr.has_option = false;
                },
                Action::Fold => {
                    plyr.folded = true;
                    println!("GAME - Player {} folds",plyr.display_name);
                    plyr.has_option = false;
                },
                Action::Bet(bet) => {
                    if bet + plyr.street_contrib == self.current_bet {
                        println!("GAME - Player {} calls {} (total {})",plyr.display_name, bet, bet + plyr.street_contrib);
                    } else if self.current_bet == 0 {
                        println!("GAME - Player {} bets {} (total {})",plyr.display_name, bet, bet + plyr.street_contrib);
                    } else {
                        println!("GAME - Player {} raises {} (total {})",plyr.display_name, bet, bet + plyr.street_contrib);
                    }

                    if bet + plyr.street_contrib > self.current_bet {
                        self.min_raise = bet + plyr.street_contrib - self.current_bet;
                        println!("DEBUG - Increasing minimum raise to {}",self.min_raise);
                    }

                    self.current_bet = bet + plyr.street_contrib;
                    println!("DEBUG - Current bet is {}",self.current_bet);

                    plyr.street_contrib += bet;
                    plyr.chips -= bet;
                    plyr.has_option = false;
                },
                Action::PostBlind(blind) => {
                    plyr.street_contrib += blind;
                    plyr.chips -= blind;
                    if blind == 2 {
                        // TODO: swap 2 for a big_blind value
                        plyr.has_option = true;
                    }
                    println!("GAME - Player {} posts blind {}",plyr.display_name, blind);
                },
            }

            if plyr.chips == 0 {
                plyr.all_in = true;
                println!("GAME - Player {} has gone all-in!",plyr.display_name);
            }
        } // End of mutable borrow block

        if self.is_hand_over() {
            self.end_hand();
        } else if self.is_street_over() {
            self.next_street();
        } else {
            self.to_act = self.next_player(self.to_act);
        }
    }

    fn is_hand_over(&self) -> bool {
        // If we're on the river and the street is done, the hand is over.
        if self.street == Street::River && self.is_street_over() {
            return true;
        }

        let players_with_action = self.players.iter()
                                              .fold(0, |sum, (_, player)| {
                                                if !player.folded && !player.eliminated && !player.all_in {
                                                    sum + 1
                                                } else {
                                                    sum
                                                }
                                              });
        
        if players_with_action == 1 {
            println!("DEBUG - HAND OVER");
            return true;
        }

        false
    }

    fn is_street_over(&self) -> bool {
        // If anyone has option we can't end the street
        if self.players.iter().any(|(_, player)| player.has_option) {
            return false;
        }

        // Otherwise, if everyone's put in the same amount we're done
        if self.players.iter().any(|(_,player)| player.street_contrib != self.current_bet){
            return false;
        } else {
            println!("DEBUG - STREET OVER");
            return true;
        }
    }

    fn next_street(&mut self) -> () {
        self.to_act = self.next_player(self.seat_order[0]);
        //self.players.get(&self.to_act).unwrap().has_option = true;

        for (_, player) in &mut self.players {
            if !player.eliminated {
                player.hand_contrib += player.street_contrib;
                player.street_contrib = 0;
                player.has_option = false;
            }
        }

        self.current_bet = 0;
        self.min_raise = 2;

        match self.street {
            Street::PreFlop => {
                self.street = Street::Flop;
                self.deal_street();
            },
            Street::Flop    => {
                self.street = Street::Turn;
                self.deal_street();
            },
            Street::Turn    => {
                self.street = Street::River;
                self.deal_street();
            },
            Street::River    => {
                println!("GAME - New Hand!");
                &self.new_hand();
            },
        }
    }

    fn deal_street(&mut self) {
        let plyr = self.players.get_mut(&self.seat_order[0]).unwrap();
        plyr.has_option = true;

        match self.street {
            Street::Flop => {
                println!("GAME - Flop: {:?}",&self.board[0..3]);
            },
            Street::Turn => {
                println!("GAME - Turn: {:?}",&self.board[3]);
            },
            Street::River => {
                println!("GAME - River: {:?}",&self.board[4]);
            }
            _ => {
                panic!("Invalid street!");
            }
        }
    }

    fn end_hand(&mut self) -> () {
        // Figure out winners, sidepots, etc
        // Eliminate players
        for (_, player) in &mut self.players {
            player.hand_contrib += player.street_contrib;
            player.street_contrib = 0;
        }

        let mut to_pay = Vec::new();

        for _ in 0..self.num_players {
            to_pay.push(0);
        }

        let mut current_pot = self.players.iter()
                                          .map(|(_, player)| {
                                            if player.folded {
                                                usize::max_value()
                                            } else {
                                                player.hand_contrib
                                            }
                                          })
                                          .min().unwrap();

        while current_pot > 0 {
            let mut payout = 0;
            let mut in_pot = Vec::new();
            for (id, player) in &mut self.players {
                if !player.folded && player.hand_contrib > 0 {
                    in_pot.push(id.clone());
                }

                let contrib = current_pot.min(player.hand_contrib);
                payout += contrib;
                player.hand_contrib -= contrib;
            }

            let winners = self.get_winners(in_pot);

            println!("DEBUG - WINNERS: {:?}",winners);

            // Split payout between winners
            let indiv_payout = payout / winners.len();

            for id in &winners {
                to_pay[*id] += indiv_payout;
            }

            let mut paid_out = indiv_payout * winners.len();

            // Any leftover change goes to the left of the button
            let mut change_target = 1;
            while paid_out < payout {
                if winners.iter().any(|&id| id == self.seat_order[change_target]) {
                    to_pay[self.seat_order[change_target]] += 1;
                    paid_out += 1;
                }
                change_target += 1;
            }

            current_pot = self.players.iter()
                                      .map(|(_, player)| {
                                        if player.folded {
                                            usize::max_value()
                                        } else {
                                            player.hand_contrib
                                        }
                                      })
                                      .min().unwrap();
        }

        // Print summary of payouts
        println!("BOARD - {:?}",self.board);
        println!("HAND PAYOUTS");
        for (&id, player) in &mut self.players {
            println!("{}:{} - {} for {:?} ({:?})",id, player.display_name, to_pay[id], player.hole_cards, player.get_rank(&self.board));
            player.chips += to_pay[id];
        }

        self.new_hand();
    }

    /// Sets up a new hand: shuffles a new deck, deals, etc.
    fn new_hand(&mut self) -> () {
        // Create a new deck
        self.deck = create_deck();

        // Move the button
        let temp = self.seat_order.remove(0);
        self.seat_order.push(temp);

        // Deal the hand
        self.board = deal_community(&mut self.deck);
        for (_, plyr) in &mut self.players {
            let cards = deal_hole(&mut self.deck);
            plyr.give_hand(&cards);
        }

        // Reset some player stuff and print chip counts
        println!("CHIP COUNTS");
        for (id, player) in &mut self.players {
            println!("{}:{} - {}", id, player.display_name, player.chips);

            if !player.eliminated {
                player.folded = false;
            }
        }

        // Reset some game stuff
        self.num_in_play = self.num_players - self.num_eliminated;
        self.num_folded = 0;

        self.street = Street::PreFlop;

        // Start action
        self.to_act = self.next_player(self.seat_order[0]);
        self.player_action(Action::PostBlind(1));
        self.player_action(Action::PostBlind(2));

        self.current_bet = 2;

    } // pub fn new_hand

    pub fn start(&mut self) -> () {
        println!("GAME - Starting");
        self.next_street();
    }

    /// Return the index of the next unfolded player in the move order
    fn next_player(&self, current_player: usize) -> usize {
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
    fn get_winners(&self, ids: Vec<usize>) -> Vec<usize> {
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