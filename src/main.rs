#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! This crate implements a flash tool for Samsung Odin-compatible devices.
//! It has both a binary CLI utility, as well as a library
//! that allows you to easily build your own tools.

use comms::Communicator;
mod comms;
mod protocol;

fn main() {
    let mut listener = comms::net::Listener::new(13579);
    println!("Listening...");
    let mut conn: Box<dyn Communicator> = Box::new(listener.accept().unwrap());
    println!("Target connected!");
    protocol::begin_session(&mut conn).unwrap();
    println!("Handshake OK");
}
