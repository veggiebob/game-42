
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

impl PlayerInput {
    pub fn new() -> Self {
        PlayerInput {
            buttons: ButtonTypeMapping::new(|_b| ButtonState::new()),
            joysticks: JoystickAxisMapping::new(|_j| JoystickState::new()),
        }
    }

    pub fn update_button(&mut self, button_type: ButtonType, pressed: bool) {
        self.buttons.get_mut(button_type).update(pressed);
    }

    pub fn update_joystick(&mut self, joystick_axis: JoystickAxis, value: f32) {
        self.joysticks.get_mut(joystick_axis).update(value);
    }
    
    pub fn is_pressed(&self, button_type: ButtonType) -> bool {
        self.buttons.get(button_type).pressed
    }
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
        // persist value until consumed or un-pressed
        self.just_pressed = (!self.pressed || self.just_pressed) && pressed;
        self.just_released = if pressed {
            false
        } else {
            self.just_released || self.pressed
        };
        self.pressed = pressed;
    }

    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    /// Consuming
    pub fn just_pressed(&mut self) -> bool {
        if self.pressed {
            // consume it
            let b = self.just_pressed;
            self.just_pressed = false;
            b
        } else {
            self.just_pressed = false;
            false
        }
    }

    /// Consuming
    pub fn just_released(&mut self) -> bool {
        if !self.pressed {
            // consume it
            let b = self.just_released;
            self.just_released = false;
            b
        } else {
            self.just_released = false;
            false
        }
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