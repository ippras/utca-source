use crate::{app::MAX_PRECISION, localization::localize};
use egui::{Grid, Slider, Ui, Window};
use egui_phosphor::regular::GEAR;
use serde::{Deserialize, Serialize};

/// Configuration control
#[derive(Default, Deserialize, Serialize)]
pub(crate) struct Control {
    pub(crate) settings: Settings,
    pub(crate) index: usize,
    pub(crate) open: bool,
}

impl Control {
    pub(crate) const fn new() -> Self {
        Self {
            settings: Settings::new(),
            index: 0,
            open: false,
        }
    }

    pub(crate) fn windows(&mut self, ui: &mut Ui) {
        Window::new(format!("{GEAR} Configuration settings"))
            .id(ui.next_auto_id())
            .default_pos(ui.next_widget_position())
            .open(&mut self.open)
            .show(ui.ctx(), |ui| self.settings.ui(ui));
    }
}

/// Configuration settings
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    #[serde(skip)]
    pub(crate) resizable: bool,
    #[serde(skip)]
    pub(crate) editable: bool,
    pub(crate) precision: usize,

    pub(crate) names: bool,
    pub(crate) properties: bool,
}

impl Settings {
    pub(crate) const fn new() -> Self {
        Self {
            resizable: false,
            editable: false,
            precision: 0,
            names: true,
            properties: true,
        }
    }

    pub(crate) fn ui(&mut self, ui: &mut Ui) {
        Grid::new("configuration").show(ui, |ui| {
            // Precision
            ui.label(localize!("precision"));
            ui.add(Slider::new(&mut self.precision, 0..=MAX_PRECISION));
            ui.end_row();

            ui.separator();
            ui.separator();
            ui.end_row();

            // Properties
            ui.label(localize!("properties"));
            ui.checkbox(&mut self.properties, "")
                .on_hover_text(localize!("properties_description"));
            ui.end_row();

            // Names
            ui.label(localize!("names"));
            ui.checkbox(&mut self.names, "")
                .on_hover_text(localize!("names_description"));
        });
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}
