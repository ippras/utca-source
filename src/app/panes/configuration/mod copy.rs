use self::{
    control::Control,
    names::Names,
    properties::Properties,
    table::{Event, TableView},
};
use crate::{
    app::{DataFrameExt, data::FattyAcids, widgets::FattyAcidWidget},
    localization::localize,
    utils::{
        polars::DataFrameExt as _,
        ui::{SubscriptedTextFormat, UiExt},
    },
};
use egui::{
    menu::bar, util::hash, vec2, ComboBox, Direction, Grid, Id, Label, Layout, RichText, ScrollArea, TextEdit, Ui, Widget as _
};
use egui_extras::{Column, Size, Strip, StripBuilder, TableBuilder};
use egui_phosphor::regular::{
    ARROW_FAT_LINE_UP, ARROWS_HORIZONTAL, CALCULATOR, ERASER, FLOPPY_DISK, GEAR, MINUS, PENCIL,
    PLUS, TAG, TRASH,
};
use lipid::fatty_acid::display::{COMMON, DisplayWithOptions as _};
use polars::prelude::*;
use ron::{extensions::Extensions, ser::PrettyConfig};
use serde::{Deserialize, Serialize};
use tracing::error;

/// Configuration pane
#[derive(Default, Deserialize, Serialize)]
pub(crate) struct Pane {
    pub(crate) data_frames: Vec<FattyAcids>,
    pub(crate) control: Control,
}

impl Pane {
    pub(crate) const fn new(data_frames: Vec<FattyAcids>) -> Self {
        Self {
            data_frames,
            control: Control::new(),
        }
    }

