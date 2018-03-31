use player::Player;
use std::collections::HashMap;
use rand::{thread_rng, Rng};
use rs_poker::core::{Deck, Card, Flattenable, FlatDeck, Rank};
use reqwest::header::{Headers, ContentType};
use reqwest::{Response};
use reqwest;
use serde::{Serialize};

#[derive(Debug)]
#[derive(PartialEq)]
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
    Call,
    AllIn,
}

#[derive(Serialize)]
struct PlayerPrivateInfo {
    info : String, // "PlayerPrivateInfo"
    ingame_id : usize,
    secret_id : String,
}

#[derive(Serialize)]
struct GameTableInfo {
    info : String, // "GameTableInfo"
    starting_stack : usize,
    seat_order : Vec<usize>,
    button_player : usize,
    display_names : Vec<(usize, String)>,
}

#[derive(Serialize)]
struct HoleCardInfo {
    info : String,
    hole_cards : (String, String),
    hand_number : usize,
}

#[derive(Serialize)]
struct MoveInfo {
    info : String, // "MoveInfo"
    player_id : usize,
    move_type : String,
    value : usize,
    hand_number : usize,
}

#[derive(Serialize)]
struct ToMoveInfo {
    info : String,
    player_id : usize,
    hand_number : usize,
}

#[derive(Serialize)]
struct StreetInfo {
    info : String, // "StreetInfo"
    street : String,
    button_player : usize,
    board_cards_revealed : Vec<String>,
    hand_number : usize,
}

#[derive(Serialize)]
struct PayoutInfo {
    info : String,                              // "PayoutInfo"
    reason : String,                            // E.g. "All others folded", "Showdown"
    payouts : Vec<(usize, usize)>,              // Player IDs and payout amounts
    hole_cards : Vec<(usize,(String, String))>, // Player IDs and revealed cards, if any
}

#[derive(Serialize)]
struct PlayerEliminatedInfo {
    info : String, // "PlayerEliminationInfo"
    eliminated_player : usize,
}

#[derive(Serialize)]
struct GameOverInfo {
    info : String, // "GameOverInfo"
    winning_player : usize,
}

#[derive(Debug)]
/// Contains the state of the poker game, including players, cards, action, etc.
pub struct Game {
    // TODO:
    //  - Push game moves to a database
    //  - Add a timer to game moves
    //  - Get rid of reqwest client, switch to well-implemented asynch
    //  - Restart Game
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

    // Private Fields
    deck : FlatDeck,                      // A deck of cards

    started : bool,                       // Has the game started?
    game_over : bool,                     // Is the game over?

    num_players : usize,                  // Player count
    num_in_play : usize,                  // Number of players able to act (not folded or all-in)
    num_folded  : usize,                  // Number of players who have folded
    num_eliminated : usize,               // Number of players who have gone to 0 chips

    // configurable
    max_players : usize,                  // Maximum number of players that can join
    starting_stack : usize,               // Number of chips we start with, with 1/2 blinds

    button : usize,                       // Position of the dealer button

    current_bet : usize,
    min_raise   : usize,

    hand_number : usize,

    client : reqwest::Client,
}

impl Game {
    /// Returns a new game object
    pub fn new(stack : usize) -> Game {
        Game{
            deck : create_deck(),
            board : Vec::new(),
            players : HashMap::new(),
            max_players : 10,
            starting_stack : stack,
            seat_order : Vec::new(),
            game_over : false,
            num_players : 0,
            num_in_play : 0,
            num_folded : 0,
            num_eliminated : 0,
            started : false,
            button : 0,
            street : Street::River,
            to_act : 0,
            //game_id : Uuid::new_v4(),
            current_bet : 0,
            min_raise : 2,
            client : reqwest::Client::new(),
            hand_number : 0,
        }
    } // pub fn new

    pub fn set_starting_stack(&mut self, stack: usize) -> bool {
        if self.started {
            return false;
        }

        self.starting_stack = stack;
        println!("CONFIG - Setting starting stack to {}",stack);

        true
    }

    pub fn set_player_limit(&mut self, limit : usize) -> bool {
        // TODO: Don't let us set the limit to less than the number of players that have joined
        //       ^ incorporate this into the return error message

        if self.started {
            return false;
        }

        self.max_players = limit;
        println!("CONFIG - Setting player limit to {}",limit);

        true
    }

