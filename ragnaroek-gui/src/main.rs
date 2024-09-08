//! This crate provides an egui-based graphical frontend for ragnaroek.

mod shared_conn;
mod tabs;

use eframe::egui::{self, RichText};

fn main() {
    let mut native_options = eframe::NativeOptions::default();
    native_options.follow_system_theme = true;
    eframe::run_native(
        "Ragnaroek",
        native_options,
        Box::new(|cc| Ok(Box::new(RagnaroekApp::new(cc)))),
    )
    .unwrap();
}

#[derive(Default)]
struct RagnaroekApp {
    tabs: Option<tabs::Tabs>,
}

impl RagnaroekApp {
    fn new(_: &eframe::CreationContext<'_>) -> Self {
        shared_conn::start();

        let mut s = Self::default();
        s.tabs = Some(tabs::Tabs::new());
        log::info!(target: "GUI", "Ragnaroek GUI started.");
        return s;
    }
}

impl eframe::App for RagnaroekApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        log::trace!(target: "GUI", "Updating GUI");
        egui::TopBottomPanel::top("title").show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                // Label displaying connection status
                if shared_conn::device_is_connected() {
                    ui.heading(
                        RichText::new("Device connected: YES ☑").color(egui::Color32::GREEN),
                    );
                } else {
                    ui.heading(RichText::new("Device connected: NO ❌").color(egui::Color32::RED));
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            self.tabs.as_mut().unwrap().ui(ui);
        });
        log::trace!(target: "GUI", "GUI updated");
    }
}