    pub(crate) fn header(&mut self, ui: &mut Ui) {
        ui.visuals_mut().button_frame = false;
        let selected_text = match self.control.index {
            Some(index) => self.data_frames[index].name(),
            None => "",
        };
        ComboBox::from_id_salt(ui.next_auto_id())
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                for index in 0..self.data_frames.len() {
                    ui.selectable_value(
                        &mut self.control.index,
                        Some(index),
                        self.data_frames[index].name(),
                    );
                }
            });
        ui.separator();

        bar(ui, |ui| {
            ScrollArea::horizontal().show(ui, |ui| {
                ui.menu_button(RichText::new(TAG).heading(), |ui| {
                    if let Some(index) = self.control.index {
                        let mut changed = false;
                        let name = self.data_frames[index].0[0].name().clone();
                        let (mut label, mut date) = name
                            .split_once(';')
                            .map_or((name.to_string(), String::new()), |(label, date)| {
                                (label.to_owned(), date.to_owned())
                            });
                        let width = ui.data_mut(|data| {
                            *data.get_temp_mut_or_default::<f32>(ui.next_auto_id())
                        });
                        ui.horizontal(|ui| {
                            let height = ui.style().spacing.interact_size.y;
                            let text_edit_width = ui.style().spacing.text_edit_width;
                            ui.add_sized(width, Label::new(text));
                            ui.label("Label12345");
                            changed |= ui
                                .add(
                                    TextEdit::singleline(&mut label)
                                        .min_size(vec2(0.0, 0.0))
                                        .hint_text("Label"),
                                )
                                .changed();
                            // StripBuilder::new(ui)
                            //     .sizes(Size::exact(height), 2)
                            //     .vertical(|mut strip| {
                            //         strip.strip(|builder| {
                            //             builder
                            //                 .size(Size::relative(0.1))
                            //                 .size(Size::remainder().at_least(text_edit_width))
                            //                 .horizontal(|mut strip| {
                            //                     strip.cell(|ui| {
                            //                         ui.label("Label");
                            //                     });
                            //                     strip.cell(|ui| {
                            //                         changed |= ui
                            //                             .add(
                            //                                 TextEdit::singleline(&mut label)
                            //                                     .min_size(vec2(0.0, 0.0))
                            //                                     .hint_text("Label"),
                            //                             )
                            //                             .changed();
                            //                     });
                            //                 });
                            //         });
                            //         strip.strip(|builder| {
                            //             builder
                            //                 .size(Size::relative(0.1))
                            //                 .size(Size::remainder().at_least(text_edit_width))
                            //                 .horizontal(|mut strip| {
                            //                     strip.cell(|ui| {
                            //                         ui.label("Date");
                            //                     });
                            //                     strip.cell(|ui| {
                            //                         changed |= ui
                            //                             .add(
                            //                                 TextEdit::singleline(&mut date)
                            //                                     .hint_text("Date"),
                            //                             )
                            //                             .changed();
                            //                     });
                            //                 });
                            //         });
                            //     });
                            // Grid::new(ui.next_auto_id()).show(ui, |ui| {
                            //     ui.label("Label");
                            //     changed |= ui
                            //         .add(
                            //             TextEdit::singleline(&mut label)
                            //                 .min_size(vec2(0.0, 0.0))
                            //                 .hint_text("Label"),
                            //         )
                            //         .changed();
                            //     ui.end_row();
                            //     ui.label("Date");
                            //     changed |= ui
                            //         .add(TextEdit::singleline(&mut date).hint_text("Date"))
                            //         .changed();
                            // });
                        });
                        ui.horizontal(|ui| {
                            ui.label("Date");
                            changed |= ui
                                .add(TextEdit::singleline(&mut date).hint_text("Date"))
                                .changed();
                        });
                        if changed {
                            self.data_frames[index]
                                .0
                                .rename(&name, format!("{label};{date}").into())
                                .unwrap();
                        }
                    }
                })
                .response
                .on_hover_text(localize!("label"));
                ui.toggle_value(
                    &mut self.control.settings.resizable,
                    RichText::new(ARROWS_HORIZONTAL).heading(),
                )
                .on_hover_text(localize!("resize"));
                ui.toggle_value(
                    &mut self.control.settings.editable,
                    RichText::new(PENCIL).heading(),
                )
                .on_hover_text(localize!("edit"));
                // Add
                if ui
                    .button(RichText::new(PLUS).heading())
                    .on_hover_text(localize!("add"))
                    .clicked()
                {
                    if let Err(error) = self.add_row() {
                        error!(%error);
                    }
                }
                // Clear
                ui.add_enabled_ui(!self.data_frames.is_empty(), |ui| {
                    if ui
                        .button(RichText::new(ERASER).heading())
                        .on_hover_text(localize!("clear"))
                        .clicked()
                    {
                        if let Some(index) = self.control.index {
                            let data_frame = &mut self.data_frames[index].0;
                            *data_frame = data_frame.clear();
                        }
                    }
                });
                // Delete
                ui.add_enabled_ui(!self.data_frames.is_empty(), |ui| {
                    if ui
                        .button(RichText::new(TRASH).heading())
                        .on_hover_text(localize!("delete"))
                        .clicked()
                    {
                        if let Some(index) = self.control.index {
                            self.data_frames.remove(index);
                            self.control.index = None;
                        }
                    }
                });
                ui.separator();
                // Settings
                ui.toggle_value(&mut self.control.open, RichText::new(GEAR).heading())
                    .on_hover_text(localize!("settings"));
                ui.separator();
                // Save
                if ui
                    .button(RichText::new(FLOPPY_DISK).heading())
                    .on_hover_text(localize!("save"))
                    .on_hover_text(&self.control.settings.label)
                    .clicked()
                {
                    // if let Err(error) = self.save() {
                    //     error!(%error);
                    // }
                }
                ui.separator();
                // Calculation
                if ui
                    .button(RichText::new(CALCULATOR).heading())
                    .on_hover_text(localize!("calculation"))
                    .clicked()
                {
                    // FATTY_ACIDS_SCHEMA.names();
                    const NAMES: [&str; 4] = ["Label", "Carbons", "Doubles", "Triples"];
                    let mut lazy_frame: Option<LazyFrame> = None;
                    for fatty_acids in &mut self.data_frames {
                        let next = fatty_acids.0.clone().lazy().select([
                            col("FA").struct_().field_by_names(["*"]),
                            as_struct(vec![col("TAG"), col("DAG1223"), col("MAG2")])
                                .alias(fatty_acids.name().clone()),
                        ]);
                        lazy_frame = Some(if let Some(current) = lazy_frame {
                            current.join(
                                next,
                                &NAMES.map(col),
                                &NAMES.map(col),
                                JoinArgs::new(JoinType::Full)
                                    .with_coalesce(JoinCoalesce::CoalesceColumns),
                            )
                        } else {
                            next
                        });
                    }
                    if let Some(mut lazy_frame) = lazy_frame {
                        lazy_frame = lazy_frame.select([
                            as_struct(vec![as_struct(vec![cols(NAMES)]).alias("FA")]).alias("Key"),
                            as_struct(vec![all().exclude(NAMES)]).alias("Values"),
                        ]);
                        ui.data_mut(|data| {
                            data.insert_temp(Id::new("Calculate"), lazy_frame.collect().unwrap());
                        });
                    }
                }
                ui.separator();
            });
        });
    }

    pub(crate) fn content(&mut self, ui: &mut Ui) {
        ui.separator();
        self.control.windows(ui);
        let Some(index) = self.control.index else {
            return;
        };
        let data_frame = &mut self.data_frames[index].0;
        if let Some(Event::DeleteRow(row)) =
            TableView::new(data_frame, &self.control.settings).ui(ui)
        {
            if let Err(error) = self.delete_row(row) {
                error!(%error);
            }
        }
        // // let mut tiles = Tiles::default();
        // // let children = vec![tiles.insert_pane(Pane {}), tiles.insert_pane(Pane {})];
        // // let root = tiles.insert_tab_tile(children);
        // // let tree = Tree::new("my_tree", root, tiles);
        // // tree.ui(behavior, ui);
        // ui.separator();
        // self.control.windows(ui);
        // if self.source.is_empty() {
        //     return;
        // }
        // let fatty_acids = &self.source[self.control.index];
        // // let Some(entry) = behavior.data.entries.iter_mut().find(|entry| entry.checked) else {
        // //     return;
        // // };
        // let height = ui.spacing().interact_size.y;
        // let width = ui.spacing().interact_size.x;
        // let total_rows = fatty_acids.0.height();
        // let labels = fatty_acids.str("Label");
        // let fatty_acid = fatty_acids.0.fatty_acid();
        // let tags = fatty_acids.0.f64("TAG");
        // let dags1223 = fatty_acids.0.f64("DAG1223");
        // let mags2 = fatty_acids.0.f64("MAG2");
        // let mut event = None;
        // let mut builder = TableBuilder::new(ui)
        //     .cell_layout(Layout::centered_and_justified(Direction::LeftToRight));
        // if self.control.settings.editable {
        //     builder = builder.column(Column::exact(width / 2.0));
        // }
        // builder = builder
        //     .column(Column::auto_with_initial_suggestion(width))
        //     .columns(Column::auto(), 3);
        // if self.control.settings.editable {
        //     builder = builder.column(Column::exact(width));
        // }
        // builder
        //     .auto_shrink(false)
        //     .resizable(self.control.settings.resizable)
        //     .striped(true)
        //     .header(height, |mut row| {
        //         if self.control.settings.editable {
        //             row.col(|_ui| {});
        //         }
        //         row.col(|ui| {
        //             ui.heading(localize!("fatty_acid.abbreviation"))
        //                 .on_hover_text(localize!("fatty_acid"));
        //         });
        //         row.col(|ui| {
        //             ui.heading(localize!("triacylglycerol.abbreviation"))
        //                 .on_hover_text(localize!("triacylglycerol"));
        //         });
        //         row.col(|ui| {
        //             ui.heading(format!(
        //                 "1,2/2,3-{}",
        //                 localize!("diacylglycerol.abbreviation"),
        //             ))
        //             .on_hover_text(format!("sn-1,2/2,3 {}", localize!("diacylglycerol")));
        //         });
        //         row.col(|ui| {
        //             ui.heading(format!("2-{}", localize!("monoacylglycerol.abbreviation")))
        //                 .on_hover_text(format!("sn-2 {}", localize!("monoacylglycerol")));
        //         });
        //     })
        //     .body(|body| {
        //         let precision = |value| format!("{value:.*}", self.control.settings.precision);
        //         body.rows(height, total_rows + 1, |mut row| {
        //             let index = row.index();
        //             if index < total_rows {
        //                 // Move row
        //                 if self.control.settings.editable {
        //                     row.col(|ui| {
        //                         if ui.button(ARROW_FAT_LINE_UP).clicked() {
        //                             event = Some(Event::Up { row: index });
        //                         }
        //                     });
        //                 }
        //                 // FA
        //                 row.col(|ui| {
        //                     let label = labels.get(index).unwrap();
        //                     let mut fatty_acid = fatty_acid.get(index).unwrap().unwrap();
        //                     let text = if label.is_empty() { "C" } else { label };
        //                     let title = ui.subscripted_text(
        //                         text,
        //                         &format!("{:#}", (&fatty_acid).display(COMMON)),
        //                         SubscriptedTextFormat {
        //                             widget: true,
        //                             ..Default::default()
        //                         },
        //                     );
        //                     let mut response = if self.control.settings.editable {
        //                         ui.menu_button(title, |ui| {
        //                             let mut label = label.to_owned();
        //                             let inner_response =
        //                                 FattyAcidWidget::new(|| self.source.fatty_acid().get(row))
        //                                     .editable(self.settings.editable)
        //                                     .hover()
        //                                     .ui(ui)?;
        //                             if let Some(value) = inner_response.inner {
        //                                 self.source.try_apply(
        //                                     "FattyAcid",
        //                                     change_fatty_acid(row, &value),
        //                                 )?;
        //                             }
        //                             // if let Some(change) =
        //                             //     FattyAcidWidget::new(&mut label, || &mut fatty_acid).ui(ui)
        //                             // {
        //                             //     let (column, value) = match change {
        //                             //         Change::Label => (
        //                             //             "Label",
        //                             //             Scalar::new(
        //                             //                 DataType::String,
        //                             //                 AnyValue::StringOwned(label.into()),
        //                             //             ),
        //                             //         ),
        //                             //         Change::Carbons => (
        //                             //             "Carbons",
        //                             //             Scalar::new(
        //                             //                 DataType::UInt8,
        //                             //                 fatty_acid.carbons.into(),
        //                             //             ),
        //                             //         ),
        //                             //         Change::Doubles => (
        //                             //             "Doubles",
        //                             //             Scalar::new(
        //                             //                 DataType::List(Box::new(DataType::Int8)),
        //                             //                 AnyValue::List(Series::from_iter(
        //                             //                     fatty_acid.doubles.clone(),
        //                             //                 )),
        //                             //             ),
        //                             //         ),
        //                             //         Change::Triples => (
        //                             //             "Triples",
        //                             //             Scalar::new(
        //                             //                 DataType::List(Box::new(DataType::Int8)),
        //                             //                 AnyValue::List(Series::from_iter(
        //                             //                     fatty_acid.triples.clone(),
        //                             //                 )),
        //                             //             ),
        //                             //         ),
        //                             //     };
        //                             //     event = Some(Event::Set {
        //                             //         row: index,
        //                             //         column,
        //                             //         value,
        //                             //     })
        //                             // }
        //                         })
        //                         .response
        //                     } else {
        //                         ui.label(title)
        //                     }
        //                     .on_hover_ui(|ui| {
        //                         ui.heading(localize!("fatty_acid"));
        //                         ui.label(format!(
        //                             "{}: {:#}",
        //                             localize!("formula"),
        //                             fatty_acid.display(COMMON),
        //                         ));
        //                         ui.label(format!(
        //                             "{}: C{}H{}O2",
        //                             localize!("formula"),
        //                             fatty_acid.c(),
        //                             fatty_acid.h(),
        //                         ));
        //                     });
        //                     if self.control.settings.properties {
        //                         response = response.on_hover_ui(|ui| {
        //                             ui.add(Properties::new(fatty_acid));
        //                         });
        //                     }
        //                     if self.control.settings.names {
        //                         response.on_hover_ui(|ui| {
        //                             ui.add(Names::new(fatty_acid));
        //                         });
        //                     }
        //                 });
        //                 // TAG
        //                 row.col(|ui| {
        //                     let mut value = tags.get(index).unwrap_or_default();
        //                     if ui
        //                         .add(Area::new(
        //                             &mut value,
        //                             self.control.settings.editable,
        //                             self.control.settings.precision,
        //                         ))
        //                         .changed()
        //                     {
        //                         event = Some(Event::Set {
        //                             row: index,
        //                             column: "TAG",
        //                             value: Scalar::new(DataType::Float64, value.into()),
        //                         });
        //                     }
        //                 });
        //                 // DAG
        //                 row.col(|ui| {
        //                     let mut value = dags1223.get(index).unwrap_or_default();
        //                     if ui
        //                         .add(Area::new(
        //                             &mut value,
        //                             self.control.settings.editable,
        //                             self.control.settings.precision,
        //                         ))
        //                         .changed()
        //                     {
        //                         event = Some(Event::Set {
        //                             row: index,
        //                             column: "DAG1223",
        //                             value: Scalar::new(DataType::Float64, value.into()),
        //                         });
        //                     }
        //                 });
        //                 // MAG
        //                 row.col(|ui| {
        //                     let mut value = mags2.get(index).unwrap_or_default();
        //                     if ui
        //                         .add(Area::new(
        //                             &mut value,
        //                             self.control.settings.editable,
        //                             self.control.settings.precision,
        //                         ))
        //                         .changed()
        //                     {
        //                         event = Some(Event::Set {
        //                             row: index,
        //                             column: "MAG2",
        //                             value: Scalar::new(DataType::Float64, value.into()),
        //                         });
        //                     }
        //                 });
        //                 // Delete row
        //                 if self.control.settings.editable {
        //                     row.col(|ui| {
        //                         if ui.button(MINUS).clicked() {
        //                             event = Some(Event::Delete { row: index });
        //                         }
        //                     });
        //                 }
        //             } else {
        //                 if self.control.settings.editable {
        //                     row.col(|_ui| {});
        //                 }
        //                 row.col(|_ui| {});
        //                 // TAG
        //                 row.col(|ui| {
        //                     let value = tags.sum().unwrap_or(NAN);
        //                     ui.label(precision(value)).on_hover_text(value.to_string());
        //                 });
        //                 // DAG
        //                 row.col(|ui| {
        //                     let value = dags1223.sum().unwrap_or(NAN);
        //                     ui.label(precision(value)).on_hover_text(value.to_string());
        //                 });
        //                 // MAG
        //                 row.col(|ui| {
        //                     let value = mags2.sum().unwrap_or(NAN);
        //                     ui.label(precision(value)).on_hover_text(value.to_string());
        //                 });
        //                 // Add row
        //                 if self.control.settings.editable {
        //                     row.col(|ui| {
        //                         if ui.button(PLUS).clicked() {
        //                             event = Some(Event::Add);
        //                         }
        //                     });
        //                 }
        //             }
        //         });
        //     });
        // // Mutable
        // if let Some(event) = event {
        //     if let Err(error) = event.apply(&mut self.source[self.control.index]) {
        //         error!(%error);
        //     }
        // }
    }

    pub(crate) fn add_row(&mut self) -> PolarsResult<()> {
        let Some(index) = self.control.index else {
            return Ok(());
        };
        let data_frame = &mut self.data_frames[index].0;
        *data_frame = concat(
            [
                data_frame.clone().lazy(),
                df! {
                    data_frame[0].name().clone() => [data_frame.height() as u32],
                    "FattyAcid" => df! {
                        "Carbons" => [0u8],
                        "Unsaturated" => [
                            df! {
                                "Index" => Series::new_empty(PlSmallStr::EMPTY, &DataType::UInt8),
                                "Isomerism" => Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8),
                                "Unsaturation" => Series::new_empty(PlSmallStr::EMPTY, &DataType::UInt8),
                            }?.into_struct(PlSmallStr::EMPTY).into_series(),
                        ],
                    }?.into_struct(PlSmallStr::EMPTY),
                    "Triacylglycerol" => [0f64],
                    "Diacylglycerol1223" => [0f64],
                    "Monoacylglycerol2" => [0f64],
                }?
                .lazy(),
            ],
            UnionArgs {
                rechunk: true,
                diagonal: true,
                ..Default::default()
            },
        )?
        .collect()?;
        Ok(())
    }

    pub(crate) fn delete_row(&mut self, row: usize) -> PolarsResult<()> {
        let Some(index) = self.control.index else {
            return Ok(());
        };
        let data_frame = &mut self.data_frames[index].0;
        let mut lazy_frame = data_frame.clone().lazy();
        lazy_frame = lazy_frame
            .filter(nth(0).neq(lit(row as u32)))
            .with_column(nth(0).cum_count(false) - lit(1));
        *data_frame = lazy_frame.collect()?;
        Ok(())
    }

    // fn save(&self) -> Result<()> {
    //     let contents = ron::ser::to_string_pretty(
    //         &self.data_frame,
    //         PrettyConfig::new().extensions(Extensions::IMPLICIT_SOME | Extensions::UNWRAP_NEWTYPES),
    //     )?;
    //     std::fs::write(format!("{}.hmf.ron", self.control.settings.label), contents)?;
    //     Ok(())
    // }

    pub(super) fn hash(&self) -> u64 {
        hash(&self.data_frames)
    }
}

// /// Event
// #[derive(Clone, Debug)]
// enum Event {
//     Add,
//     Delete {
//         row: usize,
//     },
//     Set {
//         row: usize,
//         column: &'static str,
//         value: Scalar,
//     },
//     Up {
//         row: usize,
//     },
// }

// impl Event {
//     fn apply(self, data: &mut FattyAcids) -> PolarsResult<()> {
//         match self {
//             Self::Add => data.add(),
//             Self::Delete { row } => data.delete(row),
//             Self::Set { row, column, value } => data.set(row, column, value),
//             Self::Up { row } => data.up(row),
//         }
//     }
// }

pub(crate) mod control;

mod names;
mod properties;
mod table;
// mod widgets;
