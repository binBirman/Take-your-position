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

#[tokio::main]
async fn main() {
    let next_player_id = Arc::new(AtomicUsize::new(0));
    let listener = TcpListener::bind("0.0.0.0:9000").await.unwrap();
    println!("Server listening on 9000");

    let game = Arc::new(Mutex::new(init_game()));
    let clients: Arc<Mutex<HashMap<usize, OwnedWriteHalf>>> = Arc::new(Mutex::new(HashMap::new())); // 修改为 HashMap 存储 player_id 和 TcpStream 的映射

    //建立连接
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let player_id = next_player_id.fetch_add(1, Ordering::SeqCst); // 分配 player_id

        if player_id >= 5 {
            // 检查是否超过 5 人限制
            println!("Rejecting connection: player limit reached");
            break; // 超过 5 人退出循环
        }

        println!("Client connected with player_id: {}", player_id);

        tokio::spawn({
            let game = game.clone();
            let clients = clients.clone();

            async move {
                handle_client(socket, player_id, game, clients).await;
            }
        });
    }

    //游戏大循环
    // loop {
    //     // 示例：从某个地方获取命令（例如从客户端或其他逻辑）
    //     let cmd = ...; // 获取 Command

    //     let events = game.apply(cmd)?;

    //     for event in events {
    //         match &event {
    //             // 广播事件
    //             Event::PredictionAccepted { .. }
    //             | Event::CardPlayed { .. }
    //             | Event::RoundResult { .. }
    //             | Event::PhaseChanged
    //             | Event::GameStarted
    //             | Event::GameEnded => {
    //                 //broadcast(event).await;
    //             }

    //             // 私有事件
    //             Event::CardsDealt { player_id, .. } => {
    //                 send_to_client(&clients, *player_id, &event).await;
    //             }

    //             Event::PlayerAssigned { .. } => {
    //                 // 通常在连接阶段处理
    //             }
    //         }
    //     }
    //     Ok(());
    // }
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
                        // match &event {
                        //     Event::PredictionAccepted { .. }
                        //     | Event::CardPlayed { .. }
                        //     | Event::RoundResult { .. } => {
                        //         broadcast(&clients, &event).await;
                        //     }
                        //     Event::CardsDealt { player_id, .. } => {
                        //         send_to_client(&clients, *player_id, &event).await;
                        //     }
                        //     _ => {}
                        // }
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

async fn send_to_client(
    clients: &Arc<Mutex<HashMap<usize, OwnedWriteHalf>>>,
    player_id: usize,
    msg: &NetMessage,
) {
    let text = serde_json::to_string(msg).unwrap() + "\n";
    if let Some(client) = clients.lock().await.get_mut(&player_id) {
        let _ = client.write_all(text.as_bytes()).await;
    }
}

/* ===== 初始化 GameState（临时写死 5 人） ===== */

fn init_game() -> GameState {
    let players = (0..5)
        .map(|id| PlayerState {
            id,
            hand: vec![], // 👉 第一版：先假定客户端本地有手牌
            score: 0,
            prediction: None,
        })
        .collect();

    GameState::new(players)
}
