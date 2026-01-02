use crate::card::Card;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
//系统宣布发生的事件
pub enum Event {
    // “player_id 的预测被记录了
    PredictionAccepted {
        player_id: usize,
    },
    // “player_id 出了张牌
    CardPlayed {
        player_id: usize,
    },
    // “player_id 的后验预测被记录了
    PosteriorPredictionAccepted {
        player_id: usize,
    },
    // // 出牌 card
    // CardPlayed {
    //     player_id: usize,
    //     card: Card,
    // },
    // 这一轮结算结果
    RoundResult {
        // 每个玩家的 ID 顺序与下面两个向量对应
        cards: Vec<Card>,                 // 玩家出牌
        ranking: Vec<usize>,              // 玩家排名
        prediction: Vec<usize>,           // 玩家先验预测结果
        posterior_prediction: Vec<usize>, // 玩家后验预测结果
        score_delta: Vec<i32>,            // 玩家分数变化
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
