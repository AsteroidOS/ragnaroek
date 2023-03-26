use eframe::egui;
use pit::Pit;

use crate::SharedSession;

pub struct ConnectTab {
    /// ragnaroek session with a device, shared among tabs
    shared_session: SharedSession,
}

impl ConnectTab {
    pub fn new(shared_session: SharedSession) -> ConnectTab {
        return ConnectTab { shared_session };
    }

    /// Run it's logic and draw the connection tab's UI.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // TODO: Button bar for connecting to device
        ui.horizontal(|ui| {});
    }
}
