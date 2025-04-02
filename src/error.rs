use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use crate::{entity::{EntityStateUpdate, EntityType}, model::{Log, MessageType, UserServiceParseError}};

#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("not connected")]
    NotConnected,
    #[error("device requested shutdown")]
    DeviceRequestShutdown,
    #[error("invalid password")]
    InvalidPassword,
    #[error("connection error `{0}`")]
    ConnectionError(ConnectionError),
    #[error("frame had wrong preamble `{0}`")]
    FrameHadWrongPreamble(u8),
    #[error("system time error `{0}`")]
    SystemTimeError(std::time::SystemTimeError),
    #[error("system time error `{0}`")]
    SystemTimeIntCastError(std::num::TryFromIntError),
    #[error("prost decode error `{0}`")]
    ProstDecodeError(prost::DecodeError),
    #[error("prost encode error `{0}`")]
    ProstEncodeError(prost::EncodeError),
    #[error("user service parse error `{0}`")]
    UserServiceParseError(UserServiceParseError),
    #[error("state update for unknown entity (key=`{0}`, type=`{1}`)")]
    StateUpdateForUnknownEntity(u32, EntityType),
    #[error("unknown list entities reponse `{0}`")]
    UnknownListEntitiesResponse(MessageType),
    #[error("unknown entity category `{0}`")]
    UnknownEntityCategory(i32),
    #[error("wrong message type `{0}`")]
    WrongMessageType(MessageType),
    #[error("unknown incoming message type `{0}`")]
    UnknownIncomingMessageType(MessageType),
    #[error("unknown log level `{0}`")]
    UnknownLogLevel(i32),
    #[error("log send error `{0}`")]
    LogChannelSendError(SendError<Log>),
    #[error("entity state update send error `{0}`")]
    EntityStateUpdateChannelSendError(SendError<EntityStateUpdate>),
}

impl From<ConnectionError> for DeviceError {
    fn from(value: ConnectionError) -> Self {
        Self::ConnectionError(value)
    }
}

impl From<prost::DecodeError> for DeviceError {
    fn from(value: prost::DecodeError) -> Self {
        Self::ProstDecodeError(value)
    }
}

impl From<prost::EncodeError> for DeviceError {
    fn from(value: prost::EncodeError) -> Self {
        Self::ProstEncodeError(value)
    }
}

impl From<SendError<Log>> for DeviceError {
    fn from(value: SendError<Log>) -> Self {
        Self::LogChannelSendError(value)
    }
}

impl From<SendError<EntityStateUpdate>> for DeviceError {
    fn from(value: SendError<EntityStateUpdate>) -> Self {
        Self::EntityStateUpdateChannelSendError(value)
    }
}

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("not connected")]
    NotConnected,
    #[error("unknown message type `{0}`")]
    UnknownMessageType(u16),
    #[error("noise decrypt error `{0}`")]
    NoiseDecryptError(snow::error::Error),
    #[error("tcp io error `{0}`")]
    TcpIOError(std::io::Error),
    #[error("base64 decode slice error `{0}` (noise_psk may be incorrectly sized)")]
    Base64DecodeSliceError(base64::DecodeSliceError),
    #[error("client wants unknown noise protocol `{0}`")]
    ClientWantsUnknownNoiseProtocol(u8),
    #[error("recieved message missing null terminator")]
    MessageMissingNullTerminator,
    #[error("handshake had wrong preamble `{0}`")]
    HandshakeHadWrongPreamble(u8),
    #[error("frame had wrong preamble `{0}` (may have wrong Connection type)")]
    FrameHadWrongPreamble(u8),
}

impl From<std::io::Error> for ConnectionError {
    fn from(value: std::io::Error) -> Self {
        Self::TcpIOError(value)
    }
}

impl From<snow::error::Error> for ConnectionError {
    fn from(value: snow::error::Error) -> Self {
        Self::NoiseDecryptError(value)
    }
}

impl From<base64::DecodeSliceError> for ConnectionError {
    fn from(value: base64::DecodeSliceError) -> Self {
        Self::Base64DecodeSliceError(value)
    }
}
