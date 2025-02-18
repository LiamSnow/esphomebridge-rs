use std::collections::HashMap;

use crate::error::DeviceError;
use crate::api;
use strum_macros::{Display, FromRepr};
use paste::paste;
use prost::Message;
use bytes::BytesMut;
use crate::model::MessageType;
use crate::device::ESPHomeDevice;

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
    pub typ: EntityType
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
            pub struct EntityInfos {
                $(pub [<$stateful:snake>]: HashMap<String, EntityInfo>,)*
                $(pub [<$nostate:snake>]: HashMap<String, EntityInfo>,)*
            }

            #[derive(Default)]
            pub struct EntityStates {
                $(pub [<$stateful:snake>]: HashMap<u32, Option<api::[<$stateful StateResponse>]>>,)*
            }

            #[derive(Clone, PartialEq, Eq, Debug)]
            pub enum EntityType {
                $($stateful,)*
                $($nostate,)*
            }

            impl ESPHomeDevice {
                pub(crate) fn save_entity(&mut self, msg_type: MessageType, msg: BytesMut) -> Result<(), DeviceError> {
                    match msg_type {
                        $(
                            MessageType::[<ListEntities $stateful Response>] => {
                                let res = api::[<ListEntities $stateful Response>]::decode(msg)?;
                                self.entities.[<$stateful:snake>].insert(
                                    res.object_id.clone(),
                                    EntityInfo {
                                        object_id: res.object_id,
                                        key: res.key,
                                        name: res.name,
                                        unique_id: res.unique_id,
                                        disabled_by_default: res.disabled_by_default,
                                        icon: res.icon,
                                        category: EntityCategory::from_repr(res.entity_category)
                                            .ok_or(DeviceError::UnknownEntityCategory(res.entity_category))?,
                                        typ: EntityType::$stateful
                                    }
                                );
                            },
                        )*
                        $(
                            MessageType::[<ListEntities $nostate Response>] => {
                                let res = api::[<ListEntities $nostate Response>]::decode(msg)?;
                                self.entities.[<$nostate:snake>].insert(
                                    res.object_id.clone(),
                                    EntityInfo {
                                        object_id: res.object_id,
                                        key: res.key,
                                        name: res.name,
                                        unique_id: res.unique_id,
                                        disabled_by_default: res.disabled_by_default,
                                        icon: res.icon,
                                        category: EntityCategory::from_repr(res.entity_category)
                                            .ok_or(DeviceError::UnknownEntityCategory(res.entity_category))?,
                                        typ: EntityType::$nostate
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
                                let new_state = api::[<$stateful StateResponse>]::decode(msg)?;
                                let state_val = self.states.[<$stateful:snake>].get_mut(&new_state.key)
                                    .ok_or(DeviceError::StateUpdateForUnknownEntity(new_state.key))?;
                                *state_val = Some(new_state);
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
