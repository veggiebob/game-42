
use std::thread;
use bevy::prelude::*;

fn setup() {
    // start the web server
    let (send_net, rx) = std::sync::mpsc::channel();
    let (tx, recv_net) = std::sync::mpsc::channel();
    let host_interface = game_42_net::protocol::HostInterface::new(rx, tx);
    thread::spawn(move || {
        game_42_net::main(host_interface);
    });
    thread::spawn(move || {
        for msg in recv_net {
            info!("Rocket: received message {msg:?}");
        }
    });
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}
