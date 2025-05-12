use std::sync::mpsc::{Receiver, Sender};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct UserId(u64);

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerInput {
    pub id: UserId,
    pub inp: ControllerInput,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ControllerInput {

}

/// Primitives for communication with The Host
pub struct HostInterface {
    pub recv: Receiver<()>,
    pub send: Sender<()>,
}

impl HostInterface {
    pub fn new(recv: Receiver<()>, send: Sender<()>) -> Self {
        HostInterface { recv, send }
    }
}