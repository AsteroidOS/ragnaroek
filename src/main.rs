#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! This crate implements a flash tool for Samsung Odin-compatible devices.
//! It has both a binary CLI utility, as well as a library
//! that allows you to easily build your own tools.

mod comms;
mod error;
pub use error::{Error, Result};
mod pit;
mod protocol;

use comms::Communicator;

/// All the Odin .ini files I could find only ever mention this port
const WIRELESS_PORT: u16 = 13579;

fn main() {
    let mut listener = comms::net::Listener::new(WIRELESS_PORT);
    println!("Listening...");
    let mut conn: Box<dyn Communicator> = Box::new(listener.accept().unwrap());
    println!("Target connected!");

    protocol::magic_handshake(&mut conn).unwrap();
    println!("Magic handshake OK");

    protocol::begin_session(&mut conn).unwrap();
    println!("Begin session OK");

    let pit = protocol::download_pit(&mut conn);
    println!("PIT download OK");
    println!("PIT data: {:?}", pit);

    protocol::end_session(&mut conn, false).unwrap();
    println!("End session OK");
}
