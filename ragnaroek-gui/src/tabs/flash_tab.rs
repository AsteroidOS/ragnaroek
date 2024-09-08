use crate::SharedSession;
use eframe::egui;
use pit::{Either, Pit};

use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

type OpenDialogReceiver = mpsc::Receiver<Option<PathBuf>>;

pub struct FlashTab {
    /// PIT downloaded from the device
    pit: Option<Pit>,
    /// ragnaroek session with a device, shared among tabs
    shared_session: SharedSession,
    /// Channel for receiving path to a PIT file chosen from disk
    open_pit_dialog_rx: Option<OpenDialogReceiver>,
}

impl FlashTab {
    pub fn new(shared_session: SharedSession) -> FlashTab {
        return FlashTab {
            pit: None,
            shared_session,
            open_pit_dialog_rx: None,
        };
    }

    fn start_flash(&mut self) {
        /*
        // Start separate thread for actually performing the flash
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
        */
    }

    /// Ask the user to pick a file.
    /// Emits a message into the `self.open_dialog_rx` channel when done.
    fn show_pit_open_dialog(&mut self) {
        let (send, recv) = mpsc::channel();
        thread::spawn(move || {
            let path = rfd::FileDialog::new()
                .add_filter("PIT", &["pit"])
                .pick_file();
            send.send(path).unwrap();
        });
        self.open_pit_dialog_rx = Some(recv);
    }

    /// Run it's logic and draw the connection tab's UI.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Read PIT from file
            if ui.button("Use PIT from file...").clicked() {
                self.show_pit_open_dialog();
            }

            // Read PIT from device
            if ui.button("Use PIT from device").clicked() {
                self.show_pit_open_dialog();
            }
            let dl_btn = egui::Button::new("Use PIT from device");
            if ui
                .add_enabled(self.shared_session.lock().unwrap().is_some(), dl_btn)
                .clicked()
            {
                // self.start_pit_download_from_device();
                unimplemented!();
            }

            // TODO: Add button for downloading / reading PIT
            // Show a partition selector, if PIT has already been downloaded / selected.
            // Disable combo box if not
            let selected: String = "".to_string();
            let cbox =
                egui::ComboBox::from_label("Partition to flash").selected_text(selected.clone());
            cbox.show_ui(ui, |ui| {
                if self.pit.is_some() {
                    let pit = self.pit.clone().unwrap().0;
                    match pit {
                        Either::Left(v1) => {
                            for entry in v1 {
                                ui.selectable_value(
                                    &mut selected.clone(),
                                    entry.partition_name.clone(),
                                    entry.partition_name,
                                );
                            }
                        }
                        Either::Right(v2) => {
                            for entry in v2 {
                                ui.selectable_value(
                                    &mut selected.clone(),
                                    entry.partition_name.clone(),
                                    entry.partition_name,
                                );
                            }
                        }
                    };
                }
            });

            // Offer button to flash the device if conditions are met
            let device_connected = self.shared_session.lock().unwrap().is_some();
            let can_flash = self.pit.is_some() && device_connected;
            let flash_btn = egui::Button::new("Flash");
            if ui.add_enabled(can_flash, flash_btn).clicked() {
                // TODO: self.start_flash();
            }

            ui.horizontal_centered(|ui| {
                // TODO: Don't show unless we're flashing
                let bar = egui::widgets::ProgressBar::new(0.0)
                    .text("Flashing progress")
                    .show_percentage();
                ui.add(bar);
            });
            // TODO: Provide ability to flash multiple partitions at once / use Odin tar files
        });
    }
}
