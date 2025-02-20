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
macro_rules! gen_entity_infos {
    ($($name:ident),*) => {
        paste! {
            #[derive(Default)]
            pub struct EntityInfos {
                /// name (object_id) -> info
                $(pub [<$name:snake>]: HashMap<String, EntityInfo>,)*
            }

            impl EntityInfos {
                /// converts the struct to hashmap, IE self.lights -> hashmap.get("lights")
                pub fn to_hashmap(&self) -> HashMap<String, &HashMap<String, EntityInfo>> {
                    let mut h = HashMap::new();
                    $(h.insert(stringify!([<$name:snake>]).to_string(), &self.[<$name:snake>]);)*
                    h
                }
            }

            #[derive(Clone, PartialEq, Eq, Debug)]
            pub enum EntityType {
                $($name,)*
            }

            impl ESPHomeDevice {
                ///Gets all keys, including debug and config types
                $(pub fn [<get_all_ $name:snake _keys>](&self) -> Vec<u32> {
                    self.entities.[<$name:snake>].values().map(|entity| entity.key).collect()
                })*

                ///Gets keys, excluding debug and config types
                $(pub fn [<get_primary_ $name:snake _keys>](&self) -> Vec<u32> {
                    self.entities.[<$name:snake>]
                        .values()
                        .filter(|entity| entity.category == EntityCategory::None)
                        .map(|entity| entity.key)
                        .collect()
                })*

                pub(crate) fn save_entity(&mut self, msg_type: MessageType, msg: BytesMut) -> Result<(), DeviceError> {
                    match msg_type {
                        $(
                            MessageType::[<ListEntities $name Response>] => {
                                let res = api::[<ListEntities $name Response>]::decode(msg)?;
                                self.entities.[<$name:snake>].insert(
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
                                        typ: EntityType::$name
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


            }
        }
    }
}

macro_rules! gen_entity_states {
    ($($name:ident),*) => {
        paste! {
            #[derive(Default)]
            pub struct EntityStates {
                $(pub [<$name:snake>]: HashMap<u32, Option<api::[<$name StateResponse>]>>,)*
            }

            impl ESPHomeDevice {
                pub(crate) fn process_state_update(&mut self, msg_type: &MessageType, msg: BytesMut) -> Result<bool, DeviceError> {
                    match msg_type {
                        $(
                            MessageType::[<$name StateResponse>] => {
                                let new_state = api::[<$name StateResponse>]::decode(msg)?;
                                self.states.[<$name:snake>].insert(new_state.key, Some(new_state));
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

gen_entity_infos!(
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
    Update,
    Button,
    Camera,
    Event
);

gen_entity_states!(
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
);
