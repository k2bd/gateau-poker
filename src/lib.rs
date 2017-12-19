extern crate rs_poker;

use rs_poker::core::{Deck,Card,FlatDeck,Flattenable,Rankable};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub fn test() {
    let mut deck = create_deck();
    let my_hand = deal_hole(&mut deck);
    println!("Hole Cards: {}, {}",my_hand[0],my_hand[1]);

    let your_hand = deal_hole(&mut deck);
    println!("Hole Cards: {}, {}",your_hand[0],your_hand[1]);

    let board = deal_community(&mut deck);
    println!("Board: {:?}",board);

    let test_hand = Hand::new_with_cards(my_hand + board);
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
                    suit  : tmp_card.suit
                }
        );
    }

    return cards;
}