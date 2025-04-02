use bytes::BytesMut;
use prost::Message;
use tokio::sync::mpsc::{self, Receiver, Sender};
use std::{
    collections::HashMap, hash::{Hash, Hasher}, time::{SystemTime, UNIX_EPOCH}
};

use crate::{
    api, connection::{base::{AnyConnection, Connection}, noise::NoiseConnection, plain::PlainConnection}, entity::{EntityIndexLut, EntityInfos, EntityStateUpdate}, error::DeviceError, model::{Log, LogLevel, MessageType, UserService}
};

pub struct ESPHomeDevice {
    pub(crate) conn: AnyConnection,
    password: String,
    pub connected: bool,
    pub entities: EntityInfos, // Ex. lights.rgbct_bulb -> EntityInfo
    pub entity_index_lut: EntityIndexLut,
    pub services: HashMap<u32, UserService>,
    log_tx: Option<Sender<Log>>,
    state_update_tx: Option<Sender<EntityStateUpdate>>,
    pub last_ping: Option<SystemTime>
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
            entities: EntityInfos::default(),
            entity_index_lut: EntityIndexLut::default(),
            services: HashMap::new(),
            log_tx: None,
            state_update_tx: None,
            last_ping: None
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
            return Ok(()) //TODO should this reconnect?
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

    /// Ping without waiting for response
    pub async fn ping(&mut self) -> Result<(), DeviceError> {
        self.send(
            MessageType::PingRequest,
            &api::PingRequest {},
        ).await
    }

    /// Ping and wait for response
    pub async fn ping_wait(&mut self) -> Result<(), DeviceError> {
        self.process_incoming().await?;
        let _: api::PingResponse = self.transaction(
            MessageType::PingRequest,
            &api::PingRequest {},
            MessageType::PingResponse,
        ).await?;
        self.last_ping = Some(SystemTime::now());
        Ok(())
    }

    /// Send disconnect request to device, wait for response, then disconnect socket
    pub async fn disconnect(&mut self) -> Result<(), DeviceError> {
        self.process_incoming().await?;
        let _: api::DisconnectResponse = self.transaction(
            MessageType::DisconnectRequest,
            &api::DisconnectRequest {},
            MessageType::DisconnectResponse,
        ).await?;
        self.force_disconnect().await
    }

    /// Disconnect socket (without sending disconnect request to device)
    pub async fn force_disconnect(&mut self) -> Result<(), DeviceError> {
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

    /// Request device to send state updates.
    /// Returns a mpsc channel (of `buffer_size`) where state updates will be send
    /// Note: Will only read from the socket when calling directly calling process_incoming or
    ///       indirectly through certain commands which call process_incoming (ex ping_wait)
    pub async fn subscribe_states(&mut self, buffer_size: usize) -> Result<Receiver<EntityStateUpdate>, DeviceError> {
        let (tx, rx) = mpsc::channel(buffer_size);
        self.state_update_tx = Some(tx);
        self.send(
            MessageType::SubscribeStatesRequest,
            &api::SubscribeStatesRequest {},
        ).await?;
        Ok(rx)
    }

    /// Request device to send logs.
    /// Returns a mpsc channel (of `buffer_size`) where logs will be send
    /// Note: Will only read from the socket when calling directly calling process_incoming or
    ///       indirectly through other commands (which call process_incoming first)
    pub async fn subscribe_logs(&mut self, level: LogLevel, dump_config: bool, buffer_size: usize) -> Result<Receiver<Log>, DeviceError> {
        let (tx, rx) = mpsc::channel(buffer_size);
        self.log_tx = Some(tx);
        self.send(
            MessageType::SubscribeLogsRequest,
            &api::SubscribeLogsRequest { level: level as i32, dump_config },
        ).await?;
        Ok(rx)
    }

    pub async fn execute_service(&mut self, req: &api::ExecuteServiceRequest) -> Result<(), DeviceError> {
        self.send(MessageType::ExecuteServiceRequest, req).await
    }

    pub async fn get_camera_image(&mut self, req: &api::CameraImageRequest) -> Result<api::CameraImageResponse, DeviceError> {
        self.process_incoming().await?;
        self.transaction(
            MessageType::CameraImageRequest,
            req,
            MessageType::CameraImageResponse,
        ).await
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
        let (msg_type, mut msg) = self.conn.receive_message(None).await?;
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
        while let Some(first_byte) = self.conn.try_read_byte()? {
            let (msg_type, msg) = self.conn.receive_message(Some(first_byte)).await?;

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
                MessageType::PingResponse => {
                    self.last_ping = Some(SystemTime::now());
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
                    if let Some(log_tx) = &self.log_tx {
                        let log = api::SubscribeLogsResponse::decode(msg)?;
                        log_tx.send(Log {
                            level: LogLevel::from_repr(log.level).ok_or(DeviceError::UnknownLogLevel(log.level))?,
                            message: log.message.into(),
                            send_failed: log.send_failed,
                        }).await?;
                    }
                }
                _ => {
                    let update = self.process_state_update(&msg_type, msg)?;
                    if let Some(send_update_tx) = &self.state_update_tx {
                        send_update_tx.send(update).await?;
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
            let (msg_type, msg) = self.conn.receive_message(None).await?;

            match msg_type {
                MessageType::ListEntitiesServicesResponse => {
                    let res: UserService = api::ListEntitiesServicesResponse::decode(msg)?
                        .try_into().map_err(|e| DeviceError::UserServiceParseError(e))?;
                    self.services.insert(res.key, res);
                },
                MessageType::ListEntitiesDoneResponse => break,
                _ => self.save_entity(msg_type, msg)?
            }
        }
        Ok(())
    }
}

//TODO doc comments
macro_rules! make_commands {
    ($($command:ident),*) => { paste::paste! {
        impl ESPHomeDevice {
            $(
                pub async fn [<$command:snake _command>](&mut self, req: &api::[<$command CommandRequest>]) -> Result<(), DeviceError> {
                    self.send(MessageType::[<$command CommandRequest>], req).await?;
                    Ok(())
                }

                /// Send a command to all entities
                pub async fn [<$command:snake _command_global>](&mut self, req: &mut api::[<$command CommandRequest>]) -> Result<(), DeviceError> {
                    for key in self.get_primary_light_keys() {
                        req.key = key;
                        self.send(MessageType::[<$command CommandRequest>], req).await?;
                    }
                    Ok(())
                }
            )*
        }
    }}
}

make_commands! {
    Light, Cover, Fan, Switch, Climate,
    Number, Siren, Lock, Button, MediaPlayer,
    AlarmControlPanel, Text, Date, Time, DateTime,
    Valve, Update
}
