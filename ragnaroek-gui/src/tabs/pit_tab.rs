use eframe::egui;
use egui_extras::TableBuilder;
use pit::Pit;
use ragnaroek;
use rfd;

use crate::SharedSession;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

type PitReceiver = mpsc::Receiver<ragnaroek::Result<Pit>>;
type OpenDialogReceiver = mpsc::Receiver<Option<PathBuf>>;

pub struct PitTab {
    /// PIT to render right now
    pit: Option<Pit>,
    /// Channel for receiving PIT from device
    pit_rx: Option<PitReceiver>,
    /// Channel for receiving path to a PIT file chosen from disk
    open_dialog_rx: Option<OpenDialogReceiver>,
    /// ragnaroek session with a device, shared among tabs
    shared_session: SharedSession,
}

impl PitTab {
    pub fn new(shared_session: SharedSession) -> PitTab {
        return PitTab {
            pit: None,
            pit_rx: None,
            open_dialog_rx: None,
            shared_session,
        };
    }

    /// Download PIT file from the device.
    /// Emits a message into the `self.pit_rx` channel when done.
    pub fn start_pit_download_from_device(&mut self) {
        let (send, recv) = mpsc::channel();
        let s_cloned = self.shared_session.clone();
        thread::spawn(move || {
            // TODO: Handle not acquiring session lock
            let mut s_locked = s_cloned.lock().unwrap();
            let s_locked = s_locked.deref_mut();
            // TODO: Handle None session
            let s_locked = s_locked.as_mut().unwrap();
            let pit_data = match s_locked.download_pit(s_locked.params) {
                Ok(d) => d,
                Err(e) => {
                    send.send(Err(e)).unwrap();
                    return;
                }
            };
            let pit = match Pit::deserialize(&pit_data) {
                Ok(pit) => pit,
                Err(e) => {
                    send.send(Err(ragnaroek::Error::PitError(e))).unwrap();
                    return;
                }
            };
            send.send(Ok(pit)).unwrap();
        });
        self.pit_rx = Some(recv);
    }

    /// Ask the user to pick a PIT file.
    /// Emits a message into the `self.open_dialog_rx` channel when done.
    pub fn show_open_dialog(&mut self) {
        let (send, recv) = mpsc::channel();
        thread::spawn(move || {
            let path = rfd::FileDialog::new()
                .add_filter("PIT", &["pit"])
                .pick_file();
            send.send(path).unwrap();
        });
        self.open_dialog_rx = Some(recv);
    }

    fn draw_pit_v1_table(&self, ui: &mut egui::Ui, pit: Pit) {
        ui.add_space(20.0);
        // Not part of table, but related
        ui.horizontal(|ui| {
            ui.heading("Gang Name: ");
            ui.monospace(pit.gang_name());
            ui.heading("Project Name: ");
            ui.monospace(pit.project_name());
            ui.heading("PIT version: ");
            ui.monospace("1");
        });
        let headings = [
            "Type",
            "Device Type",
            "Partition ID",
            "Attributes",
            "Update Attributes",
            "Block Size",
            "Block Count",
            "File Offset",
            "File Size",
            "Partition Name",
            "Flash Filename",
            "FOTA Filename",
        ];
        TableBuilder::new(ui)
            .resizable(true)
            .columns(egui_extras::Column::remainder(), headings.len())
            .header(60.0, |mut header| {
                for heading in headings {
                    header.col(|ui| {
                        ui.heading(heading);
                    });
                }
            })
            .body(|mut body| {
                for entry in pit.0.left().unwrap() {
                    body.row(25.0, |mut row| {
                        for text in [
                            format!("{}", entry.pit_type),
                            format!("{}", entry.pit_device_type),
                            format!("{}", entry.partition_id),
                            format!("{:?}", entry.pit_attributes),
                            format!("{:?}", entry.pit_update_attributes),
                            format!("{}", entry.block_size),
                            format!("{}", entry.block_count),
                            format!("{}", entry.file_offset),
                            format!("{}", entry.file_size),
                            entry.partition_name.clone(),
                            entry.flash_filename.clone(),
                            entry.fota_filename.clone(),
                        ] {
                            row.col(|ui| {
                                ui.label(text);
                            });
                        }
                    });
                }
            });
    }

