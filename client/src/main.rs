use game_core::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
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
        // Debug: Print the received JSON line
        println!("Received JSON: {}", line);

        // Attempt to parse the JSON
        let msg: NetMessage = match serde_json::from_str(&line) {
            Ok(parsed_msg) => parsed_msg,
            Err(e) => {
                println!("Failed to parse message: {}", e);
                continue;
            }
        };

        // 处理服务器发出的event消息
        match msg {
            //
            NetMessage::Event(Event::PlayerAssigned { player_id }) => {
                my_id = Some(player_id);
                println!("You are player {}", player_id);
            }

            NetMessage::Event(Event::CardsDealt { player_id, cards }) => {
                if Some(player_id) == my_id {
                    println!("Your cards:");
                    for card in cards {
                        println!("{}", card.to_string());
                    }
                }
            }

            NetMessage::Event(e) => {
                println!("Event: {:?}", e);
            }

            NetMessage::Command(_) => {
                println!("Received a Command message, which is unexpected on the client.");
            }
        }
    }
}
