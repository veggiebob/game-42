mod track;
mod scene;
mod style;

use crate::config::{
    Config, ConfigAccessor, ConfigPath, ConfigPathElem, ConfigType, ConfigValue, ConfigValueMap,
    GetConfig,
};
use crate::debug_input::{DebugPlayer, DebugPlayerInput};
use crate::games::Player;
use crate::games::racing::track::{LapCounter, Tether, Tragnet, TragnetAnchor};
use crate::{PlayerInputs, PlayerMapping, RandomSource};
use avian3d::PhysicsPlugins;
use avian3d::math::Quaternion;
use avian3d::prelude::{
    Collider, Friction, Gravity, LinearVelocity, LockedAxes, MaxLinearSpeed, Physics,
    PhysicsDebugPlugin, Restitution, RigidBody, RigidBodyDisabled,
};
use bevy::app::{App, FixedUpdate, Startup};
use bevy::asset::Handle;
use bevy::color::palettes::css::{ORANGE_RED, WHITE};
use bevy::gltf::{GltfAssetLabel, GltfMaterialName};
use bevy::log::warn;
use bevy::math::{Quat, ShapeSample, vec3};
use bevy::pbr::light_consts::lux::AMBIENT_DAYLIGHT;
use bevy::pbr::{MaterialPlugin, MeshMaterial3d};
use bevy::prelude::{AlphaMode, AmbientLight, AssetServer, Assets, Bundle, Camera3d, Children, Circle, Color, Commands, Component, DirectionalLight, Entity, Fixed, GlobalTransform, IntoScheduleConfigs, LinearRgba, Mesh, Mesh3d, Meshable, Name, Or, Query, Res, ResMut, Resource, Scene, SceneRoot, Single, Sphere, Time, Transform, TransformHelper, Trigger, Update, Vec3, With, Without, default, info, Hsla};
use bevy::render::mesh::MeshAabb;
use bevy::scene::SceneInstanceReady;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use game_42_net::controls::{ButtonType, PlayerInput};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;
use bevy::text::cosmic_text::ttf_parser::colr::CompositeMode;
use values_macro_derive::EnumValues;
use crate::games::racing::scene::on_scene_load;
use crate::games::racing::style::CarStyle;

const RACE_CHECKPOINTS: usize = 3;

const GAME: &str = "racing";

const COLLISION_MAT_NAME: &str = "collision";
const TRAGNET_MAT_NAME: &str = "tragnet";
const CAR_SIZE: &str = "car-size";
const CAR_RESTITUTION: &str = "car-restitution";
const CAR_FRICTION: &str = "car-friction";
const GROUND_FRICTION: &str = "ground-friction";
const GROUND_RESTITUTION: &str = "ground-restitution";
const MAX_SPEED: &str = "max-speed";
const ACC_SPEED: &str = "acc-speed";
const TURN_SPEED: &str = "turn-speed";
const TRACK_RADIUS: &str = "track-radius";
const TRAGNET_STRENGTH: &str = "tragnet-strength";
const TRAGNET_STRENGTH_EXP: &str = "tragnet-strength-exp";
const TRAGNET_K: &str = "tragnet-k";

macro_rules! cfloat {
    ($config:expr, $i:expr) => {
        $config[GAME][$i].as_f64().expect(
            format!(
                "Config value {} does not exist or isn't floating point.",
                $i
            )
            .as_str(),
        ) as f32
    };
}

macro_rules! cint {
    ($config:expr, $i:expr) => {
        $config[GAME][$i]
            .as_i64()
            .expect(format!("Config value {} does not exist or isn't an i64.", $i).as_str())
            as i32
    };
}

macro_rules! cusize {
    ($config:expr, $i:expr) => {
        $config[GAME][$i]
            .as_u64()
            .expect(format!("Config value {} does not exist or isn't a u64.", $i).as_str())
            as usize
    };
}

#[derive(Component)]
struct DistanceSensitiveStaticCollider {
    distance: f32,
}

#[derive(Resource)]
struct Started(bool);
fn has_started(started: Res<Started>) -> bool {
    started.0
}
fn not_started(started: Res<Started>) -> bool {
    !started.0
}

#[derive(Resource, Default)]
struct SceneInfo {
    handle: Handle<Scene>,
    car_handle: Handle<Scene>,
    race_start: Transform,
}

#[derive(Resource)]
struct GameStateInfo {
    /// Checkpoints per lap. Must be >= 1
    checkpoints: usize,
}

