use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_websockets::{Message, ServerBuilder, WebSocketStream};

async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: WebSocketStream<TcpStream>,
    bcast_tx: broadcast::Sender<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ws_stream.send(Message::text("Welcome to pachill chat!".to_string())).await?;

    let mut bcast_rx = bcast_tx.subscribe();
    
    loop {
        tokio::select! {
            incoming = ws_stream.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        if let Some(text) = msg.as_text() {
                            bcast_tx.send(text.into())?;
                        }
                    }
                    Some(Err(err)) => return Err(err.into()),
                    None => return Ok(()),
                }
            }
            msg = bcast_rx.recv() => {
                ws_stream.send(Message::text(msg?)).await?;
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (bcast_tx, _) = broadcast::channel(16);
    let listener = TcpListener::bind("127.0.0.1:2000").await?;
    println!("listening on port 2000");
    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from {addr:?}");
        let bcast_tx = bcast_tx.clone();
        tokio::spawn(async move {
            let ws_stream = ServerBuilder::new().accept(socket).await?;
            if let Err(e) = handle_connection(addr, ws_stream, bcast_tx).await {
                println!("Error handling connection: {e:?}");
            }
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
        });
    }
}