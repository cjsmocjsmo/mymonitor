use crate::metrics::snapshot::MetricSnapshot;
use futures_util::{SinkExt, StreamExt};
use std::error::Error;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio_tungstenite::{accept_async, tungstenite::Message};

pub async fn run_server(addr: &str, tx: broadcast::Sender<MetricSnapshot>) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on ws://{}", addr);

    loop {
        let (stream, peer) = listener.accept().await?;
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let ws = match accept_async(stream).await {
                Ok(ws) => ws,
                Err(err) => {
                    eprintln!("WebSocket handshake failed for {}: {}", peer, err);
                    return;
                }
            };

            let (mut ws_write, mut ws_read) = ws.split();

            tokio::spawn(async move {
                while let Some(msg) = ws_read.next().await {
                    match msg {
                        Ok(message) => {
                            if message.is_close() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            });

            loop {
                match rx.recv().await {
                    Ok(snapshot) => {
                        match serde_json::to_string(&snapshot) {
                            Ok(payload) => {
                                if ws_write.send(Message::Text(payload)).await.is_err() {
                                    break;
                                }
                            }
                            Err(err) => {
                                eprintln!("Failed to serialize snapshot for {}: {}", peer, err);
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
    }
}
