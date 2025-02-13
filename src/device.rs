use bytes::BytesMut;
use prost::Message;
use thiserror::Error;
use std::{
    collections::HashMap, time::{SystemTime, UNIX_EPOCH}
};

use crate::{
    api, connection::{base::{Connection, ConnectionError}, noise::NoiseConnection}, entity::Entities, model::{MessageType, UserService, UserServiceParseError}
};

pub struct Device<T: Connection> {
    pub(crate) conn: T,
    pub connected: bool,
    pub entities: Entities,
    pub services: HashMap<u32, UserService>,
}

impl Device<NoiseConnection> {
    /// helper function to create a NoiseConnection and Device
    pub async fn new_noise(ip: String, noise_psk: String) -> Result<Self, DeviceError> {
        let conn = NoiseConnection::new(ip, noise_psk).await?;
        Self::new(conn).await
    }
}

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
    #[error("state update for unknown entity `{0}`")]
    StateUpdateForUnknownEntity(u32),
    #[error("unknown list entities reponse `{0}`")]
    UnknownListEntitiesResponse(MessageType),
    #[error("unknown entity category `{0}`")]
    UnknownEntityCategory(i32),
    #[error("wrong message type `{0}`")]
    WrongMessageType(MessageType),
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

impl<T: Connection> Device<T> {
    pub async fn new(conn: T) -> Result<Self, DeviceError> {
        let me = Device {
            conn,
            connected: false,
            entities: Entities::default(),
            services: HashMap::new(),
        };
        Ok(me)
    }

    pub async fn connect(&mut self) -> Result<(), DeviceError> {
        if self.connected {
            return Ok(())
        }

        self.conn.connect().await?;
        let _: api::HelloResponse = self.transaction(
            MessageType::HelloRequest,
            &api::HelloRequest {
                client_info: "iron-esphome".to_string(),
                api_version_major: 1,
                api_version_minor: 9,
            },
            MessageType::HelloResponse,
        ).await?;
        let res = self.transaction::<api::ConnectResponse>(
            MessageType::ConnectRequest,
            &api::ConnectRequest {
                password: "".to_string(),
            },
            MessageType::ConnectResponse,
        ).await;
        if let Ok(msg) = res {
            if msg.invalid_password {
                return Err(DeviceError::InvalidPassword);
            }
        }
        self.fetch_entities_and_services().await?;
        self.connected = true;
        Ok(())
    }

    pub async fn ping(&mut self) -> Result<(), DeviceError> {
        self.process_incoming().await?;
        let _: api::PingResponse = self.transaction(
            MessageType::PingRequest,
            &api::PingRequest {},
            MessageType::PingResponse,
        ).await?;
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), DeviceError> {
        self.process_incoming().await?;
        let _: api::DisconnectResponse = self.transaction(
            MessageType::DisconnectRequest,
            &api::DisconnectRequest {},
            MessageType::DisconnectResponse,
        ).await?;
        self.conn.disconnect().await?;
        self.connected = false;
        Ok(())
    }

    pub async fn device_info(&mut self) -> Result<api::DeviceInfoResponse, DeviceError> {
        self.process_incoming().await?;
        let res: api::DeviceInfoResponse = self.transaction(
            MessageType::DeviceInfoRequest,
            &api::DeviceInfoRequest {},
            MessageType::DeviceInfoResponse,
        ).await?;
        Ok(res)
    }

    pub async fn subscribe_states(&mut self) -> Result<(), DeviceError> {
        self.process_incoming().await?;
        self.send(
            MessageType::SubscribeStatesRequest,
            &api::SubscribeStatesRequest {},
        ).await?;
        Ok(())
    }

    pub async fn subscribe_logs(&mut self, level: i32, dump_config: bool) -> Result<(), DeviceError> {
        self.process_incoming().await?;
        self.send(
            MessageType::SubscribeLogsRequest,
            &api::SubscribeLogsRequest { level, dump_config },
        ).await?;
        Ok(())
    }

    pub async fn light_command(&mut self, req: api::LightCommandRequest) -> Result<(), DeviceError> {
        self.process_incoming().await?;
        self.send(MessageType::LightCommandRequest, &req).await?;
        Ok(())
    }

    pub async fn send(&mut self, msg_type: MessageType, msg: &impl prost::Message) -> Result<(), DeviceError> {
        let msg_len = msg.encoded_len();
        let mut bytes = BytesMut::with_capacity(msg_len);
        msg.encode(&mut bytes)?;
        bytes.truncate(msg_len);
        self.conn.send_message(msg_type, &bytes).await?;
        Ok(())
    }

    pub async fn recieve<U: prost::Message + Default>(&mut self, expected_msg_type: MessageType) -> Result<U, DeviceError> {
        let (msg_type, mut msg) = self.conn.receive_message().await?;
        if msg_type != expected_msg_type {
            return Err(DeviceError::WrongMessageType(msg_type));
        }
        Ok(U::decode(&mut msg)?)
    }

    pub async fn transaction<U: prost::Message + Default>(
        &mut self,
        req_type: MessageType,
        req: &impl prost::Message,
        res_type: MessageType,
    ) -> Result<U, DeviceError> {
        self.send(req_type, req).await?;
        Ok(self.recieve(res_type).await?)
    }


    pub async fn process_incoming(&mut self) -> Result<(), DeviceError> {
        while !self.conn.buffer_empty().await {
            let (msg_type, msg) = self.conn.receive_message().await?;

            match msg_type {
                MessageType::DisconnectRequest => {
                    self.send(
                        MessageType::DisconnectResponse,
                        &api::DisconnectResponse {},
                    ).await?;
                    self.conn.disconnect().await?;
                    return Err(DeviceError::DeviceRequestShutdown);
                }
                MessageType::PingRequest => {
                    self.send(MessageType::PingResponse, &api::PingResponse {}).await?;
                }
                MessageType::GetTimeRequest => {
                    self.send(
                        MessageType::GetTimeResponse,
                        &api::GetTimeResponse {
                            epoch_seconds: SystemTime::now()
                                .duration_since(UNIX_EPOCH).map_err(|e| DeviceError::SystemTimeError(e))?
                                .as_secs()
                                .try_into().map_err(|e| DeviceError::SystemTimeIntCastError(e))?,
                        },
                    ).await?;
                }
                MessageType::SubscribeLogsResponse => {
                    //TODO
                }
                _ => {
                    if !self.process_state_update(&msg_type, msg)? {
                        println!("unexpectedly got incoming msg: {:#?}", msg_type);
                    }
                },
            }
        }
        Ok(())
    }

    pub async fn fetch_entities_and_services(&mut self) -> Result<(), DeviceError> {
        self.process_incoming().await?;
        self.send(MessageType::ListEntitiesRequest, &api::ListEntitiesRequest {}).await?;
        loop {
            let (msg_type, msg) = self.conn.receive_message().await?;

            match msg_type {
                MessageType::ListEntitiesServicesResponse => {
                    let res: UserService = api::ListEntitiesServicesResponse::decode(msg)?
                        .try_into().map_err(|e| DeviceError::UserServiceParseError(e))?;
                    self.services.insert(res.key, res);
                },
                MessageType::ListEntitiesDoneResponse => {
                    return Ok(());
                },
                _ => self.save_entity(msg_type, msg)?
            }
        }
    }
}
