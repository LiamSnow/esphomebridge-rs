use std::collections::HashMap;

use crate::api;
use strum_macros::FromRepr;
use paste::paste;
use crate::connection::base::Connection;
use std::error::Error;
use prost::Message;
use bytes::BytesMut;
use crate::model::MessageType;
use crate::Device;

#[derive(FromRepr, Debug, PartialEq, Clone)]
#[repr(i32)]
pub enum EntityCategory {
    None = 0,
    Config = 1,
    Diagnostic = 2,
}

#[derive(Debug, PartialEq, Clone)]
pub struct EntityInfo {
    pub object_id: String,
    pub key: u32,
    pub name: String,
    pub unique_id: String,
    pub disabled_by_default: bool,
    pub icon: String,
    pub category: EntityCategory,
}

macro_rules! gen_entities {
    (
        stateful {
            $($stateful:ident),*
        }
        nostate {
            $($nostate:ident),*
        }
    ) => {
        paste! {
            #[derive(Default)]
            pub struct Entities {
                $(
                    pub [<$stateful:snake s>]: HashMap<u32, $stateful>,
                )*
                $(
                    pub [<$nostate:snake s>]: HashMap<u32, $nostate>,
                )*
            }

            $(
                pub struct $stateful {
                    pub info: EntityInfo,
                    pub state: Option<api::[<$stateful StateResponse>]>
                }
            )*

            $(
                pub struct $nostate {
                    pub info: EntityInfo,
                }
            )*

            impl<T: Connection> Device<T> {
                pub(crate) fn save_entity(&mut self, msg_type: MessageType, msg: BytesMut) -> Result<(), Box<dyn Error>> {
                    match msg_type {
                        $(
                            MessageType::[<ListEntities $stateful Response>] => {
                                let res = api::[<ListEntities $stateful Response>]::decode(msg)?;
                                self.entities.[<$stateful:snake s>].insert(
                                    res.key,
                                    [<$stateful>] {
                                        info: EntityInfo {
                                            object_id: res.object_id,
                                            key: res.key,
                                            name: res.name,
                                            unique_id: res.unique_id,
                                            disabled_by_default: res.disabled_by_default,
                                            icon: res.icon,
                                            category: EntityCategory::from_repr(res.entity_category).ok_or("unknown entity category")?,
                                        },
                                        state: None
                                    }
                                );
                            },
                        )*
                        $(
                            MessageType::[<ListEntities $nostate Response>] => {
                                let res = api::[<ListEntities $nostate Response>]::decode(msg)?;
                                self.entities.[<$nostate:snake s>].insert(
                                    res.key,
                                    [<$nostate>] {
                                        info: EntityInfo {
                                            object_id: res.object_id,
                                            key: res.key,
                                            name: res.name,
                                            unique_id: res.unique_id,
                                            disabled_by_default: res.disabled_by_default,
                                            icon: res.icon,
                                            category: EntityCategory::from_repr(res.entity_category).ok_or("unknown entity category")?,
                                        }
                                    }
                                );
                            },
                        )*
                        _ => {
                            return Err("Unknown ListEntities response!".into());
                        }
                    }
                    Ok(())
                }

                pub(crate) fn process_state_update(&mut self, msg_type: &MessageType, msg: BytesMut) -> Result<bool, Box<dyn Error>> {
                    match msg_type {
                        $(
                            MessageType::[<$stateful StateResponse>] => {
                                let state = api::[<$stateful StateResponse>]::decode(msg)?;
                                let entity = self.entities.[<$stateful:snake s>].get_mut(&state.key).ok_or("state update for unknown entity!")?;
                                entity.state = Some(state);
                                Ok(true)
                            }
                        )*
                        _ => Ok(false)
                    }
                }
            }
        }
    }
}

gen_entities!(
    stateful {
        BinarySensor,
        Cover,
        Fan,
        Light,
        Sensor,
        Switch,
        TextSensor,
        Climate,
        Number,
        Select,
        Siren,
        Lock,
        MediaPlayer,
        AlarmControlPanel,
        Text,
        Date,
        Time,
        Valve,
        DateTime,
        Update
    }
    nostate {
        Button,
        Camera,
        Event
    }
);
