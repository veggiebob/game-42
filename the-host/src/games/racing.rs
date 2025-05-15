use crate::games::Player;
use crate::{PlayerInputs, PlayerMapping};
use bevy::gltf::GltfAssetLabel;
use bevy::math::{Quat, vec3};
use bevy::prelude::{
    AssetServer, Camera3d, Color, Commands, Component, DirectionalLight, Entity, Query, Res,
    SceneRoot, Transform, Vec3, With, default,
};
use game_42_net::controls::ButtonType;
use std::f32::consts::PI;

/// Marks that it belongs to this mini-game, so that it can be
/// despawned later easily.
#[derive(Component)]
pub struct RaceGameMarker;

/// Racing game is a game where each player is a car and they drive
/// it around a track :)
pub fn start_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        RaceGameMarker,
        Camera3d::default(),
        Transform::from_xyz(5., 5., 5.).looking_at(vec3(0., 0., 0.), Vec3::Y),
    ));
    commands.spawn((
        RaceGameMarker,
        DirectionalLight {
            color: Color::WHITE,
            ..default()
        },
    ));

    // spawn a controllable car for player 1
    commands.spawn((
        RaceGameMarker,
        Player(1),
        Transform::from_xyz(0., 0., 0.),
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("gltf/car/car.glb"))),
    ));
    commands.spawn((
        RaceGameMarker,
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("gltf/race-1/race-1.glb"))),
    ));
}

// use player inputs to control car based on the player number
pub fn control_cars(
    cars: Query<(&mut Transform, &Player), With<RaceGameMarker>>,
    player_inputs: Res<PlayerInputs>,
    player_mapping: Res<PlayerMapping>,
) {
    let speed = 0.05;
    let rotation_speed = 0.01;
    for (mut pos, player) in cars {
        // get the player number mapping (first connection is player 1)
        // and then get the input state for that player
        if let Some(pi) = player_mapping
            .0
            .get(&player.0)
            .and_then(|pn| player_inputs.0.get(pn))
        {
            if pi.is_pressed(ButtonType::Up) {
                let v = pos.rotation.mul_vec3(vec3(0., 0., 1.)) * speed;
                pos.translation += v;
            }
            if pi.is_pressed(ButtonType::Down) {
                let v = pos.rotation.mul_vec3(vec3(0., 0., -1.)) * speed;
                pos.translation += v;
            }
            if pi.is_pressed(ButtonType::Right) {
                let rotate_right = Quat::from_rotation_y(-PI / 2.);
                pos.rotation = pos
                    .rotation
                    .rotate_towards(pos.rotation.mul_quat(rotate_right), rotation_speed);
            }

            if pi.is_pressed(ButtonType::Left) {
                let rotate_right = Quat::from_rotation_y(PI / 2.);
                pos.rotation = pos
                    .rotation
                    .rotate_towards(pos.rotation.mul_quat(rotate_right), rotation_speed);
            }
        }
    }
}

/// Despawn all the entities that have a RaceGameMarker
pub fn shutdown_game(mut commands: Commands, objects: Query<Entity, With<RaceGameMarker>>) {
    for entity in objects {
        commands.entity(entity).despawn();
    }
}
