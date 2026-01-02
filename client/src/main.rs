use game_core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

#[derive(Debug, Serialize, Deserialize)]
enum NetMessage {
    Command(Command),
    Event(Event),
}

#[tokio::main]
async fn main() {
    let stream = TcpStream::connect("127.0.0.1:9000").await.unwrap();
    println!("Connected to server");
    let mut my_id: Option<usize> = None;

    let (r, mut w) = stream.into_split();
    let mut reader = BufReader::new(r).lines();

    while let Ok(Some(line)) = reader.next_line().await {
        let msg: NetMessage = serde_json::from_str(&line).unwrap();

        match msg {
            //
            NetMessage::Event(Event::PlayerAssigned { player_id }) => {
                my_id = Some(player_id);
                println!("You are player {}", player_id);
            }

            NetMessage::Event(e) => {
                println!("Event: {:?}", e);
            }

            NetMessage::Command(_) => {
                println!("Received a Command message, which is unexpected on the client.");
            }
        }
    }

    // 示例：直接发一个预测（真实版本你从 stdin 读）
    // let cmd = NetMessage::Command(Command::Predict {
    //     player_id: 0,
    //     rank: Some(1),
    // });

    // w.write_all((serde_json::to_string(&cmd).unwrap() + "\n").as_bytes())
    //     .await
    //     .unwrap();

    //游戏大循环
    //决定顺序
    //发牌

    //游戏内循环
    //先验预测
    //出牌
    //计算排名，不公布
    //后验预测
    //计算分数，公布（包括排名）

    //计算总分
    //下一轮投票
}
