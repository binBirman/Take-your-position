//use crate::card::Card;
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
    //后验预测
    PosteriorPredict {
        player_id: usize,
        rank_list: Option<Vec<usize>>,
    },
    // 重开投票
    Restart {
        player_id: usize,
        yes: bool,
    },
}
