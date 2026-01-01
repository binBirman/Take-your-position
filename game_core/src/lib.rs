mod card;
mod command;
mod state;

pub use card::*;
pub use command::*;
pub use state::*;

use std::cmp::Ordering;

impl GameState {
    pub fn new(players: Vec<PlayerState>) -> Self {
        GameState {
            players,
            round: 0,
            start_player: 0,
            phase: Phase::Prediction,
            table: vec![],
        }
    }

    pub fn apply(&mut self, cmd: Command) -> Result<Vec<Event>, String> {
        match cmd {
            Command::Predict { player_id, rank } => {
                // 1️⃣ 阶段校验
                if self.phase != Phase::Prediction {
                    return Err("Not prediction phase".into());
                }

                // 2️⃣ 身份校验
                let player = self
                    .players
                    .iter_mut()
                    .find(|p| p.id == player_id)
                    .ok_or("Invalid player")?;

                // 3️⃣ 状态修改
                player.prediction = rank;

                // 4️⃣ 生成事件（不直接打印、不发网络）
                Ok(vec![Event::PredictionAccepted { player_id }])
            }
            Command::PlayCard {
                player_id,
                card_index,
            } => {
                // 暂时未实现逻辑，使用占位符
                todo!("Handle PlayCard command");
            }
        }
    }

    fn finish_round(&mut self) -> Vec<Event> {
        let mut table = self.table.clone();
        let cards: Vec<Card> = table.iter().map(|(_, c)| c.clone()).collect();

        table.sort_by(|a, b| Card::compare(&a.1, &b.1, &cards));

        let scores = [2, 1, 0, -1, -2];
        let mut delta = vec![0; self.players.len()];
        let mut ranking = vec![];

        for (rank, (pid, _)) in table.iter().rev().enumerate() {
            delta[*pid] += scores[rank];
            ranking.push(*pid);

            if let Some(p) = self.players[*pid].prediction {
                if p == rank + 1 {
                    delta[*pid] += 2;
                } else {
                    delta[*pid] -= 2;
                }
            }
        }

        for i in 0..self.players.len() {
            self.players[i].score += delta[i];
            self.players[i].prediction = None;
        }

        self.table.clear();
        self.round += 1;
        self.phase = if self.round >= 5 {
            Phase::End
        } else {
            Phase::Prediction
        };

        vec![Event::RoundResult {
            ranking,
            score_delta: delta,
        }]
    }
}
