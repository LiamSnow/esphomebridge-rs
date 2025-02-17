use std::collections::HashMap;

use crate::error::DeviceError;
use crate::api;
use strum_macros::{Display, FromRepr};
use paste::paste;
use prost::Message;
use bytes::BytesMut;
use crate::model::MessageType;
use crate::device::Device;

#[derive(FromRepr, Display, Debug, PartialEq, Clone)]
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

//TODO doc comments
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

            impl Device {
                pub(crate) fn save_entity(&mut self, msg_type: MessageType, msg: BytesMut) -> Result<(), DeviceError> {
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
                                            category: EntityCategory::from_repr(res.entity_category)
                                                .ok_or(DeviceError::UnknownEntityCategory(res.entity_category))?,
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
                                            category: EntityCategory::from_repr(res.entity_category)
                                                .ok_or(DeviceError::UnknownEntityCategory(res.entity_category))?,
                                        }
                                    }
                                );
                            },
                        )*
                        _ => {
                            return Err(DeviceError::UnknownListEntitiesResponse(msg_type));
                        }
                    }
                    Ok(())
                }

                pub(crate) fn process_state_update(&mut self, msg_type: &MessageType, msg: BytesMut) -> Result<bool, DeviceError> {
                    match msg_type {
                        $(
                            MessageType::[<$stateful StateResponse>] => {
                                let state = api::[<$stateful StateResponse>]::decode(msg)?;
                                let entity = self.entities.[<$stateful:snake s>].get_mut(&state.key)
                                    .ok_or(DeviceError::StateUpdateForUnknownEntity(state.key))?;
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
