use crate::api;
use bytes::Bytes;
use strum_macros::{Display, FromRepr};
use thiserror::Error;

#[derive(FromRepr, Display, Debug, PartialEq, Clone)]
#[repr(i32)]
pub enum LogLevel {
    None = 0,
    Error = 1,
    Warn = 2,
    Info = 3,
    Config = 4,
    Debug = 5,
    Verbose = 6,
    VeryVerbose = 7,
}

pub struct Log {
    pub level: LogLevel,
    pub message: Bytes,
    pub send_failed: bool,
}

#[derive(FromRepr, Display, Debug, PartialEq, Clone)]
#[repr(i32)]
pub enum UserServiceArgType {
    Bool = 0,
    Int = 1,
    Float = 2,
    String = 3,
    BoolArray = 4,
    IntArray = 5,
    FloatArray = 6,
    StringArray = 7,
}

#[derive(Error, Debug, Clone)]
pub enum UserServiceParseError {
    #[error("unknown arg type `{0}`")]
    UnknownArgType(i32),
}

#[derive(Debug, Clone)]
pub struct UserServiceArg {
    pub name: String,
    pub typ: UserServiceArgType
}

impl TryFrom<api::ListEntitiesServicesArgument> for UserServiceArg {
    type Error = UserServiceParseError;

    fn try_from(value: api::ListEntitiesServicesArgument) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            typ: UserServiceArgType::from_repr(value.r#type).ok_or(UserServiceParseError::UnknownArgType(value.r#type))?
        })
    }
}

#[derive(Debug, Clone)]
pub struct UserService {
    pub name: String,
    pub key: u32,
    pub args: Vec<UserServiceArg>
}

impl TryFrom<api::ListEntitiesServicesResponse> for UserService {
    type Error = UserServiceParseError;

    fn try_from(value: api::ListEntitiesServicesResponse) -> Result<Self, Self::Error> {
        let mut args: Vec<UserServiceArg> = Vec::new();
        for arg in value.args {
            args.push(arg.try_into()?);
        }
        Ok(Self {
            name: value.name,
            key: value.key,
            args
        })
    }
}

