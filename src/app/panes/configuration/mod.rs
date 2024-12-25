use self::{
    area::Area,
    control::Control,
    names::Names,
    properties::Properties,
    widgets::{Change, FattyAcidWidget},
};
use crate::{
    app::data::{FattyAcids, File},
    localization::localize,
    special::fatty_acid::{COMMON, DisplayWithOptions, FattyAcid},
    utils::{
        polars::DataFrameExt as _,
        ui::{SubscriptedTextFormat, UiExt},
    },
};
use egui::{ComboBox, Direction, Id, Layout, RichText, Ui, util::hash};
use egui_extras::{Column, TableBuilder};
use egui_phosphor::regular::{
    ARROW_FAT_LINE_UP, ARROWS_HORIZONTAL, CALCULATOR, GEAR, MINUS, PENCIL, PLUS,
};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::f64::NAN;
use tracing::error;

/// Configuration pane
#[derive(Default, Deserialize, Serialize)]
pub(crate) struct Pane {
    pub(crate) source: Vec<File>,
    pub(crate) control: Control,
}

impl Pane {
    pub(crate) const fn new(data: Vec<File>) -> Self {
        Self {
            source: data,
            control: Control::new(),
        }
    }

    pub(crate) fn header(&mut self, ui: &mut Ui) {
        ui.visuals_mut().button_frame = false;
        let selected_text = match self.source.get(0) {
            Some(file) => &file.name,
            None => "",
        };
        ComboBox::from_id_salt(ui.next_auto_id())
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                for index in 0..self.source.len() {
                    ui.selectable_value(&mut self.control.index, index, &self.source[index].name);
                }
            });
        ui.separator();
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
        ui.toggle_value(&mut self.control.open, RichText::new(GEAR).heading())
            .on_hover_text(localize!("settings"));
        if ui
            .button(RichText::new(CALCULATOR).heading())
            .on_hover_text(localize!("calculation"))
            .clicked()
        {
            // FATTY_ACIDS_SCHEMA.names();
            const NAMES: [&str; 4] = ["Label", "Carbons", "Doubles", "Triples"];
            let mut lazy_frame: Option<LazyFrame> = None;
            for file in &mut self.source {
                let next = file.fatty_acids.0.clone().lazy().select([
                    col("FA").struct_().field_by_names(["*"]),
                    as_struct(vec![col("TAG"), col("DAG1223"), col("MAG2")])
                        .alias(file.name.clone()),
                ]);
                lazy_frame = Some(if let Some(current) = lazy_frame {
                    current.join(
                        next,
                        &NAMES.map(col),
                        &NAMES.map(col),
                        JoinArgs::new(JoinType::Full).with_coalesce(JoinCoalesce::CoalesceColumns),
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
    }

    pub(crate) fn content(&mut self, ui: &mut Ui) {
        // let mut tiles = Tiles::default();
        // let children = vec![tiles.insert_pane(Pane {}), tiles.insert_pane(Pane {})];
        // let root = tiles.insert_tab_tile(children);
        // let tree = Tree::new("my_tree", root, tiles);
        // tree.ui(behavior, ui);
        ui.separator();
        self.control.windows(ui);
        if self.source.is_empty() {
            return;
        }
        let file = &self.source[self.control.index];
        // let Some(entry) = behavior.data.entries.iter_mut().find(|entry| entry.checked) else {
        //     return;
        // };
        let height = ui.spacing().interact_size.y;
        let width = ui.spacing().interact_size.x;
        let total_rows = file.fatty_acids.height();
        let fatty_acids = file.fatty_acids.destruct("FA");
        // let triples = fatty_acids.explode(["Triples"])?;
        // let triples = triples["Triples"].i8()?;
        let labels = fatty_acids.str("Label");
        let carbons = fatty_acids.u8("Carbons");
        let doubles = fatty_acids.list("Doubles");
        let triples = fatty_acids.list("Triples");
        let tags = file.fatty_acids.f64("TAG");
        let dags1223 = file.fatty_acids.f64("DAG1223");
        let mags2 = file.fatty_acids.f64("MAG2");
        let mut event = None;
        let mut builder = TableBuilder::new(ui)
            .cell_layout(Layout::centered_and_justified(Direction::LeftToRight));
        if self.control.settings.editable {
            builder = builder.column(Column::exact(width / 2.0));
        }
        builder = builder
            .column(Column::auto_with_initial_suggestion(width))
            .columns(Column::auto(), 3);
        if self.control.settings.editable {
            builder = builder.column(Column::exact(width));
        }
        builder
            .auto_shrink(false)
            .resizable(self.control.settings.resizable)
            .striped(true)
            .header(height, |mut row| {
                if self.control.settings.editable {
                    row.col(|_ui| {});
                }
                row.col(|ui| {
                    ui.heading(localize!("fatty_acid.abbreviation"))
                        .on_hover_text(localize!("fatty_acid"));
                });
                row.col(|ui| {
                    ui.heading(localize!("triacylglycerol.abbreviation"))
                        .on_hover_text(localize!("triacylglycerol"));
                });
                row.col(|ui| {
                    ui.heading(format!(
                        "1,2/2,3-{}",
                        localize!("diacylglycerol.abbreviation"),
                    ))
                    .on_hover_text(format!("sn-1,2/2,3 {}", localize!("diacylglycerol")));
                });
                row.col(|ui| {
                    ui.heading(format!("2-{}", localize!("monoacylglycerol.abbreviation")))
                        .on_hover_text(format!("sn-2 {}", localize!("monoacylglycerol")));
                });
            })
            .body(|body| {
                let precision = |value| format!("{value:.*}", self.control.settings.precision);
                body.rows(height, total_rows + 1, |mut row| {
                    let index = row.index();
                    if index < total_rows {
                        // Move row
                        if self.control.settings.editable {
                            row.col(|ui| {
                                if ui.button(ARROW_FAT_LINE_UP).clicked() {
                                    event = Some(Event::Up { row: index });
                                }
                            });
                        }
                        // FA
                        row.col(|ui| {
                            let label = labels.get(index).unwrap();
                            let carbons = carbons.get(index).unwrap();
                            let doubles = doubles.get_as_series(index).unwrap();
                            let triples = triples.get_as_series(index).unwrap();
                            let fatty_acid = &mut FattyAcid {
                                carbons,
                                doubles: doubles.i8().unwrap().to_vec_null_aware().left().unwrap(),
                                triples: triples.i8().unwrap().to_vec_null_aware().left().unwrap(),
                            };
                            let text = if label.is_empty() { "C" } else { label };
                            let title = ui.subscripted_text(
                                text,
                                &format!("{:#}", fatty_acid.display(COMMON)),
                                SubscriptedTextFormat {
                                    widget: true,
                                    ..Default::default()
                                },
                            );
                            let mut response = if self.control.settings.editable {
                                ui.menu_button(title, |ui| {
                                    let mut label = label.to_owned();
                                    if let Some(change) =
                                        FattyAcidWidget::new(&mut label, fatty_acid).ui(ui)
                                    {
                                        let (column, value) = match change {
                                            Change::Label => (
                                                "Label",
                                                Scalar::new(
                                                    DataType::String,
                                                    AnyValue::StringOwned(label.into()),
                                                ),
                                            ),
                                            Change::Carbons => (
                                                "Carbons",
                                                Scalar::new(
                                                    DataType::UInt8,
                                                    fatty_acid.carbons.into(),
                                                ),
                                            ),
                                            Change::Doubles => (
                                                "Doubles",
                                                Scalar::new(
                                                    DataType::List(Box::new(DataType::Int8)),
                                                    AnyValue::List(Series::from_iter(
                                                        fatty_acid.doubles.clone(),
                                                    )),
                                                ),
                                            ),
                                            Change::Triples => (
                                                "Triples",
                                                Scalar::new(
                                                    DataType::List(Box::new(DataType::Int8)),
                                                    AnyValue::List(Series::from_iter(
                                                        fatty_acid.triples.clone(),
                                                    )),
                                                ),
                                            ),
                                        };
                                        event = Some(Event::Set {
                                            row: index,
                                            column,
                                            value,
                                        })
                                    }
                                })
                                .response
                            } else {
                                ui.label(title)
                            }
                            .on_hover_ui(|ui| {
                                ui.heading(localize!("fatty_acid"));
                                ui.label(format!(
                                    "{}: {:#}",
                                    localize!("formula"),
                                    fatty_acid.display(COMMON),
                                ));
                                ui.label(format!(
                                    "{}: C{}H{}O2",
                                    localize!("formula"),
                                    fatty_acid.c(),
                                    fatty_acid.h(),
                                ));
                            });
                            if self.control.settings.properties {
                                response = response.on_hover_ui(|ui| {
                                    ui.add(Properties::new(fatty_acid));
                                });
                            }
                            if self.control.settings.names {
                                response.on_hover_ui(|ui| {
                                    ui.add(Names::new(fatty_acid));
                                });
                            }
                        });
                        // TAG
                        row.col(|ui| {
                            let mut value = tags.get(index).unwrap_or_default();
                            if ui
                                .add(Area::new(
                                    &mut value,
                                    self.control.settings.editable,
                                    self.control.settings.precision,
                                ))
                                .changed()
                            {
                                event = Some(Event::Set {
                                    row: index,
                                    column: "TAG",
                                    value: Scalar::new(DataType::Float64, value.into()),
                                });
                            }
                        });
                        // DAG
                        row.col(|ui| {
                            let mut value = dags1223.get(index).unwrap_or_default();
                            if ui
                                .add(Area::new(
                                    &mut value,
                                    self.control.settings.editable,
                                    self.control.settings.precision,
                                ))
                                .changed()
                            {
                                event = Some(Event::Set {
                                    row: index,
                                    column: "DAG1223",
                                    value: Scalar::new(DataType::Float64, value.into()),
                                });
                            }
                        });
                        // MAG
                        row.col(|ui| {
                            let mut value = mags2.get(index).unwrap_or_default();
                            if ui
                                .add(Area::new(
                                    &mut value,
                                    self.control.settings.editable,
                                    self.control.settings.precision,
                                ))
                                .changed()
                            {
                                event = Some(Event::Set {
                                    row: index,
                                    column: "MAG2",
                                    value: Scalar::new(DataType::Float64, value.into()),
                                });
                            }
                        });
                        // Delete row
                        if self.control.settings.editable {
                            row.col(|ui| {
                                if ui.button(MINUS).clicked() {
                                    event = Some(Event::Delete { row: index });
                                }
                            });
                        }
                    } else {
                        if self.control.settings.editable {
                            row.col(|_ui| {});
                        }
                        row.col(|_ui| {});
                        // TAG
                        row.col(|ui| {
                            let value = tags.sum().unwrap_or(NAN);
                            ui.label(precision(value)).on_hover_text(value.to_string());
                        });
                        // DAG
                        row.col(|ui| {
                            let value = dags1223.sum().unwrap_or(NAN);
                            ui.label(precision(value)).on_hover_text(value.to_string());
                        });
                        // MAG
                        row.col(|ui| {
                            let value = mags2.sum().unwrap_or(NAN);
                            ui.label(precision(value)).on_hover_text(value.to_string());
                        });
                        // Add row
                        if self.control.settings.editable {
                            row.col(|ui| {
                                if ui.button(PLUS).clicked() {
                                    event = Some(Event::Add);
                                }
                            });
                        }
                    }
                });
            });
        // Mutable
        if let Some(event) = event {
            if let Err(error) = event.apply(&mut self.source[self.control.index].fatty_acids) {
                error!(%error);
            }
        }
    }

    pub(super) fn hash(&self) -> u64 {
        hash(&self.source)
    }
}

/// Event
#[derive(Clone, Debug)]
enum Event {
    Add,
    Delete {
        row: usize,
    },
    Set {
        row: usize,
        column: &'static str,
        value: Scalar,
    },
    Up {
        row: usize,
    },
}

impl Event {
    fn apply(self, data: &mut FattyAcids) -> PolarsResult<()> {
        match self {
            Self::Add => data.add(),
            Self::Delete { row } => data.delete(row),
            Self::Set { row, column, value } => data.set(row, column, value),
            Self::Up { row } => data.up(row),
        }
    }
}

pub(crate) mod control;

mod area;
mod names;
mod properties;
mod widgets;
