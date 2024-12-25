use self::{control::Control, table::CompositionTable};
use crate::{
    app::computers::{CompositionComputed, CompositionKey},
    localization::localize,
};
use egui::{RichText, Ui, Visuals, util::hash};
use egui_phosphor::regular::{CHART_BAR, GEAR};
use plot::CompositionPlot;
use polars::prelude::*;
use serde::{Deserialize, Serialize};

/// Composition pane
#[derive(Default, Deserialize, Serialize)]
pub(crate) struct Pane {
    pub(crate) source: DataFrame,
    pub(crate) target: DataFrame,
    pub(crate) control: Control,
    pub(crate) plot: bool,
}

impl Pane {
    pub fn new(data_frame: DataFrame) -> Self {
        Self {
            source: data_frame,
            target: DataFrame::empty(),
            control: Control::new(),
            plot: false,
        }
    }

    pub(crate) fn header(&mut self, ui: &mut Ui) {
        ui.visuals_mut().button_frame = false;
        ui.toggle_value(&mut self.control.open, RichText::new(GEAR).heading());
        ui.visuals_mut().widgets.hovered = Visuals::default().widgets.hovered;
        self.plot ^= ui
            .button(RichText::new(CHART_BAR).heading())
            .on_hover_text(localize!("visualization"))
            .clicked();
    }

    pub(crate) fn content(&mut self, ui: &mut Ui) {
        ui.separator();
        self.control.windows(ui);
        self.target = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<CompositionComputed>()
                .get(CompositionKey {
                    data_frame: &self.source,
                    settings: &self.control.confirmed,
                })
        });
        if self.plot {
            CompositionPlot::new(&self.target).ui(ui);
        } else {
            CompositionTable::new(&self.target, &self.control.confirmed).ui(ui);
        }
    }

    pub(super) fn hash(&self) -> u64 {
        // hash(&self.source)
        0
    }
}

pub(crate) mod control;

mod plot;
mod table;
mod widgets;
