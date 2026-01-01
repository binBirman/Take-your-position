use crate::card::Card;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
//玩家发起的事件
pub enum Command {
    //预测
    Predict {
        player_id: usize,
        rank: Option<usize>,
    },
    //出牌
    PlayCard {
        player_id: usize,
        card_index: usize,
    },
}

#[derive(Debug, Serialize, Deserialize)]
//系统宣布发生的事件
pub enum Event {
    // “player_id 的预测被记录了
    PredictionAccepted {
        player_id: usize,
    },
    // player_id 已经出了一张牌 card
    CardPlayed {
        player_id: usize,
        card: Card,
    },
    // 这一轮结算结果
    RoundResult {
        ranking: Vec<usize>,
        score_delta: Vec<i32>,
    },
    // 所有人已就绪
    GameStarted,
    // 游戏阶段切换
    PhaseChanged,
    // 整局游戏结束
    GameEnded,
    // 身份确认
    PlayerAssigned {
        player_id: usize,
    },
    // player_id 这一局的手牌
    CardsDealt {
        player_id: usize,
        cards: Vec<Card>,
    },
}
