use std::ops::Range;

use super::control::{From, Settings};
use crate::{
    app::{MARGIN, widgets::FloatValue},
    localization::localize,
    special::{
        new_fatty_acid::{COMMON, DisplayWithOptions},
        polars::{DataFrameExt as _, columns::fatty_acids::ColumnExt as _},
    },
    utils::{polars::DataFrameExt as _, ui::UiExt as _},
};
use egui::{Frame, Id, Margin, TextStyle, TextWrapMode, Ui};
use egui_table::{AutoSizeMode, CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate};
use polars::prelude::*;

const META: &str = "Meta";

const KEY: Range<usize> = 0..2;
const EXPERIMENTAL: Range<usize> = KEY.end..KEY.end + 3;
const THEORETICAL: Range<usize> = EXPERIMENTAL.end..EXPERIMENTAL.end + 5;
const FACTORS: Range<usize> = THEORETICAL.end..THEORETICAL.end + 4;
const LEN: usize = FACTORS.end;

const TOP: &[Range<usize>] = &[KEY, EXPERIMENTAL, THEORETICAL, FACTORS];

const MIDDLE: &[Range<usize>] = &[
    key::INDEX,
    key::FA,
    experimental::TAG123,
    experimental::DAG1223,
    experimental::MAG2,
    theoretical::TAG123,
    theoretical::DAG1223,
    theoretical::MAG2,
    theoretical::DAG13,
    factors::EF,
    factors::SF,
];

/// Kind
pub(crate) enum Kind {
    Single,
    Mean,
}

/// Calculation table
pub(crate) struct CalculationTable<'a> {
    data_frame: DataFrame,
    kind: Kind,
    settings: &'a Settings,
}

impl<'a> CalculationTable<'a> {
    pub(crate) fn new(data_frame: DataFrame, kind: Kind, settings: &'a Settings) -> Self {
        Self {
            data_frame,
            kind,
            settings,
        }
    }
}

