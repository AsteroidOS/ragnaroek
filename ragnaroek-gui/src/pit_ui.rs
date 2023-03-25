use eframe::egui;
use egui_extras::TableBuilder;
use pit;
use pit::Pit;
use ragnaroek;
use rfd;

use crate::SharedSession;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

/// Download PIT file from the device.
/// Does not block.
/// Instead, result is returned as a message on the returned channel when ready.
pub fn start_download(s: SharedSession) -> mpsc::Receiver<ragnaroek::Result<pit::Pit>> {
    let (send, recv) = mpsc::channel();
    thread::spawn(move || {
        let mut s_locked = s.lock().unwrap();
        let s_locked = s_locked.deref_mut();
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
    return recv;
}

/// Ask the user to pick a PIT file.
/// Does not block.
/// Instead, result is returned as a message on the returned channel when ready.
/// If a `None` is received over the channel, the user cancelled.
pub fn open_dialog() -> mpsc::Receiver<Option<PathBuf>> {
    let (send, recv) = mpsc::channel();
    thread::spawn(move || {
        let path = rfd::FileDialog::new()
            .add_filter("PIT", &["pit"])
            .pick_file();
        send.send(path).unwrap();
    });
    return recv;
}

fn draw_pitv1_table(ui: &mut egui::Ui, pit: pit::Pit) {
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

fn draw_pitv2_table(ui: &mut egui::Ui, pit: pit::Pit) {
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

/// Draw the PIT contents as a table.
pub fn draw_table(ui: &mut egui::Ui, pit: pit::Pit) {
    if pit.0.clone().left().is_some() {
        draw_pitv1_table(ui, pit);
    } else {
        draw_pitv2_table(ui, pit);
    }
}
