use std::sync::{Arc, Mutex};
use std::sync::mpsc::SendError;
use std::sync::mpsc::Sender;
use rocket::futures::channel::mpsc::{UnboundedReceiver};
use rocket::futures::StreamExt;
use rocket::State;
use rocket_ws::Message::Close;
use rocket_ws::result::Error;
use rocket_ws::stream::DuplexStream;
use tokio::select;
use tokio::task::JoinHandle;
use log::error;
use crate::protocol::{AnnotatedClientPacket, ClientPacket, HostInterface, Packet};
use crate::protocol::Packet::Connected;
use crate::Users;
use crate::websocket::ClientStreamError::HostClosed;

#[derive(Debug)]
pub enum ClientStreamError {
    Socket(rocket_ws::result::Error),
    HostClosed(SendError<AnnotatedClientPacket>),
    Json(serde_json::Error)
}

pub(crate) async fn handle_socket(
    channel: DuplexStream,
    host_interface: &State<HostInterface>,
    users: &State<Arc<Mutex<Users>>>,
) -> rocket_ws::result::Result<(), Error> {
    let to_host = host_interface.send.clone();
    let dis_host = to_host.clone();
    let uid = users.lock().map(|mut users| {
        users.add_next()
    }).unwrap();
    match to_host.send(AnnotatedClientPacket {
        user_id: uid,
        packet: Packet::Connected,
    }) {
        Err(e) => {
            error!("Error while sending connection message: {e:?}");
            return Ok(())
        }
        Ok(_) => {}
    }
    let (mut sender, mut receiver) = channel.split();
    let receive_task: JoinHandle<Result<(), ClientStreamError>> = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Close(_c) => {
                    info!("Closing connection.");
                    break;
                }
                msg => {
                    // info!("Received message: {msg:?}");
                    match ClientPacket::from_ws_message(msg) {
                        Ok(client_packet) => {
                            to_host.send(AnnotatedClientPacket {
                                packet: Packet::Client(client_packet),
                                user_id: uid
                            })?;
                        }
                        Err(e) => {
                            error!("Failed to decode message: {e:?}");
                            continue;
                        }
                    }
                },
            }
        }
        Ok(())
    });

    // Sending task (handles outgoing messages)
    // let send_task: JoinHandle<Result<(), Error>> = tokio::spawn(async move {
    //
    // });

    // Wait for either task to complete
    select! {
        e = receive_task => info!("Channel closed from receiver end with {e:?}"),
        // _ = send_task => info!("Channel closed from sender end"),
    }

    match dis_host.send(AnnotatedClientPacket {
        user_id: uid,
        packet: Packet::Disconnected,
    }) {
        Err(e) => {
            error!("Error while sending disconnect message: {e:?}");
        }
        Ok(_) => {}
    }

    Ok(())
}

impl From<SendError<AnnotatedClientPacket>> for ClientStreamError {
    fn from(value: SendError<AnnotatedClientPacket>) -> Self {
        HostClosed(value)
    }
}

impl From<serde_json::Error> for ClientStreamError {
    fn from(value: serde_json::Error) -> Self {
        ClientStreamError::Json(value)
    }
}