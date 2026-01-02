use game_core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

#[derive(Serialize, Deserialize)]
enum NetMessage {
    Command(Command),
    Event(Event),
}

enum ServerPhase {
    Waiting,  // 等人
    Playing,  // 游戏中
    Finished, // 已结束，等待是否重开
}

#[tokio::main]
async fn main() {
    let phase = Arc::new(Mutex::new(ServerPhase::Waiting));
    let next_player_id = Arc::new(AtomicUsize::new(0));
    let listener = TcpListener::bind("0.0.0.0:9000").await.unwrap();
    println!("Server listening on 9000");

    let game = Arc::new(Mutex::new(init_game()));
    let clients: Arc<Mutex<HashMap<usize, OwnedWriteHalf>>> = Arc::new(Mutex::new(HashMap::new())); // 修改为 HashMap 存储 player_id 和 TcpStream 的映射

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        let mut phase_guard = phase.lock().await;
        if !matches!(*phase_guard, ServerPhase::Waiting) {
            // 游戏中 / 已结束，不接新玩家
            let (_r, mut w) = socket.into_split();
            let _ = w.write_all(b"Game already started\n").await;
            continue;
        }

        let player_id = next_player_id.fetch_add(1, Ordering::SeqCst);

        if player_id >= 5 {
            let (_r, mut w) = socket.into_split();
            let _ = w.write_all(b"Player limit reached\n").await;
            continue;
        }

        println!("Client connected with player_id {}", player_id);

        tokio::spawn({
            let game = game.clone();
            let clients = clients.clone();

            async move {
                handle_client(socket, player_id, game, clients).await;
            }
        });

        // 如果正好 5 人，立刻发牌
        if player_id == 4 && game.lock().await.is_card == false {
            *phase_guard = ServerPhase::Playing;
            start_game(game.clone(), clients.clone()).await;
            game.lock().await.is_card = true;
        }
    }
}

async fn handle_client(
    socket: TcpStream,
    player_id: usize,
    game: Arc<Mutex<GameState>>,
    clients: Arc<Mutex<HashMap<usize, OwnedWriteHalf>>>, // 修改为 HashMap
) {
    let (r, mut w) = socket.into_split();
    let mut reader = BufReader::new(r).lines();

    // 将客户端添加到 clients 列表
    clients.lock().await.insert(player_id, w);

    // 发放 player_id
    {
        let assign = NetMessage::Event(Event::PlayerAssigned { player_id });
        let text = serde_json::to_string(&assign).unwrap() + "\n";

        // 需要重新借用写半部
        if let Some(writer) = clients.lock().await.get_mut(&player_id) {
            let _ = writer.write_all(text.as_bytes()).await;
        }
    }

    while let Ok(Some(line)) = reader.next_line().await {
        let msg: NetMessage = serde_json::from_str(&line).unwrap();

        if let NetMessage::Command(cmd) = msg {
            let mut game = game.lock().await;
            match game.apply(cmd) {
                Ok(events) => {
                    for event in events {
                        match &event {
                            //
                            Event::PredictionAccepted { .. }
                            | Event::CardPlayed { .. }
                            | Event::PosteriorPredictionAccepted { .. }
                            | Event::RoundResult { .. } => {
                                broadcast(&clients, &event).await;
                            }
                            Event::CardsDealt { player_id, .. } => {
                                send_to_player(&clients, *player_id, &event).await;
                            }
                            _ => {}
                        }
                    }
                }
                Err(err) => {
                    if let Some(writer) = clients.lock().await.get_mut(&player_id) {
                        let _ = writer.write_all(format!("ERR {}\n", err).as_bytes()).await;
                    }
                }
            }
        }
    }
}

async fn send_to_player(
    clients: &Arc<Mutex<HashMap<usize, OwnedWriteHalf>>>,
    player_id: usize,
    msg: &game_core::Event,
) {
    let text = serde_json::to_string(msg).unwrap() + "\n";

    // 只在这里短暂加锁
    let mut client = {
        let mut guard = clients.lock().await;
        guard.remove(&player_id)
    };

    if let Some(mut writer) = client {
        let _ = writer.write_all(text.as_bytes()).await;

        // 写完再放回去
        clients.lock().await.insert(player_id, writer);
    }
}

async fn broadcast(clients: &Arc<Mutex<HashMap<usize, OwnedWriteHalf>>>, msg: &game_core::Event) {
    let text = serde_json::to_string(msg).unwrap() + "\n";

    // 先把所有 writer 拿出来
    let mut writers: Vec<(usize, OwnedWriteHalf)> = {
        let mut guard = clients.lock().await;
        guard.drain().collect()
    };

    // 逐个写（无锁）
    for (_, writer) in writers.iter_mut() {
        let _ = writer.write_all(text.as_bytes()).await;
    }

    // 再全部放回
    let mut guard = clients.lock().await;
    for (pid, writer) in writers {
        guard.insert(pid, writer);
    }
}

/* ===== 初始化 GameState（5人） ===== */

fn init_game() -> GameState {
    let players = (0..5)
        .map(|id| PlayerState {
            id,
            is_first: false,
            hand: vec![],
            score: 0,
            prediction: None,
            posterior_prediction: None,
            has_predicted: false,
            has_played: false,
        })
        .collect();

    GameState::new(players)
}

/* ================= 发牌阶段 ================= */
async fn start_game(
    game: Arc<Mutex<GameState>>,
    clients: Arc<Mutex<HashMap<usize, OwnedWriteHalf>>>,
) {
    let events = {
        let mut game = game.lock().await;
        game.deal_cards()
    };

    for event in events {
        if let Event::CardsDealt { player_id, .. } = &event {
            send_to_player(&clients, *player_id, &event).await;
        }
    }

    broadcast(&clients, &Event::GameStarted).await;
}

/* ================= 预测阶段 ================= */

/* ================= 出牌阶段 ================= */

/* ================= 后验预测阶段 ================= */

/* ================= 重开投票阶段 ================= */
async fn reset_game(
    game: &Arc<Mutex<GameState>>,
    clients: &Arc<Mutex<HashMap<usize, OwnedWriteHalf>>>,
    phase: &Arc<Mutex<ServerPhase>>,
) {
    *game.lock().await = init_game();
    *phase.lock().await = ServerPhase::Waiting;

    broadcast(clients, &Event::PhaseChanged).await;
}
