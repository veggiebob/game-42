use std::sync::mpsc::{Receiver, Sender};
use rocket_ws::Message;
use serde::{Deserialize, Serialize};
use serde::ser::Error;
use crate::controls::InputUpdate;

#[derive(Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub struct UserId(pub u64);

/// Packet going from net (here) to host, annotated with user ID
#[derive(Debug)]
pub struct AnnotatedClientPacket {
    pub user_id: UserId,
    pub client_packet: ClientPacket
}

/// Packet going from client to net (here)
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ClientPacket {
    Input(InputUpdate),
    // seldom other things
}

/// Primitives for communication with The Host
pub struct HostInterface {
    /// Receiving from Host
    // pub recv: Receiver<()>,
    /// Sending to host
    pub send: Sender<AnnotatedClientPacket>,
}

impl HostInterface {
    pub fn new(recv: Receiver<()>, send: Sender<AnnotatedClientPacket>) -> Self {
        // todo: make a router for recv
        HostInterface { send }
    }
}

impl ClientPacket {
    pub fn from_ws_message(msg: Message) -> Result<ClientPacket, serde_json::Error> {
        match msg {
            Message::Text(text) => serde_json::from_str(&text),
            Message::Binary(_) => Err(serde_json::Error::custom("Binary message not supported yet.")),
            _ => Err(serde_json::Error::custom("Unknown message type.")),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::controls::{ButtonType, InputUpdate, JoystickAxis};
    use crate::protocol::ClientPacket;

    #[test]
    fn example_serialize() {
        let packet = ClientPacket::Input(InputUpdate::Button(ButtonType::A, true));
        let json = serde_json::to_string_pretty(&packet).unwrap();
        println!("{packet:?} is \n{json}");

        let packet = ClientPacket::Input(InputUpdate::Joystick(JoystickAxis::LeftX, 0.5));
        let json = serde_json::to_string_pretty(&packet).unwrap();
        println!("{packet:?} is \n{json}");
    }
}