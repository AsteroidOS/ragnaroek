//! This crate provides an egui-based graphical frontend for ragnaroek.

mod tabs;

use eframe::egui::{self, RichText};
use std::sync::{Arc, Mutex};

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

#[derive(Default)]
struct RagnaroekApp {
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

impl eframe::App for RagnaroekApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::TopBottomPanel::top("title").show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                // Label displaying connection status
                if self.shared_session.lock().unwrap().is_some() {
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
    }
}
