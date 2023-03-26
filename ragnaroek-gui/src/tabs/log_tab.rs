use eframe::egui;

pub struct LogTab {}

impl LogTab {
    pub fn new() -> LogTab {
        egui_logger::init().unwrap();
        return LogTab {};
    }

    /// Run it's logic and draw the log tab's UI.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        egui_logger::logger_ui(ui);
    }
}
