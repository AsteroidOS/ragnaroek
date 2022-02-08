#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! This crate implements a flash tool for Samsung Odin-compatible devices.
//! It has both a binary CLI utility, as well as a library
//! that allows you to easily build your own tools.

mod comms;
mod error;
mod pit;
mod protocol;
mod upload_mode;

use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

pub use comms::Communicator;
pub use error::{Error, Result};

use clap::{App, AppSettings, Arg, ArgMatches};

/// All the Odin .ini files I could find only ever mention this port
const WIRELESS_PORT: u16 = 13579;

fn main() {
    let args = define_cli();
    match args.subcommand() {
        Some(("detect", sub_args)) => detect(sub_args),
        Some(("wait-for-device", sub_args)) => wait_for_device(sub_args),
        Some(("print-pit", sub_args)) => print_pit(sub_args),
        Some(("parse-pit", sub_args)) => parse_pit(sub_args),
        Some(("flash", sub_args)) => flash(sub_args),
        Some(("upload-mode", sub_args)) => upload_mode(sub_args),
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
    let reboot = Arg::new("reboot")
        .long("reboot")
        .short('r')
        .help("Choose whether to reboot target at the end.")
        .possible_values(["false", "true"])
        .default_value("false");

    // Subcommands
    let detect = App::new("detect")
        .about("Test whether a supported device is connected, returning failure if not. Use wait-for-device if this is not what you want.")
        .arg(transport.clone())
        .arg(reboot.clone());

    let print_pit = App::new("print-pit")
        .about("Print the target's Partition Information Table (PIT).")
        .arg(transport.clone())
        .arg(reboot.clone());

    let parse_pit = App::new("parse-pit")
        .about("Parse the provided Partition Information Table (PIT). This command does not interact with a target in any way.")
        .arg(Arg::new("pit-path")
            .long("pit-path")
            .short('p')
            .help("Specify which PIT file to use.")
            .takes_value(true)
            .required(true)
        );

    let flash = App::new("flash").about("Flash the given image to the given partition. Remember that flashing certain partitions incorrectly may brick your device!")
    .arg(transport.clone())
        .arg(reboot.clone())
    .arg(Arg::new("partition")
        .short('p')
        .long("partition")
        .required(true)
        .takes_value(true)
        .help("The partition to flash, as named in the device's PIT. Required. If unsure, you may want to run print-pit first.")
    )
    .arg(Arg::new("filename")
        .short('f')
        .long("filename")
        .required(true)
        .takes_value(true)
        .help("The filename of the file containing the partition contents you want to flash. Required.")
    );

    let wait_for_device = App::new("wait-for-device")
        .about(
            "Wait until a supported device is connected. Then return with a successful exit code.",
        )
        .arg(transport.clone())
        .arg(reboot.clone());

    let upload_mode = App::new("upload-mode")
        .about(
            "Receive a memory dump from a device in upload mode.
             Note that this is not the regular Odin mode!
             The device usually enters this mode after entering a key combo or a kernel panic.",
        )
        .arg(transport.clone())
        .arg(Arg::new("filename")
            .short('f')
            .long("filename")
            .required(true)
            .takes_value(true)
            .help("The filename of the file where the memory dump should be written to. Required.")
        )
        .arg(Arg::new("start-address")
            .short('s')
            .long("start-address")
            .required(true)
            .takes_value(true)
            .help("Memory address the dump should start at (inclusive). Required.")
        )
        .arg(Arg::new("end-address")
            .short('e')
            .long("end-address")
            .required(true)
            .takes_value(true)
            .help("Memory address the dump should end at (inclusive). Required.")
        );

    // Putting it all together
    return App::new("ragnaroek")
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommands([
            detect,
            wait_for_device,
            print_pit,
            parse_pit,
            flash,
            upload_mode,
        ])
        .get_matches();
}

fn detect(args: &ArgMatches) {
    let _ = args;
    // TODO: Respect transport once USB is implemented

    let mut listener = comms::net::Listener::new(WIRELESS_PORT);
    let mut conn: Box<dyn Communicator> = Box::new(listener.accept().unwrap());

    protocol::magic_handshake(&mut conn).unwrap();
    protocol::begin_session(&mut conn).unwrap();
    let reboot: bool = args.value_of_t_or_exit("reboot");
    protocol::end_session(&mut conn, reboot).unwrap();
}

fn wait_for_device(args: &ArgMatches) {
    let _ = args;
    // TODO: Respect transport once USB is implemented

    let mut listener = comms::net::Listener::new(WIRELESS_PORT);
    // This loop is pretty much the only difference to detect
    let mut conn: Box<dyn Communicator>;
    loop {
        match listener.accept() {
            Ok(c) => {
                conn = Box::new(c);
                break;
            }
            Err(_) => {}
        }
    }

    protocol::magic_handshake(&mut conn).unwrap();
    protocol::begin_session(&mut conn).unwrap();
    let reboot: bool = args.value_of_t_or_exit("reboot");
    protocol::end_session(&mut conn, reboot).unwrap();
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
    let reboot: bool = args.value_of_t_or_exit("reboot");
    protocol::end_session(&mut conn, reboot).unwrap();
}

fn parse_pit(args: &ArgMatches) {
    let path = args
        .value_of("pit-path")
        .expect("Required argument not set! This is probably a clap bug.");
    let path = Path::new(path);
    let mut f = File::open(path).unwrap();

    let mut pit_data: Vec<u8> = Vec::new();
    f.read_to_end(&mut pit_data).unwrap();

    let pit = pit::Pit::deserialize(&pit_data).unwrap();
    println!("{:?}", pit);
}

fn flash(args: &ArgMatches) {
    let _ = args;
    // TODO: Respect transport once USB is implemented

    let mut listener = comms::net::Listener::new(WIRELESS_PORT);
    let mut conn: Box<dyn Communicator> = Box::new(listener.accept().unwrap());

    protocol::magic_handshake(&mut conn).unwrap();
    protocol::begin_session(&mut conn).unwrap();
    let mut pit = protocol::download_pit(&mut conn).unwrap();

    // FIXME: This is the wrong type of flash!
    protocol::flash(&mut conn, &[]).unwrap();

    let reboot: bool = args.value_of_t_or_exit("reboot");
    protocol::end_session(&mut conn, reboot).unwrap();
}

fn upload_mode(args: &ArgMatches) {
    // TODO: Respect transport once USB is implemented

    let mut listener = comms::net::Listener::new(WIRELESS_PORT);
    let mut conn: Box<dyn Communicator> = Box::new(listener.accept().unwrap());

    let start_addr: u32 = args.value_of_t_or_exit("start-address");
    let end_addr: u32 = args.value_of_t_or_exit("end-address");
    let data = upload_mode::dump_memory(&mut conn, start_addr, end_addr).unwrap();

    // Write to file
    // TODO: OS strings may be more appropriate here
    let path = args
        .value_of("filename")
        .expect("Required argument not set! This is probably a clap bug.");
    let path = Path::new(path);
    let mut f = File::open(path).unwrap();
    f.write_all(&data).unwrap();
}
