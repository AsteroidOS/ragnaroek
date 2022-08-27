//! This crate provides an egui-based graphical frontend for ragnaroek.

mod pit_ui;

use eframe::egui;
use pit;
use ragnaroek;
use std::{
    fs,
    path::PathBuf,
    sync::mpsc::{self, TryRecvError},
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
    conn: Option<Box<dyn ragnaroek::Communicator>>,
    pit_dialog_receiver: Option<mpsc::Receiver<Option<PathBuf>>>,
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

impl eframe::App for RagnaroekApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.heading("Ragnaroek");
            });

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        // Allow user to pick how to communicate with device
                        ui.heading("Communications mode");
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
                    // Button for establishing connection
                    if ui.button("Detect").clicked() {
                        let c = connect(self.comms_mode).unwrap();
                        self.conn = Some(c);
                    }

                    // Label displaying connection status
                    if self.conn.is_some() {
                        ui.colored_label(egui::Color32::GREEN, "Connected: YES ✓");
                    } else {
                        ui.colored_label(egui::Color32::RED, "Connected: NO ❌");
                    }
                });

                // Print PIT from device
                let print_pit_btn = egui::Button::new("Print PIT");
                if ui.add_enabled(self.conn.is_some(), print_pit_btn).clicked() {
                    let conn = self.conn.as_mut().unwrap();
                    self.rendering_pit = Some(pit_ui::download(ui, conn));
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
                // If there's a PIT file to render, do it
                if self.rendering_pit.is_some() {
                    let p = self.rendering_pit.as_ref().unwrap();
                    pit_ui::draw_table(ui, p.clone());
                }
            });
        });
    }
}