impl CalculationTable<'_> {
    pub(crate) fn ui(&mut self, ui: &mut Ui) {
        let id_salt = Id::new("CalculationDataTable");
        let height = ui.text_style_height(&TextStyle::Heading);
        let num_rows = self.data_frame.height() as u64 + 1;
        let num_columns = LEN;
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
                    groups: TOP.to_vec(),
                },
                HeaderRow {
                    height,
                    groups: MIDDLE.to_vec(),
                },
                HeaderRow::new(height),
            ])
            .auto_size_mode(AutoSizeMode::OnParentResize)
            .show(ui, self);
    }

    fn header_cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: usize) {
        if self.settings.truncate {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
        }
        match (row, column) {
            // Top
            (0, 0) => {
                ui.heading("Key");
            }
            (0, 1) => {
                ui.heading("Experimental");
            }
            (0, 2) => {
                ui.heading("Theoretical");
            }
            (0, 3) => {
                ui.heading("Factors");
            }
            // Middle
            (1, 0) => {
                ui.heading("Index");
            }
            (1, 1) => {
                ui.heading(localize!("fatty_acid.abbreviation"))
                    .on_hover_text(localize!("fatty_acid"));
            }
            (1, 2) => {
                ui.heading("TAG")
                    .on_hover_text(localize!("triacylglycerol"));
            }
            (1, 3) => {
                ui.heading("DAG1223")
                    .on_hover_text(format!("sn-1,2/2,3 {}", localize!("diacylglycerol")));
            }
            (1, 4) => {
                ui.heading("MAG2")
                    .on_hover_text(format!("sn-2 {}", localize!("monoacylglycerol")));
            }
            (1, 5) => {
                ui.heading("TAG")
                    .on_hover_text(localize!("triacylglycerol"));
            }
            (1, 6) => {
                ui.heading("DAG1223")
                    .on_hover_text(format!("sn-1,2/2,3 {}", localize!("diacylglycerol")));
            }
            (1, 7) => {
                ui.heading("MAG2")
                    .on_hover_text(format!("sn-2 {}", localize!("monoacylglycerol")));
            }
            (1, 8) => {
                ui.heading("DAG13")
                    .on_hover_text(format!("sn-13 {}", localize!("diacylglycerol")));
            }
            (1, 9) => {
                ui.heading("EF")
                    .on_hover_text(localize!("enrichment_factor"));
            }
            (1, 10) => {
                ui.heading("SF")
                    .on_hover_text(localize!("selectivity_factor"));
            }
            // Bottom
            (2, 0..8) => {}
            (2, 8) => {
                ui.heading("DAG1223");
            }
            (2, 9) => {
                ui.heading("MAG2");
            }
            (2, 10) => {
                ui.heading("MAG2");
            }
            (2, 11) => {
                ui.heading("DAG13");
            }
            (2, 12) => {
                ui.heading("MAG2");
            }
            (2, 13) => {
                ui.heading("DAG13");
            }
            _ => unreachable!(),
        };
    }

    fn body_cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: usize) {
        let footer = row == self.data_frame.height();
        // let footer = row == self.data_frame[column].height();
        match (row, column) {
            (row, 0) => {
                if row != self.data_frame.height() {
                    let indices = self.data_frame.u32("Index");
                    let index = indices.get(row).unwrap();
                    ui.label(index.to_string());
                }
            }
            (row, 1) => {
                if row != self.data_frame.height() {
                    let fatty_acids = self.data_frame["FA"].fa();
                    let index = row;
                    let (label, fatty_acid) = fatty_acids.get(index).unwrap();
                    let text = if label.is_empty() { "C" } else { &label };
                    let title = ui.subscripted_text(
                        text,
                        &format!("{:#}", &fatty_acid.display(COMMON)),
                        Default::default(),
                    );
                    ui.label(title);
                }
            }
            // (row, 2) => match self.kind {
            //     Kind::Mean => self.mean(
            //         ui,
            //         &["Experimental", "TAG"],
            //         row,
            //         false,
            //         self.settings.percent,
            //         footer,
            //     ),
            //     _ => self.data(
            //         ui,
            //         &["Experimental", "TAG"],
            //         row,
            //         false,
            //         self.settings.percent,
            //         footer,
            //     ),
            // },
            // (row, 3) => {
            //     let disable = self.settings.from != From::Dag1223;
            //     match self.kind {
            //         Kind::Mean => self.mean(
            //             ui,
            //             &["Experimental", "DAG1223"],
            //             row,
            //             disable,
            //             self.settings.percent,
            //             footer,
            //         ),
            //         _ => self.data(
            //             ui,
            //             &["Experimental", "DAG1223"],
            //             row,
            //             disable,
            //             self.settings.percent,
            //             footer,
            //         ),
            //     }
            // }
            // (row, 4) => {
            //     let disable = self.settings.from != From::Mag2;
            //     match self.kind {
            //         Kind::Mean => self.mean(
            //             ui,
            //             &["Experimental", "MAG2"],
            //             row,
            //             disable,
            //             self.settings.percent,
            //             footer,
            //         ),
            //         _ => self.data(
            //             ui,
            //             &["Experimental", "MAG2"],
            //             row,
            //             disable,
            //             self.settings.percent,
            //             footer,
            //         ),
            //     }
            // }
            // (row, 5) => match self.kind {
            //     Kind::Mean => self.mean(
            //         ui,
            //         &["Theoretical", "TAG"],
            //         row,
            //         true,
            //         self.settings.percent,
            //         footer,
            //     ),
            //     _ => self.data(
            //         ui,
            //         &["Theoretical", "TAG"],
            //         row,
            //         true,
            //         self.settings.percent,
            //         footer,
            //     ),
            // },
            // (row, 6) => match self.kind {
            //     Kind::Mean => self.mean(
            //         ui,
            //         &["Theoretical", "DAG1223"],
            //         row,
            //         true,
            //         self.settings.percent,
            //         footer,
            //     ),
            //     _ => self.data(
            //         ui,
            //         &["Theoretical", "DAG1223"],
            //         row,
            //         true,
            //         self.settings.percent,
            //         footer,
            //     ),
            // },
            // (row, 7) => match self.kind {
            //     Kind::Mean => self.mean(
            //         ui,
            //         &["Theoretical", "MAG2"],
            //         row,
            //         true,
            //         self.settings.percent,
            //         footer,
            //     ),
            //     _ => self.data(
            //         ui,
            //         &["Theoretical", "MAG2"],
            //         row,
            //         true,
            //         self.settings.percent,
            //         footer,
            //     ),
            // },
            // (row, 8) => {
            //     let disable = self.settings.from != From::Dag1223;
            //     match self.kind {
            //         Kind::Mean => self.mean(
            //             ui,
            //             &["Theoretical", "DAG13", "DAG1223"],
            //             row,
            //             disable,
            //             self.settings.percent,
            //             footer,
            //         ),
            //         _ => self.data(
            //             ui,
            //             &["Theoretical", "DAG13", "DAG1223"],
            //             row,
            //             disable,
            //             self.settings.percent,
            //             footer,
            //         ),
            //     }
            // }
            // (row, 9) => {
            //     let disable = self.settings.from != From::Mag2;
            //     match self.kind {
            //         Kind::Mean => self.mean(
            //             ui,
            //             &["Theoretical", "DAG13", "MAG2"],
            //             row,
            //             disable,
            //             self.settings.percent,
            //             footer,
            //         ),
            //         _ => self.data(
            //             ui,
            //             &["Theoretical", "DAG13", "MAG2"],
            //             row,
            //             disable,
            //             self.settings.percent,
            //             footer,
            //         ),
            //     }
            // }
            // (row, 10) => match self.kind {
            //     Kind::Mean => self.mean(ui, &["EF", "MAG2"], row, true, false, footer),
            //     _ => self.data(ui, &["EF", "MAG2"], row, true, false, footer),
            // },
            // (row, 11) => match self.kind {
            //     Kind::Mean => self.mean(ui, &["EF", "DAG13"], row, true, false, footer),
            //     _ => self.data(ui, &["EF", "DAG13"], row, true, false, footer),
            // },
            // (row, 12) => match self.kind {
            //     Kind::Mean => self.mean(ui, &["SF", "MAG2"], row, true, false, footer),
            //     _ => self.data(ui, &["SF", "MAG2"], row, true, false, footer),
            // },
            // (row, 13) => match self.kind {
            //     Kind::Mean => self.mean(ui, &["SF", "DAG13"], row, true, false, footer),
            //     _ => self.data(ui, &["SF", "DAG13"], row, true, false, footer),
            // },
            _ => {}
            // _ => unreachable!(),
        }
        // match column {
        //     _ => {}
        //     0..=1 => {}
        //     2 => {
        //         let experimental = self
        //             .data_frame
        //             .destruct(&self.name)
        //             .destruct("Experimental");
        //         let sum = experimental.f64("TAG").sum();
        //         ui.add(
        //             FloatValue::new(sum)
        //                 .percent(self.settings.percent)
        //                 .precision(Some(self.settings.precision))
        //                 .hover(),
        //         );
        //     }
        //     3 => {
        //         let experimental = self
        //             .data_frame
        //             .destruct(&self.name)
        //             .destruct("Experimental");
        //         let sum = experimental.f64("DAG1223").sum();
        //         ui.add(
        //             FloatValue::new(sum)
        //                 .disable(self.settings.from != From::Dag1223)
        //                 .percent(self.settings.percent)
        //                 .precision(Some(self.settings.precision))
        //                 .hover(),
        //         );
        //     }
        //     4 => {
        //         let experimental = self
        //             .data_frame
        //             .destruct(&self.name)
        //             .destruct("Experimental");
        //         let sum = experimental.f64("MAG2").sum();
        //         ui.add(
        //             FloatValue::new(sum)
        //                 .disable(self.settings.from != From::Mag2)
        //                 .percent(self.settings.percent)
        //                 .precision(Some(self.settings.precision))
        //                 .hover(),
        //         );
        //     }
        //     5 => {
        //         let theoretical = self.data_frame.destruct(&self.name).destruct("Theoretical");
        //         let sum = theoretical.f64("TAG").sum();
        //         ui.add(
        //             FloatValue::new(sum)
        //                 .disable(true)
        //                 .percent(self.settings.percent)
        //                 .precision(Some(self.settings.precision))
        //                 .hover(),
        //         );
        //     }
        //     6 => {
        //         let theoretical = self.data_frame.destruct(&self.name).destruct("Theoretical");
        //         let sum = theoretical.f64("DAG1223").sum();
        //         ui.add(
        //             FloatValue::new(sum)
        //                 .disable(true)
        //                 .percent(self.settings.percent)
        //                 .precision(Some(self.settings.precision))
        //                 .hover(),
        //         );
        //     }
        //     7 => {
        //         let theoretical = self.data_frame.destruct(&self.name).destruct("Theoretical");
        //         let sum = theoretical.f64("MAG2").sum();
        //         ui.add(
        //             FloatValue::new(sum)
        //                 .disable(true)
        //                 .percent(self.settings.percent)
        //                 .precision(Some(self.settings.precision))
        //                 .hover(),
        //         );
        //     }
        //     8 => {
        //         let theoretical = self
        //             .data_frame
        //             .destruct(&self.name)
        //             .destruct("Theoretical")
        //             .destruct("DAG13");
        //         let sum = theoretical.f64("DAG1223").sum();
        //         ui.add(
        //             FloatValue::new(sum)
        //                 .disable(self.settings.from != From::Dag1223)
        //                 .percent(self.settings.percent)
        //                 .precision(Some(self.settings.precision))
        //                 .hover(),
        //         );
        //     }
        //     9 => {
        //         let theoretical = self
        //             .data_frame
        //             .destruct(&self.name)
        //             .destruct("Theoretical")
        //             .destruct("DAG13");
        //         let sum = theoretical.f64("MAG2").sum();
        //         ui.add(
        //             FloatValue::new(sum)
        //                 .disable(self.settings.from != From::Mag2)
        //                 .percent(self.settings.percent)
        //                 .precision(Some(self.settings.precision))
        //                 .hover(),
        //         );
        //     }
        //     _ => unreachable!(),
        // }
    }

    fn data(
        &self,
        ui: &mut Ui,
        names: &[&str],
        row: usize,
        disable: bool,
        percent: bool,
        footer: bool,
    ) {
        let r#struct = self.data_frame.destructs(&names[0..names.len() - 1]);
        let values = r#struct.f64(names[names.len() - 1]);
        let value = if footer {
            values.sum()
        } else {
            values.get(row)
        };
        ui.add(
            FloatValue::new(value)
                .disable(disable)
                .percent(percent)
                .precision(Some(self.settings.precision))
                .hover(),
        );
    }

    fn mean(
        &self,
        ui: &mut Ui,
        names: &[&str],
        row: usize,
        disable: bool,
        percent: bool,
        footer: bool,
    ) {
        let r#struct = self.data_frame.destructs(names);
        let means = r#struct.f64("Mean");
        let standard_deviations = r#struct.f64("Std");
        let mean = if footer { means.sum() } else { means.get(row) };
        let standard_deviation = if footer {
            standard_deviations.sum()
        } else {
            standard_deviations.get(row)
        };
        ui.add(
            FloatValue::new(mean)
                .disable(disable)
                .percent(percent)
                .precision(Some(self.settings.precision))
                .hover(),
        );
        ui.label("±");
        ui.add(
            FloatValue::new(standard_deviation)
                .disable(disable)
                .percent(percent)
                .precision(Some(self.settings.precision))
                .hover(),
        );
    }
}

