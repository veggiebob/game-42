use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use bevy::prelude::*;
use game_42_net::controls::{InputUpdate, PlayerInput};
use game_42_net::protocol::{AnnotatedClientPacket, Packet, UserId};
use game_42_net::protocol::ClientPacket::Input;

/// Use this to message the net (probably a client?)
#[derive(Resource)]
pub struct MessageNet(pub Sender<()>);

#[derive(Resource)]
pub struct NetMessages(pub Mutex<Receiver<AnnotatedClientPacket>>);

#[derive(Resource)]
pub struct PlayerInputs(pub HashMap<UserId, PlayerInput>);

fn setup(
    mut commands: Commands
) {
    // start the web server
    let (send_net, rx) = std::sync::mpsc::channel();
    let (tx, recv_net) = std::sync::mpsc::channel();
    let host_interface = game_42_net::protocol::HostInterface::new(rx, tx);
    thread::spawn(move || {
        game_42_net::main(host_interface);
    });
    commands.insert_resource(NetMessages(Mutex::new(recv_net)));
    commands.insert_resource(MessageNet(send_net));
    commands.insert_resource(PlayerInputs(HashMap::new()));
}

fn process_messages(
    receiver: Res<NetMessages>,
    mut player_inputs: ResMut<PlayerInputs>,
    mut commands: Commands
) {
    let mut pi = &mut player_inputs.as_mut().0;
    while let Ok(Ok(msg)) = receiver.0.lock().map(|l| l.try_recv()) {
        match msg.packet {
            Packet::Connected => {
                info!("Player {} connected!", msg.user_id);
                // spawn something here?
                pi.insert(msg.user_id, PlayerInput::new());
            }
            Packet::Disconnected => {
                info!("Player {} disconnected!", msg.user_id);
                // despawn something here?
                match pi.remove(&msg.user_id) {
                    None => {
                        error!("Player {} disconnected but had no controls to begin with.", msg.user_id);
                    },
                    _ => {}
                }
            }
            Packet::Client(packet) => {
                if let Some(entry) = pi.get_mut(&msg.user_id) {
                    if let Input(inp) = packet {
                        match inp {
                            InputUpdate::Button(but, pressed) => {
                                info!("Updated button");
                                entry.update_button(but, pressed);
                            }
                            InputUpdate::Joystick(joy, v) => {
                                info!("Updated joystick");
                                entry.update_joystick(joy, v);
                            }
                        }
                    } else {
                        error!("Unsupported variant of ClientPacket.");
                    }
                } else {
                    error!("Player Input for {} does not exist!", msg.user_id);
                }
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, process_messages)
        .run();
}
