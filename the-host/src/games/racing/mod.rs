mod scene;
mod style;
mod track;

use crate::config::{
    Config, ConfigAccessor, ConfigPath, ConfigPathElem, ConfigType, ConfigValue, ConfigValueMap,
    GetConfig,
};
use crate::debug_input::{DebugPlayer, DebugPlayerInput};
use crate::games::racing::scene::on_scene_load;
use crate::games::racing::style::CarStyle;
use crate::games::racing::track::{LapCounter, Tether, Tragnet, TragnetAnchor};
use crate::games::{ConfigLoadState, CurrentGame, GamePhase, Player};
use crate::{PlayerInputs, PlayerMapping, RandomSource, is_debug_mode};
use avian3d::PhysicsPlugins;
use avian3d::prelude::{
    Collider, Friction, Gravity, LinearVelocity, LockedAxes, MaxLinearSpeed, Physics,
    PhysicsDebugPlugin, Restitution, RigidBody, RigidBodyDisabled,
};
use bevy::app::{App, FixedUpdate, Startup};
use bevy::asset::Handle;
use bevy::color::palettes::css::{ORANGE_RED, WHITE};
use bevy::gltf::{GltfAssetLabel, GltfMaterialName};
use bevy::math::{Quat, ShapeSample, vec3};
use bevy::pbr::{MaterialPlugin, MeshMaterial3d};
use bevy::prelude::{
    AlphaMode, AmbientLight, AppExtStates, AssetServer, Assets, Bundle, Camera3d, Children, Circle,
    Color, Commands, Component, ComputedStates, Condition, DirectionalLight, Entity, Fixed,
    GlobalTransform, Hsla, IntoScheduleConfigs, LinearRgba, Local, Mesh, Mesh3d, Meshable, Name,
    NextState, OnEnter, OnExit, Or, Query, Res, ResMut, Resource, Scene, SceneRoot, Single, Sphere,
    Time, Timer, TimerMode, Transform, TransformHelper, Trigger, Update, Vec3, With, Without,
    default, in_state, info,
};
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use game_42_net::controls::{ButtonType, PlayerInput};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;

// The first few sections before the "GAME" section should probably
// be replicated for other games. I haven't figured out how to abstract
// them yet and I don't care (yet).

// --- SPECIAL CONSTANTS ---
// these are constants that don't really need to be hot reloaded or anything
// because they change very infrequently
const RACE_CHECKPOINTS: usize = 3;
const RACE_LAPS: usize = 1;
const GRAVITY: f32 = 20.0;
const COLLISION_MAT_NAME: &str = "collision";
const TRAGNET_MAT_NAME: &str = "tragnet";
pub const CAR_BODY_MAT_NAME: &str = "body";

// --- CONFIG FILE CONSTANTS ---
const GAME: &str = "racing"; // top-level name of game in config file
const INIT_CARS_PER_TRACK_WIDTH: &str = "init-cars-per-track-width";
const INIT_CAR_FRONT_BACK_SPACING: &str = "init-car-front-back-spacing";
const STARTING_LINE_OFFSET: &str = "starting-line-offset";
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

// --- CONFIG FILE MACROS ---

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

// --- GAME STATE ---

// I will use these computed states instead of the global states
// to manage gameplay correctly.

/// this is analogous to the PreGame state
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
struct PreRacing;

/// This is analogous to the Playing state
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
struct PlayingRacing;

/// This is analogous to the PostGame state
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
struct PostRacing;

impl ComputedStates for PreRacing {
    type SourceStates = (ConfigLoadState, CurrentGame, GamePhase);

    fn compute(sources: Self::SourceStates) -> Option<Self> {
        match sources {
            (ConfigLoadState::Loaded, CurrentGame::Racing, GamePhase::PreGame) => Some(Self),
            _ => None,
        }
    }
}

impl ComputedStates for PlayingRacing {
    type SourceStates = (ConfigLoadState, CurrentGame, GamePhase);

