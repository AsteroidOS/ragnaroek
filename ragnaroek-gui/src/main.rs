//! This crate provides an egui-based graphical frontend for ragnaroek.

mod pit_ui;

use eframe::egui::{self, RichText};
use pit;
use ragnaroek::{self, download_protocol};
use std::{
    fs,
    path::PathBuf,
    sync::mpsc::{self, TryRecvError},
    thread,
};

/// All the Odin .ini files I could find only ever mention this port
const WIRELESS_PORT: u16 = 13579;
/// All the targets implementing wireless mode seem to use this IP
const WIRELESS_TARGET_IP: &str = "192.168.49.1";

fn main() {
    let mut native_options = eframe::NativeOptions::default();
    native_options.follow_system_theme = true;
    eframe::run_native(
        "Ragnaroek",
        native_options,
        Box::new(|cc| Box::new(RagnaroekApp::new(cc))),
    );
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
    comms_mode_radio_enabled: bool,
    comm: Option<Box<dyn ragnaroek::Communicator>>,
    pit_dialog_receiver: Option<mpsc::Receiver<Option<PathBuf>>>,
    comm_receiver: Option<mpsc::Receiver<ragnaroek::Result<Box<dyn ragnaroek::Communicator>>>>,
    rendering_pit: Option<pit::Pit>,
}

impl RagnaroekApp {
    fn new(_: &eframe::CreationContext<'_>) -> Self {
        let mut s = Self::default();
        s.comms_mode_radio_enabled = true;
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

fn connect_and_detect(
    m: CommsMode,
) -> mpsc::Receiver<ragnaroek::Result<Box<dyn ragnaroek::Communicator>>> {
    let (send, recv) = mpsc::channel();
    thread::spawn(move || {
        let mut c = connect(m).unwrap();
        download_protocol::magic_handshake(&mut c).unwrap();
        download_protocol::begin_session(&mut c).unwrap();
        download_protocol::end_session(&mut c, false).unwrap();
        send.send(Ok(c)).unwrap();
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

        egui::TopBottomPanel::bottom("actions").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // If we're in the middle of connection establishment, check if it's done
                if self.comm_receiver.is_some() {
                    let r = self.comm_receiver.as_mut().unwrap();
                    match r.try_recv() {
                        // Have a Communicator
                        Ok(Ok(comm)) => {
                            self.comm_receiver = None;
                            ui.set_enabled(true);
                            self.comm = Some(comm);
                        }
                        // Error occurred while obtaining communicator
                        Ok(Err(err)) => panic!("{:?}", err),
                        // Nothing happened yet, keep waiting
                        Err(TryRecvError::Empty) => {
                            ui.set_enabled(false);
                        }
                        _ => panic!("Unexpected state while reading communicator channel"),
                    }
                }
                // Button for establishing connection
                if ui.button("Connect to device").clicked() {
                    // Because devices take forever to respond, this function runs it in a separate thread.
                    self.comm_receiver = Some(connect_and_detect(self.comms_mode));
                }

                // Print PIT from device
                let print_pit_btn = egui::Button::new("Parse PIT from device");
                if ui.add_enabled(self.comm.is_some(), print_pit_btn).clicked() {
                    let conn = self.comm.as_mut().unwrap();
                    self.rendering_pit = Some(pit_ui::download(conn));
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
                            let pit = fs::read(path).unwrap();
                            let pit = pit::Pit::deserialize(&pit).unwrap();
                            self.rendering_pit = Some(pit);
                        }
                        // User cancelled, we're done
                        Ok(None) => {
                            self.pit_dialog_receiver = None;
                        }
                        // Nothing happened yet, keep waiting
                        Err(TryRecvError::Empty) => {}
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

        egui::CentralPanel::default().show(ctx, |ui| {
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
                        if ui
                            .add_enabled(self.comms_mode_radio_enabled, rbtn)
                            .clicked()
                        {
                            self.comms_mode = mode;
                        }
                    }
                });
                // If there's a PIT file to render, do it
                if self.rendering_pit.is_some() {
                    let p = self.rendering_pit.as_ref().unwrap();
                    pit_ui::draw_table(ui, p.clone());
                }
            });
        });
    }
}
