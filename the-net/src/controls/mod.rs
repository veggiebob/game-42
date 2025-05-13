mod message;

use values_macro_derive::{EnumValues, Mapping};
use serde::{Deserialize, Serialize};

/// Identifies a button
#[derive(EnumValues, Mapping, Serialize, Deserialize, Debug, Clone)]
pub enum ButtonType {
    A,
    B,
    X,
    Y,
    Up,
    Down,
    Left,
    Right
}

/// Identifies which axis it is (a traditional joystick has 2 axes, X and Y).
#[derive(EnumValues, Mapping, Serialize, Deserialize, Debug, Clone)]
pub enum JoystickAxis {
    LeftX,
    LeftY,
    RightX,
    RightY,
}

/// Most recent state of input from player
pub struct PlayerInput {
    buttons: ButtonTypeMapping<ButtonState>,
    joysticks: JoystickAxisMapping<JoystickState>,
}

pub struct ButtonState {
    pressed: bool,
    just_pressed: bool,
    just_released: bool,
}

pub struct JoystickState {
    value: f32
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InputUpdate {
    Button(ButtonType, bool),
    Joystick(JoystickAxis, f32),
}

impl ButtonState {
    pub fn new() -> Self {
        ButtonState {
            pressed: false,
            just_pressed: false,
            just_released: false,
        }
    }

    pub fn update(&mut self, pressed: bool) {
        self.just_pressed = !self.pressed && pressed;
        self.just_released = self.pressed && !pressed;
        self.pressed = pressed;
    }

    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    pub fn just_pressed(&self) -> bool {
        self.just_pressed
    }

    pub fn just_released(&self) -> bool {
        self.just_released
    }
}

impl JoystickState {
    pub fn new() -> Self {
        JoystickState {
            value: 0.0,
        }
    }

    pub fn update(&mut self, value: f32) {
        self.value = value;
    }

    pub fn get(&self) -> f32 {
        self.value
    }
}