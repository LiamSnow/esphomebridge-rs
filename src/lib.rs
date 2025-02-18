pub mod connection;
pub mod device;
pub mod model;
pub mod entity;
pub mod error;
pub mod api {
    include!(concat!(env!("OUT_DIR"), "/_.rs"));
}

