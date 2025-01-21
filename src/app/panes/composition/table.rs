use super::{ID_SOURCE, Settings, State};
use crate::{
    app::{ResultExt, panes::MARGIN, text::Text, widgets::FloatWidget},
    special::composition::{MC, NC, PMC, PNC, PSC, PTC, PUC, SC, SMC, SNC, SSC, STC, SUC, TC, UC},
};
use egui::{Frame, Id, Margin, Response, TextStyle, Ui};
use egui_table::{
    AutoSizeMode, CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState,
};
use polars::prelude::*;
use std::ops::Range;

const INDEX: Range<usize> = 0..1;

/// Composition table
pub(super) struct TableView<'a> {
    data_frame: &'a DataFrame,
    settings: &'a Settings,
    state: &'a mut State,
    // is_row_expanded: BTreeMap<u64, bool>,
    // prefetched: Vec<PrefetchInfo>,
}

impl<'a> TableView<'a> {
    pub(crate) fn new(
        data_frame: &'a DataFrame,
        settings: &'a Settings,
        state: &'a mut State,
    ) -> Self {
        Self {
            data_frame,
            settings,
            state,
        }
    }
}

impl TableView<'_> {
    pub(super) fn show(&mut self, ui: &mut Ui) {
        let id_salt = Id::new(ID_SOURCE).with("Table");
        if self.state.reset_table_state {
            let id = TableState::id(ui, Id::new(id_salt));
            TableState::reset(ui.ctx(), id);
            self.state.reset_table_state = false;
        }
        let height = ui.text_style_height(&TextStyle::Heading);
        let num_rows = self.data_frame.height() as u64 + 1;
        let num_columns = self.settings.confirmed.groups.len() * 2 + 1;
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
                    let composition = self.settings.confirmed.groups[index].composition;
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
                let indices = self.data_frame["Index"].u32()?;
                let index = indices.get(row).unwrap();
                ui.label(index.to_string());
            }
            (row, column) => {
                let index = (column.start + 1) / 2 - 1;
                if column.start % 2 == 1 {
                    let keys = self.data_frame["Keys"].struct_()?;
                    let key = &keys.fields_as_series()[index];
                    match self.settings.confirmed.groups[index].composition {
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
                        // SC => {
                        //     ui.label(key.str_value(row)?);
                        // }
                        SC | PSC | SSC => {
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
                    // self.data_frame["Experimental"]
                    //     .struct_()?
                    //     .field_by_name("Triacylglycerol")?
                    self.value(
                        ui,
                        self.data_frame["Values"].as_materialized_series(),
                        Some(row),
                        index,
                        self.settings.percent,
                    )?;
                    // FloatWidget::new(|| {
                    //     let Some(values) = self.data_frame["Values"].list()?.get_as_series(row)
                    //     else {
                    //         return Ok(None);
                    //     };
                    //     Ok(values.f64()?.get(index))
                    // })
                    // .percent(self.settings.percent)
                    // .precision(Some(self.settings.precision))
                    // .hover()
                    // .show(ui);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn footer_cell_content_ui(&mut self, ui: &mut Ui, column: Range<usize>) -> PolarsResult<()> {
        // // Last
        // let index = self.settings.groups.len();
        // if column.start == index * 2 {
        //     let composition = self.data_frame[index].struct_()?;
        //     FloatWidget::new(|| Ok(composition.field_by_name("Value")?.f64()?.sum()))
        //         .percent(self.settings.percent)
        //         .precision(Some(self.settings.precision))
        //         .hover()
        //         .show(ui);
        // }
        Ok(())
    }

    fn value(
        &self,
        ui: &mut Ui,
        series: &Series,
        row: Option<usize>,
        index: usize,
        percent: bool,
    ) -> PolarsResult<()> {
        let list = series.list()?;
        Ok(match list.inner_dtype() {
            DataType::Float64 => {
                FloatWidget::new(|| {
                    Ok(if let Some(row) = row {
                        let Some(values) = list.get_as_series(row) else {
                            return Ok(None);
                        };
                        values.f64()?.get(index)
                    } else {
                        let Some(values) = list.get_as_series(0) else {
                            return Ok(None);
                        };
                        values.f64()?.sum()
                    })
                })
                .percent(percent)
                .precision(Some(self.settings.precision))
                .hover()
                .show(ui);
            }
            DataType::Struct(_) => {
                FloatWidget::new(|| {
                    Ok(if let Some(row) = row {
                        let Some(mean) = list.get_as_series(row) else {
                            return Ok(None);
                        };
                        mean.struct_()?.field_by_name("Mean")?.f64()?.get(index)
                    } else {
                        let Some(mean) = list.get_as_series(0) else {
                            return Ok(None);
                        };
                        mean.struct_()?.field_by_name("Mean")?.f64()?.sum()
                    })
                })
                .percent(percent)
                .precision(Some(self.settings.precision))
                .hover()
                .show(ui);
                ui.label("±");
                FloatWidget::new(|| {
                    Ok(if let Some(row) = row {
                        let Some(standard_deviation) = list.get_as_series(row) else {
                            return Ok(None);
                        };
                        standard_deviation
                            .struct_()?
                            .field_by_name("StandardDeviation")?
                            .f64()?
                            .get(index)
                    } else {
                        let Some(standard_deviation) = list.get_as_series(0) else {
                            return Ok(None);
                        };
                        standard_deviation
                            .struct_()?
                            .field_by_name("StandardDeviation")?
                            .f64()?
                            .sum()
                    })
                })
                .percent(percent)
                .precision(Some(self.settings.precision))
                .hover()
                .show(ui);
            }
            _ => unreachable!(),
        })
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
                    .context(ui.ctx())
            });
    }
}
