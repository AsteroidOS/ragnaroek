use eframe::egui;

use crate::shared_conn::*;

pub struct ConnectTab {}

impl ConnectTab {
    pub fn new() -> ConnectTab {
        return ConnectTab {};
    }

    /// Run it's logic and draw the connection tab's UI.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // Button bar for connecting to device
        ui.horizontal(|ui| {
            if ui.button("Connect (USB)").clicked() {
                send_cmd(ConnectionRequest::Connect(CommsMode::Usb));
            }
            if ui.button("Connect (Net Bind)").clicked() {
                send_cmd(ConnectionRequest::Connect(CommsMode::NetBind));
            }
            if ui.button("Connect (Net Connect)").clicked() {
                send_cmd(ConnectionRequest::Connect(CommsMode::NetConnect));
            }
            if ui.button("Disconnect").clicked() {
                send_cmd(ConnectionRequest::Disconnect);
            }
        });
    }
}
