use std::error::Error;

use bytes::BytesMut;

use crate::model::MessageType;

pub trait Connection {
    fn send_message(&mut self, req: impl prost::Message, msg_type: MessageType) -> Result<(), Box<dyn Error>>;
    fn receive_message_raw(&mut self) -> Result<(MessageType, BytesMut), Box<dyn Error>>;
    fn receive_message<T: prost::Message + Default>(&mut self, expected_msg_type: MessageType) -> Result<T, Box<dyn Error>>;
    fn transaction<F: prost::Message + Default>(&mut self, req: impl prost::Message, req_type: MessageType, res_type: MessageType)
        -> Result<F, Box<dyn Error>>;
    fn buffer_empty(&mut self) -> Result<bool, Box<dyn Error>>;
    fn disconnect(&mut self);
}