    fn compute(sources: Self::SourceStates) -> Option<Self> {
        match sources {
            (ConfigLoadState::Loaded, CurrentGame::Racing, GamePhase::PlayingGame) => Some(Self),
            _ => None,
        }
    }
}

impl ComputedStates for PostRacing {
    type SourceStates = (ConfigLoadState, CurrentGame, GamePhase);

    fn compute(sources: Self::SourceStates) -> Option<Self> {
        match sources {
            (ConfigLoadState::Loaded, CurrentGame::Racing, GamePhase::PostGame) => Some(Self),
            _ => None,
        }
    }
}

// --- GAME ---

/// goes with the track scene
#[derive(Component)]
struct RacingSceneMarker;

#[derive(Resource, Default)]
struct SceneInfo {
    scene_handle: Handle<Scene>,
    car_handle: Handle<Scene>,
    race_start: Transform,
}

#[derive(Resource)]
struct GameInfo {
    /// Checkpoints per lap. Must be >= 1
    checkpoints: usize,
}

struct EverySecondTimer(Timer);
impl Default for EverySecondTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1., TimerMode::Repeating))
    }
}

fn schedule_1hz(mut timer: Local<EverySecondTimer>, time: Res<Time>) -> bool {
    timer.0.tick(time.delta()).just_finished()
}

pub fn init_app(app: &mut App) {
    app.add_plugins(FlyCameraPlugin)
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(Gravity(Vec3::NEG_Y * GRAVITY))
        .insert_resource(GameInfo {
            checkpoints: RACE_CHECKPOINTS,
        })
        .insert_resource(SceneInfo::default())
        // states
        .add_computed_state::<PreRacing>()
        .add_computed_state::<PlayingRacing>()
        .add_computed_state::<PostRacing>()
        // systems & observers
        .add_systems(OnEnter(PreRacing), start_game)
        .add_observer(on_scene_load)
        .add_systems(Update, everyone_ready.run_if(in_state(PreRacing))) // this actually starts the game
        .add_systems(
            Update,
            arrange_cars_pre_race.run_if(in_state(PreRacing).and(schedule_1hz)),
        )
        .add_systems(
            FixedUpdate,
            (tragnet_players, control_cars, orient_cars).run_if(in_state(PlayingRacing)),
        )
        .add_systems(
            FixedUpdate,
            (print_debug_information, control_debug_car).run_if(is_debug_mode),
        )
        .add_systems(
            Update,
            (despawn_disconnected_players, spawn_new_players)
                .run_if(in_state(PreRacing).and(schedule_1hz)),
        )
        .add_systems(
            Update,
            (step_physics, count_laps).run_if(in_state(PlayingRacing)),
        )
        .add_systems(
            Update,
            someone_finished.run_if(in_state(PlayingRacing).and(schedule_1hz)),
        )
        .add_systems(OnExit(PostRacing), shutdown_game);

    if is_debug_mode() {
        app.add_plugins(PhysicsDebugPlugin::default()); // to be removed
    }
}

/// Marks that it belongs to this mini-game, so that it can be
/// despawned later easily.
#[derive(Component)]
pub struct RaceGameMarker;

/// Racing game is a game where each player is a car, and they drive
/// it around a track :)
/// This doesn't start the game directly, but it is started when
/// the track scene is loaded (see on_scene_load)
fn start_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    configs: Res<Assets<Config>>,
    config_resource: Res<ConfigAccessor>,
    mut scene_info: ResMut<SceneInfo>,
) {
    info!("Starting racing game!");
    let config = configs
        .get(&config_resource.handle)
        .expect("Config does not exist");
    let ground_friction = cfloat![config, GROUND_FRICTION];
    let ground_restitution = cfloat![config, GROUND_RESTITUTION];

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
    commands.spawn((
        RaceGameMarker,
        RacingSceneMarker,
        SceneRoot(scene_handle.clone()),
    ));
    scene_info.scene_handle = scene_handle;

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

    if is_debug_mode() {
        // debug car
        commands.spawn((
            DebugPlayer,
            car_bundle(
                Transform::from_xyz(0., 2., 0.),
                scene_info.as_ref(),
                &configs,
                &config_resource,
                Color::BLACK,
            ),
        ));
    }
}

