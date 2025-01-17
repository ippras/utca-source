use super::{ContextExt as _, control::Settings};
use crate::app::{
    panes::MARGIN,
    widgets::{FattyAcidWidget, FloatWidget},
};
use egui::{Context, Frame, Id, Margin, Response, TextStyle, TextWrapMode, Ui};
use egui_phosphor::regular::{MINUS, PLUS};
use egui_table::{AutoSizeMode, CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate};
use lipid::fatty_acid::{
    FattyAcid,
    polars::{DataFrameExt as _, SeriesExt as _},
};
use polars::{chunked_array::builder::AnonymousOwnedListBuilder, prelude::*};
use std::ops::Range;

const INDEX: Range<usize> = 0..1;
const LABEL: Range<usize> = INDEX.end..INDEX.end + 1;
const FA: Range<usize> = LABEL.end..LABEL.end + 1;
const TAG: Range<usize> = FA.end..FA.end + 1;
const DAG1223: Range<usize> = TAG.end..TAG.end + 1;
const MAG2: Range<usize> = DAG1223.end..DAG1223.end + 1;
const LEN: usize = MAG2.end;

/// Table view
pub(super) struct TableView<'a> {
    data_frame: &'a mut DataFrame,
    settings: &'a Settings,
    event: Option<Event>,
}

impl<'a> TableView<'a> {
    pub(super) fn new(data_frame: &'a mut DataFrame, settings: &'a Settings) -> Self {
        Self {
            data_frame,
            settings,
            event: None,
        }
    }
}

impl TableView<'_> {
    pub(super) fn show(&mut self, ui: &mut Ui) -> Option<Event> {
        let id_salt = Id::new("ConfigurationTable");
        let height = ui.text_style_height(&TextStyle::Heading) + 2.0 * MARGIN.y;
        let num_rows = self.data_frame.height() as u64 + 1;
        let num_columns = LEN;
        Table::new()
            .id_salt(id_salt)
            .num_rows(num_rows)
            .columns(vec![
                Column::default().resizable(self.settings.resizable);
                num_columns
            ])
            .num_sticky_cols(self.settings.sticky)
            .headers([HeaderRow::new(height)])
            .auto_size_mode(AutoSizeMode::OnParentResize)
            .show(ui, self);
        self.event
    }

    fn header_cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: Range<usize>) {
        if self.settings.truncate {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
        }
        match (row, column) {
            // Top
            (0, INDEX) => {
                ui.heading("Index");
            }
            (0, LABEL) => {
                ui.heading("Label");
            }
            (0, FA) => {
                ui.heading("FA");
            }
            (0, TAG) => {
                ui.heading("TAG");
            }
            (0, DAG1223) => {
                ui.heading("DAG1223");
            }
            (0, MAG2) => {
                ui.heading("MAG2");
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
                if self.settings.editable {
                    if ui.button(MINUS).clicked() {
                        self.event = Some(Event::DeleteRow(row));
                    }
                }
                let indices = self.data_frame[0].u32()?;
                let index = indices.get(row).unwrap();
                ui.label(index.to_string());
            }
            (row, LABEL) => {
                let labels = self.data_frame["Label"].str()?;
                let label = labels.get(row).unwrap();
                if self.settings.editable {
                    let mut label = label.to_owned();
                    if ui.text_edit_singleline(&mut label).changed() {
                        self.data_frame
                            .try_apply("Label", change_label(row, &label))?;
                    }
                } else {
                    ui.label(label);
                }
            }
            (row, FA) => {
                let inner_response = FattyAcidWidget::new(|| self.data_frame.fatty_acid().get(row))
                    .editable(self.settings.editable)
                    .hover()
                    .names(self.settings.names)
                    .try_show(ui)?;
                if let Some(value) = inner_response.inner {
                    self.data_frame
                        .try_apply("FattyAcid", change_fatty_acid(row, &value))?;
                }
            }
            (row, TAG) => {
                self.rw(ui, row, "Triacylglycerol")?;
            }
            (row, DAG1223) => {
                self.rw(ui, row, "Diacylglycerol1223")?;
            }
            (row, MAG2) => {
                self.rw(ui, row, "Monoacylglycerol2")?;
            }
            _ => {}
        }
        Ok(())
    }

    fn footer_cell_content_ui(&mut self, ui: &mut Ui, column: Range<usize>) -> PolarsResult<()> {
        match column {
            INDEX => {
                if self.settings.editable {
                    if ui.button(PLUS).clicked() {
                        self.event = Some(Event::AddRow);
                    }
                }
            }
            TAG => {
                FloatWidget::new(|| Ok(self.data_frame["Triacylglycerol"].f64()?.sum()))
                    .precision(Some(self.settings.precision))
                    .hover()
                    .show(ui)
                    .response
                    .on_hover_text("∑ TAG");
            }
            DAG1223 => {
                FloatWidget::new(|| Ok(self.data_frame["Diacylglycerol1223"].f64()?.sum()))
                    .precision(Some(self.settings.precision))
                    .hover()
                    .show(ui)
                    .response
                    .on_hover_text("∑ DAG1223");
            }
            MAG2 => {
                FloatWidget::new(|| Ok(self.data_frame["Monoacylglycerol2"].f64()?.sum()))
                    .precision(Some(self.settings.precision))
                    .hover()
                    .show(ui)
                    .response
                    .on_hover_text("∑ MAG");
            }
            _ => {}
        }
        Ok(())
    }

    fn rw(&mut self, ui: &mut Ui, row: usize, column: &str) -> PolarsResult<Response> {
        let inner_response = FloatWidget::new(|| Ok(self.data_frame[column].f64()?.get(row)))
            .editable(self.settings.editable)
            .precision(Some(self.settings.precision))
            .hover()
            .show(ui);
        if let Some(value) = inner_response.inner {
            self.data_frame
                .try_apply(column, change_experimental(row, value))?;
        }
        Ok(inner_response.response)
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
            .inner_margin(Margin::symmetric(MARGIN.x, 2.0))
            .show(ui, |ui| {
                if let Err(error) =
                    self.cell_content_ui(ui, cell.row_nr as _, cell.col_nr..cell.col_nr + 1)
                {
                    ui.ctx()
                        .error(error.context("Configuration table cell ui".into()));
                }
            });
    }

    fn row_top_offset(&self, ctx: &Context, _table_id: Id, row_nr: u64) -> f32 {
        row_nr as f32 * (ctx.style().spacing.interact_size.y + 2.0 * MARGIN.y)
    }
}

