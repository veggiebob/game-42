use crate::config::{Config, ConfigAccessor};
use bevy::app::App;
use bevy::input::ButtonInput;
use bevy::prelude::{Assets, Commands, Component, KeyCode, Res, ResMut, Resource, Update, info};
use game_42_net::controls::{ButtonState, ButtonType, PlayerInput};

pub fn init(app: &mut App) {
    app.insert_resource(DebugPlayerInput {
        player_input: PlayerInput::new(),
        button_1: ButtonState::default(),
    });
    app.add_systems(Update, handle_debug_input);
}

#[derive(Component)]
pub struct DebugPlayer;

#[derive(Resource)]
pub struct DebugPlayerInput {
    pub player_input: PlayerInput,
    pub button_1: ButtonState,
}

fn map_debug_button_pi(
    keyboard_input: &Res<ButtonInput<KeyCode>>,
    pi: &mut PlayerInput,
    key_code: KeyCode,
    button_type: ButtonType,
) {
    if keyboard_input.just_pressed(key_code) {
        pi.update_button(button_type, true);
    }
    if keyboard_input.just_released(key_code) {
        pi.update_button(button_type, false);
    }
}

fn map_debug_button(
    keyboard_input: &Res<ButtonInput<KeyCode>>,
    button_state: &mut ButtonState,
    key_code: KeyCode,
) {
    if keyboard_input.just_pressed(key_code) {
        button_state.update(true);
    }
    if keyboard_input.just_released(key_code) {
        button_state.update(false);
    }
}

pub fn handle_debug_input(
    mut debug_player: ResMut<DebugPlayerInput>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    map_debug_button(&keyboard_input, &mut debug_player.button_1, KeyCode::Space);
    let dp = &mut debug_player.as_mut().player_input;
    map_debug_button_pi(&keyboard_input, dp, KeyCode::ArrowUp, ButtonType::Up);
    map_debug_button_pi(&keyboard_input, dp, KeyCode::ArrowDown, ButtonType::Down);
    map_debug_button_pi(&keyboard_input, dp, KeyCode::ArrowLeft, ButtonType::Left);
    map_debug_button_pi(&keyboard_input, dp, KeyCode::ArrowRight, ButtonType::Right);
}