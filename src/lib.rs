extern crate rs_poker;

mod player;

use std::collections::HashMap;
use rs_poker::core::{Deck,Card,FlatDeck,Flattenable,Rank};
use player::Player;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub fn test() {
    let mut deck = create_deck();

    let mut players = HashMap::new();

    for i in 0..3 {
        players.insert(
            i, 
            Player{
                hole_cards : deal_hole(&mut deck),
                folded : false,
            } 
        );
    }

    for (id, player) in &players {
        println!("Player {} hole cards: {:?}",id,player.hole_cards);
    }

    let board = deal_community(&mut deck);
    println!("Board: {:?}",board);

    let winners = get_winners(&players, &board);
    println!("Winning players: {:?}, with {:?}",winners,players.get(&winners[0])
                                                               .unwrap()
                                                               .get_rank(&board));
}

/// Returns a shuffled and dealable deck
pub fn create_deck() -> FlatDeck {
    let mut deck = Deck::default().flatten();
    deck.shuffle();
    return deck;
}

/// Returns a vec of 2 cards as hole cards
pub fn deal_hole(deck: &mut FlatDeck) -> Vec<Card> {
    deal_cards(deck, 2)
}

/// Returns a vec of 5 cards as community cards
pub fn deal_community(deck: &mut FlatDeck) -> Vec<Card> {
    deal_cards(deck, 5)
}

/// Returns a vec with capacity `num` filled with cards
pub fn deal_cards(deck: &mut FlatDeck, num: usize) -> Vec<Card> {
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

pub fn get_winners(players: &HashMap<usize,Player>, community: &Vec<Card>) -> Vec<usize> {    
    let mut best_hands = Vec::<usize>::new();

    let best_rank = players.iter()
                           .fold(Rank::HighCard(0), |best, (_, player)| {
                               if !player.folded {
                                   let new_rank = &player.get_rank(community);
                                   if new_rank > &best {
                                       &new_rank
                                   } else {
                                       &best
                                   }
                               } else {
                                   best
                               }
                           });
    
    for (id, player) in players {
        if player.get_rank(community) == best_rank {
            best_hands.push(id.clone());
        }
    }

    return best_hands;
}