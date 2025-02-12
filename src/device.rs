use prost::Message;
use std::{
    collections::HashMap, error::Error, time::{SystemTime, UNIX_EPOCH}
};

use crate::{
    api,
    connection::base::Connection,
    model::{MessageType, UserService},
    entity::Entities
};

pub struct Device<T: Connection> {
    pub(crate) conn: T,
    pub entities: Entities,
    pub services: HashMap<u32, UserService>,
}

impl<T: Connection> Device<T> {
    pub async fn new(conn: T) -> Result<Self, Box<dyn Error>> {
        let mut me = Device {
            conn,
            entities: Entities::default(),
            services: HashMap::new(),
        };
        let _: api::HelloResponse = me.conn.transaction(
            api::HelloRequest {
                client_info: "iron-esphome".to_string(),
                api_version_major: 1,
                api_version_minor: 9,
            },
            MessageType::HelloRequest,
            MessageType::HelloResponse,
        ).await?;
        let res = me.conn.transaction::<api::ConnectResponse>(
            api::ConnectRequest {
                password: "".to_string(),
            },
            MessageType::ConnectRequest,
            MessageType::ConnectResponse,
        ).await;
        if let Ok(msg) = res {
            if msg.invalid_password {
                return Err("Invalid password".into());
            }
        }
        me.fetch_entities_and_services().await?;
        me.subscribe_states().await?;
        Ok(me)
    }

    pub async fn ping(&mut self) -> Result<(), Box<dyn Error>> {
        self.process_incoming().await?;
        let _: api::PingResponse = self.conn.transaction(
            api::PingRequest {},
            MessageType::PingRequest,
            MessageType::PingResponse,
        ).await?;
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), Box<dyn Error>> {
        self.process_incoming().await?;
        let _: api::DisconnectResponse = self.conn.transaction(
            api::DisconnectRequest {},
            MessageType::DisconnectRequest,
            MessageType::DisconnectResponse,
        ).await?;
        Ok(())
    }

    pub async fn device_info(&mut self) -> Result<api::DeviceInfoResponse, Box<dyn Error>> {
        self.process_incoming().await?;
        let res: api::DeviceInfoResponse = self.conn.transaction(
            api::DeviceInfoRequest {},
            MessageType::DeviceInfoRequest,
            MessageType::DeviceInfoResponse,
        ).await?;
        Ok(res)
    }

    pub async fn subscribe_states(&mut self) -> Result<(), Box<dyn Error>> {
        self.process_incoming().await?;
        self.conn.send_message(
            api::SubscribeStatesRequest {},
            MessageType::SubscribeStatesRequest,
        ).await?;
        Ok(())
    }

    pub async fn subscribe_logs(&mut self, level: i32, dump_config: bool) -> Result<(), Box<dyn Error>> {
        self.process_incoming().await?;
        self.conn.send_message(
            api::SubscribeLogsRequest { level, dump_config },
            MessageType::SubscribeLogsRequest,
        ).await?;
        Ok(())
    }

    pub async fn light_command(&mut self, req: api::LightCommandRequest) -> Result<(), Box<dyn Error>> {
        self.process_incoming().await?;
        self.conn
            .send_message(req, MessageType::LightCommandRequest).await?;
        Ok(())
    }

    pub async fn process_incoming(&mut self) -> Result<(), Box<dyn Error>> {
        while !self.conn.buffer_empty().await {
            let (msg_type, msg) = self.conn.receive_message_raw().await?;

            match msg_type {
                MessageType::DisconnectRequest => {
                    self.conn.send_message(
                        api::DisconnectResponse {},
                        MessageType::DisconnectResponse,
                    ).await?;
                    self.conn.disconnect().await?;
                    return Err("device requested shutdown".into());
                }
                MessageType::PingRequest => {
                    self.conn
                        .send_message(api::PingResponse {}, MessageType::PingResponse).await?;
                }
                MessageType::GetTimeRequest => {
                    self.conn.send_message(
                        api::GetTimeResponse {
                            epoch_seconds: SystemTime::now()
                                .duration_since(UNIX_EPOCH)?
                                .as_secs()
                                .try_into()?,
                        },
                        MessageType::GetTimeResponse,
                    ).await?;
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

    pub async fn fetch_entities_and_services(&mut self) -> Result<(), Box<dyn Error>> {
        self.process_incoming().await?;
        self.conn.send_message(api::ListEntitiesRequest {}, MessageType::ListEntitiesRequest).await?;
        loop {
            let (msg_type, msg) = self.conn.receive_message_raw().await?;

            match msg_type {
                MessageType::ListEntitiesServicesResponse => {
                    let res: UserService = api::ListEntitiesServicesResponse::decode(msg)?.try_into()?;
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
