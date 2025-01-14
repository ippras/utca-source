use self::{control::Control, table::TableView};
use super::PaneDelegate;
use crate::{
    app::computers::{CompositionComputed, CompositionKey},
    localize,
};
use egui::{CursorIcon, Response, RichText, ScrollArea, Ui, Visuals, menu::bar, util::hash};
use egui_phosphor::regular::{ARROWS_HORIZONTAL, CHART_BAR, GEAR, INTERSECT_THREE, LIST};
use metadata::MetaDataFrame;
use plot::PlotView;
use polars::prelude::*;
use serde::{Deserialize, Serialize};

/// Composition pane
#[derive(Default, Deserialize, Serialize)]
pub(crate) struct Pane {
    pub(crate) source: Vec<MetaDataFrame>,
    pub(crate) target: DataFrame,
    pub(crate) control: Control,
    pub(crate) plot: bool,
}

impl Pane {
    pub fn new(frames: Vec<MetaDataFrame>, index: Option<usize>) -> Self {
        Self {
            source: frames,
            target: DataFrame::empty(),
            control: Control::new(index),
            plot: false,
        }
    }

    pub(crate) fn title(&self) -> String {
        match self.control.index {
            Some(index) => self.source[index].meta.title(),
            None => localize!("composition"),
        }
    }

    fn header_content(&mut self, ui: &mut Ui) -> Response {
        let mut response = ui
            .heading(INTERSECT_THREE)
            .on_hover_text(localize!("composition"));
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
        // Resize
        ui.toggle_value(
            &mut self.control.confirmed.resizable,
            RichText::new(ARROWS_HORIZONTAL).heading(),
        )
        .on_hover_text(localize!("resize"));
        // Settings
        ui.toggle_value(&mut self.control.open, RichText::new(GEAR).heading());
        // View
        ui.visuals_mut().widgets.hovered = Visuals::default().widgets.hovered;
        self.plot ^= ui
            .button(RichText::new(CHART_BAR).heading())
            .on_hover_text(localize!("visualization"))
            .clicked();
        response
    }

    fn body_content(&mut self, ui: &mut Ui) {
        self.target = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<CompositionComputed>()
                .get(CompositionKey {
                    frames: &self.source,
                    index: &self.control.index,
                    settings: &self.control.confirmed,
                })
        });
        // if self.plot {
        //     PlotView::new(&self.target).ui(ui);
        // } else {
        TableView::new(&self.target, &self.control.confirmed).show(ui);
        // }
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
        ui.separator();
        self.control.windows(ui);
        self.body_content(ui);
    }
}

pub(crate) mod control;

mod plot;
mod table;
mod widgets;
