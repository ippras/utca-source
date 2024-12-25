use crate::{app::MAX_PRECISION, localization::localize};
use egui::{Grid, RichText, Slider, Ui, Window};
use egui_phosphor::regular::GEAR;
use serde::{Deserialize, Serialize};

/// Visualization control
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct Control {
    pub(crate) settings: Settings,
    pub(crate) index: Option<usize>,
    pub(crate) open: bool,
}

impl Control {
    pub(crate) const fn new() -> Self {
        Self {
            settings: Settings::new(),
            index: None,
            open: false,
        }
    }

    pub(crate) fn windows(&mut self, ui: &mut Ui) {
        Window::new(format!("{GEAR} Visualization settings"))
            .id(ui.next_auto_id())
            .default_pos(ui.next_widget_position())
            .open(&mut self.open)
            .show(ui.ctx(), |ui| {
                self.settings.ui(ui);
            });
    }
}

/// Visualization settings
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) precision: usize,
}

impl Settings {
    pub(crate) const fn new() -> Self {
        Self { precision: 0 }
    }

    pub(crate) fn ui(&mut self, ui: &mut Ui) {
        ui.visuals_mut().collapsing_header_frame = true;
        ui.collapsing(RichText::new(localize!("configuration")).heading(), |ui| {
            Grid::new("configuration").show(ui, |ui| {
                ui.label(localize!("precision"));
                ui.add(Slider::new(&mut self.precision, 0..=MAX_PRECISION));
                ui.end_row();

                ui.separator();
                ui.separator();
                ui.end_row();
            });
        });
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}
