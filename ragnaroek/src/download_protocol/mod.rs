//! This module is the core of the actual protocol implementation.

mod begin_session;
mod download_pit;
mod end_session;
mod flash;
mod magic_handshake;
mod types;

pub use types::*;
