use super::control::Settings;
use crate::{
    app::{MARGIN, text::Text, widgets::FloatWidget},
    special::composition::{MC, NC, PMC, PNC, PSC, PTC, PUC, SC, SMC, SNC, SSC, STC, SUC, TC, UC},
};
use egui::{Frame, Id, Margin, TextStyle, Ui};
use egui_table::{AutoSizeMode, CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate};
use polars::prelude::*;
use std::ops::Range;

const INDEX: Range<usize> = 0..1;

/// Composition table
pub(super) struct TableView<'a> {
    data_frame: &'a DataFrame,
    settings: &'a Settings,
    // is_row_expanded: BTreeMap<u64, bool>,
    // prefetched: Vec<PrefetchInfo>,
}

impl<'a> TableView<'a> {
    pub(crate) fn new(data_frame: &'a DataFrame, settings: &'a Settings) -> Self {
        Self {
            data_frame,
            settings,
        }
    }
}

impl TableView<'_> {
    pub(super) fn show(&mut self, ui: &mut Ui) {
        let id_salt = Id::new("CompositionTable");
        let height = ui.text_style_height(&TextStyle::Heading);
        let num_rows = self.data_frame.height() as u64 + 1;
        let num_columns = self.settings.groups.len() * 2 + 1;
        let top = vec![0..1, 1..num_columns];
        let mut middle = vec![0..1];
        const STEP: usize = 2;
        for index in (1..num_columns).step_by(STEP) {
            middle.push(index..index + STEP);
        }
        Table::new()
            .id_salt(id_salt)
            .num_rows(num_rows)
            .columns(vec![
                Column::default().resizable(self.settings.resizable);
                num_columns
            ])
            .num_sticky_cols(self.settings.sticky_columns)
            .headers([
                HeaderRow {
                    height,
                    groups: top,
                },
                HeaderRow {
                    height,
                    groups: middle,
                },
                HeaderRow::new(height),
            ])
            .auto_size_mode(AutoSizeMode::OnParentResize)
            .show(ui, self);
    }

    fn header_cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: Range<usize>) {
        match (row, column) {
            (0, INDEX) => {
                ui.heading("Index");
            }
            (0, _) => {
                ui.heading("Compositions");
            }
            (1, column) => {
                if column.start % 2 == 1 {
                    let index = column.start / 2;
                    let composition = self.settings.groups[index].composition;
                    ui.heading(composition.text())
                        .on_hover_text(composition.hover_text());
                } else if column.start != 0 {
                    ui.heading("Value");
                }
            }
            (2, column) => {
                if column.start % 2 == 1 {
                    ui.heading("Key");
                } else if column.start != 0 {
                    ui.heading("Value");
                }
            }
            _ => {}
        }
    }

    fn cell_content_ui(
        &mut self,
        ui: &mut Ui,
        row: usize,
        column: Range<usize>,
    ) -> PolarsResult<()> {
        if row != self.data_frame.height() {
            self.body_cell_content_ui(ui, row, column)?;
        } else {
            self.footer_cell_content_ui(ui, column)?;
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
                ui.label(row.to_string());
            }
            (row, column) => {
                let index = (column.start + 1) / 2;
                let composition = self.data_frame[index].struct_()?;
                if column.start % 2 == 1 {
                    let key = composition.field_by_name("Key")?;
                    match self.settings.groups[index - 1].composition {
                        MC => {
                            FloatWidget::new(|| Ok(key.f64()?.get(row)))
                                .hover()
                                .show(ui);
                        }
                        PMC => {
                            ui.label(key.str_value(row)?);
                        }
                        SMC => {
                            ui.label(key.str_value(row)?);
                        }
                        NC => {
                            ui.label(key.str_value(row)?);
                        }
                        PNC => {
                            ui.label(key.str_value(row)?);
                        }
                        SNC => {
                            ui.label(key.str_value(row)?);
                        }
                        TC | PTC | STC => {
                            let sn1 = key.struct_()?.field_by_name("StereospecificNumber1")?;
                            let sn2 = key.struct_()?.field_by_name("StereospecificNumber2")?;
                            let sn3 = key.struct_()?.field_by_name("StereospecificNumber3")?;
                            let r#type = |series: Series| -> PolarsResult<&str> {
                                let saturated = series.bool()?.get(row).unwrap();
                                Ok(if saturated { "S" } else { "U" })
                            };
                            ui.label(format!("{}{}{}", r#type(sn1)?, r#type(sn2)?, r#type(sn3)?));
                        }
                        SC => {
                            ui.label(key.str_value(row)?);
                        }
                        PSC | SSC => {
                            let sn1 = key.struct_()?.field_by_name("StereospecificNumber1")?;
                            let sn2 = key.struct_()?.field_by_name("StereospecificNumber2")?;
                            let sn3 = key.struct_()?.field_by_name("StereospecificNumber3")?;
                            let r#type = |series: Series| -> PolarsResult<String> {
                                Ok(series.str_value(row)?.to_string())
                            };
                            ui.label(format!(
                                "{}-{}-{}",
                                r#type(sn1)?,
                                r#type(sn2)?,
                                r#type(sn3)?,
                            ));
                        }
                        UC => {
                            ui.label(key.str_value(row)?);
                        }
                        PUC => {
                            ui.label(key.str_value(row)?);
                        }
                        SUC => {
                            ui.label(key.str_value(row)?);
                        }
                    }
                } else {
                    FloatWidget::new(|| Ok(composition.field_by_name("Value")?.f64()?.get(row)))
                        .percent(self.settings.percent)
                        .precision(Some(self.settings.precision))
                        .hover()
                        .show(ui);
                }
            }
        }
        Ok(())
    }

    fn footer_cell_content_ui(&mut self, ui: &mut Ui, column: Range<usize>) -> PolarsResult<()> {
        // Last
        let index = self.settings.groups.len();
        if column.start == index * 2 {
            let composition = self.data_frame[index].struct_()?;
            FloatWidget::new(|| Ok(composition.field_by_name("Value")?.f64()?.sum()))
                .percent(self.settings.percent)
                .precision(Some(self.settings.precision))
                .hover()
                .show(ui);
        }
        Ok(())
    }
}

impl TableDelegate for TableView<'_> {
    fn header_cell_ui(&mut self, ui: &mut Ui, cell: &HeaderCellInfo) {
        ui.painter()
            .rect_filled(ui.max_rect(), 0.0, ui.visuals().faint_bg_color);
        Frame::none()
            .inner_margin(Margin::symmetric(MARGIN.x, MARGIN.y))
            .show(ui, |ui| {
                self.header_cell_content_ui(ui, cell.row_nr, cell.col_range.clone())
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
                self.cell_content_ui(ui, cell.row_nr as _, cell.col_nr..cell.col_nr + 1)
                    .unwrap()
            });
    }
}
