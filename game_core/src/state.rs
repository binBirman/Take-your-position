use crate::Rank;
use crate::Suit;
use crate::card::Card;
use crate::command::Command;
use crate::event::Event;
use rand::rng;
use rand::seq::SliceRandom;

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub id: usize,                                // 玩家 ID
    pub is_first: bool,                           // 是否为首位玩家
    pub hand: Vec<Card>,                          // 玩家手牌
    pub score: i32,                               // 玩家分数
    pub prediction: Option<usize>,                // 先验预测,记录自己的排名,不记录为none
    pub has_predicted: bool,                      // 是否已经做出预测
    pub has_played: bool,                         // 是否已经出牌
    pub posterior_prediction: Option<Vec<usize>>, // 后验预测,按从预测排名高到低顺序记录玩家 ID,不记录为none
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    //Prediction, // 预测
    PriorPrediction,     // 先验预测
    Play,                // 出牌
    PosteriorPrediction, // 后验预测
    End,                 // 结束
}

#[derive(Debug)]
pub struct GameState {
    pub players: Vec<PlayerState>, // 玩家状态列表
    pub round: u8,                 // 当前轮数
    pub start_player: usize,       // 本轮起始玩家 ID
    pub phase: Phase,              // 当前阶段
    pub current_player: usize,     // 当前行动玩家 ID
    pub table: Vec<(usize, Card)>, // 本轮牌桌上的牌（玩家 ID，牌）
    pub is_card: bool,             // 是否发牌
}

impl GameState {
    pub fn deal_cards(&mut self) -> Vec<Event> {
        let mut rng = rng(); // 使用线程安全的随机数生成器

        // 固定牌堆内容
        let mut small_deck: Vec<Card> = vec![
            Card {
                rank: Rank::A,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Two,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Four,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Five,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Six,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Seven,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::A,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Two,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Four,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Five,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Six,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Seven,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::A,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Two,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Four,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Five,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Six,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Seven,
                suit: Suit::Diamond,
            },
        ];

        let mut big_deck: Vec<Card> = vec![
            Card {
                rank: Rank::Eight,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Nine,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Ten,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::J,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Q,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::K,
                suit: Suit::Heart,
            },
            Card {
                rank: Rank::Eight,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Nine,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Ten,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::J,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Q,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::K,
                suit: Suit::Club,
            },
            Card {
                rank: Rank::Eight,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Nine,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Ten,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::J,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::Q,
                suit: Suit::Diamond,
            },
            Card {
                rank: Rank::K,
                suit: Suit::Diamond,
            },
        ];

        let mut spade_deck: Vec<Card> = vec![
            Card {
                rank: Rank::A,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::Two,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::Four,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::Five,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::Six,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::Seven,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::Eight,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::Nine,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::Ten,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::J,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::Q,
                suit: Suit::Spade,
            },
            Card {
                rank: Rank::K,
                suit: Suit::Spade,
            },
        ];

        // 洗牌
        small_deck.shuffle(&mut rng);
        big_deck.shuffle(&mut rng);
        spade_deck.shuffle(&mut rng);

        let mut events = vec![];

        for p in &mut self.players {
            // 从小牌堆、大牌堆和黑桃牌堆中抽取牌
            let cards = vec![
                small_deck.pop().unwrap(),
                small_deck.pop().unwrap(),
                big_deck.pop().unwrap(),
                big_deck.pop().unwrap(),
                spade_deck.pop().unwrap(),
            ];

            p.hand = cards.clone(); // 更新玩家的手牌

            events.push(Event::CardsDealt {
                player_id: p.id,
                cards,
            });
        }

        events // 返回发牌事件
    }
}
