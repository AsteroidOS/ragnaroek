mod pit_tab;
use pit_tab::PitTab;
mod connect_tab;
use connect_tab::ConnectTab;

use eframe::egui;
use egui_dock::{NodeIndex, Tree};

use crate::SharedSession;

pub struct Tabs {
    tree: Tree<String>,
    pit_tab: PitTab,
    connect_tab: ConnectTab,
}

impl Tabs {
    pub fn new(s: SharedSession) -> Self {
        let pit_tab_title = "PIT".to_string();
        let pit_tab = PitTab::new(s.clone());
        let connect_tab_title = "Connect".to_string();
        let connect_tab = ConnectTab::new(s);

        let mut tree = Tree::new(vec![pit_tab_title]);
        tree.split_left(NodeIndex::root(), 0.50, vec![connect_tab_title]);

        Self {
            tree,
            pit_tab,
            connect_tab,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let style = egui_dock::Style::from_egui(ui.style().as_ref());
        egui_dock::DockArea::new(&mut self.tree)
            .style(style)
            .show_inside(
                ui,
                &mut TabViewer {
                    pit_tab: &mut self.pit_tab,
                    connect_tab: &mut self.connect_tab,
                },
            );
    }
}

pub struct TabViewer<'a> {
    pit_tab: &'a mut PitTab,
    connect_tab: &'a mut ConnectTab,
}

impl<'a> egui_dock::TabViewer for TabViewer<'a> {
    type Tab = String;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab.as_str() {
            "PIT" => {
                self.pit_tab.ui(ui);
            }
            "Connect" => {
                self.connect_tab.ui(ui);
            }
            _ => {
                ui.label(format!("Content of {tab}"));
            }
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        (&*tab).into()
    }
}