/// Event
#[derive(Clone, Copy, Debug)]
pub(super) enum Event {
    AddRow,
    DeleteRow(usize),
}

fn change_label(row: usize, new: &str) -> impl FnMut(&Series) -> PolarsResult<Series> {
    move |series| {
        Ok(series
            .str()?
            .iter()
            .enumerate()
            .map(|(index, mut value)| {
                if index == row {
                    value = Some(new);
                }
                Ok(value)
            })
            .collect::<PolarsResult<StringChunked>>()?
            .into_series())
    }
}

fn change_fatty_acid(
    row: usize,
    new: &FattyAcid,
) -> impl FnMut(&Series) -> PolarsResult<Series> + '_ {
    move |series| {
        let fatty_acid_series = series.fatty_acid();
        let mut carbons = PrimitiveChunkedBuilder::<UInt8Type>::new(
            fatty_acid_series.carbons.name().clone(),
            fatty_acid_series.len(),
        );
        let mut unsaturated = AnonymousOwnedListBuilder::new(
            fatty_acid_series.unsaturated.name().clone(),
            fatty_acid_series.len(),
            fatty_acid_series.unsaturated.dtype().inner_dtype().cloned(),
        );
        for index in 0..fatty_acid_series.len() {
            let mut fatty_acid = fatty_acid_series.get(index)?;
            if index == row {
                fatty_acid = Some(new.clone());
            }
            let fatty_acid = fatty_acid.as_ref();
            // Carbons
            carbons.append_option(fatty_acid.map(|fatty_acid| fatty_acid.carbons));
            // Unsaturated
            if let Some(fatty_acid) = fatty_acid {
                // let mut fields = Vec::with_capacity(fatty_acid.unsaturated.len());
                // if let Some(unsaturated_series) = fatty_acid_series.unsaturated(index)? {
                //     fields.push(unsaturated_series.index);
                //     fields.push(unsaturated_series.isomerism);
                //     fields.push(unsaturated_series.unsaturation);
                // }
                // unsaturated.append_series(
                //     &StructChunked::from_series(
                //         PlSmallStr::EMPTY,
                //         fatty_acid.unsaturated.len(),
                //         fields.iter(),
                //     )?
                //     .into_series(),
                // )?;
                let mut index = PrimitiveChunkedBuilder::<UInt8Type>::new(
                    "Index".into(),
                    fatty_acid.unsaturated.len(),
                );
                let mut isomerism = PrimitiveChunkedBuilder::<Int8Type>::new(
                    "Isomerism".into(),
                    fatty_acid.unsaturated.len(),
                );
                let mut unsaturation = PrimitiveChunkedBuilder::<UInt8Type>::new(
                    "Unsaturation".into(),
                    fatty_acid.unsaturated.len(),
                );
                for unsaturated in &fatty_acid.unsaturated {
                    index.append_option(unsaturated.index);
                    isomerism.append_option(unsaturated.isomerism.map(|isomerism| isomerism as _));
                    unsaturation.append_option(
                        unsaturated
                            .unsaturation
                            .map(|unsaturation| unsaturation as _),
                    );
                }
                unsaturated.append_series(
                    &StructChunked::from_series(
                        PlSmallStr::EMPTY,
                        fatty_acid.unsaturated.len(),
                        [
                            index.finish().into_series(),
                            isomerism.finish().into_series(),
                            unsaturation.finish().into_series(),
                        ]
                        .iter(),
                    )?
                    .into_series(),
                )?;
            } else {
                println!("HERE1");
                unsaturated.append_opt_series(None)?;
            }
        }
        Ok(StructChunked::from_series(
            series.name().clone(),
            fatty_acid_series.len(),
            [
                carbons.finish().into_series(),
                unsaturated.finish().into_series(),
            ]
            .iter(),
        )?
        .into_series())
    }
}

fn change_experimental(row: usize, new: f64) -> impl FnMut(&Series) -> PolarsResult<Series> {
    move |series| {
        Ok(series
            .f64()?
            .iter()
            .enumerate()
            .map(|(index, mut value)| {
                if index == row {
                    value = Some(new);
                }
                Ok(value)
            })
            .collect::<PolarsResult<Float64Chunked>>()?
            .into_series())
    }
}
