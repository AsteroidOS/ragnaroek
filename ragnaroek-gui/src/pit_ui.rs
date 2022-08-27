use eframe::egui;
use egui_extras::TableBuilder;
use pit;
use ragnaroek;
use rfd;

use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

pub fn download(c: &mut Box<dyn ragnaroek::Communicator>) -> pit::Pit {
    ragnaroek::download_protocol::begin_session(c).unwrap();
    let pit = ragnaroek::download_protocol::download_pit(c).unwrap();
    return pit;
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

/// Draw the PIT contents as a table.
pub fn draw_table(ui: &mut egui::Ui, pit: pit::Pit) {
    ui.add_space(20.0);
    // Not part of table, but related
    ui.horizontal(|ui| {
        ui.heading("Gang Name: ");
        ui.monospace(pit.gang_name.clone());
        ui.heading("Project Name: ");
        ui.monospace(pit.project_name.clone());
        ui.heading("Version: ");
        ui.monospace(format!("{}", pit.proto_version));
    });
    let headings = [
        "Type",
        "Device Type",
        "ID",
        "Attributes",
        "Update Attributes",
        "Block Size / Offset",
        "Block Count",
        "File Offset",
        "File Size",
        "Partition Name",
        "Flash Filename",
        "FOTA Filename",
    ];
    TableBuilder::new(ui)
        .resizable(true)
        .columns(egui_extras::Size::remainder(), headings.len())
        .header(60.0, |mut header| {
            for heading in headings {
                header.col(|ui| {
                    ui.heading(heading);
                });
            }
        })
        .body(|mut body| {
            for entry in pit {
                body.row(25.0, |mut row| {
                    for text in [
                        format!("{}", entry.pit_type),
                        format!("{}", entry.pit_device_type),
                        format!("{}", entry.pit_id),
                        format!("{:?}", entry.pit_attributes),
                        format!("{:?}", entry.pit_update_attributes),
                        format!("{}", entry.block_size_or_offset),
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