    /// Add a player to the game. Things like ID, starting stack, etc are handled automatically. 
    pub fn add_player(&mut self, name : &str, address: &str) -> bool {
        if (self.num_players == self.max_players) || self.started {
            return false;
        }

        let id = self.num_players;
        self.players.insert(
            id,
            Player::new(String::from(name), String::from(address), self.starting_stack)
        );
        self.seat_order.push(id);
        
        thread_rng().shuffle(&mut self.seat_order);

        self.num_players += 1;

        println!("DEBUG - Added player {}:{}",id,name);

        true
    } // pub fn add_player

    /// Takes a player action and applies it to the game
    /// 
    /// # Valid actions
    /// * Bet(amount)
    ///   This is a basic action that can cover calling, betting, raising.
    ///   This action moves a number of chips from your stack to the table in front of you.
    ///   In this way, if you bet 10, someone else raises to 30, then Bet(20) is a call. 
    ///   An under-call is interpreted as a call.
    ///   A raise under the min-raise is a min-raise.
    ///   Any bet that puts you above all-in just puts you all-in.
    ///   Bet(0) is a special case and is a Check if it's legal, otherwise it's a Fold. 
    /// * Check
    ///   This is a basic action that just checks if it's legal to, otherwise it folds.
    /// * Fold
    ///   This folds your hand.
    /// * Call
    ///   This is a special action that bets to match the current bet, or just checks.
    /// * AllIn
    ///   This puts you all in. 
    pub fn player_action(&mut self, recv_action: Action) -> () {

        let real_action;
        let move_type;
        let mut move_amnt = 0;

        { // Block to contain mutable borrow in plyr
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
                Action::Call => {
                    if self.current_bet == plyr.street_contrib {
                        // If we can just check this is a check!
                        real_action = Action::Check;
                    } else {
                        // Try to call, up to an all-in
                        real_action = Action::Bet(plyr.chips.min(self.current_bet
                                                                - plyr.street_contrib));
                    }
                },
                Action::AllIn => {
                    real_action = Action::Bet(plyr.chips);
                }
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
                    move_type = "Check";
                    println!("GAME - Player {} checks",plyr.display_name);
                    plyr.has_option = false;
                },
                Action::Fold => {
                    move_type = "Fold";
                    plyr.folded = true;
                    println!("GAME - Player {} folds",plyr.display_name);
                    plyr.has_option = false;
                },
                Action::Bet(bet) => {
                    move_type = "Bet";
                    move_amnt = bet;
                    if bet + plyr.street_contrib == self.current_bet {
                        println!("GAME - Player {} calls {} (total {})",plyr.display_name, bet, bet + plyr.street_contrib);
                    } else if self.current_bet == 0 || (self.street == Street::PreFlop && self.current_bet == 2) {
                        // TODO: Swap 2 for a big blind varaible
                        println!("GAME - Player {} bets {} (total {})",plyr.display_name, bet, bet + plyr.street_contrib);
                    } else {
                        println!("GAME - Player {} raises {} (total {})",plyr.display_name, bet, bet + plyr.street_contrib);
                    }

                    if bet + plyr.street_contrib > self.current_bet {
                        self.min_raise = bet + plyr.street_contrib - self.current_bet;
                        println!("DEBUG - Increasing minimum raise to {}",self.min_raise);
                    }

                    self.current_bet = self.current_bet.max(bet + plyr.street_contrib);

                    plyr.street_contrib += bet;
                    plyr.chips -= bet;
                    plyr.has_option = false;
                },
                Action::PostBlind(blind) => {
                    move_type = "Blind";
                    move_amnt = blind;
                    plyr.street_contrib += blind;
                    plyr.chips -= blind;
                    println!("GAME - Player {} posts blind {}",plyr.display_name, blind);
                },
                _ => {
                    panic!("Invalid action got here");
                }
            }

