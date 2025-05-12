pub mod protocol;


#[macro_use]
extern crate rocket;
use rocket_ws::{Channel, WebSocket};

use std::sync::mpsc::{Receiver, Sender};
use rocket::{Build, Rocket, State};
use rocket::fs::{relative, FileServer, Options};
use rocket::response::status;
use crate::protocol::HostInterface;

#[get("/")]
fn index() -> &'static str {
    "Hello world!"
}

// #[get("/ws")]
// fn updates<'r>() -> Result<Channel<'r>, status::Forbidden<&'static str>> {
//     let server2 = server;
//     let mut server = server.lock().unwrap();
//     let rx = server
//         .register(make_user_id(uid.to_string()))
//         .map_err(|_| status::Forbidden("Already registered"))?;
//     let tx = server_sender.0.clone();
//     Ok(ws.channel(move |stream| Box::pin(handle_socket(server2, tx, rx, stream, uid.to_string()))))
// }

fn rocket() -> Rocket<Build> {
    rocket::build()
        .mount("/", FileServer::new(relative!["static"], Options::default()))
        .mount("/helloworld", routes![index])
}

pub fn main(host_interface: HostInterface) {
    ::rocket::async_main(async {
        let _ = rocket().launch().await;
    });
}

// #[launch]
// fn rocket(host_interface: HostInterface) -> Rocket<Build> {
//     rocket::build()
//         .mount("/", FileServer::new(relative!["static"], Options::default()))
//         .mount("/helloworld", routes![index])
//         .mount("/ws", routes![updates])
// }