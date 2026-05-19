use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct MessageData {
    from: String,
    message: String,
}

type Users = Arc<Mutex<HashMap<SocketAddr, String>>>;

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    bcast_tx: broadcast::Sender<String>,
    users: Users,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut bcast_rx = bcast_tx.subscribe();
    let mut username = String::from("anonymous");

    loop {
        tokio::select! {
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        if let Some(text) = msg.as_text() {
                            if let Ok(ws_msg) = serde_json::from_str::<WebSocketMessage>(text) {
                                match ws_msg.message_type {
                                    MsgTypes::Register => {
                                        if let Some(name) = ws_msg.data {
                                            username = name;
                                            users.lock().unwrap().insert(addr, username.clone());
                                            let user_list: Vec<String> = users.lock().unwrap().values().cloned().collect();
                                            let response = WebSocketMessage {
                                                message_type: MsgTypes::Users,
                                                data_array: Some(user_list),
                                                data: None,
                                            };
                                            bcast_tx.send(serde_json::to_string(&response)?)?;
                                        }
                                    }
                                    MsgTypes::Message => {
                                        if let Some(text) = ws_msg.data {
                                            let msg_data = MessageData {
                                                from: username.clone(),
                                                message: text,
                                            };
                                            let response = WebSocketMessage {
                                                message_type: MsgTypes::Message,
                                                data: Some(serde_json::to_string(&msg_data)?),
                                                data_array: None,
                                            };
                                            bcast_tx.send(serde_json::to_string(&response)?)?;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    Some(Err(err)) => return Err(err.into()),
                    None => break,
                }
            }
            msg = bcast_rx.recv() => {
                if ws_stream.send(Message::text(msg?)).await.is_err() {
                    break;
                }
            }
        }
    }

    // cleanup on disconnect
    users.lock().unwrap().remove(&addr);
    let user_list: Vec<String> = users.lock().unwrap().values().cloned().collect();
    let response = WebSocketMessage {
        message_type: MsgTypes::Users,
        data_array: Some(user_list),
        data: None,
    };
    let _ = bcast_tx.send(serde_json::to_string(&response)?);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (bcast_tx, _) = broadcast::channel(64);
    let users: Users = Arc::new(Mutex::new(HashMap::new()));
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("YewChat server listening on port 8080");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {addr:?}");
        let bcast_tx = bcast_tx.clone();
        let users = users.clone();
        tokio::spawn(async move {
            let ws_stream = ServerBuilder::new().accept(socket).await?;
            if let Err(e) = handle_connection(addr, ws_stream, bcast_tx, users).await {
                println!("Error: {e:?}");
            }
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
        });
    }
}
