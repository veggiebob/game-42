
// not really a game, just a waiting screen where players can do a funny little thing

use std::collections::HashSet;
use bevy::prelude::{Commands, Query, Res};
use crate::games::Player;
use crate::PlayerMapping;

fn spawn_new_players(
    mut commands: Commands,
    player_mapping: Res<PlayerMapping>,
    current_players: Query<(&Player)>,
) {
    let connected_players = player_mapping.0.keys();
    let current_players: HashSet<_> = current_players.iter().map(|p| p.0).collect();
    for connected_player in connected_players {
        if !current_players.contains(connected_player) {
            commands.spawn((

                ));
        }
    }
}