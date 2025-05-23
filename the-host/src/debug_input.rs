use bevy::app::App;
use crate::assets::DoReloadAssets;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButtonInput;
use bevy::log::info;
use bevy::prelude::{Commands, Component, KeyCode, MouseButton, Res, Resource, Update};
use game_42_net::controls::PlayerInput;

pub fn init(app: &mut App) {
    app.insert_resource(DebugPlayerInput(PlayerInput::new()));
    app.add_systems(Update, handle_debug_input);
}

#[derive(Component)]
pub struct DebugPlayer;

#[derive(Resource)]
pub struct DebugPlayerInput(PlayerInput);

// again, maybe to be used later
pub fn handle_debug_input(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    // if keyboard_input.just_pressed(KeyCode::Space) {
    //     info!("Reloading...");
    //     commands.spawn(DoReloadAssets);
    // }
}
