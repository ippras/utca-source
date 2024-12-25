use egui::{Frame, Id, Margin, TextStyle, Ui};
use egui_table::{AutoSizeMode, CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate};
use polars::prelude::*;

use super::control::Settings;
use crate::{
    app::{MARGIN, text::Text, widgets::FloatValue},
    special::polars::column::ColumnExt as _,
};

const INDEX: usize = 0;
const COMPOSITION: usize = 1;
const FA: usize = 2;
const TIME: usize = 3;
const ECL: usize = 4;
const ECN: usize = 5;
const MASS: usize = 6;

/// Composition table
pub(super) struct CompositionTable<'a> {
    data_frame: &'a DataFrame,
    settings: &'a Settings,
    // is_row_expanded: BTreeMap<u64, bool>,
    // prefetched: Vec<PrefetchInfo>,
}

impl<'a> CompositionTable<'a> {
    pub(super) fn new(data_frame: &'a DataFrame, settings: &'a Settings) -> Self {
        Self {
            data_frame,
            settings,
        }
    }

    pub(super) fn ui(&mut self, ui: &mut Ui) {
        let id_salt = Id::new("CompositionTable");
        let height = ui.text_style_height(&TextStyle::Heading);
        let num_rows = self.data_frame.height() as u64 + 1;
        let num_columns = self.data_frame.width() * 2 + 1;
        let mut groups = vec![0..1];
        const STEP: usize = 2;
        for index in (1..num_columns).step_by(STEP) {
            groups.push(index..index + STEP);
        }
        Table::new()
            .id_salt(id_salt)
            .num_rows(num_rows)
            .columns(vec![Column::default().resizable(true); num_columns])
            .num_sticky_cols(self.settings.sticky_columns)
            .headers([HeaderRow { height, groups }, HeaderRow::new(height)])
            .auto_size_mode(AutoSizeMode::OnParentResize)
            .show(ui, self);
    }

    fn header_cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: usize) {
        let settings = &self.settings;
        match (row, column) {
            (0, INDEX) => {
                ui.heading("Index");
            }
            (0, column) => {
                let composition = settings.groups[column - 1].composition;
                ui.heading(composition.text());
            }
            (1, INDEX) => {}
            (1, column) => {
                if column % 2 == 1 {
                    ui.heading("Key");
                } else {
                    ui.heading("Value");
                }
            }
            _ => unreachable!(),
        }
    }

    fn body_cell_content_ui(&mut self, ui: &mut Ui, row: usize, col: usize) {
        let settings = &self.settings;
        if row == self.data_frame.height() {
            if col == self.data_frame.width() * 2 {
                let index = (col - 1) / 2;
                let compositions = self.data_frame[index].compositions();
                let sum = compositions.sum().unwrap();
                ui.add(
                    FloatValue::new(Some(sum.mean))
                        .percent(settings.percent)
                        .precision(Some(settings.precision))
                        .hover(),
                );
                ui.label("±");
                ui.add(
                    FloatValue::new(Some(sum.standard_deviation))
                        .percent(settings.percent)
                        .precision(Some(settings.precision))
                        .hover(),
                );
            }
            return;
        }
        match (row, col) {
            (row, INDEX) => {
                ui.label(row.to_string());
            }
            (row, column) => {
                let index = (column - 1) / 2;
                let composition = self.data_frame[index].struct_().unwrap();
                if column % 2 == 1 {
                    let keys = composition.field_by_name("Key").unwrap();
                    let key = keys.str_value(row).unwrap();
                    ui.label(key);
                } else {
                    let values = composition.field_by_name("Value").unwrap();
                    let value = values.struct_().unwrap();
                    let means = value.field_by_name("Mean").unwrap();
                    let standard_deviations = value.field_by_name("StandardDeviation").unwrap();
                    ui.add(
                        FloatValue::new(means.f64().unwrap().get(row))
                            .percent(settings.percent)
                            .precision(Some(settings.precision))
                            .hover(),
                    );
                    ui.label("±");
                    ui.add(
                        FloatValue::new(standard_deviations.f64().unwrap().get(row))
                            .percent(settings.percent)
                            .precision(Some(settings.precision))
                            .hover(),
                    );
                }
            }
        }
    }
}

impl TableDelegate for CompositionTable<'_> {
    fn header_cell_ui(&mut self, ui: &mut Ui, cell: &HeaderCellInfo) {
        Frame::none()
            .inner_margin(Margin::symmetric(MARGIN.x, MARGIN.y))
            .show(ui, |ui| {
                self.header_cell_content_ui(ui, cell.row_nr, cell.group_index)
            });
    }

    fn cell_ui(&mut self, ui: &mut Ui, cell: &CellInfo) {
        if cell.row_nr % 2 == 1 {
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, ui.visuals().faint_bg_color);
        }
        Frame::none()
            .inner_margin(Margin::symmetric(MARGIN.x, MARGIN.y))
            .show(ui, |ui| {
                self.body_cell_content_ui(ui, cell.row_nr as _, cell.col_nr)
            });
    }
}
