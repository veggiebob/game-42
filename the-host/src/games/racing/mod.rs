pub mod materials;

use crate::games::Player;
use crate::games::racing::materials::race_rails::RailsMaterial;
use crate::games::racing::materials::{MaterialOverride, MaterialOverrides};
use crate::{PlayerInputs, PlayerMapping, RandomSource};
use avian3d::PhysicsPlugins;
use avian3d::prelude::{Collider, Physics, RigidBody};
use bevy::app::{App, FixedUpdate, Startup};
use bevy::color::palettes::css::{ORANGE_RED, WHITE};
use bevy::gltf::{GltfAssetLabel, GltfMaterialName};
use bevy::math::{Quat, ShapeSample, vec3};
use bevy::pbr::light_consts::lux::AMBIENT_DAYLIGHT;
use bevy::pbr::{MaterialPlugin, MeshMaterial3d};
use bevy::prelude::{
    AlphaMode, AmbientLight, AssetServer, Assets, Camera3d, Children, Circle, Color, Commands,
    Component, DirectionalLight, Entity, Fixed, IntoScheduleConfigs, LinearRgba, Mesh, Mesh3d,
    Meshable, Name, Query, Res, ResMut, SceneRoot, Single, Sphere, Time, Transform, Trigger,
    Update, Vec3, With, default, info,
};
use bevy::scene::SceneInstanceReady;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use game_42_net::controls::ButtonType;
use materials::racetrack::RacetrackMaterial;
use std::collections::HashSet;
use std::f32::consts::PI;

const COLLISION_MAT_NAME: &str = "collision";

pub fn init_app(app: &mut App) {
    app.add_plugins(MaterialPlugin::<RacetrackMaterial>::default())
        .add_plugins(MaterialPlugin::<RailsMaterial>::default())
        .add_plugins(FlyCameraPlugin)
        .add_plugins(PhysicsPlugins::default())
        .add_systems(Startup, start_game.after(crate::setup))
        .add_systems(FixedUpdate, (control_cars, spawn_new_players))
        .add_systems(Update, step)
        .add_observer(on_scene_load);
}

/// Marks that it belongs to this mini-game, so that it can be
/// despawned later easily.
#[derive(Component)]
pub struct RaceGameMarker;

/// Racing game is a game where each player is a car, and they drive
/// it around a track :)
pub fn start_game(mut commands: Commands, asset_server: Res<AssetServer>) {
    // ambient lights do nothing??
    // commands.spawn((
    //     RaceGameMarker,
    //     AmbientLight {
    //         brightness: AMBIENT_DAYLIGHT,
    //         affects_lightmapped_meshes: true,
    //         color: WHITE.into()
    //     }
    // ));

    commands.spawn((
        RaceGameMarker,
        Camera3d::default(),
        FlyCamera::default(),
        Transform::from_xyz(0., 5., 0.).looking_at(vec3(1., 0., 0.), Vec3::Y),
    ));
    commands.spawn((
        RaceGameMarker,
        DirectionalLight {
            color: Color::WHITE,
            shadows_enabled: true,
            affects_lightmapped_mesh_diffuse: true,
            ..default()
        },
        Transform::from_xyz(0., 5., 0.).looking_at(vec3(-2., -2., 0.), vec3(0., 1., 0.)),
    ));

    commands.spawn((
        RaceGameMarker,
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("gltf/race-1/race-1.glb"))),
    ));

    commands.spawn((
        RigidBody::Static,
        Collider::cuboid(5., 0.5, 5.),
        Transform::from_xyz(0., -0.5, 0.),
    ));

    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(1., 1., 1.),
        Transform::from_xyz(0., 5., 0.),
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("gltf/car/car.glb"))),
    ));
}

fn step(mut physics_time: ResMut<Time<Physics>>, fixed_time: Res<Time<Fixed>>) {
    physics_time.advance_by(fixed_time.delta());
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
            Transform::from_xyz(pos.x, 5.0, pos.y).with_scale(vec3(0.2, 0.2, 0.2)),
            RigidBody::Dynamic,
            Collider::cuboid(1., 1., 1.),
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

fn on_scene_load(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
    gltf_children: Query<(&GltfMaterialName, &Transform)>,
    children: Query<&Children>,
) {
    info!("Scene Instance Ready: {:?}", trigger.target());
    for descendant in children
        .iter_descendants(trigger.target())
        .collect::<Vec<_>>()
        .into_iter()
    {
        if let Ok((gltf_name, transform)) = gltf_children.get(descendant) {
            // add collider to it
            if gltf_name.0 == COLLISION_MAT_NAME {
                commands
                    .entity(descendant)
                    .insert((RigidBody::Static, Collider::cuboid(5., 0.5, 5.)));
                info!("inserted collider!")
            }
        }
    }
}
