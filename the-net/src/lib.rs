pub mod protocol;
pub mod websocket;
pub mod controls;

#[macro_use]
extern crate rocket;

use std::collections::HashSet;
use rocket_ws::{Channel, WebSocket};

use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use rocket::{Build, Rocket, State};
use rocket::fs::{relative, FileServer, Options};
use rocket::response::status;
use crate::protocol::{HostInterface, UserId};
use crate::websocket::handle_socket;

#[derive(Default)]
pub(crate) struct Users {
    connected: HashSet<UserId>
}

impl Users {
    pub fn add_next(&mut self) -> UserId {
        let x = self.connected.iter().min().map(|ui| ui.0).unwrap_or(0);
        let mut uid = UserId(x);
        while self.connected.contains(&uid) {
            uid.0 += 1;
        }
        self.connected.insert(uid.clone());
        uid
    }
}

#[get("/")]
fn index() -> &'static str {
    "Hello world!"
}

#[get("/ws")]
fn updates<'r>(
    ws: WebSocket,
    host_interface: &'r State<HostInterface>,
    users: &'r State<Arc<Mutex<Users>>>,
) -> Result<Channel<'r>, status::Forbidden<&'static str>> {
    Ok(ws.channel(move |stream| Box::pin(handle_socket(stream, host_interface, users))))
}

fn rocket(host_interface: HostInterface) -> Rocket<Build> {
    let figment = rocket::Config::figment()
        .merge(("port", 8000))
        .merge(("address", "0.0.0.0")) // when you want to visit it from outside
    ;
    rocket::custom(figment)
        .manage(host_interface)
        .manage(Arc::new(Mutex::new(Users::default())))
        .mount("/", FileServer::new(relative!["static"], Options::default()))
        .mount("/game", routes![index, updates])
}

pub fn main(host_interface: HostInterface) {
    rocket::async_main(async move {
        let _ = rocket(host_interface).launch().await;
    });
}