fn everyone_ready(mut game_phase: ResMut<NextState<GamePhase>>, player_inputs: Res<PlayerInputs>) {
    if !player_inputs.0.is_empty()
        && player_inputs
            .0
            .values()
            .all(|pi| pi.is_pressed(ButtonType::A))
    {
        // start the game!
        info!("Everyone pressed ready button... starting game!");
        game_phase.set(GamePhase::PlayingGame);
    }
}

fn someone_finished(
    mut commands: Commands,
    mut game_phase: ResMut<NextState<GamePhase>>,
    lap_counters: Query<(Entity, &LapCounter, &Player)>,
) {
    if lap_counters.is_empty() {
        info!("All players finished!");
        game_phase.set(GamePhase::PostGame);
    }
    for (entity, lap_counter, player) in lap_counters {
        if lap_counter.lap() >= RACE_LAPS {
            info!("Player {} finished!", player.0);
            commands.entity(entity).despawn();
        }
    }
}

fn arrange_cars_pre_race(
    players: Query<(&mut Transform, &Player)>,
    debug_car: Query<&mut Transform, (With<DebugPlayer>, Without<Player>)>,
    scene_info: Res<SceneInfo>,
    configs: Res<Assets<Config>>,
    config_resource: Res<ConfigAccessor>,
) {
    let config = configs
        .get(&config_resource.handle)
        .expect("Config does not exist");
    let track_radius = cfloat![config, TRACK_RADIUS];
    let cars_per_track = cint![config, INIT_CARS_PER_TRACK_WIDTH];
    let car_size = cfloat![config, CAR_SIZE];
    let front_back_car_spacing = cfloat![config, INIT_CAR_FRONT_BACK_SPACING];
    let starting_line_offset = cfloat![config, STARTING_LINE_OFFSET];
    let mut sorted_players: Vec<_> = players.into_iter().collect();
    sorted_players.sort_by_key(|(_, player)| player.0);
    let right = scene_info.race_start.right();
    let behind = scene_info.race_start.forward();
    let start_transform = scene_info
        .race_start
        .with_translation(scene_info.race_start.translation - right * track_radius);
    for (row, chunk) in sorted_players
        .into_iter()
        .map(|(t, p)| t)
        .chain(debug_car)
        .chunks(cars_per_track as usize)
        .into_iter()
        .enumerate()
    {
        let cars: Vec<_> = chunk.collect();
        let num_cars_in_row = cars.len();
        for (i, mut t) in cars.into_iter().enumerate() {
            let transform = start_transform.with_translation(
                start_transform.translation
                    + ((i as f32 + 0.5) / num_cars_in_row as f32) * track_radius * 2. * right
                    + (row as f32 * front_back_car_spacing * car_size + starting_line_offset)
                        * behind
                    + Vec3::Y * car_size * 0.5,
            );
            t.translation = transform.translation;
            t.rotation = scene_info.race_start.rotation;
        }
    }
}

fn step_physics(mut physics_time: ResMut<Time<Physics>>, fixed_time: Res<Time<Fixed>>) {
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

/// Keep the cars from tipping over and make them move in the direction
/// consistent with their wheels.
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
    let spawned_cars: HashSet<_> = cars.into_iter().map(|p| p.0).collect();
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
                    Color::Hsla(Hsla::hsl((65. * *player as f32) % 360., 1.0, 0.5)),
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
fn shutdown_game(mut commands: Commands, objects: Query<Entity, With<RaceGameMarker>>) {
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
            info!(
                "{name}: lap={}, sector={}, current sector={}",
                counter.lap(),
                counter.sector(),
                current_sector
            );
        }
        info!("-----debug-end-----");
    }
}
