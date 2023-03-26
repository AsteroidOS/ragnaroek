use crate::SharedSession;

use eframe::egui;
use ragnaroek::download_protocol;

use std::thread;

/// All the Odin .ini files I could find only ever mention this port
const WIRELESS_PORT: u16 = 13579;
/// All the targets implementing wireless mode seem to use this IP
const WIRELESS_TARGET_IP: &str = "192.168.49.1";

/// How to communicate with the target.

#[derive(PartialEq, Default, Copy, Clone)]
enum CommsMode {
    #[default]
    Usb,
    NetBind,
    NetConnect,
}

pub struct ConnectTab {
    /// ragnaroek session with a device, shared among tabs
    shared_session: SharedSession,
}

fn connect(m: CommsMode) -> ragnaroek::Result<Box<dyn ragnaroek::Communicator>> {
    use CommsMode::*;
    match m {
        Usb => {
            let c = ragnaroek::UsbConnection::establish()?;
            return Ok(Box::new(c));
        }
        NetBind => {
            let mut l = ragnaroek::NetBindListener::new(WIRELESS_PORT);
            let c = l.accept()?;
            return Ok(Box::new(c));
        }
        NetConnect => {
            let c = ragnaroek::NetConnectConnection::new(WIRELESS_TARGET_IP, WIRELESS_PORT);
            return Ok(Box::new(c));
        }
    }
}

impl ConnectTab {
    pub fn new(shared_session: SharedSession) -> ConnectTab {
        return ConnectTab { shared_session };
    }

    fn connect_and_detect(&mut self, m: CommsMode) {
        let shared_session_clone = self.shared_session.clone();
        thread::spawn(move || {
            let conn = connect(m).unwrap();
            let sess = download_protocol::Session::begin(conn).unwrap();
            let mut locked = shared_session_clone.lock().unwrap();
            locked.replace(sess);
        });
    }

    /// Run it's logic and draw the connection tab's UI.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // Button bar for connecting to device
        ui.horizontal(|ui| {
            if ui.button("Connect (USB)").clicked() {
                self.connect_and_detect(CommsMode::Usb);
            }
            if ui.button("Connect (Net Bind)").clicked() {
                self.connect_and_detect(CommsMode::NetBind);
            }
            if ui.button("Connect (Net Connect)").clicked() {
                self.connect_and_detect(CommsMode::NetConnect);
            }
        });
    }
}
