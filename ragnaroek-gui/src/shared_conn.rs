//! Implements a shared asynchronous abstraction over a device connection,
//! which can be used by all tabs without blocking or callback hell.

use std::sync::{mpsc, Arc, Mutex, MutexGuard};

use bus::Bus;
pub use bus::BusReader;
use lazy_static::lazy_static;
use log;
use pit::Pit;
use ragnaroek::download_protocol::DownloadProtocolError;
use ragnaroek::Result;

/// All the Odin .ini files I could find only ever mention this port
const WIRELESS_PORT: u16 = 13579;
/// All the targets implementing wireless mode seem to use this IP
const WIRELESS_TARGET_IP: &str = "192.168.49.1";

lazy_static! {
    // Connection singleton
    static ref SHARED_CONNECTION: Arc<Mutex<SharedConnection>> =
        Arc::new(Mutex::new(SharedConnection::new()));
}

/// How to communicate with the target.
#[derive(PartialEq, Default, Copy, Clone, Debug)]
pub enum CommsMode {
    #[default]
    Usb,
    NetBind,
    NetConnect,
}

/// A request to the connection thread.
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum ConnectionRequest {
    /// Request to connect to the device.
    Connect(CommsMode),
    /// Request to initiate a download mode session.
    BeginDLSession,
    /// Request to disconnect from the device.
    Disconnect,
    /// Request to download the PIT file from the device.
    DownloadPit,
}

/// An event that occurred on the connection.
#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    /// A connection was established.
    Connected,
    /// A connection was lost.
    Disconnected,
    /// A PIT file was received.
    PitReceived(Result<Pit>),
}

struct SharedConnection {
    /// Channel for sending commands to this connection.
    pub cmd_send: mpsc::Sender<ConnectionRequest>,
    pub cmd_chan: mpsc::Receiver<ConnectionRequest>,

    /// Bus for disseminating connection events to tabs.
    pub event_bus: bus::Bus<ConnectionEvent>,
    /// The current download mode session, if any.
    pub dl_sess: Option<ragnaroek::download_protocol::Session>,
    /// The current communicator, if any.
    pub comm: Option<Box<dyn ragnaroek::Communicator>>,
}

impl SharedConnection {
    pub fn new() -> SharedConnection {
        let (cmd_send, cmd_chan) = mpsc::channel();
        let event_bus = Bus::new(32);

        return SharedConnection {
            cmd_chan,
            cmd_send,
            event_bus,
            dl_sess: None,
            comm: None,
        };
    }
}

fn process_cmd(cmd: ConnectionRequest) -> bool {
    println!("Received command: {:?}", cmd);
    let mut sc = SHARED_CONNECTION.lock().unwrap();
    match cmd {
        ConnectionRequest::Connect(m) => {
            log::info!(target: "GUI", "Connecting to device in mode {:?}...", m);
            use CommsMode::*;
            sc.comm = match m {
                Usb => {
                    let c = ragnaroek::UsbConnection::establish().unwrap();
                    Some(Box::new(c))
                }
                NetBind => {
                    let mut l = ragnaroek::NetBindListener::new(WIRELESS_PORT);
                    let c = l.accept().unwrap();
                    Some(Box::new(c))
                }
                NetConnect => {
                    let c = ragnaroek::NetConnectConnection::new(WIRELESS_TARGET_IP, WIRELESS_PORT);
                    Some(Box::new(c))
                }
            };
            log::info!(target: "GUI", "Connected!");
            sc.event_bus.broadcast(ConnectionEvent::Connected);
        }
        ConnectionRequest::BeginDLSession => {
            log::info!(target: "GUI", "Beginning download mode session...");
            if sc.comm.is_some() {
                // Communicator is consumed here, so we can't use it again
                let comm = std::mem::take(&mut sc.comm).unwrap();
                sc.dl_sess = Some(ragnaroek::download_protocol::Session::begin(comm).unwrap());
            }
            log::info!(target: "GUI", "Download mode session begun!");
        }
        ConnectionRequest::Disconnect => {
            log::info!(target: "GUI", "Disconnecting...");
            let comm = std::mem::take(&mut sc.comm);
            drop(comm);
            sc.event_bus.broadcast(ConnectionEvent::Disconnected);
            log::info!(target: "GUI", "Disconnected!");
        }
        ConnectionRequest::DownloadPit => {
            log::info!(target: "GUI", "Downloading PIT file...");
            if let Some(c) = sc.dl_sess.as_mut() {
                let pit_bytes = c.download_pit(c.params);
                match pit_bytes {
                    Ok(pit) => match Pit::deserialize(&pit) {
                        Ok(pit) => {
                            sc.event_bus
                                .broadcast(ConnectionEvent::PitReceived(Ok(pit)));
                        }
                        Err(e) => {
                            sc.event_bus.broadcast(ConnectionEvent::PitReceived(Err(
                                DownloadProtocolError::InvalidPitFile(e).into(),
                            )));
                            return false;
                        }
                    },
                    Err(e) => {
                        sc.event_bus.broadcast(ConnectionEvent::PitReceived(Err(e)));
                        return false;
                    }
                }
                log::info!(target: "GUI", "PIT file downloaded!");
            } else {
                // TODO: Deal with session being None
                log::warn!(target: "GUI", "Attempted to download PIT file without a session, ignoring!");
            }
        }
    }
    return true;
}

fn process_cmds() {
    loop {
        let mut sc = SHARED_CONNECTION.lock().unwrap();
        match sc.cmd_chan.try_recv() {
            Ok(cmd) => {
                drop(sc); // Drop the lock before processing the command
                if !process_cmd(cmd) {
                    break;
                }
            }
            Err(mpsc::TryRecvError::Empty) => {
                // Give up the lock if there are no commands to process
                drop(sc);
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                log::error!(target: "GUI", "Connection thread channel disconnected!");
                break;
            }
        }
    }
    log::error!(target: "GUI", "Connection thread exited!");
}

/// Send a command to the global shared connection.
pub fn send_cmd(msg: ConnectionRequest) {
    let shared_conn = SHARED_CONNECTION.lock().unwrap();
    shared_conn.cmd_send.send(msg).unwrap();
}

/// Get a reference to the event bus for the global shared connection.
pub fn get_event_bus() -> BusReader<ConnectionEvent> {
    let mut shared_conn = SHARED_CONNECTION.lock().unwrap();
    return shared_conn.event_bus.add_rx();
}

/// Check if a device is connected.
pub fn device_is_connected() -> bool {
    let shared_conn = SHARED_CONNECTION.lock().unwrap();
    return shared_conn.comm.is_some();
}

/// Start the connection thread.
pub fn start() {
    std::thread::spawn(|| {
        process_cmds();
    });
}