            if plyr.chips == 0 {
                plyr.all_in = true;
                println!("GAME - Player {} has gone all-in!",plyr.display_name);
            }
        } // End of block to free mutable borrow

        let move_info = MoveInfo {
            info : "MoveInfo".to_string(),
            player_id : self.to_act,
            move_type : move_type.to_string(),
            value : move_amnt,
            hand_number : self.hand_number,
        };

        self.send_to_all_players(&move_info);
        
        if self.is_hand_over() {
            self.end_hand();
        } else if self.is_street_over() {
            self.next_street();
        } else {
            self.to_act = self.next_player(self.to_act);
            let to_move = ToMoveInfo {
                info : "ToMoveInfo".to_string(),
                player_id : self.to_act,
                hand_number : self.hand_number,
            };
            self.send_to_all_players(&to_move);
        }
    }

    fn is_hand_over(&self) -> bool {
        // If we're on the river and the street is done, the hand is over.
        if self.street == Street::River && self.is_street_over() {
            return true;
        }

        let players_with_action = self.players.iter()
                                              .fold(0, |sum, (_, player)| {
                                                if !player.folded && !player.eliminated {// && !player.all_in {
                                                    sum + 1
                                                } else {
                                                    sum
                                                }
                                              });

        let players_all_in = self.players.iter()
                                         .fold(0, |sum, (_, player)| {
                                             if player.all_in {
                                                 sum + 1
                                             } else {
                                                 sum
                                             }
                                         });
        
        if players_with_action == 1 || players_with_action == players_all_in {
            println!("DEBUG - HAND OVER");
            return true;
        }

        false
    }

    fn is_street_over(&self) -> bool {
        // If anyone has option we can't end the street
        if self.players.iter().any(|(_, player)| !player.all_in && !player.folded && !player.eliminated && player.has_option) {
            return false;
        }

        // Otherwise, if everyone's put in the same amount we're done
        if self.players.iter().any(|(_,player)| !player.folded && player.street_contrib != self.current_bet && !player.all_in){
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
        // Give option to the player that will act LAST!
        let option_player = self.prev_player(self.next_player(self.seat_order[0]));
        self.players.get_mut(&option_player).unwrap().has_option = true;

        // Deal the cards for the street
        let revealed_cards_raw : Vec<Card>;
        let revealed_cards: Vec<String>;
        let street_name;
        match self.street {
            Street::Flop => {
                println!("GAME - Flop: {:?}",&self.board[0..3]);
                revealed_cards_raw = self.board[0..3].to_vec();
                street_name = "Flop";
            },
            Street::Turn => {
                println!("GAME - Turn: {:?}",&self.board[3]);
                revealed_cards_raw = vec!(self.board[3]);
                street_name = "Turn";
            },
            Street::River => {
                println!("GAME - River: {:?}",&self.board[4]);
                revealed_cards_raw = vec!(self.board[4]);
                street_name = "River";
            }
            _ => {
                panic!("Invalid street!");
            }
        }

        revealed_cards = revealed_cards_raw.iter()
                                           .map(|card| card_to_string(card))
                                           .collect::<Vec<_>>();

        let street_info = StreetInfo {
            info : "StreetInfo".to_string(),
            street : street_name.to_string(),
            button_player : self.seat_order[self.button],
            board_cards_revealed : revealed_cards,
            hand_number : self.hand_number,
        };

        self.send_to_all_players(&street_info);

        let to_move = ToMoveInfo {
            info : "ToMoveInfo".to_string(),
            player_id : self.to_act,
            hand_number : self.hand_number,
        };
        self.send_to_all_players(&to_move);
    }

    fn end_hand(&mut self) -> () {
        // Figure out winners, sidepots, etc
        // Eliminate players
        for (_, player) in &mut self.players {
            player.hand_contrib += player.street_contrib;
            player.street_contrib = 0;
        }

        let mut to_pay = Vec::new();
        // TODO: Auto-muck hands that wouldn't be forced to reveal!
        // TODO: Winner doesn't have to show hand if he won by fold!
        //       Implement win-by-fold message.
        let mut hand_revealed = Vec::new();

        for _ in 0..self.num_players {
            to_pay.push(0);
            hand_revealed.push(false);
        }

        let mut current_pot = self.players.iter()
                                          .map(|(_, player)| {
                                            if player.folded || player.hand_contrib == 0 {
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
                if !player.folded && !player.eliminated && player.hand_contrib > 0 {
                    in_pot.push(id.clone());
                    hand_revealed[*id] = true;
                }

                let contrib = current_pot.min(player.hand_contrib);
                payout += contrib;
                player.hand_contrib -= contrib;
            }
            println!("DEBUG - players in pot {:?}",in_pot);

            let winners = self.get_winners(in_pot);

            println!("DEBUG - WINNERS: {:?}",winners);

            // Split payout between winners
            let indiv_payout = payout / winners.len();

            for &id in &winners {
                to_pay[id] += indiv_payout;
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

            if self.players.iter().all(|(_, player)| player.hand_contrib == 0) {
                current_pot = 0;
            } else {
                current_pot = self.players.iter()
                                          .map(|(_, player)| {
                                            if player.folded || player.eliminated || player.hand_contrib == 0 {
                                                usize::max_value()
                                            } else {
                                                player.hand_contrib
                                            }
                                          })
                                          .min().unwrap();
            }
        }

        let payouts = (0..self.num_players).map(|x| (x, to_pay[x]))
                                           .collect::<Vec<(usize, usize)>>();
        
        // Create a Vec<(id, (card, card))> for each player whose hand is revealed
        // during showdown
        let hands_revealed = (0..self.num_players)
                                .filter_map(|id| {
                                    if hand_revealed[id] {
                                        let player_cards = &self.players[&id].hole_cards;
                                        Some(
                                            (id, (card_to_string(&player_cards[0]),
                                                  card_to_string(&player_cards[1])))
                                        )
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>();

        let payout_info = PayoutInfo {
            info : "PayoutInfo".to_string(),
            reason : "Showdown".to_string(),
            payouts : payouts,
            hole_cards : hands_revealed,
        };

        self.send_to_all_players(&payout_info);

        // Print summary of payouts
        println!("BOARD - {:?}",self.board);
        println!("HAND PAYOUTS");
        for (&id, player) in &mut self.players {
            if player.folded {
                println!("{}:{} folded",id, player.display_name);
            } else {
                println!("{}:{} - {} for {:?} ({:?})",id, player.display_name, to_pay[id], 
                                            player.hole_cards, player.get_rank(&self.board));
            }
            player.chips += to_pay[id];

            if player.chips == 0 && !player.eliminated {
                player.eliminated = true;
                player.folded = true;
                println!("{} eliminated!",player.display_name);
                
                // TODO: Move this somewhere out of mutable borrow of game!!
                //let player_eliminated_info = PlayerEliminatedInfo {
                //    info : "PlayerEliminatedInfo".to_string(),
                //    eliminated_player : id,
                //};
                //self.send_to_all_players(&player_eliminated_info);
            }
        }

        // If the game's over, for now just set the internal variable to true
        self.game_over = self.players.iter()
                                     .fold(0, |sum, (_, player)| 
                                        if player.eliminated { sum + 1 } else { sum }
                                    ) == self.num_players - 1;

        if self.game_over {
            let mut winning_player = self.num_players;

            for (&id, player) in self.players.iter() {
                if player.chips > 0 {
                    winning_player = id;
                    break;
                }
            }

            let game_over_info = GameOverInfo {
                info : "GameOverInfo".to_string(),
                winning_player : winning_player,
            };
            self.send_to_all_players(&game_over_info);
        } else {
            self.new_hand();
        }
    }

    /// Sets up a new hand: shuffles a new deck, deals, etc.
    fn new_hand(&mut self) -> () {
        // Increase hand number
        self.hand_number += 1;

        // Create a new deck
        self.deck = create_deck();

        // Reset min-raise
        self.min_raise = 2;

        // Move the button
        let temp = self.seat_order.remove(0);
        self.seat_order.push(temp);

        // Deal the hand
        self.board = deal_community(&mut self.deck);
        for (_, plyr) in &mut self.players {
            let cards = deal_hole(&mut self.deck);
            plyr.give_hand(&cards);
        }

        for (&id, player) in self.players.iter() {
            let hole_card_info = HoleCardInfo {
                info : "HoleCardInfo".to_string(),
                hole_cards : (card_to_string(&player.hole_cards[0]),
                              card_to_string(&player.hole_cards[1])),
                hand_number : self.hand_number,
            };
            let response = &self.send_to_player(id, &hole_card_info);
            println!("Sent player {} hole cards: {}",id,response.status());
        }

        // Reset some player stuff and print chip counts
        println!("CHIP COUNTS");
        for (id, player) in &mut self.players {
            println!("{}:{} - {}", id, player.display_name, player.chips);

            if !player.eliminated {
                player.folded = false;
                player.all_in = false;
            }
        }

        // Reset some game stuff
        self.num_in_play = self.num_players - self.num_eliminated;
        self.num_folded = 0;

        self.street = Street::PreFlop;

        // Start action
        self.to_act = self.next_player(self.seat_order[0]);
        if self.num_in_play() == 2 {
            self.to_act = self.next_player(self.to_act);
        }
        let big_blind = &self.next_player(self.to_act);
        self.players.get_mut(big_blind).unwrap().has_option = true;
        self.player_action(Action::PostBlind(1));
        self.player_action(Action::PostBlind(2));

        self.current_bet = 2;

        let street_info = StreetInfo {
            info : "StreetInfo".to_string(),
            street : "PreFlop".to_string(),
            button_player : self.seat_order[self.button],
            board_cards_revealed : Vec::<String>::new(),
            hand_number : self.hand_number,
        };
        self.send_to_all_players(&street_info);

        let to_move = ToMoveInfo {
            info : "ToMoveInfo".to_string(),
            player_id : self.to_act,
            hand_number : self.hand_number,
        };
        self.send_to_all_players(&to_move);

    } // pub fn new_hand

    /// Call this function to indicate the players are in and the game is ready to start.
    pub fn start(&mut self) -> bool {
        if self.started {
            return false;
        }

        println!("DEBUG - Sending player information");
        
        let game_info = GameTableInfo {
            info : "GameTableInfo".to_string(),
            starting_stack : self.starting_stack,
            seat_order : self.seat_order.clone(), // TODO: differnet soln
            button_player : self.seat_order[self.button],
            display_names : self.seat_order.iter()
                                           .map(|&id| (id, self.players[&id].display_name.clone()))
                                           .collect::<Vec<_>>()
        };

        for (&id, player) in self.players.iter() {
            let player_info = PlayerPrivateInfo {
                info : "PlayerPrivateInfo".to_string(),
                ingame_id : id,
                secret_id : player.secret_id.simple().to_string(),
            };

            println!("DEBUG - Sending info to Player {}",player.display_name);

            // TODO: remove synch comms
            let response = self.send_to_player(id, &player_info);

            println!("DEBUG - {}: {}",self.players[&id].display_name, 
                                                    response.status());
        }

        self.send_to_all_players(&game_info);
        
        println!("GAME - Starting");
        self.next_street();

        self.started = true;

        true
    }

    fn num_in_play(&self) -> usize {
        self.players.iter().fold(0,|sum, (_, ref player)| if !player.eliminated {sum + 1} else {sum})
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
                //panic!("Only one unfolded player!");
                // We only have one player left
                return i;
            } else {
                let plyr = self.players.get(&i).unwrap();
                if !plyr.folded && !plyr.all_in && !plyr.eliminated {
                    return i;
                }
            }
        }

        panic!("Something wrong in Game::next_player");
    }

        /// Return the index of the next unfolded player in the move order
    fn prev_player(&self, current_player: usize) -> usize {
        let mut found_current = false;

        let mut reverse_seat = self.seat_order.clone();
        reverse_seat.reverse();
        
        for &i in &reverse_seat {
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
        for &i in &reverse_seat {
            if i == current_player {
                //panic!("Only one unfolded player!");
                // We only have one player left
                return i;
            } else {
                let plyr = self.players.get(&i).unwrap();
                if !plyr.folded && !plyr.all_in && !plyr.eliminated {
                    return i;
                }
            }
        }

        panic!("Something wrong in Game::next_player");
    }

    /// Send a seralizable JSON message to all players
    fn send_to_all_players<T: Serialize>(&self, message : &T) -> () {

        let mut responses = Vec::new();

        // Don't require responses from eliminated players
        for (&id, _) in self.players
                                .iter()
                                .filter(|&(_, player)| !player.eliminated ) {
            responses.push((id, self.send_to_player(id, message)));
        }

        println!("DEBUG - Responses:"); // TODO: create a trait barrier so we can grab the info field from T
        for (id, response) in responses {
            println!("DEBUG - {}: {}",self.players[&id].display_name, response.status());
        }
     }

    fn send_to_player<T: Serialize>(&self, player_id: usize, message: &T) -> Response {
        let mut header = Headers::new();
        header.set(
            ContentType::json()
        );

        let player = &self.players[&player_id];

        let post_addr = player.address.to_owned()+"/player";

        self.client.post(&post_addr[..])
                   .headers(header)
                   .json(message)
                   .send()
                   .unwrap()
    }

    /// Of the players still in the hand, return a `Vec<usize>` of 
    /// the ID(s) of the player(s) with the best hand
    fn get_winners(&self, ids: Vec<usize>) -> Vec<usize> {
        let mut best_hands = Vec::<usize>::new();

        let best_rank = ids.iter()
                           .fold(Rank::HighCard(0), |best, id| {
                                let player = self.players.get(&id).unwrap();
                                if !player.folded && !player.eliminated {
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
            if !player.folded && !player.eliminated && player.get_rank(&self.board) == best_rank {
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

fn card_to_string(card: &Card) -> String {
    format!("{}{}",card.value.to_char(),card.suit.to_char())
}