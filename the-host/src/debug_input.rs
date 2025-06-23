use bevy::app::App;
use bevy::input::ButtonInput;
use bevy::prelude::{info, Assets, Commands, Component, KeyCode, Res, ResMut, Resource, Update};
use game_42_net::controls::{ButtonType, PlayerInput};
use crate::config::{Config, ConfigAccessor};

pub fn init(app: &mut App) {
    app.insert_resource(DebugPlayerInput(PlayerInput::new()));
    app.add_systems(Update, handle_debug_input);
}

#[derive(Component)]
pub struct DebugPlayer;

#[derive(Resource)]
pub struct DebugPlayerInput(pub PlayerInput);

// again, maybe to be used later
pub fn handle_debug_input(
    mut commands: Commands,
    mut debug_player: ResMut<DebugPlayerInput>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    configs: Res<Assets<Config>>,
    config_accessor: Res<ConfigAccessor>,
) {
    let config = configs.get(&config_accessor.handle)
        .expect("no config");
    // if keyboard_input.just_pressed(KeyCode::Space) {
    //     info!("config is {}", config.0);
    // }
    let dp = &mut debug_player.as_mut().0;
    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        dp.update_button(ButtonType::Up, true);
    }
    if keyboard_input.just_released(KeyCode::ArrowUp) {
        dp.update_button(ButtonType::Up, false);
    }
    if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        dp.update_button(ButtonType::Down, true);
    }
    if keyboard_input.just_released(KeyCode::ArrowDown) {
        dp.update_button(ButtonType::Down, false);
    }
    if keyboard_input.just_pressed(KeyCode::ArrowLeft) {
        dp.update_button(ButtonType::Left, true);
    }
    if keyboard_input.just_released(KeyCode::ArrowLeft) {
        dp.update_button(ButtonType::Left, false);
    }
    if keyboard_input.just_pressed(KeyCode::ArrowRight) {
        dp.update_button(ButtonType::Right, true);
    }
    if keyboard_input.just_released(KeyCode::ArrowRight) {
        dp.update_button(ButtonType::Right, false);
    }
}
