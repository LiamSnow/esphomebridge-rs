use bytes::BytesMut;
use prost::Message;
use std::{
    collections::HashMap, hash::{Hash, Hasher}, time::{SystemTime, UNIX_EPOCH}
};

use crate::{
    api, connection::{base::{AnyConnection, Connection}, noise::NoiseConnection, plain::PlainConnection}, entity::Entities, error::DeviceError, model::{Log, LogLevel, MessageType, UserService}
};

pub struct ESPHomeDevice {
    pub(crate) conn: AnyConnection,
    password: String,
    pub connected: bool,
    pub entities: Entities,
    pub services: HashMap<u32, UserService>,
    pub logs: Vec<Log>
}

impl Hash for ESPHomeDevice {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.conn.hash(state);
    }
}

impl ESPHomeDevice {
    pub fn new(conn: AnyConnection, password: Option<String>) -> Self {
        ESPHomeDevice {
            conn,
            password: password.unwrap_or("".to_string()),
            connected: false,
            entities: Entities::default(),
            services: HashMap::new(),
            logs: Vec::new()
        }
    }

    /// helper function to create a NoiseConnection and Device
    pub fn new_noise(ip: String, noise_psk: String) -> Self {
        Self::new(NoiseConnection::new(ip, noise_psk).into(), None)
    }

    /// helper function to create a PlainConnection and Device
    pub fn new_plain(ip: String, password: String) -> Self {
        Self::new(PlainConnection::new(ip).into(), Some(password))
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
                password: self.password.clone(),
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


    pub async fn execute_service(&mut self, req: &api::ExecuteServiceRequest) -> Result<(), DeviceError> {
        self.process_incoming().await?;
        self.send(MessageType::ExecuteServiceRequest, req).await?;
        Ok(())
    }

    pub async fn get_camera_image(&mut self, req: &api::CameraImageRequest) -> Result<api::CameraImageResponse, DeviceError> {
        self.process_incoming().await?;
        let res: api::CameraImageResponse = self.transaction(
            MessageType::CameraImageRequest,
            req,
            MessageType::CameraImageResponse,
        ).await?;
        Ok(res)
    }

    ///WARNING: Call process_incoming first
    pub async fn send(&mut self, msg_type: MessageType, msg: &impl prost::Message) -> Result<(), DeviceError> {
        let msg_len = msg.encoded_len();
        let mut bytes = BytesMut::with_capacity(msg_len);
        msg.encode(&mut bytes)?;
        bytes.truncate(msg_len);
        self.conn.send_message(msg_type, &bytes).await?;
        Ok(())
    }

    ///WARNING: Call process_incoming first
    pub async fn recieve<U: prost::Message + Default>(&mut self, expected_msg_type: MessageType) -> Result<U, DeviceError> {
        let (msg_type, mut msg) = self.conn.receive_message().await?;
        if msg_type != expected_msg_type {
            return Err(DeviceError::WrongMessageType(msg_type));
        }
        Ok(U::decode(&mut msg)?)
    }

    ///WARNING: Call process_incoming first
    pub async fn transaction<U: prost::Message + Default>(
        &mut self,
        req_type: MessageType,
        req: &impl prost::Message,
        res_type: MessageType,
    ) -> Result<U, DeviceError> {
        self.send(req_type, req).await?;
        Ok(self.recieve(res_type).await?)
    }

    pub fn first_light_key(&self) -> Option<u32> {
        let first_light = self.entities.lights.iter().next()?;
        Some(first_light.1.info.key)
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
                    let log = api::SubscribeLogsResponse::decode(msg)?;
                    self.logs.push(Log {
                        level: LogLevel::from_repr(log.level).ok_or(DeviceError::UnknownLogLevel(log.level))?,
                        message: log.message.into(),
                        send_failed: log.send_failed,
                    });
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

//TODO doc comments
macro_rules! get_commands {
    ($($command:ident),*) => { paste::paste! {
        impl ESPHomeDevice {
            $(
                pub async fn [<$command:snake _command>](&mut self, req: &api::[<$command CommandRequest>]) -> Result<(), DeviceError> {
                    self.process_incoming().await?;
                    self.send(MessageType::[<$command CommandRequest>], req).await?;
                    Ok(())
                }
            )*
        }
    }}
}

get_commands! {
    Light, Cover, Fan, Switch, Climate,
    Number, Siren, Lock, Button, MediaPlayer,
    AlarmControlPanel, Text, Date, Time, DateTime,
    Valve, Update
}
