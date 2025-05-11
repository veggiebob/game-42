mod controls;

use bevy::prelude::*;
use net::{spawn as spawn_net, Input as NetInput, ServerEvent};
use tokio::sync::mpsc::UnboundedReceiver;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_event::<PlayerInput>()
        .add_event::<PlayerJoined>()
        .add_event::<PlayerLeft>()
        .add_systems(Startup, start_net_server)
        .add_systems(Update, poll_network)
        .add_systems(Update, handle_player_input)
        .run();
}

/* --------------- Resources & events --------------- */

#[derive(Resource)]
struct NetRx(UnboundedReceiver<ServerEvent>);

#[derive(Event)]
struct PlayerInput { id: u64, inp: NetInput }

#[derive(Event)] struct PlayerJoined(u64);
#[derive(Event)] struct PlayerLeft(u64);

/* --------------- Systems --------------- */

fn start_net_server(mut commands: Commands) {
    // listen on localhost:4000
    let rx = spawn_net(([0, 0, 0, 0], 4000).into());
    commands.insert_resource(NetRx(rx));
    println!("server spawned!")
}

fn poll_network(
    mut net_rx: ResMut<NetRx>,
    mut ev_inp: EventWriter<PlayerInput>,
    mut ev_join: EventWriter<PlayerJoined>,
    mut ev_left: EventWriter<PlayerLeft>,
) {
    while let Ok(ev) = net_rx.0.try_recv() {
        match ev {
            ServerEvent::Connected    { id }       => { let _ = ev_join.send(PlayerJoined(id)); }
            ServerEvent::Disconnected { id }       => { let _ = ev_left.send(PlayerLeft(id));  }
            ServerEvent::Input        { id, inp }  => { let _ = ev_inp.send(PlayerInput { id, inp }); }
        }
    }
}

fn handle_player_input(mut reader: EventReader<PlayerInput>) {
    for PlayerInput { id, inp } in reader.read() {
        info!("{id} → {inp:?}");
        println!("input!");
    }
}