pub fn init_app(app: &mut App) {
    app.add_plugins(FlyCameraPlugin)
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(Gravity(Vec3::NEG_Y * 20.0))
        .insert_resource(GameStateInfo { checkpoints: 3 })
        .insert_resource(Started(false))
        .insert_resource(SceneInfo::default())
        .add_systems(Update, start_game.run_if(not_started))
        .add_systems(
            FixedUpdate,
            (
                tragnet_players,
                spawn_new_players,
                despawn_disconnected_players,
                control_cars,
                orient_cars,
            )
                .run_if(has_started),
        )
        .add_systems(Update, (step, print_debug_information, count_laps).run_if(has_started))
        .add_observer(on_scene_load)
        // debug
        .add_systems(FixedUpdate, control_debug_car.run_if(has_started))
        .add_plugins(PhysicsDebugPlugin::default());
}

/// Marks that it belongs to this mini-game, so that it can be
/// despawned later easily.
#[derive(Component)]
pub struct RaceGameMarker;

/// Racing game is a game where each player is a car, and they drive
/// it around a track :)
pub fn start_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    configs: Res<Assets<Config>>,
    config_resource: Res<ConfigAccessor>,
    mut started: ResMut<Started>,
    mut scene_info: ResMut<SceneInfo>,
) {
    if started.0 {
        return;
    }
    let config = configs.get(&config_resource.handle);
    if config.is_none() {
        return;
    }
    info!("Starting racing game!");
    started.0 = true;
    let config = config.unwrap();
    let car_friction = cfloat![config, CAR_FRICTION];
    let car_size = cfloat![config, CAR_SIZE];
    let car_restitution = cfloat![config, CAR_RESTITUTION];
    let ground_friction = cfloat![config, GROUND_FRICTION];
    let ground_restitution = cfloat![config, GROUND_RESTITUTION];
    // ambient lights do nothing??
    // commands.spawn((
    //     RaceGameMarker,
    //     AmbientLight {
    //         brightness: AMBIENT_DAYLIGHT,
    //         affects_lightmapped_meshes: true,
    //         color: WHITE.into()
    //     }
    // ));

    // debug camera
    commands.spawn((
        RaceGameMarker,
        Camera3d::default(),
        FlyCamera::default(),
        Transform::from_xyz(0., 3., 0.),
    ));

    // sun as a light
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

    // spawn the actual scene
    let scene_handle =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("gltf/race-1/race-1.glb"));
    commands.spawn((RaceGameMarker, SceneRoot(scene_handle.clone())));
    scene_info.handle = scene_handle;

    // ground collider
    commands.spawn((
        RigidBody::Static,
        Friction::new(ground_friction),
        Restitution::new(ground_restitution),
        Collider::cuboid(15., 0.5, 15.),
        Transform::from_xyz(0., -0.5, 0.),
    ));

    let car_handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset("gltf/car/car.glb"));
    scene_info.as_mut().car_handle = car_handle.clone();
    // debug car
    commands.spawn((
        DebugPlayer,
        car_bundle(
            Transform::from_xyz(0., 2., 0.),
            scene_info.as_ref(),
            &configs,
            &config_resource,
            Color::BLACK
        ),
    ));
}

fn update_colliders(
    mut commands: Commands,
    cars: Query<&GlobalTransform, With<Player>>,
    debug_car: Single<&GlobalTransform, With<DebugPlayer>>,
    inactive: Query<
        (Entity, &GlobalTransform, &DistanceSensitiveStaticCollider),
        With<RigidBodyDisabled>,
    >,
    active: Query<
        (Entity, &GlobalTransform, &DistanceSensitiveStaticCollider),
        Without<RigidBodyDisabled>,
    >,
) {
    let mut pts = cars
        .into_iter()
        .map(|t| t.translation())
        .collect::<Vec<_>>();
    pts.push(debug_car.translation());
    for (ent, pos, mut dist) in inactive {
        for t in pts.iter() {
            if t.distance(pos.translation()) < dist.distance {
                commands.entity(ent).remove::<RigidBodyDisabled>();
                break;
            }
        }
    }
    'colliders: for (ent, pos, mut dist) in active {
        for t in pts.iter() {
            if t.distance(pos.translation()) < dist.distance {
                // keep it active
                continue 'colliders;
            }
        }
        // now inactive
        commands.entity(ent).insert(RigidBodyDisabled);
    }
}
fn step(mut physics_time: ResMut<Time<Physics>>, fixed_time: Res<Time<Fixed>>) {
    physics_time.advance_by(fixed_time.delta());
}

// keep cars from going crazy

