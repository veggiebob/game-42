pub mod materials;

use crate::config::{
    Config, ConfigAccessor, ConfigPath, ConfigPathElem, ConfigType, ConfigValue, ConfigValueMap,
    GetConfig,
};
use crate::debug_input::{DebugPlayer, DebugPlayerInput};
use crate::games::Player;
use crate::games::racing::materials::race_rails::RailsMaterial;
use crate::games::racing::materials::{MaterialOverride, MaterialOverrides};
use crate::{PlayerInputs, PlayerMapping, RandomSource};
use avian3d::PhysicsPlugins;
use avian3d::prelude::{
    Collider, Friction, Gravity, LinearVelocity, LockedAxes, MaxLinearSpeed, Physics,
    PhysicsDebugPlugin, Restitution, RigidBody, RigidBodyDisabled,
};
use bevy::app::{App, FixedUpdate, Startup};
use bevy::color::palettes::css::{ORANGE_RED, WHITE};
use bevy::gltf::{GltfAssetLabel, GltfMaterialName};
use bevy::log::warn;
use bevy::math::{Quat, ShapeSample, vec3};
use bevy::pbr::light_consts::lux::AMBIENT_DAYLIGHT;
use bevy::pbr::{MaterialPlugin, MeshMaterial3d};
use bevy::prelude::{
    AlphaMode, AmbientLight, AssetServer, Assets, Camera3d, Children, Circle, Color, Commands,
    Component, DirectionalLight, Entity, Fixed, GlobalTransform, IntoScheduleConfigs, LinearRgba,
    Mesh, Mesh3d, Meshable, Name, Or, Query, Res, ResMut, Resource, SceneRoot, Single, Sphere,
    Time, Transform, Trigger, Update, Vec3, With, Without, default, info,
};
use bevy::scene::SceneInstanceReady;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};
use game_42_net::controls::{ButtonType, PlayerInput};
use materials::racetrack::RacetrackMaterial;
use std::collections::HashSet;
use std::f32::consts::PI;
use values_macro_derive::EnumValues;

const GAME: &str = "racing";

const COLLISION_MAT_NAME: &str = "collision";
const CAR_SIZE: &str = "car-size";
const CAR_RESTITUTION: &str = "car-restitution";
const CAR_FRICTION: &str = "car-friction";
const GROUND_FRICTION: &str = "ground-friction";
const GROUND_RESTITUTION: &str = "ground-restitution";
const MAX_SPEED: &str = "max-speed";
const ACC_SPEED: &str = "acc-speed";
const TURN_SPEED: &str = "turn-speed";

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

pub fn init_app(app: &mut App) {
    app.add_plugins(MaterialPlugin::<RacetrackMaterial>::default())
        .add_plugins(MaterialPlugin::<RailsMaterial>::default())
        .add_plugins(FlyCameraPlugin)
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(Gravity(Vec3::NEG_Y * 20.0))
        .insert_resource(Started(false))
        .add_systems(Update, start_game.run_if(not_started))
        .add_systems(
            FixedUpdate,
            (control_cars, spawn_new_players).run_if(has_started),
        )
        .add_systems(Update, step.run_if(has_started))
        // .add_observer(on_scene_load)
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
    // let config = asset_server.load("config/racing.json");
    // let make_cv = |key, et| ConfigValue::new(ConfigPath::key(key, et), config.clone());
    // let config_map = RacingConfig::values()
    //     .map(|cfg| (cfg, make_cv(cfg.get_key(), cfg.get_expected_type())))
    //     .collect();
    // commands.spawn((RaceGameMarker, ConfigValueMap(config_map)));

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

    // commands.spawn((
    //     RaceGameMarker,
    //     SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("gltf/race-1/race-1.glb"))),
    // ));
    commands.spawn((
        RaceGameMarker,
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("gltf/test.glb"))),
    ));

    commands.spawn((
        RigidBody::Static,
        Friction::new(ground_friction),
        Restitution::new(ground_restitution),
        Collider::cuboid(15., 0.5, 15.),
        Transform::from_xyz(0., -0.5, 0.),
    ));

    // commands.spawn((
    //     RigidBody::Dynamic,
    //     DebugPlayer,
    //     LockedAxes::new().lock_rotation_z(),
    //     // MaxLinearSpeed(MAX_SPEED),
    //     Friction::new(car_friction),
    //     Restitution::new(car_restitution),
    //     Collider::cuboid(0.5, 0.5, 1.),
    //     Transform::from_xyz(0., 2., 0.).with_scale(Vec3::splat(car_size)),
    //     SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("gltf/car/car.glb"))),
    // ));
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

fn spawn_new_players(
    mut commands: Commands,
    mut random_source: ResMut<RandomSource>,
    asset_server: Res<AssetServer>,
    cars: Query<&Player, With<RaceGameMarker>>,
    player_mapping: Res<PlayerMapping>,
    configs: Res<Assets<Config>>,
    config_resource: Res<ConfigAccessor>,
) {
    let config = configs.get(&config_resource.handle).expect("no config!");
    let car_friction = cfloat![config, CAR_FRICTION];
    let max_speed = cfloat![config, MAX_SPEED];
    let car_size = cfloat![config, CAR_SIZE];
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
            Transform::from_xyz(pos.x, 5.0, pos.y).with_scale(Vec3::splat(car_size)),
            RigidBody::Dynamic,
            Friction::new(car_friction),
            MaxLinearSpeed(max_speed),
            Collider::cuboid(1., 1., 1.),
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("gltf/car/car.glb"))),
        ));
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
    // let acc_speed: f32 = configs.get_config_value(&config[&RacingConfig::AccSpeed]);
    // let acc_speed: f32 = 1.0;
    let config = configs.get(&config_resource.handle).expect("no config!");
    // let acc_speed = cfloat!(config, ACC_SPEED);
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
            let forward = pos.forward();
            linear_velocity.0 += Vec3::Z * co.acceleration * acc_speed;
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
    let co = get_control_acc(&dpi.0, acc_speed, turn_speed);
    let rotate_right = Quat::from_rotation_y(PI / 2.);
    let forward = pos.rotation.mul_vec3(Vec3::Z);
    linear_velocity.0 += Vec3::Z * co.acceleration;
    pos.rotation = pos
        .rotation
        .rotate_towards(pos.rotation.mul_quat(rotate_right), co.turn);
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
    gltf_children: Query<(&GltfMaterialName, &Transform, &Mesh3d, &Name)>,
    meshes: Res<Assets<Mesh>>,
    children: Query<&Children>,
) {
    info!("Scene Instance Ready: {:?}", trigger.target());
    for descendant in children
        .iter_descendants(trigger.target())
        .collect::<Vec<_>>()
        .into_iter()
    {
        if let Ok((gltf_name, transform, mesh, name)) = gltf_children.get(descendant) {
            // add collider to it
            if gltf_name.0 == COLLISION_MAT_NAME {
                if let Some(mesh) = meshes.get(&mesh.0) {
                    if let Some(collider) = Collider::convex_hull_from_mesh(mesh) {
                        commands
                            .entity(descendant)
                            .insert((
                                RigidBody::Static,
                                DistanceSensitiveStaticCollider { distance: 2.0 },
                            ))
                            .remove::<Mesh3d>()
                            .insert(collider);
                    } else {
                        warn!("Unable to generate collider for {name}!",)
                    }
                }
            }
        }
    }
}
