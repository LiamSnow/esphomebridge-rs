use std::collections::HashMap;

use crate::error::DeviceError;
use crate::api;
use paste::paste;
use prost::Message;
use bytes::BytesMut;
use crate::model::MessageType;
use crate::device::ESPHomeDevice;
use strum_macros::Display;

pub const ENTITY_CATEGORY_NONE: i32 = 0;
pub const ENTITY_CATEGORY_CONFIG: i32 = 1;
pub const ENTITY_CATEGORY_DIAGNOSTIC: i32 = 2;

#[derive(Debug, PartialEq, Clone)]
pub struct EntityInfo<'a> {
    pub object_id: &'a str,
    pub key: u32,
    pub name: &'a str,
    pub unique_id: &'a str,
    pub disabled_by_default: bool,
    pub icon: &'a str,
    pub category: i32,
    pub typ: EntityType
}

//TODO doc comments
macro_rules! gen_entity_infos {
    ($($name:ident),*) => {
        paste! {
            #[derive(Default)]
            pub struct EntityInfos {
                /// name (object_id) -> info
                $(pub [<$name:snake>]: HashMap<String, api::[<ListEntities $name Response>]>,)*
            }

            impl EntityInfos {
                /// converts every entity to EntityInfo
                pub fn get_all<'a>(&'a self) -> Vec<EntityInfo<'a>> {
                    let mut v = Vec::new();
                    $(
                        for (_, entity) in &self.[<$name:snake>] {
                            v.push(EntityInfo {
                                object_id: &entity.object_id,
                                key: entity.key,
                                name: &entity.name,
                                unique_id: &entity.unique_id,
                                disabled_by_default: entity.disabled_by_default,
                                icon: &entity.icon,
                                category: entity.entity_category,
                                typ: EntityType::$name
                            });
                        }
                    )*
                    v
                }
            }

            #[derive(Clone, PartialEq, Eq, Debug, Display)]
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
                        .filter(|entity| entity.entity_category == ENTITY_CATEGORY_NONE)
                        .map(|entity| entity.key)
                        .collect()
                })*

                pub(crate) fn save_entity(&mut self, msg_type: MessageType, msg: BytesMut) -> Result<(), DeviceError> {
                    match msg_type {
                        $(
                            MessageType::[<ListEntities $name Response>] => {
                                let res = api::[<ListEntities $name Response>]::decode(msg)?;
                                self.entities.[<$name:snake>].insert(res.object_id.clone(), res);
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
            #[derive(Debug)]
            pub struct EntityStateUpdate {
                pub subdev_name: String,
                pub value: EntityStateUpdateValue
            }

            #[derive(Debug)]
            pub enum EntityStateUpdateValue {
                $($name(api::[<$name StateResponse>]),)*
            }

            impl ESPHomeDevice {
                pub(crate) fn process_state_update(&mut self, msg_type: &MessageType, msg: BytesMut) -> Result<EntityStateUpdate, DeviceError> {
                    match msg_type {
                        $(
                            MessageType::[<$name StateResponse>] => {
                                let new_state = api::[<$name StateResponse>]::decode(msg)?;
                                let subdev_name = self.entity_key_to_name.get(&new_state.key)
                                    .ok_or(DeviceError::StateUpdateForUnknownEntity(new_state.key))?
                                    .to_string();
                                let value = EntityStateUpdateValue::$name(new_state);
                                Ok(EntityStateUpdate { subdev_name, value })
                            }
                        )*
                        _ => Err(DeviceError::UnknownIncomingMessageType(msg_type.clone()))
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
