pub mod materials;

use crate::games::Player;
use crate::games::racing::materials::race_rails::RailsMaterial;
use crate::games::racing::materials::{MaterialOverride, MaterialOverrides, material_override};
use crate::{PlayerInputs, PlayerMapping, RandomSource};
use bevy::app::{App, FixedUpdate, Startup};
use bevy::gltf::GltfAssetLabel;
use bevy::math::{Quat, ShapeSample, vec3};
use bevy::pbr::{MaterialPlugin, MeshMaterial3d};
use bevy::prelude::{AssetServer, Assets, Camera3d, Circle, Color, Commands, Component, DirectionalLight, Entity, IntoScheduleConfigs, LinearRgba, Query, Res, ResMut, SceneRoot, Transform, Vec3, With, default, AlphaMode, info, Mesh, Sphere, Meshable, Mesh3d};
use game_42_net::controls::ButtonType;
use materials::racetrack::RacetrackMaterial;
use std::collections::HashSet;
use std::f32::consts::PI;

pub fn init_app(app: &mut App) {
    app
        .add_plugins(MaterialPlugin::<RacetrackMaterial>::default())
        .add_plugins(MaterialPlugin::<RailsMaterial>::default())
        .add_systems(Startup, start_game.after(crate::setup))
        .add_systems(FixedUpdate, (control_cars, spawn_new_players))
        // .add_observer(material_override)
    ;
}

/// Marks that it belongs to this mini-game, so that it can be
/// despawned later easily.
#[derive(Component)]
pub struct RaceGameMarker;

/// Racing game is a game where each player is a car, and they drive
/// it around a track :)
pub fn start_game(
    mut commands: Commands,
    mut racetrack_material: ResMut<Assets<RacetrackMaterial>>,
    mut rails_material: ResMut<Assets<RailsMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
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
    let rails = rails_material.add(RailsMaterial {
        color: LinearRgba::rgb(1., 1., 1.),
        color_texture: None,
        alpha_mode: Default::default(),
    });
    let racetrack = racetrack_material.add(RacetrackMaterial {
        color: LinearRgba::rgb(1., 1., 1.),
        color_texture: Some(asset_server.load("textures/asphalt2.png")),
        alpha_mode: AlphaMode::Opaque,
    });
    let material_overrides = MaterialOverrides::new(
        vec![
            MaterialOverride::Racetrack(racetrack.clone()),
            MaterialOverride::Rails(racetrack.clone()),
        ]
            .into_iter(),
    );
    info!("material overrides: {:#?}", material_overrides);
    commands.spawn((
        RaceGameMarker,
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("gltf/race-1/race-1.glb"))),
        material_overrides
    ));
    
    
    // commands.spawn((
    //     Mesh3d(meshes.add(Sphere::default().mesh().uv(32, 18))),
    //     MeshMaterial3d(racetrack.clone()),
    //     Transform::from_xyz(2., 0., 0.)
    // ));
    // commands.spawn((
    //     Mesh3d(meshes.add(Sphere::default().mesh().uv(32, 18))),
    //     MeshMaterial3d(rails.clone()),
    //     Transform::from_xyz(-2., 0., 0.)
    // ));
}

pub fn spawn_new_players(
    mut commands: Commands,
    mut random_source: ResMut<RandomSource>,
    asset_server: Res<AssetServer>,
    cars: Query<&Player, With<RaceGameMarker>>,
    player_mapping: Res<PlayerMapping>,
) {
    let car_players: HashSet<_> = cars.into_iter().map(|x| x.0).collect();
    let players_without_cars = player_mapping
        .get_players()
        .filter(|p| !car_players.contains(p));
    let spawn_area = Circle::new(3.);
    for player in players_without_cars {
        let pos = spawn_area.sample_interior(&mut random_source.0);
        commands.spawn((
            RaceGameMarker,
            Player(*player),
            Transform::from_xyz(pos.x, 0.0, pos.y),
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("gltf/car/car.glb"))),
        ));
    }
}

// use player inputs to control car based on the player number
pub fn control_cars(
    cars: Query<(&mut Transform, &Player), With<RaceGameMarker>>,
    player_inputs: Res<PlayerInputs>,
    player_mapping: Res<PlayerMapping>,
) {
    let speed = 0.05;
    let rotation_speed = 0.05;
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
