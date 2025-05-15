use crate::PlayerNum;
use bevy::prelude::Component;

pub mod racing;

/// Component to identify player by a player number
#[derive(Component)]
pub struct Player(PlayerNum);
