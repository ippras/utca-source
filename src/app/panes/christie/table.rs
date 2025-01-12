use crate::app::{
    MARGIN,
    widgets::{FattyAcidWidget, FloatWidget},
};
use egui::{Frame, Id, Margin, Response, TextStyle, TextWrapMode, Ui, Vec2, vec2};
use egui_phosphor::regular::MINUS;
use egui_table::{AutoSizeMode, CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate};
use lipid::fatty_acid::polars::{DataFrameExt as _, SeriesExt as _};
use polars::prelude::*;
use std::ops::Range;

const INDEX: Range<usize> = 0..1;
const FA: Range<usize> = INDEX.end..INDEX.end + 1;
const FACTOR: Range<usize> = FA.end..FA.end + 1;
const LEN: usize = FACTOR.end;

/// Table view
pub(super) struct TableView<'a> {
    data_frame: &'a DataFrame,
    // metadata: &'a Metadata,
}

impl<'a> TableView<'a> {
    pub(super) fn new(data_frame: &'a DataFrame) -> Self {
        Self { data_frame }
    }
}

impl TableView<'_> {
    pub(super) fn ui(&mut self, ui: &mut Ui) {
        let id_salt = Id::new("Christie");
        let height = ui.text_style_height(&TextStyle::Heading);
        let num_rows = self.data_frame.height() as _;
        let num_columns = LEN;
        Table::new()
            .id_salt(id_salt)
            .num_rows(num_rows)
            .columns(vec![Column::default(); num_columns])
            .headers([HeaderRow::new(height)])
            .auto_size_mode(AutoSizeMode::Never)
            .show(ui, self);
    }

    fn header_cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: Range<usize>) {
        match (row, column) {
            // Top
            (0, INDEX) => {
                ui.heading("Index");
            }
            (0, FA) => {
                ui.heading("FA");
            }
            (0, FACTOR) => {
                ui.heading("Christie");
            }
            _ => {} // _ => unreachable!(),
        };
    }

    fn cell_content_ui(
        &mut self,
        ui: &mut Ui,
        row: usize,
        column: Range<usize>,
    ) -> PolarsResult<()> {
        if !self.data_frame.is_empty() {
            self.body_cell_content_ui(ui, row, column)?;
        }
        Ok(())
    }

    fn body_cell_content_ui(
        &mut self,
        ui: &mut Ui,
        row: usize,
        column: Range<usize>,
    ) -> PolarsResult<()> {
        match (row, column) {
            (row, INDEX) => {
                let indices = self.data_frame["Index"].u32()?;
                let index = indices.get(row).unwrap();
                ui.label(index.to_string());
            }
            (row, FA) => {
                FattyAcidWidget::new(|| self.data_frame.fatty_acid().get(row))
                    .hover()
                    .ui(ui)?;
            }
            (row, FACTOR) => {
                FloatWidget::new(|| Ok(self.data_frame["Christie"].f64()?.get(row)))
                    .hover()
                    .show(ui);
            }
            _ => {} // _ => unreachable!(),
        }
        Ok(())
    }
}

impl TableDelegate for TableView<'_> {
    fn header_cell_ui(&mut self, ui: &mut Ui, cell: &HeaderCellInfo) {
        Frame::none()
            .inner_margin(Margin::symmetric(MARGIN.x, MARGIN.y))
            .show(ui, |ui| {
                self.header_cell_content_ui(ui, cell.row_nr, cell.col_range.clone())
            });
    }

    fn cell_ui(&mut self, ui: &mut Ui, cell: &CellInfo) {
        if cell.row_nr % 2 == 0 {
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, ui.visuals().faint_bg_color);
        }
        Frame::none()
            .inner_margin(Margin::symmetric(MARGIN.x, MARGIN.y))
            .show(ui, |ui| {
                self.cell_content_ui(ui, cell.row_nr as _, cell.col_nr..cell.col_nr + 1)
                    .unwrap()
            });
    }
}
