#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(dead_code)]
#![allow(unused_variables)]

//! This crate implements various Samsung protocols used for communicating with their devices in
//! recovery modes, such as Odin (Download) and Upload mode.
//! It aims to support both wired (via USB) and wireless (via Wi-Fi) operation.

mod comms;
pub mod download_protocol;
mod error;
pub mod upload_protocol;

pub use comms::net_bind::Connection as NetBindConnection;
pub use comms::net_bind::Listener as NetBindListener;
pub use comms::net_connect::Connection as NetConnectConnection;
pub use comms::usb::Connection as UsbConnection;
pub use comms::Communicator;
pub use error::{Error, Result};
