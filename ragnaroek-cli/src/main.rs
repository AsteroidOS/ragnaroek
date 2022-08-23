#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! This crate implements the CLI for a flash tool for Samsung Odin-compatible devices.

use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use ragnaroek::*;

use clap::{Arg, ArgMatches, Command};
use pit::*;

/// All the Odin .ini files I could find only ever mention this port
const WIRELESS_PORT: u16 = 13579;
/// All the targets implementing wireless mode seem to use this IP
const WIRELESS_TARGET_IP: &str = "192.168.49.1";

fn main() {
    env_logger::init();

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
        .help("Choose how to communicate with the target. USB is even more experimental than everything else about ragnaroek.")
        .possible_values(["net", "usb"])
        .default_value("net");
    let reboot = Arg::new("reboot")
        .long("reboot")
        .short('r')
        .help("Choose whether to reboot target at the end.")
        .possible_values(["false", "true"])
        .default_value("false");
    let output_format = Arg::new("output-format")
        .long("output-format")
        .short('o')
        .help("Specify which output format to use.")
        .takes_value(true)
        .possible_values(["human", "json"])
        .default_value("human")
        .required(false);

    // Subcommands
    let detect = Command::new("detect")
        .about("Test whether a supported device is connected, returning failure if not. Use wait-for-device if this is not what you want.")
        .arg(transport.clone())
        .arg(reboot.clone());

    let print_pit = Command::new("print-pit")
        .about("Print the target's Partition Information Table (PIT).")
        .arg(transport.clone())
        .arg(reboot.clone())
        .arg(output_format.clone());

    let parse_pit = Command::new("parse-pit")
        .about("Parse the provided Partition Information Table (PIT). This command does not interact with a target in any way.")
        .arg(Arg::new("pit-path")
            .long("pit-path")
            .short('p')
            .help("Specify which PIT file to use.")
            .takes_value(true)
            .required(true)
        )
        .arg(output_format.clone());

    let flash = Command::new("flash").about("Flash the given image to the given partition. Remember that flashing certain partitions incorrectly may brick your device!")
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

    let wait_for_device = Command::new("wait-for-device")
        .about(
            "Wait until a supported device is connected. Then return with a successful exit code.",
        )
        .arg(transport.clone())
        .arg(reboot.clone());

    // TODO: Add subcommands for displaying probe table etc.
    // TODO: Add support for specifying name of probe table memory range to dump
    // TODO: This is getting pretty hefty. Consider moving out of here.
    let upload_mode =Command::new("upload-mode")
        .about(
            "Interact with a device in upload mode.
             Note that this is not the regular Odin mode!
             The device usually enters this mode after entering a key combo or a kernel panic.",
        )
        .subcommand(Command::new("dump")
            .about("Dump the given memory region to a file.")
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
            )
        )
        .subcommand(Command::new("probe")
            .about("Dump the probe table to stdout. This is a listing of memory regions and their properties.")
            .arg(transport.clone())
        );

    // Putting it all together
    return Command::new("ragnaroek")
        .arg_required_else_help(true)
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

fn get_download_communicator(args: &ArgMatches) -> Result<Box<dyn Communicator>> {
    let transport = args
        .value_of("transport")
        .expect("Transport must have been set! This is probably clap bug.");
    match transport {
        "usb" => {
            let conn = UsbConnection::establish()?;
            return Ok(Box::new(conn));
        }
        "net" => {
            let mut listener = NetBindListener::new(WIRELESS_PORT);
            let conn = listener.accept()?;
            return Ok(Box::new(conn));
        }
        _ => panic!("Unexpected invalid transport! This should've been caught by clap."),
    }
}

fn detect(args: &ArgMatches) {
    let mut conn: Box<dyn Communicator> = get_download_communicator(args).unwrap();

    download_protocol::magic_handshake(&mut conn).unwrap();
    download_protocol::begin_session(&mut conn).unwrap();
    let reboot: bool = args.value_of_t_or_exit("reboot");
    download_protocol::end_session(&mut conn, reboot).unwrap();
}

fn wait_for_device(args: &ArgMatches) {
    // This loop is pretty much the only difference to detect
    let mut conn: Box<dyn Communicator>;
    loop {
        match get_download_communicator(args) {
            Ok(c) => {
                conn = c;
                break;
            }
            Err(_) => {}
        }
    }

    download_protocol::magic_handshake(&mut conn).unwrap();
    download_protocol::begin_session(&mut conn).unwrap();
    let reboot: bool = args.value_of_t_or_exit("reboot");
    download_protocol::end_session(&mut conn, reboot).unwrap();
}

