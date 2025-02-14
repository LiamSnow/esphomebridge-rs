use bytes::BytesMut;
use crate::{error::ConnectionError, model::MessageType};

#[allow(async_fn_in_trait)]
pub trait Connection {
    async fn send_message(&mut self, msg_type: MessageType, msg_bytes: &BytesMut) -> Result<(), ConnectionError>;
    async fn receive_message(&mut self) -> Result<(MessageType, BytesMut), ConnectionError>;
    async fn buffer_empty(&mut self) -> bool;
    async fn connect(&mut self) -> Result<(), ConnectionError>;
    async fn disconnect(&mut self) -> Result<(), ConnectionError>;
}