fn tragnet_players(
    tragnet: Single<&Tragnet>,
    cars: Query<
        (&GlobalTransform, &mut LinearVelocity, &mut Tether),
        (With<RaceGameMarker>, With<Player>),
    >,
    debug_car: Query<
        (&GlobalTransform, &mut LinearVelocity, &mut Tether),
        (With<RaceGameMarker>, With<DebugPlayer>, Without<Player>),
    >,
    configs: Res<Assets<Config>>,
    config_resource: Res<ConfigAccessor>,
) {
    let config = configs.get(&config_resource.handle).expect("no config!");
    let track_radius = cfloat![config, TRACK_RADIUS];
    let tragnet_strength = cfloat![config, TRAGNET_STRENGTH];
    let tragnet_exp = cfloat![config, TRAGNET_STRENGTH_EXP];
    let tragnet_k = cusize![config, TRAGNET_K];
    let all_cars = cars.into_iter().chain(debug_car);
    for (i, (transform, mut lv, mut tether)) in all_cars.enumerate() {
        tragnet.update_tether(tether.as_mut(), transform.translation(), tragnet_k);
        let target = tragnet.get_tether_transform(tether.as_ref());
        let to_target = target.translation - transform.translation();
        let to_target_dist = to_target.length();
        let strength = f32::max((to_target_dist - track_radius).signum(), 0.0);
        let tragnet_pull =
            to_target.normalize() * f32::powf(strength, tragnet_exp) * tragnet_strength;
        if strength > 0.0 {
            lv.0 *= tragnet_strength;
        }
        lv.0 += tragnet_pull;
    }
}

fn orient_cars(
    cars: Query<
        (&mut Transform, &mut LinearVelocity),
        (With<RaceGameMarker>, Without<DebugPlayer>, With<Player>),
    >,
    debug_car: Query<
        (&mut Transform, &mut LinearVelocity),
        (With<RaceGameMarker>, With<DebugPlayer>, Without<Player>),
    >,
) {
    let all_cars = cars.into_iter().chain(debug_car);
    for (mut transform, mut lv) in all_cars {
        // rotate up y, and see how far it is from actual upright
        let car_up = transform.rotation.mul_vec3(Vec3::Y);
        let upright_angle = car_up.angle_between(Vec3::Y);
        if upright_angle > PI / 4. {
            let rotation = Quat::from_rotation_arc(car_up, Vec3::Y);
            let correction = Quat::slerp(Quat::IDENTITY, rotation, 0.1);
            transform.rotation = correction * transform.rotation;
        }

        // make it actually go in the direction it's travelling
        let speed = lv.0.dot(transform.forward().as_vec3());
        lv.0 = speed * transform.forward().as_vec3();
    }
}

fn car_bundle(
    transform: Transform,
    scene_info: &SceneInfo,
    configs: &Res<Assets<Config>>,
    config_resource: &Res<ConfigAccessor>,
    color: Color,
) -> impl Bundle {
    let config = configs.get(&config_resource.handle).expect("no config!");
    let car_friction = cfloat![config, CAR_FRICTION];
    let car_size = cfloat![config, CAR_SIZE];
    let car_restitution = cfloat![config, CAR_RESTITUTION];
    (
        RaceGameMarker,
        RigidBody::Dynamic,
        Friction::new(car_friction),
        Restitution::new(car_restitution),
        Collider::cuboid(0.5, 0.5, 1.),
        transform.with_scale(Vec3::splat(car_size)),
        SceneRoot(scene_info.car_handle.clone()),
        Tether::Lost,
        LapCounter::at_start(RACE_CHECKPOINTS),
        CarStyle::new(color),
    )
}
fn spawn_new_players(
    mut commands: Commands,
    mut random_source: ResMut<RandomSource>,
    cars: Query<&Player, With<RaceGameMarker>>,
    player_mapping: Res<PlayerMapping>,
    configs: Res<Assets<Config>>,
    config_resource: Res<ConfigAccessor>,
    scene_info: Res<SceneInfo>,
) {
    let config = configs.get(&config_resource.handle).expect("no config!");
    let track_radius = cfloat![config, TRACK_RADIUS];
    let spawn_area = Circle::new(track_radius);
    let spawned_cars: HashSet<_> = cars.into_iter()
        .map(|p| p.0).collect();
    for player in player_mapping.0.keys() {
        if !spawned_cars.contains(player) {
            let pos = spawn_area.sample_interior(&mut random_source.0);
            let mut spawn_transform = scene_info.race_start;
            spawn_transform.translation += vec3(pos.x, 0.0, pos.y);
            commands.spawn((
                Player(*player),
                car_bundle(
                    spawn_transform,
                    scene_info.as_ref(),
                    &configs,
                    &config_resource,
                    Color::Hsla(Hsla::hsl((65. * *player as f32) % 360., 1.0, 0.5))
                ),
            ));
        }
    }
}