impl TableDelegate for CalculationTable<'_> {
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

mod key {
    use super::*;

    pub(super) const INDEX: Range<usize> = KEY.start..KEY.start + 1;
    pub(super) const FA: Range<usize> = INDEX.end..INDEX.end + 1;
}

mod experimental {
    use super::*;

    pub(super) const TAG123: Range<usize> = EXPERIMENTAL.start..EXPERIMENTAL.start + 1;
    pub(super) const DAG1223: Range<usize> = TAG123.end..TAG123.end + 1;
    pub(super) const MAG2: Range<usize> = DAG1223.end..DAG1223.end + 1;
}

mod theoretical {
    use super::*;

    pub(super) const TAG123: Range<usize> = THEORETICAL.start..THEORETICAL.start + 1;
    pub(super) const DAG1223: Range<usize> = TAG123.end..TAG123.end + 1;
    pub(super) const MAG2: Range<usize> = DAG1223.end..DAG1223.end + 1;
    pub(super) const DAG13: Range<usize> = MAG2.end..MAG2.end + 2;

    // mod dag13 {
    //     use super::*;

    //     pub(super) const DAG1223: Range<usize> = DAG13.start..DAG13.start + 1;
    //     pub(super) const MAG2: Range<usize> = DAG1223.end..DAG1223.end + 1;
    // }
}

mod factors {
    use super::*;

    pub(super) const EF: Range<usize> = FACTORS.start..FACTORS.start + 2;
    pub(super) const SF: Range<usize> = EF.end..EF.end + 2;

    // mod enrichment_factor {
    //     use super::*;

    //     pub(super) const MAG2: Range<usize> = EF.start..EF.start + 1;
    //     pub(super) const DAG13: Range<usize> = MAG2.end..MAG2.end + 1;
    // }

    // mod selectivity_factor {
    //     use super::*;

    //     pub(super) const MAG2: Range<usize> = SF.start..SF.start + 1;
    //     pub(super) const DAG13: Range<usize> = MAG2.end..MAG2.end + 1;
    // }
}
