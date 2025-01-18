use crate::{app::MAX_PRECISION, localize};
use egui::{Grid, Slider, Ui};
use serde::{Deserialize, Serialize};

/// Configuration settings
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) index: usize,

    pub(crate) resizable: bool,
    pub(crate) editable: bool,
    pub(crate) precision: usize,
    pub(crate) sticky: usize,
    pub(crate) truncate: bool,

    pub(crate) names: bool,
    pub(crate) properties: bool,
}

impl Settings {
    pub(crate) const fn new() -> Self {
        Self {
            index: 0,
            resizable: false,
            editable: false,
            precision: 2,
            sticky: 0,
            truncate: false,
            names: true,
            properties: true,
        }
    }

    pub(crate) fn show(&mut self, ui: &mut Ui) {
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