fn despawn_disconnected_players(
    mut commands: Commands,
    cars: Query<(Entity, &Player), With<RaceGameMarker>>,
    player_mapping: Res<PlayerMapping>,
) {
    for (entity, player) in cars {
        if !player_mapping.0.contains_key(&player.0) {
            commands.entity(entity).despawn();
        }
    }
}

struct ControlOutput {
    acceleration: f32,
    turn: f32,
}
fn get_control_acc(pi: &PlayerInput, acc_speed: f32, turn_speed: f32) -> ControlOutput {
    let mut co = ControlOutput {
        acceleration: 0.,
        turn: 0.,
    };
    if pi.is_pressed(ButtonType::Up) {
        co.acceleration += acc_speed;
    }
    if pi.is_pressed(ButtonType::Down) {
        co.acceleration -= acc_speed;
    }
    if pi.is_pressed(ButtonType::Right) {
        co.turn -= turn_speed;
    }
    if pi.is_pressed(ButtonType::Left) {
        co.turn += turn_speed;
    }
    co
}

// use player inputs to control car based on the player number
pub fn control_cars(
    cars: Query<(&mut Transform, &Player, &mut LinearVelocity), With<RaceGameMarker>>,
    configs: Res<Assets<Config>>,
    config_resource: Res<ConfigAccessor>,
    player_inputs: Res<PlayerInputs>,
    player_mapping: Res<PlayerMapping>,
) {
    let config = configs.get(&config_resource.handle).expect("no config!");
    let acc_speed = config[GAME][ACC_SPEED].as_f64().expect("beep beep") as f32;
    let turn_speed = cfloat![config, TURN_SPEED];
    for (mut pos, player, mut linear_velocity) in cars {
        // get the player number mapping (first connection is player 1)
        // and then get the input state for that player
        if let Some(pi) = player_mapping
            .0
            .get(&player.0)
            .and_then(|pn| player_inputs.0.get(pn))
        {
            let co = get_control_acc(pi, acc_speed, turn_speed);
            let rotate_right = Quat::from_rotation_y(PI / 2.);
            let forward = pos.rotation.mul_vec3(Vec3::Z);
            linear_velocity.0 += forward * co.acceleration;
            pos.rotation = pos
                .rotation
                .rotate_towards(pos.rotation.mul_quat(rotate_right), co.turn);
            // linear_velocity.0 = rotate_right.mul_vec3(linear_velocity.0);
        }
    }
}

fn control_debug_car(
    mut debug_car: Single<(&mut Transform, &mut LinearVelocity), With<DebugPlayer>>,
    dpi: Res<DebugPlayerInput>,
    configs: Res<Assets<Config>>,
    config_resource: Res<ConfigAccessor>,
) {
    let config = configs.get(&config_resource.handle).expect("no config!");
    let acc_speed = cfloat![config, ACC_SPEED];
    let turn_speed = cfloat![config, TURN_SPEED];
    let (mut pos, mut linear_velocity) = debug_car.into_inner();
    let co = get_control_acc(&dpi.player_input, acc_speed, turn_speed);
    let rotate_right = Quat::from_rotation_y(PI / 2.);
    let forward = pos.rotation.mul_vec3(Vec3::Z);
    linear_velocity.0 += forward * co.acceleration;
    pos.rotation = pos
        .rotation
        .rotate_towards(pos.rotation.mul_quat(rotate_right), co.turn);
}

fn count_laps(tethered_things: Query<(&Tether, &mut LapCounter)>, tragnet: Single<&Tragnet>) {
    for (tether, mut counter) in tethered_things {
        let current_sector = tragnet.get_current_sector(tether);
        counter.update_sector(current_sector);
    }
}

/// Despawn all the entities that have a RaceGameMarker
pub fn shutdown_game(mut commands: Commands, objects: Query<Entity, With<RaceGameMarker>>) {
    for entity in objects {
        commands.entity(entity).despawn();
    }
}

fn print_debug_information(
    mut debug_player_input: ResMut<DebugPlayerInput>,
    lap_things: Query<(Option<&Name>, &Tether, &LapCounter)>,
    tragnet: Single<&Tragnet>,
) {
    if debug_player_input.button_1.just_pressed() {
        info!("-----debug-start-----");
        for (name, tether, counter) in lap_things.iter() {
            let name = name.map(|s| s.as_str()).unwrap_or("unnamed");
            let current_sector = tragnet.get_current_sector(tether);
            info!("{name}: lap={}, sector={}, current sector={}", counter.lap(), counter.sector(), current_sector);
        }
        info!("-----debug-end-----");
    }
}
