use eframe::egui;

pub struct LogTab {}

impl LogTab {
    pub fn new() -> LogTab {
        let ok = egui_logger::builder().init();
        if let Err(e) = ok {
            log::error!(target: "GUI", "Failed to initialize logger tab: {}", e);
        }
        return LogTab {};
    }

    /// Run it's logic and draw the log tab's UI.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        egui_logger::logger_ui().show(ui);
    }
}
