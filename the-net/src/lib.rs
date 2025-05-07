// net/src/lib.rs
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::{get, Router},
};
use rand::random;
use tower_http::services::ServeDir;
use serde::Deserialize;
use std::{collections::HashMap, net::SocketAddr};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use once_cell::sync::Lazy;
use std::path::PathBuf;

/// What the Bevy game will receive.
#[derive(Debug)]
pub enum ServerEvent {
    Connected { id: u64 },
    Disconnected { id: u64 },
    Input { id: u64, inp: Input },
}

#[derive(Debug, Deserialize)]
pub struct Input {
    #[serde(default)] up: bool,
    #[serde(default)] down: bool,
    #[serde(default)] left: bool,
    #[serde(default)] right: bool,
    #[serde(default)] action: bool,
}




/* ---------- single global runtime ---------- */
static RT: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("create Tokio runtime")
});

/* ---------- public API ---------- */
pub fn spawn(addr: SocketAddr) -> UnboundedReceiver<ServerEvent> {
    let (tx_game, rx_game) = tokio::sync::mpsc::unbounded_channel();
    // run the async server inside the runtime
    RT.spawn(async move {
        if let Err(e) = run(addr, tx_game).await {
            eprintln!("websocket server error: {e:?}");
        }
    });
    rx_game
}

/// Launches the async server and returns a receiver the game can poll.
// pub fn spawn(addr: SocketAddr) -> UnboundedReceiver<ServerEvent> {
//     let (tx_game, rx_game) = tokio::sync::mpsc::unbounded_channel();
//     tokio::spawn(async move { run(addr, tx_game).await });
//     rx_game
// }

fn static_dir() -> PathBuf {
    // expands to ".../net"
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("static")
}

async fn run(addr: SocketAddr, tx_game: UnboundedSender<ServerEvent>) -> anyhow::Result<()> {
    let router = Router::new()
        .route("/ws", get(move |ws: WebSocketUpgrade| async move {
            ws.on_upgrade(move |socket| client(socket, tx_game.clone()))
        }))
        .nest_service("/", ServeDir::new(static_dir()));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router.into_make_service()).await?;
    Ok(())
}


async fn client(mut socket: WebSocket, tx_game: UnboundedSender<ServerEvent>) {
    // very small “hand-rolled” id generator
    let id = random::<u64>();
    let _ = tx_game.send(ServerEvent::Connected { id });

    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(txt) => {
                if let Ok(inp) = serde_json::from_str::<Input>(&txt) {
                    let _ = tx_game.send(ServerEvent::Input { id, inp });
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    let _ = tx_game.send(ServerEvent::Disconnected { id });
}
