use crate::assets::DoReloadAssets;
use bevy::input::ButtonInput;
use bevy::input::mouse::MouseButtonInput;
use bevy::log::info;
use bevy::prelude::{Commands, KeyCode, MouseButton, Res};

// again, maybe to be used later
pub fn handle_input(mut commands: Commands, keyboard_input: Res<ButtonInput<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        info!("Reloading...");
        commands.spawn(DoReloadAssets);
    }
}
