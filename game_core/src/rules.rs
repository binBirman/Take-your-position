use crate::GameState;
use crate::PlayerState;
use crate::Rank;
use crate::Suit;
use crate::card::Card;
use crate::command::Command;
use crate::event::Event;
use crate::state::Phase;

use std::cmp::Ordering;

impl GameState {
    pub fn new(players: Vec<PlayerState>) -> Self {
        let cp = players[0].id;
        GameState {
            players,
            round: 0,
            start_player: 0,
            phase: Phase::PriorPrediction,
            current_player: cp,
            table: vec![],
            is_card: false,
        }
    }

    // 阶段校验
    fn check_phase(&self, expected_phase: Phase) -> Result<(), String> {
        if self.phase != expected_phase {
            Err(format!(
                "Invalid phase: expected {:?}, found {:?}",
                expected_phase, self.phase
            ))
        } else {
            Ok(())
        }
    }

    // 玩家回合校验
    fn check_current_player(&self, player_id: usize) -> Result<(), String> {
        if player_id != self.current_player {
            return Err(format!("Not your turn, player {}", player_id));
        }
        Ok(())
    }

    // 首位玩家校验
    fn check_first_player(&self, player_id: usize) -> Result<(), String> {
        if player_id != self.start_player {
            return Err(format!("Player {} is not the first player", player_id));
        }
        Ok(())
    }

    pub fn apply(&mut self, cmd: Command) -> Result<Vec<Event>, String> {
        match cmd {
            // 先验预测
            Command::Predict { player_id, rank } => {
                // 校验
                self.check_phase(Phase::PriorPrediction)?;
                self.check_current_player(player_id)?;

                let player = self
                    .players
                    .iter_mut()
                    .find(|p| p.id == player_id)
                    .ok_or("Invalid player")?;

                // 记录预测值
                player.prediction = rank;
                player.has_predicted = true;

                // 生成事件
                Ok(vec![Event::PredictionAccepted { player_id }])
            }
            // 出牌
            Command::PlayCard {
                player_id,
                card_index,
            } => {
                // 校验
                self.check_phase(Phase::Play)?;
                self.check_current_player(player_id)?;

                let player = self
                    .players
                    .iter_mut()
                    .find(|p| p.id == player_id)
                    .ok_or("Invalid player")?;

                // 出牌校验
                player.hand.get(card_index).ok_or("Invalid card index")?;

                //出牌
                let card = player.hand.remove(card_index);
                player.has_predicted = true;

                // 生成事件
                Ok(vec![Event::CardPlayed { player_id }])
            }

            // 后验预测
            Command::PosteriorPredict {
                player_id,
                rank_list,
            } => {
                // 校验
                self.check_phase(Phase::PosteriorPrediction)?;
                self.check_first_player(player_id)?;

                let player = self
                    .players
                    .iter_mut()
                    .find(|p| p.id == player_id)
                    .ok_or("Invalid player")?;

                // 记录预测值
                //player.posterior_prediction = Some(rank_list);
                player.posterior_prediction = rank_list;
                player.has_predicted = true;

                // 生成事件
                Ok(vec![Event::PosteriorPredictionAccepted { player_id }])
            }

            // 下一局投票
            Command::Restart { player_id, yes } => Ok(vec![]),
        }
    }

    fn finish_round(&mut self) -> Vec<Event> {
        // 克隆牌桌
        let mut table = self.table.clone();
        let first_player = table.first().map(|(pid, _)| *pid).unwrap_or(0);
        let cards: Vec<Card> = table.iter().map(|(_, c)| c.clone()).collect();

        // 按牌面大小排序
        table.sort_by(|a, b| Card::compare(&a.1, &b.1, &cards));

        // 初始化分数和排名
        let scores = [2, 1, 0, -1, -2]; // 不同排名对应的分数变化
        let mut delta = vec![0; self.players.len()]; // 每个玩家的分数变化
        let mut ranking = vec![]; // 本轮排名

        let mut prediction: Vec<usize> = vec![];
        let mut posterior_prediction: Vec<usize> = vec![];

        // 统计后验预测结果
        let mut accurate_count = 0; // 准确预测个数
        if let Some(p) = self.players[first_player].posterior_prediction.clone() {
            posterior_prediction = p;

            // 统计排名预测的准确个数
            for (predicted_rank, &player_id) in posterior_prediction.iter().enumerate() {
                if ranking.get(predicted_rank) == Some(&player_id) {
                    accurate_count += 1;
                }
            }
            //计算分数变化
            delta[first_player] += scores[5 - accurate_count];
        }

        // 遍历牌桌，按排名计算分数和排名
        for (player_rank, (player_id, _)) in table.iter().rev().enumerate() {
            // 根据排名调整分数
            delta[*player_id] += scores[player_rank];
            ranking.push(*player_id);

            // 如果玩家进行了预测，调整分数
            if let Some(prediction) = self.players[*player_id].prediction {
                if prediction == player_rank + 1 {
                    // 预测正确，加 2 分
                    delta[*player_id] += 2;
                } else {
                    // 预测错误，减 2 分
                    delta[*player_id] -= 2;
                }
            }
        }

        for i in 0..self.players.len() {
            self.players[i].score += delta[i];
            prediction.push(self.players[i].prediction.unwrap_or(0));
            self.players[i].prediction = None;
        }

        self.table.clear();
        self.round += 1;
        self.phase = if self.round >= 5 {
            Phase::End
        } else {
            Phase::PriorPrediction
        };

        vec![Event::RoundResult {
            cards,
            ranking,
            prediction,
            posterior_prediction,
            score_delta: delta,
        }]
    }
}
