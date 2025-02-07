use std::error::Error;

use crate::core::MessageType;

pub trait Connection {
    fn send_message(&mut self, req: impl prost::Message, msg_type: MessageType) -> Result<(), Box<dyn Error>>;
    fn receive_message<T: prost::Message + Default>(&mut self, expected_msg_type: MessageType) -> Result<T, Box<dyn Error>>;
    fn close(&mut self);
}