#[derive(FromRepr, Display, Debug, PartialEq, Clone)]
#[repr(u16)]
pub enum MessageType {
    HelloRequest = 1,
    HelloResponse = 2,
    ConnectRequest = 3,
    ConnectResponse = 4,
    DisconnectRequest = 5,
    DisconnectResponse = 6,
    PingRequest = 7,
    PingResponse = 8,
    DeviceInfoRequest = 9,
    DeviceInfoResponse = 10,
    ListEntitiesRequest = 11,
    ListEntitiesBinarySensorResponse = 12,
    ListEntitiesCoverResponse = 13,
    ListEntitiesFanResponse = 14,
    ListEntitiesLightResponse = 15,
    ListEntitiesSensorResponse = 16,
    ListEntitiesSwitchResponse = 17,
    ListEntitiesTextSensorResponse = 18,
    ListEntitiesDoneResponse = 19,
    SubscribeStatesRequest = 20,
    BinarySensorStateResponse = 21,
    CoverStateResponse = 22,
    FanStateResponse = 23,
    LightStateResponse = 24,
    SensorStateResponse = 25,
    SwitchStateResponse = 26,
    TextSensorStateResponse = 27,
    SubscribeLogsRequest = 28,
    SubscribeLogsResponse = 29,
    CoverCommandRequest = 30,
    FanCommandRequest = 31,
    LightCommandRequest = 32,
    SwitchCommandRequest = 33,
    SubscribeHomeassistantServicesRequest = 34,
    HomeassistantServiceResponse = 35,
    GetTimeRequest = 36,
    GetTimeResponse = 37,
    SubscribeHomeAssistantStatesRequest = 38,
    SubscribeHomeAssistantStateResponse = 39,
    HomeAssistantStateResponse = 40,
    ListEntitiesServicesResponse = 41,
    ExecuteServiceRequest = 42,
    ListEntitiesCameraResponse = 43,
    CameraImageResponse = 44,
    CameraImageRequest = 45,
    ListEntitiesClimateResponse = 46,
    ClimateStateResponse = 47,
    ClimateCommandRequest = 48,
    ListEntitiesNumberResponse = 49,
    NumberStateResponse = 50,
    NumberCommandRequest = 51,
    ListEntitiesSelectResponse = 52,
    SelectStateResponse = 53,
    SelectCommandRequest = 54,
    ListEntitiesSirenResponse = 55,
    SirenStateResponse = 56,
    SirenCommandRequest = 57,
    ListEntitiesLockResponse = 58,
    LockStateResponse = 59,
    LockCommandRequest = 60,
    ListEntitiesButtonResponse = 61,
    ButtonCommandRequest = 62,
    ListEntitiesMediaPlayerResponse = 63,
    MediaPlayerStateResponse = 64,
    MediaPlayerCommandRequest = 65,
    SubscribeBluetoothLEAdvertisementsRequest = 66,
    BluetoothLEAdvertisementResponse = 67,
    BluetoothDeviceRequest = 68,
    BluetoothDeviceConnectionResponse = 69,
    BluetoothGATTGetServicesRequest = 70,
    BluetoothGATTGetServicesResponse = 71,
    BluetoothGATTGetServicesDoneResponse = 72,
    BluetoothGATTReadRequest = 73,
    BluetoothGATTReadResponse = 74,
    BluetoothGATTWriteRequest = 75,
    BluetoothGATTReadDescriptorRequest = 76,
    BluetoothGATTWriteDescriptorRequest = 77,
    BluetoothGATTNotifyRequest = 78,
    BluetoothGATTNotifyDataResponse = 79,
    SubscribeBluetoothConnectionsFreeRequest = 80,
    BluetoothConnectionsFreeResponse = 81,
    BluetoothGATTErrorResponse = 82,
    BluetoothGATTWriteResponse = 83,
    BluetoothGATTNotifyResponse = 84,
    BluetoothDevicePairingResponse = 85,
    BluetoothDeviceUnpairingResponse = 86,
    UnsubscribeBluetoothLEAdvertisementsRequest = 87,
    BluetoothDeviceClearCacheResponse = 88,
    SubscribeVoiceAssistantRequest = 89,
    VoiceAssistantRequest = 90,
    VoiceAssistantResponse = 91,
    VoiceAssistantEventResponse = 92,
    BluetoothLERawAdvertisementsResponse = 93,
    ListEntitiesAlarmControlPanelResponse = 94,
    AlarmControlPanelStateResponse = 95,
    AlarmControlPanelCommandRequest = 96,
    ListEntitiesTextResponse = 97,
    TextStateResponse = 98,
    TextCommandRequest = 99,
    ListEntitiesDateResponse = 100,
    DateStateResponse = 101,
    DateCommandRequest = 102,
    ListEntitiesTimeResponse = 103,
    TimeStateResponse = 104,
    TimeCommandRequest = 105,
    VoiceAssistantAudio = 106,
    ListEntitiesEventResponse = 107,
    EventResponse = 108,
    ListEntitiesValveResponse = 109,
    ValveStateResponse = 110,
    ValveCommandRequest = 111,
    ListEntitiesDateTimeResponse = 112,
    DateTimeStateResponse = 113,
    DateTimeCommandRequest = 114,
    VoiceAssistantTimerEventResponse = 115,
    ListEntitiesUpdateResponse = 116,
    UpdateStateResponse = 117,
    UpdateCommandRequest = 118,
    VoiceAssistantAnnounceRequest = 119,
    VoiceAssistantAnnounceFinished = 120,
    VoiceAssistantConfigurationRequest = 121,
    VoiceAssistantConfigurationResponse = 122,
    VoiceAssistantSetConfiguration = 123,
}


