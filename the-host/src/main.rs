mod assets;
mod debug_input;
pub mod games;
mod config;

use std::collections::hash_map::Keys;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, WindowResized};
use game_42_net::controls::{InputUpdate, PlayerInput};
use game_42_net::protocol::ClientPacket::Input;
use game_42_net::protocol::{AnnotatedClientPacket, Packet, UserId};
use games::racing;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use bevy::remote::http::RemoteHttpPlugin;
use bevy::remote::RemotePlugin;
use rand_chacha::rand_core::SeedableRng;
use crate::config::{Config, ConfigAccessor, ConfigAssetLoaderError, ConfigLoader};

#[derive(Resource)]
pub(crate) struct RandomSource(rand_chacha::ChaCha8Rng);

/// Use this to message the net (probably a client?)
#[derive(Resource)]
pub struct MessageNet(pub Sender<()>);

#[derive(Resource)]
pub struct NetMessages(pub Mutex<Receiver<AnnotatedClientPacket>>);

#[derive(Resource)]
pub struct PlayerInputs(pub HashMap<UserId, PlayerInput>);

// Map UserIds (connections) to player numbers (1, 2, 3, ...)
// Not necessary to use this interface; see example at games::racing::control_cars
type PlayerNum = u8;
#[derive(Resource)]
pub struct PlayerMapping(pub HashMap<PlayerNum, UserId>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Setting up The Host");
    // start the web server
    let (send_net, rx) = std::sync::mpsc::channel();
    let (tx, recv_net) = std::sync::mpsc::channel();
    let host_interface = game_42_net::protocol::HostInterface::new(rx, tx);
    thread::spawn(move || {
        game_42_net::main(host_interface);
    });
    
    // communications
    commands.insert_resource(NetMessages(Mutex::new(recv_net)));
    commands.insert_resource(MessageNet(send_net));
    commands.insert_resource(PlayerInputs(HashMap::new()));
    commands.insert_resource(PlayerMapping(HashMap::new()));
    
    // https://bevyengine.org/examples/math/random-sampling/
    let seeded_rng = rand_chacha::ChaCha8Rng::seed_from_u64(1029301923);
    // let seeded_rng = ChaCha8Rng::from_os_rng();
    commands.insert_resource(RandomSource(seeded_rng));
    
    commands.insert_resource(ConfigAccessor {
        handle: asset_server.load("config.json")
    });
}

// handle player connections
fn process_messages(
    receiver: Res<NetMessages>,
    mut pm: ResMut<PlayerMapping>,
    mut player_inputs: ResMut<PlayerInputs>,
    mut commands: Commands,
) {
    let mut pi = &mut player_inputs.as_mut().0;
    let mut pm = pm.as_mut();
    while let Ok(Ok(msg)) = receiver.0.lock().map(|l| l.try_recv()) {
        match msg.packet {
            Packet::Connected => {
                let player_number = pm.connect_lowest_num(&msg.user_id);
                info!(
                    "Player {} connected as player {}!",
                    msg.user_id, player_number
                );
                // spawn something here?
                pi.insert(msg.user_id, PlayerInput::new());
            }
            Packet::Disconnected => {
                let disconnected = pm.remove(&msg.user_id);
                info!(
                    "Player {} disconnected from player {}!",
                    msg.user_id,
                    disconnected.unwrap()
                );
                // despawn something here?
                if let None = pi.remove(&msg.user_id) {
                    error!(
                        "Player {} disconnected but had no controls to begin with.",
                        msg.user_id
                    );
                }
            }
            Packet::Client(packet) => {
                if let Some(entry) = pi.get_mut(&msg.user_id) {
                    if let Input(inp) = packet {
                        match inp {
                            InputUpdate::Button(but, pressed) => {
                                // info!("Updated button");
                                entry.update_button(but, pressed);
                            }
                            InputUpdate::Joystick(joy, v) => {
                                // info!("Updated joystick");
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

// don't use rn, maybe later
// fn keep_aspect_ratio(
//     mut window: Single<&mut Window>,
//     mut resize_reader: EventReader<WindowResized>
// ) {
//     static ASPECT_RATIO: f32 = 7. / 6.; // height / width
//     if let Some(res) = resize_reader.read().last() {
//         let fwidth = res.height / ASPECT_RATIO;
//         let fheight = fwidth * ASPECT_RATIO;
//         window.resolution.set(fwidth, fwidth);
//         println!("Resized window to {fwidth}x{fheight}")
//     }
// }

fn grab_mouse(
    mut window: Query<&mut Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if let Ok(mut window) = window.single_mut() {
        if mouse.just_pressed(MouseButton::Left) {
            window.cursor_options.visible = false;
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
        }
        if keys.just_pressed(KeyCode::Escape) {
            window.cursor_options.visible = true;
            window.cursor_options.grab_mode = CursorGrabMode::None;
        }
    }
}

pub fn is_debug_mode() -> bool {
    false
}

fn main() {
    let mut app = App::new();
    app
        .add_plugins(RemotePlugin::default())
        .add_plugins(RemoteHttpPlugin::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (1080., 1260.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(First, grab_mouse)
        .add_systems(Update, process_messages)
        .init_asset::<Config>()
        .init_asset_loader::<ConfigLoader>()
        ;
    games::init_games(&mut app);
    debug_input::init(&mut app);
    app.run();
}

impl PlayerMapping {
    pub fn connect_lowest_num(&mut self, user_id: &UserId) -> PlayerNum {
        let mut min = 1;
        while self.0.contains_key(&min) {
            min += 1;
        }
        self.0.insert(min, *user_id);
        min
    }
    
    pub fn remove(&mut self, user_id: &UserId) -> Option<PlayerNum> {
        if let Some((&player_num, _uid)) = self.0.iter().find(|(k, v)| v == &user_id) {
            self.0.remove(&player_num);
            Some(player_num)
        } else {
            None
        }
    }
    
    pub fn get_players(&self) -> Keys<PlayerNum, UserId> {
        self.0.keys()
    }
}
