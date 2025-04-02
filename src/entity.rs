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
                /// type.get(ID) -> info
                $(pub [<$name:snake>]: Vec<api::[<ListEntities $name Response>]>,)*
            }

            #[derive(Default)]
            pub struct EntityIndexLut {
                /// maps entity.key -> index of the entity inside EntityInfos.type
                $(pub [<$name:snake _by_key>]: HashMap<u32, usize>,)*
                /// maps entity.object_id -> index of the entity inside EntityInfos.type
                $(pub [<$name:snake _by_name>]: HashMap<String, usize>,)*
            }

            #[derive(Clone, PartialEq, Eq, Debug, Display)]
            pub enum EntityType {
                $($name,)*
            }

            impl EntityInfos {
                /// converts every entity to EntityInfo
                pub fn get_all<'a>(&'a self) -> Vec<EntityInfo<'a>> {
                    let mut v = Vec::new();
                    $(
                        for entity in &self.[<$name:snake>] {
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

            impl ESPHomeDevice {
                /// get entity info from entity.object_id
                $(pub fn [<get_ $name:snake _from_name>](&self, object_id: &str) -> Option<&api::[<ListEntities $name Response>]> {
                    let entity_index = self.entity_index_lut.[<$name:snake _by_name>].get(object_id)?;
                    self.entities.[<$name:snake>].get(*entity_index)
                })*

                /// get entity key from entity.object_id
                $(pub fn [<get_ $name:snake _key_from_name>](&self, object_id: &str) -> Option<u32> {
                    Some(self.[<get_ $name:snake _from_name>](object_id)?.key)
                })*

                /// get entity info from entity.key
                $(pub fn [<get_ $name:snake _from_key>](&self, key: &u32) -> Option<&api::[<ListEntities $name Response>]> {
                    let entity_index = self.entity_index_lut.[<$name:snake _by_key>].get(key)?;
                    self.entities.[<$name:snake>].get(*entity_index)
                })*

                ///Gets all keys, including debug and config types
                $(pub fn [<get_all_ $name:snake _keys>](&self) -> Vec<u32> {
                    self.entities.[<$name:snake>].iter().map(|info| info.key).collect()
                })*

                ///Gets keys, excluding debug and config types
                $(pub fn [<get_primary_ $name:snake _keys>](&self) -> Vec<u32> {
                    self.entities.[<$name:snake>].iter()
                        .filter(|info| info.entity_category == ENTITY_CATEGORY_NONE)
                        .map(|info| info.key)
                        .collect()
                })*

                pub(crate) fn save_entity(&mut self, msg_type: MessageType, msg: BytesMut) -> Result<(), DeviceError> {
                    match msg_type {
                        $(
                            MessageType::[<ListEntities $name Response>] => {
                                let res = api::[<ListEntities $name Response>]::decode(msg)?;
                                let id = self.entities.[<$name:snake>].len();
                                self.entity_index_lut.[<$name:snake _by_key>].insert(res.key, id);
                                self.entity_index_lut.[<$name:snake _by_name>].insert(res.object_id.clone(), id);
                                self.entities.[<$name:snake>].push(res);
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
                pub entity_key: u32,
                pub entity_index: usize,
                pub entity_name: String,
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
                                let entity_key = new_state.key;
                                let entity_index = *self.entity_index_lut.[<$name:snake _by_key>].get(&entity_key)
                                    .ok_or(DeviceError::StateUpdateForUnknownEntity(entity_key, EntityType::$name))?;
                                let entity_name = self.entities.[<$name:snake>].get(entity_index).unwrap().object_id.clone();
                                let value = EntityStateUpdateValue::$name(new_state);
                                Ok(EntityStateUpdate { entity_key, entity_index, entity_name, value })
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
