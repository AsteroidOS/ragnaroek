//! This crate provides an egui-based graphical frontend for ragnaroek.

mod tabs;

use eframe::egui::{self, RichText};
use ragnaroek::{self, download_protocol};
use std::{
    sync::mpsc::{self},
    sync::{Arc, Mutex},
    thread,
};

/// All the Odin .ini files I could find only ever mention this port
const WIRELESS_PORT: u16 = 13579;
/// All the targets implementing wireless mode seem to use this IP
const WIRELESS_TARGET_IP: &str = "192.168.49.1";

pub type SharedSession = Arc<Mutex<Option<ragnaroek::download_protocol::Session>>>;

fn main() {
    let mut native_options = eframe::NativeOptions::default();
    native_options.follow_system_theme = true;
    eframe::run_native(
        "Ragnaroek",
        native_options,
        Box::new(|cc| Box::new(RagnaroekApp::new(cc))),
    )
    .unwrap();
}

/// How to communicate with the target.

#[derive(PartialEq, Default, Copy, Clone)]
enum CommsMode {
    #[default]
    Usb,
    NetBind,
    NetConnect,
}

#[derive(Default)]
struct RagnaroekApp {
    comms_mode: CommsMode,
    comm: Option<SharedSession>,
    comm_receiver: Option<mpsc::Receiver<ragnaroek::Result<SharedSession>>>,
    tabs: Option<tabs::Tabs>,
    shared_session: SharedSession,
}

impl RagnaroekApp {
    fn new(_: &eframe::CreationContext<'_>) -> Self {
        let mut s = Self::default();
        s.tabs = Some(tabs::Tabs::new(s.shared_session.clone()));
        return s;
    }
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

fn connect_and_detect(m: CommsMode) -> mpsc::Receiver<ragnaroek::Result<SharedSession>> {
    let (send, recv) = mpsc::channel();
    thread::spawn(move || {
        let c = connect(m).unwrap();
        let s = download_protocol::Session::begin(c).unwrap();
        send.send(ragnaroek::Result::Ok(Arc::new(Mutex::new(Some(s)))))
            .unwrap();
    });
    return recv;
}

impl eframe::App for RagnaroekApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::TopBottomPanel::top("title").show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                // Label displaying connection status
                if self.comm_receiver.is_some() {
                    ui.heading(
                        RichText::new("Device connected: Connecting...")
                            .color(egui::Color32::YELLOW),
                    );
                } else if self.comm.is_some() {
                    ui.heading(
                        RichText::new("Device connected: YES ☑").color(egui::Color32::GREEN),
                    );
                } else {
                    ui.heading(RichText::new("Device connected: NO ❌").color(egui::Color32::RED));
                }
            });
        });

        /*
        egui::TopBottomPanel::bottom("actions").show(ctx, |ui| {
            ui.set_enabled(self.interaction_enabled);
            ui.horizontal(|ui| {
                // If we're in the middle of connection establishment, check if it's done
                if self.comm_receiver.is_some() {
                    let r = self.comm_receiver.as_mut().unwrap();
                    match r.try_recv() {
                        // Have a Communicator
                        Ok(Ok(comm)) => {
                            self.comm_receiver = None;
                            self.interaction_enabled = true;
                            self.comm = Some(comm);
                        }
                        // Error occurred while obtaining communicator
                        Ok(Err(err)) => panic!("{:?}", err),
                        // Nothing happened yet, keep waiting
                        Err(TryRecvError::Empty) => {
                            self.interaction_enabled = false;
                        }
                        _ => panic!("Unexpected state while reading communicator channel"),
                    }
                }
                // Button for establishing connection
                if ui.button("Connect to device").clicked() {
                    // Because devices take forever to respond, this function runs it in a separate thread.
                    self.comm_receiver = Some(connect_and_detect(self.comms_mode));
                }

                // If we're in the middle of PIT download, check if it's done
                if self.pit_receiver.is_some() {
                    let r = self.pit_receiver.as_mut().unwrap();
                    match r.try_recv() {
                        Ok(Ok(pit)) => {
                            self.pit_receiver = None;
                            self.interaction_enabled = true;
                            self.rendering_pit = Some(pit);
                        }
                        // Error occurred while obtaining PIT
                        Ok(Err(err)) => panic!("{:?}", err),
                        // Nothing happened yet, keep waiting
                        Err(TryRecvError::Empty) => {
                            self.interaction_enabled = false;
                        }
                        _ => panic!("Unexpected state while reading PIT download channel"),
                    }
                }

                // Print PIT from device
                let print_pit_btn = egui::Button::new("Parse PIT from device");
                if ui.add_enabled(self.comm.is_some(), print_pit_btn).clicked() {
                    // Start thread in background for downloading PIT
                    self.pit_receiver =
                        Some(pit_ui::start_download(self.comm.clone().unwrap().clone()));
                }

                // Parse PIT from file
                if ui.button("Parse PIT from file").clicked() {
                    self.pit_dialog_receiver = Some(pit_ui::open_dialog());
                }
                // In order to avoid blocking the main thread, file dialog is displayed in a worker thread.
                // Wait for it's result here.
                if self.pit_dialog_receiver.is_some() {
                    let r = self.pit_dialog_receiver.as_mut().unwrap();
                    match r.try_recv() {
                        // Have path, set PIT to render
                        Ok(Some(path)) => {
                            self.pit_dialog_receiver = None;
                            self.interaction_enabled = true;
                            let pit = fs::read(path).unwrap();
                            let pit = pit::Pit::deserialize(&pit).unwrap();
                            self.rendering_pit = Some(pit);
                        }
                        // User cancelled, we're done
                        Ok(None) => {
                            self.pit_dialog_receiver = None;
                            self.interaction_enabled = true;
                        }
                        // Nothing happened yet, keep waiting
                        Err(TryRecvError::Empty) => {
                            self.interaction_enabled = false;
                        }
                        // Disconnection, shouldn't happen
                        Err(TryRecvError::Disconnected) => {
                            panic!("Disconnect");
                        }
                    }
                }

                // Clear PIT display
                if ui.button("Clear PIT display").clicked() {
                    self.rendering_pit = None;
                }
            });
        });
        */

        egui::CentralPanel::default().show(ctx, |ui| {
            self.tabs.as_mut().unwrap().ui(ui);
            /*
            ui.set_enabled(self.interaction_enabled);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    // Allow user to pick how to communicate with device
                    ui.heading("Communications mode: ");
                    for (mode, text) in [
                        (CommsMode::Usb, "USB"),
                        (CommsMode::NetBind, "Net (Bind)"),
                        (CommsMode::NetConnect, "Net (Connect)"),
                    ] {
                        let rbtn = egui::RadioButton::new(self.comms_mode == mode, text);
                        if ui.add_enabled(self.interaction_enabled, rbtn).clicked() {
                            self.comms_mode = mode;
                        }
                    }
                });
            });
            */
        });
    }
}
