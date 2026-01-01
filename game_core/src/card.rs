use serde::Deserialize;
use serde::Serialize;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Suit {
    Spade,
    Heart,
    Diamond,
    Club,
}

impl Suit {
    pub fn to_string(&self) -> String {
        match self {
            Suit::Spade => "♠".to_string(),
            Suit::Heart => "♥".to_string(),
            Suit::Diamond => "♦".to_string(),
            Suit::Club => "♣".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rank {
    A = 1,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    J,
    Q,
    K,
}

impl Rank {
    pub fn value(&self) -> u8 {
        *self as u8
    }

    pub fn to_string(&self) -> String {
        match self {
            Rank::A => "A".to_string(),
            Rank::Two => "2".to_string(),
            Rank::Three => "3".to_string(),
            Rank::Four => "4".to_string(),
            Rank::Five => "5".to_string(),
            Rank::Six => "6".to_string(),
            Rank::Seven => "7".to_string(),
            Rank::Eight => "8".to_string(),
            Rank::Nine => "9".to_string(),
            Rank::Ten => "10".to_string(),
            Rank::J => "J".to_string(),
            Rank::Q => "Q".to_string(),
            Rank::K => "K".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    pub fn compare(a: &Card, b: &Card, table: &[Card]) -> Ordering {
        let has_a = table.iter().any(|c| c.rank == Rank::A);
        let has_k = table.iter().any(|c| c.rank == Rank::K);

        if has_a && has_k {
            match (a.rank, b.rank) {
                (Rank::A, _) => return Ordering::Greater, // A 最大
                (_, Rank::A) => return Ordering::Less,    // A 最大
                (Rank::K, _) => return Ordering::Greater, // K 次大
                (_, Rank::K) => return Ordering::Less,    // K 次大
                _ => {}                                   // 其他情况继续比较
            }
        }

        match a.rank.cmp(&b.rank) {
            Ordering::Equal => suit_cmp(a.suit, b.suit),
            other => other,
        }
    }

    pub fn to_string(&self) -> String {
        let rank_str = self.rank.to_string();
        let suit_str = self.suit.to_string();
        format!("{}{}", suit_str, rank_str)
    }
}

fn suit_cmp(a: Suit, b: Suit) -> Ordering {
    use Suit::*;
    let v = |s| match s {
        Spade => 4,
        Heart => 3,
        Diamond => 2,
        Club => 1,
    };
    v(a).cmp(&v(b))
}
