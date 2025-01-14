use self::{control::Control, table::TableView};
use super::PaneDelegate;
use crate::{
    app::computers::{CalculationComputed, CalculationKey},
    localize,
};
use egui::{CursorIcon, Id, Response, RichText, ScrollArea, Ui, menu::bar, util::hash};
use egui_phosphor::regular::{
    ARROWS_CLOCKWISE, ARROWS_HORIZONTAL, CALCULATOR, GEAR, INTERSECT_THREE, LIST,
};
use metadata::MetaDataFrame;
use polars::prelude::*;
use serde::{Deserialize, Serialize};

/// Calculation pane
#[derive(Deserialize, Serialize)]
pub(crate) struct Pane {
    pub(crate) source: Vec<MetaDataFrame>,
    pub(crate) target: DataFrame,
    pub(crate) control: Control,
}

impl Pane {
    pub const fn new(frames: Vec<MetaDataFrame>, index: usize) -> Self {
        Self {
            source: frames,
            target: DataFrame::empty(),
            control: Control {
                index: Some(index),
                ..Control::new()
            },
        }
    }

    pub(crate) fn title(&self) -> String {
        match self.control.index {
            Some(index) => self.source[index].meta.title(),
            None => localize!("calculation"),
        }
    }

    fn header_content(&mut self, ui: &mut Ui) -> Response {
        let mut response = ui
            .heading(CALCULATOR)
            .on_hover_text(localize!("calculation"));
        response |= ui.heading(self.title());
        response = response
            .on_hover_text(format!("{:x}", self.hash()))
            .on_hover_cursor(CursorIcon::Grab);
        ui.separator();
        // List
        ui.menu_button(RichText::new(LIST).heading(), |ui| {
            let mut clicked = false;
            for index in 0..self.source.len() {
                clicked |= ui
                    .selectable_value(
                        &mut self.control.index,
                        Some(index),
                        self.source[index].meta.title(),
                    )
                    .clicked()
            }
            ui.selectable_value(&mut self.control.index, None, "Mean ± standard deviations");
            if clicked {
                ui.close_menu();
            }
        })
        .response
        .on_hover_text(localize!("list"));
        ui.separator();
        // Reset
        if ui
            .button(RichText::new(ARROWS_CLOCKWISE).heading())
            .clicked()
        {
            self.control.settings.reset = true;
        }
        // Resize
        ui.toggle_value(
            &mut self.control.settings.resizable,
            RichText::new(ARROWS_HORIZONTAL).heading(),
        )
        .on_hover_text(localize!("resize"));
        // Settings
        ui.toggle_value(&mut self.control.open, RichText::new(GEAR).heading());
        ui.separator();
        // Composition
        if ui
            .button(RichText::new(INTERSECT_THREE).heading())
            .on_hover_text(localize!("composition"))
            .clicked()
        {
            let mut target = Vec::with_capacity(self.source.len());
            for index in 0..self.source.len() {
                let meta = self.source[index].meta.clone();
                let data = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<CalculationComputed>()
                        .get(CalculationKey {
                            frames: &self.source,
                            index: &Some(index),
                            settings: &self.control.settings,
                        })
                });
                target.push(MetaDataFrame::new(meta, data));
            }
            ui.data_mut(|data| data.insert_temp(Id::new("Compose"), (target, self.control.index)));
        }
        ui.separator();
        response
    }

    fn body_content(&mut self, ui: &mut Ui) {
        self.target = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<CalculationComputed>()
                .get(CalculationKey {
                    frames: &self.source,
                    index: &self.control.index,
                    settings: &self.control.settings,
                })
        });
        TableView::new(&mut self.target, &self.control.settings).show(ui)
    }

    fn hash(&self) -> u64 {
        hash(&self.source)
    }
}

impl PaneDelegate for Pane {
    fn header(&mut self, ui: &mut Ui) -> Response {
        bar(ui, |ui| {
            ScrollArea::horizontal()
                .show(ui, |ui| {
                    ui.visuals_mut().button_frame = false;
                    self.header_content(ui)
                })
                .inner
        })
        .inner
    }

    fn body(&mut self, ui: &mut Ui) {
        self.control.windows(ui);
        self.body_content(ui);
    }
}

pub(crate) mod control;

mod table;