fn pretty_print_pit(pit: pit::Pit) {
    println!("Gang: {}", pit.gang_name);
    println!("Project: {}", pit.project_name);
    println!("Version: {}", pit.proto_version);
    println!("Entries:");
    println!("{}", tabled::Table::new(pit).to_string());
}

fn print_pit(args: &ArgMatches) {
    let mut conn: Box<dyn Communicator> = get_download_communicator(args).unwrap();

    download_protocol::magic_handshake(&mut conn).unwrap();
    download_protocol::begin_session(&mut conn).unwrap();
    let pit = download_protocol::download_pit(&mut conn).unwrap();
    pretty_print_pit(pit);

    let reboot: bool = args.value_of_t_or_exit("reboot");
    download_protocol::end_session(&mut conn, reboot).unwrap();
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

    let output_format = args
        .value_of("output-format")
        .expect("Required argument not set! This is probably a clap bug.");
    match output_format {
        "human" => pretty_print_pit(pit),
        "json" => println!(
            "{}",
            serde_json::to_string(&pit).expect("Failed to serialize PIT! This is probably a bug.")
        ),
        _ => panic!("Unexpected output format! This is probably a clap bug."),
    }
}

fn flash(args: &ArgMatches) {
    let mut conn: Box<dyn Communicator> = get_download_communicator(args).unwrap();

    download_protocol::magic_handshake(&mut conn).unwrap();
    let params = download_protocol::begin_session(&mut conn).unwrap();

    // Find the PIT entry matching the partition to flash
    let pit = download_protocol::download_pit(&mut conn).unwrap();
    let partition_name: String = args.value_of_t_or_exit("partition");
    let pit_entry = pit
        .get_entry_by_name(&partition_name)
        .expect("A partition by that name could not be found! Make sure it exists");

    // TODO: Do this in a more efficient way than loading everything into RAM
    let path = args
        .value_of("filename")
        .expect("Required argument not set! This is probably a clap bug.");
    let path = Path::new(path);
    let mut f = File::open(path).unwrap();
    let mut data: Vec<u8> = Vec::new();
    f.read_to_end(&mut data).unwrap();

    download_protocol::flash(&mut conn, params, &data, pit_entry).unwrap();

    let reboot: bool = args.value_of_t_or_exit("reboot");
    download_protocol::end_session(&mut conn, reboot).unwrap();
}

fn get_upload_communicator(args: &ArgMatches) -> Result<Box<dyn Communicator>> {
    let transport = args
        .value_of("transport")
        .expect("Transport must have been set! This is probably clap bug.");
    match transport {
        "usb" => {
            let conn = UsbConnection::establish()?;
            return Ok(Box::new(conn));
        }
        "net" => {
            let conn = NetConnectConnection::new(WIRELESS_TARGET_IP, WIRELESS_PORT);
            return Ok(Box::new(conn));
        }
        _ => panic!("Unexpected invalid transport! This should've been caught by clap."),
    }
}

fn upload_mode(args: &ArgMatches) {
    match args.subcommand() {
        Some(("dump", sub_args)) => upload_mode_dump(sub_args),
        Some(("probe", sub_args)) => upload_mode_probe(sub_args),
        _ => panic!("Unexpected missing subcommand! This should've been caught by clap."),
    }
}

fn upload_mode_dump(args: &ArgMatches) {
    let mut conn: Box<dyn Communicator> = get_upload_communicator(args).unwrap();
    println!("[DEBUG] Target connected!");

    upload_protocol::handshake(&mut conn).unwrap();

    let start_addr: u64 = args.value_of_t_or_exit("start-address");
    let end_addr: u64 = args.value_of_t_or_exit("end-address");
    let data = upload_protocol::dump(&mut conn, start_addr, end_addr).unwrap();

    // Write to file
    // TODO: OS strings may be more appropriate here
    let path = args
        .value_of("filename")
        .expect("Required argument not set! This is probably a clap bug.");
    let path = Path::new(path);
    let mut f = File::create(path).unwrap();
    f.write_all(&data).unwrap();

    upload_protocol::end_session(&mut conn).unwrap();
}

fn upload_mode_probe(args: &ArgMatches) {
    let mut conn: Box<dyn Communicator> = get_upload_communicator(args).unwrap();

    upload_protocol::handshake(&mut conn).unwrap();

    let table = upload_protocol::probe(&mut conn).unwrap();
    println!("{:?}", table);

    upload_protocol::end_session(&mut conn).unwrap();
}
