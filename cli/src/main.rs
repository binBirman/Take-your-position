use game_core::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::io::{self, Write};

fn main() {
    let players = init_players();
    let mut game = GameState::new(players);

    println!("Take Your Position - CLI 联机前版本");

    loop {
        match game.phase {
            Phase::Prediction => prediction_phase(&mut game),
            Phase::Play => play_phase(&mut game),
            Phase::End => {
                println!("\n=== 游戏结束 ===");
                for p in &game.players {
                    println!("玩家 {}：{} 分", p.id, p.score);
                }
                break;
            }
        }
    }
}

/* ================= 初始化 ================= */

fn init_players() -> Vec<PlayerState> {
    let mut players: Vec<PlayerState> = (0..5)
        .map(|id| PlayerState {
            id,
            hand: vec![],
            score: 0,
            prediction: None,
        })
        .collect();

    // === 构造牌堆（CLI 层允许做） ===
    let mut small = vec![];
    let mut big = vec![];
    let mut spades = vec![];

    for &suit in &[Suit::Heart, Suit::Club, Suit::Diamond] {
        for &rank in &[
            Rank::A,
            Rank::Two,
            Rank::Three,
            Rank::Four,
            Rank::Five,
            Rank::Six,
            Rank::Seven,
        ] {
            small.push(Card { rank, suit });
        }
        for &rank in &[
            Rank::Eight,
            Rank::Nine,
            Rank::Ten,
            Rank::J,
            Rank::Q,
            Rank::K,
        ] {
            big.push(Card { rank, suit });
        }
    }

    for &rank in &[
        Rank::A,
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::J,
        Rank::Q,
        Rank::K,
    ] {
        spades.push(Card {
            rank,
            suit: Suit::Spade,
        });
    }

    let mut rng = thread_rng();
    small.shuffle(&mut rng);
    big.shuffle(&mut rng);
    spades.shuffle(&mut rng);

    for p in &mut players {
        p.hand.push(small.pop().unwrap());
        p.hand.push(small.pop().unwrap());
        p.hand.push(big.pop().unwrap());
        p.hand.push(big.pop().unwrap());
        p.hand.push(spades.pop().unwrap());
    }

    players
}

/* ================= 预测阶段 ================= */

fn prediction_phase(game: &mut GameState) {
    println!("\n--- 预测阶段 ---");

    // 逆时针
    for i in 0..game.players.len() {
        let idx = (game.start_player + game.players.len() - i) % game.players.len();
        let p = &game.players[idx];

        print!("玩家 {} 预测名次 (1-5 或 -): ", p.id);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        let cmd = Command::Predict {
            player_id: p.id,
            rank: if input == "-" {
                None
            } else {
                Some(input.parse().unwrap())
            },
        };

        dispatch(game, cmd);
    }

    game.phase = Phase::Play;
}

/* ================= 出牌阶段 ================= */

fn play_phase(game: &mut GameState) {
    println!("\n--- 出牌阶段 ---");

    for pid in 0..game.players.len() {
        let p = &game.players[pid];
        println!("玩家 {} 手牌：", p.id);
        for (i, c) in p.hand.iter().enumerate() {
            println!("  [{}] {:?}{:?}", i, c.rank, c.suit);
        }

        print!("选择出牌编号: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let idx: usize = input.trim().parse().unwrap();

        let cmd = Command::PlayCard {
            player_id: p.id,
            card_index: idx,
        };

        dispatch(game, cmd);
    }
}

/* ================= Command → Event ================= */

fn dispatch(game: &mut GameState, cmd: Command) {
    match game.apply(cmd) {
        Ok(events) => {
            for e in events {
                handle_event(game, e);
            }
        }
        Err(e) => {
            println!("❌ 错误：{}", e);
        }
    }
}

fn handle_event(game: &mut GameState, event: Event) {
    match event {
        Event::PredictionAccepted { player_id } => {
            println!("玩家 {} 已完成预测", player_id);
        }
        Event::CardPlayed { player_id, card } => {
            println!("玩家 {} 出牌 {:?}", player_id, card);
        }
        Event::RoundResult {
            ranking,
            score_delta,
        } => {
            println!("\n--- 本轮结果 ---");
            for (i, pid) in ranking.iter().enumerate() {
                println!("第 {} 名：玩家 {}", i + 1, pid);
            }

            for (i, d) in score_delta.iter().enumerate() {
                if *d != 0 {
                    println!("玩家 {} 积分变化 {}", i, d);
                }
            }

            game.start_player = (game.start_player + game.players.len() - 1) % game.players.len();
        }
        Event::GameEnded => {
            println!("游戏结束");
        }
        Event::PhaseChanged => {}
    }
}