    fn draw_pit_v2_table(&self, ui: &mut egui::Ui, pit: Pit) {
        ui.add_space(20.0);
        // Not part of table, but related
        ui.horizontal(|ui| {
            ui.heading("Gang Name: ");
            ui.monospace(pit.gang_name());
            ui.heading("Project Name: ");
            ui.monospace(pit.project_name());
            ui.heading("PIT version: ");
            ui.monospace("2");
        });
        let headings = [
            "Type",
            "Device Type",
            "Partition ID",
            "Partition Type",
            "PIT Filesystem",
            "Start Block",
            "Block Count",
            "File Offset",
            "File Size",
            "Partition Name",
            "Flash Filename",
            "FOTA Filename",
        ];
        TableBuilder::new(ui)
            .resizable(true)
            .columns(egui_extras::Column::remainder(), headings.len())
            .header(60.0, |mut header| {
                for heading in headings {
                    header.col(|ui| {
                        ui.heading(heading);
                    });
                }
            })
            .body(|mut body| {
                for entry in pit.0.right().unwrap() {
                    body.row(25.0, |mut row| {
                        for text in [
                            format!("{}", entry.pit_type),
                            format!("{}", entry.pit_device_type),
                            format!("{}", entry.partition_id),
                            format!("{}", entry.partition_type),
                            format!("{}", entry.pit_filesystem),
                            format!("{}", entry.start_block),
                            format!("{}", entry.block_num),
                            format!("{}", entry.file_offset),
                            format!("{}", entry.file_size),
                            entry.partition_name.clone(),
                            entry.flash_filename.clone(),
                            entry.fota_filename.clone(),
                        ] {
                            row.col(|ui| {
                                ui.label(text);
                            });
                        }
                    });
                }
            });
    }

    /// Run it's logic and draw the PIT tab's UI.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // Button bar for PIT import
        ui.horizontal(|ui| {
            if ui.button("Parse PIT from file").clicked() {
                self.show_open_dialog();
            }

            let dl_btn = egui::Button::new("Download PIT from device");
            if self.shared_session.lock().unwrap().is_none() {
                // Can't download if no device is connected
                ui.add_enabled(false, dl_btn);
            } else {
                if ui.add_enabled(true, dl_btn).clicked() {
                    self.start_pit_download_from_device();
                }
            }

            if ui.button("Clear").clicked() {
                self.pit = None;
            }
        });

        // Has the user chosen a new PIT file to open?
        if self.open_dialog_rx.is_some() {
            match self.open_dialog_rx.as_ref().unwrap().try_recv() {
                // Nothing in the channel yet
                Err(_) => {}
                // User selected a file
                Ok(Some(path)) => {
                    let pit = std::fs::read(path).unwrap();
                    let pit = Pit::deserialize(&pit).unwrap();
                    self.pit = Some(pit);
                    self.open_dialog_rx = None;
                }
                // User cancelled
                Ok(None) => {
                    self.open_dialog_rx = None;
                }
            }
        }

        // Has a PIT file arrived from the device?
        if self.pit_rx.is_some() {
            match self.pit_rx.as_ref().unwrap().try_recv() {
                // Nothing in the channel yet
                Err(_) => {}

                // File download failed
                Ok(Err(_)) => {
                    // TODO: Handle (show error, maybe offer retry)
                    self.pit_rx = None;
                }
                // File download succeeded
                Ok(Ok(pit)) => {
                    self.pit = Some(pit);
                    self.pit_rx = None;
                }
            }
        }

        // Render PIT contents
        match &self.pit {
            None => {
                ui.heading("Open or download a PIT to display");
            }
            Some(pit) => {
                if pit.0.clone().left().is_some() {
                    self.draw_pit_v1_table(ui, pit.clone());
                } else {
                    self.draw_pit_v2_table(ui, pit.clone());
                }
            }
        }
    }
}
