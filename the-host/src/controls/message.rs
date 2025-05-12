use game_42_net::protocol::UserId;
use crate::controls::{ButtonType, JoystickAxis};

pub enum InputUpdate {
    Button(ButtonType, bool),
    Joystick(JoystickAxis, f32),
}

pub struct PlayerInputUpdate {
    pub id: UserId,
    pub inp: InputUpdate,
}