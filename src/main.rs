#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! This crate implements a flash tool for Samsung Odin-compatible devices.
//! It has both a binary CLI utility, as well as a library
//! that allows you to easily build your own tools.

mod comms;
mod error;
mod pit;
mod protocol;

pub use comms::Communicator;
pub use error::{Error, Result};

use clap::{App, AppSettings, Arg, ArgMatches};

/// All the Odin .ini files I could find only ever mention this port
const WIRELESS_PORT: u16 = 13579;

fn main() {
    let args = define_cli();
    match args.subcommand() {
        Some(("detect", sub_args)) => detect(sub_args),
        Some(("print-pit", sub_args)) => print_pit(sub_args),
        _ => panic!("Unexpected missing subcommand! This should've been caught by clap."),
    }
}

fn define_cli() -> ArgMatches {
    // Arguments common to all subcommands
    let transport = Arg::new("transport")
        .long("transport")
        .short('t')
        .help("Choose how to communicate with the target. USB doesn't work yet.")
        // TODO: Add USB
        .possible_values(["net", "usb"])
        .default_value("net");

    // Subcommands
    let detect = App::new("detect")
        .about("Test whether a supported device is connected.")
        .arg(transport.clone());
    let print_pit = App::new("print-pit")
        .about("Print the target's Partition Information Table (PIT).")
        .arg(transport);

    // Putting it all together
    return App::new("ragnaroek")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommands([detect, print_pit])
        .get_matches();
}

fn detect(args: &ArgMatches) {
    let _ = args;
    // TODO: Respect transport once USB is implemented

    let mut listener = comms::net::Listener::new(WIRELESS_PORT);
    let mut conn: Box<dyn Communicator> = Box::new(listener.accept().unwrap());

    protocol::magic_handshake(&mut conn).unwrap();
    protocol::begin_session(&mut conn).unwrap();
    protocol::end_session(&mut conn, false).unwrap();
}

fn print_pit(args: &ArgMatches) {
    let _ = args;
    // TODO: Respect transport once USB is implemented

    let mut listener = comms::net::Listener::new(WIRELESS_PORT);
    let mut conn: Box<dyn Communicator> = Box::new(listener.accept().unwrap());

    protocol::magic_handshake(&mut conn).unwrap();
    protocol::begin_session(&mut conn).unwrap();
    let pit = protocol::download_pit(&mut conn).unwrap();
    println!("{:?}", pit);
    protocol::end_session(&mut conn, false).unwrap();
}
