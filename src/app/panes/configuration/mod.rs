use self::{
    control::Control,
    table::{Event, TableView},
};
use super::PaneDelegate;
use crate::{localize, utils::save};
use anyhow::Result;
use egui::{
    CursorIcon, DragValue, Grid, Id, Response, RichText, ScrollArea, Ui, menu::bar, util::hash,
};
use egui_extras::{Column, DatePickerButton, TableBuilder};
use egui_phosphor::regular::{
    ARROWS_HORIZONTAL, CALCULATOR, ERASER, FLOPPY_DISK, GEAR, LIST, NOTE_PENCIL, PENCIL, TAG, TRASH,
};
use metadata::MetaDataFrame;
use polars::prelude::*;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use tracing::error;

pub(crate) static SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::from_iter([
        Field::new("Index".into(), DataType::UInt32),
        Field::new("Label".into(), DataType::String),
        Field::new(
            "FattyAcid".into(),
            DataType::Struct(vec![
                Field::new("Carbons".into(), DataType::UInt8),
                Field::new(
                    "Unsaturated".into(),
                    DataType::List(Box::new(DataType::Struct(vec![
                        Field::new("Index".into(), DataType::UInt8),
                        Field::new("Isomerism".into(), DataType::Int8),
                        Field::new("Unsaturation".into(), DataType::UInt8),
                    ]))),
                ),
            ]),
        ),
        Field::new("Triacylglycerol".into(), DataType::Float64),
        Field::new("Diacylglycerol1223".into(), DataType::Float64),
        Field::new("Monoacylglycerol2".into(), DataType::Float64),
    ])
});

/// Configuration pane
#[derive(Default, Deserialize, Serialize)]
pub(crate) struct Pane {
    pub(crate) frames: Vec<MetaDataFrame>,
    pub(crate) control: Control,
}

impl Pane {
    pub(crate) const fn new(frames: Vec<MetaDataFrame>) -> Self {
        Self {
            frames,
            control: Control::new(),
        }
    }

    pub(crate) const fn icon() -> &'static str {
        NOTE_PENCIL
    }

    pub(crate) fn title(&self) -> String {
        self.frames[self.control.index].meta.title()
    }

    fn header_content(&mut self, ui: &mut Ui) -> Response {
        let mut response = ui
            .heading(NOTE_PENCIL)
            .on_hover_text(localize!("configuration"));
        response |= ui.heading(self.title());
        response = response
            .on_hover_text(format!("{:x}", self.hash()))
            .on_hover_cursor(CursorIcon::Grab);
        ui.separator();
        // List
        ui.menu_button(RichText::new(LIST).heading(), |ui| {
            for index in 0..self.frames.len() {
                if ui
                    .selectable_value(
                        &mut self.control.index,
                        index,
                        self.frames[index].meta.title(),
                    )
                    .clicked()
                {
                    ui.close_menu();
                }
            }
        })
        .response
        .on_hover_text(localize!("list"));
        ui.separator();
        // Resize
        ui.toggle_value(
            &mut self.control.settings.resizable,
            RichText::new(ARROWS_HORIZONTAL).heading(),
        )
        .on_hover_text(localize!("resize"));
        // Edit
        ui.toggle_value(
            &mut self.control.settings.editable,
            RichText::new(PENCIL).heading(),
        )
        .on_hover_text(localize!("edit"));
        // Clear
        ui.add_enabled_ui(
            self.control.settings.editable && self.frames[self.control.index].data.height() > 0,
            |ui| {
                if ui
                    .button(RichText::new(ERASER).heading())
                    .on_hover_text(localize!("clear"))
                    .clicked()
                {
                    let data_frame = &mut self.frames[self.control.index].data;
                    *data_frame = data_frame.clear();
                }
            },
        );
        // Delete
        ui.add_enabled_ui(
            self.control.settings.editable && self.frames.len() > 1,
            |ui| {
                if ui
                    .button(RichText::new(TRASH).heading())
                    .on_hover_text(localize!("delete"))
                    .clicked()
                {
                    self.frames.remove(self.control.index);
                    self.control.index = 0;
                }
            },
        );
        ui.separator();
        // Settings
        ui.toggle_value(&mut self.control.open, RichText::new(GEAR).heading())
            .on_hover_text(localize!("settings"));
        ui.separator();
        // Save
        if ui
            .button(RichText::new(FLOPPY_DISK).heading())
            .on_hover_text(localize!("save"))
            .on_hover_text(format!("{}.utca.ipc", self.title()))
            .clicked()
        {
            if let Err(error) = self.save() {
                error!(%error);
            }
        }
        ui.separator();
        // Calculation
        if ui
            .button(RichText::new(CALCULATOR).heading())
            .on_hover_text(localize!("calculation"))
            .clicked()
        {
            ui.data_mut(|data| {
                data.insert_temp(
                    Id::new("Calculate"),
                    (self.frames.clone(), self.control.index),
                );
            });
        }
        ui.separator();
        response
    }

    fn body_meta(&mut self, ui: &mut Ui, index: usize) {
        ui.style_mut().visuals.collapsing_header_frame = true;
        ui.collapsing(RichText::new(format!("{TAG} Metadata")).heading(), |ui| {
            let height = ui.style().spacing.interact_size.y;
            let meta = &mut self.frames[index].meta;
            TableBuilder::new(ui)
                .column(Column::auto())
                .column(Column::remainder())
                .body(|mut body| {
                    // Name
                    body.row(height, |mut row| {
                        row.col(|ui| {
                            ui.label("Name");
                        });
                        row.col(|ui| {
                            ui.text_edit_singleline(&mut meta.name);
                        });
                    });
                    // Description
                    body.row(height, |mut row| {
                        row.col(|ui| {
                            ui.label("Description");
                        });
                        row.col(|ui| {
                            ui.text_edit_singleline(&mut meta.description);
                        });
                    });
                    // Authors
                    body.row(height, |mut row| {
                        row.col(|ui| {
                            ui.label("Authors");
                        });
                        row.col(|ui| {
                            let mut authors = meta.authors.join(",");
                            if ui.text_edit_singleline(&mut authors).changed() {
                                meta.authors =
                                    authors.split(",").map(|author| author.to_owned()).collect()
                            }
                            // for author in &mut meta.authors {
                            //     ui.text_edit_singleline(author);
                            // }
                        });
                    });
                    // Version
                    body.row(height, |mut row| {
                        row.col(|ui| {
                            ui.label("Version");
                        });
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                let mut checked = meta.version.is_some();
                                if ui.checkbox(&mut checked, "").changed() {
                                    Some(Version::new(0, 0, 0));
                                    meta.version = if checked {
                                        Some(Version::new(0, 0, 0))
                                    } else {
                                        None
                                    };
                                }
                                if let Some(version) = &mut meta.version {
                                    ui.menu_button(version.to_string(), |ui| {
                                        ui.visuals_mut().widgets.inactive =
                                            ui.visuals().widgets.active;
                                        Grid::new(ui.next_auto_id()).show(ui, |ui| {
                                            ui.label("Major");
                                            ui.add(DragValue::new(&mut version.major));
                                            ui.end_row();

                                            ui.label("Minor");
                                            ui.add(DragValue::new(&mut version.minor));
                                            ui.end_row();

                                            ui.label("Patch");
                                            ui.add(DragValue::new(&mut version.patch));
                                            ui.end_row();
                                        });
                                    });
                                }
                            });
                        });
                    });
                    // Date
                    body.row(height, |mut row| {
                        row.col(|ui| {
                            ui.label("Date");
                        });
                        row.col(|ui| {
                            ui.add(
                                DatePickerButton::new(meta.date.get_or_insert_default())
                                    .show_icon(false),
                            );
                        });
                    });
                });
            // StripBuilder::new(ui)
            //     .sizes(Size::exact(height), 2)
            //     .vertical(|mut strip| {
            //         // let mut row = |text, value| {
            //         //     strip.strip(|builder| {
            //         //         builder
            //         //             .size(Size::remainder())
            //         //             .size(Size::remainder().at_least(text_edit_width))
            //         //             .horizontal(|mut strip| {
            //         //                 strip.cell(|ui| {
            //         //                     ui.label(text);
            //         //                 });
            //         //                 strip.cell(|ui| {
            //         //                     changed |= ui
            //         //                         .add(TextEdit::singleline(value).hint_text(text))
            //         //                         .changed();
            //         //                 });
            //         //             });
            //         //     });
            //         // };
            //         // strip.strip(|builder| {
            //         //     builder
            //         //         .size(Size::initial(width))
            //         //         .size(Size::remainder().at_least(text_edit_width))
            //         //         .horizontal(|mut strip| {
            //         //             strip.cell(|ui| {
            //         //                 ui.label("Name");
            //         //             });
            //         //             strip.cell(|ui| {
            //         //                 ui.add(
            //         //                     TextEdit::singleline(&mut frame.meta.name)
            //         //                         .hint_text("Name"),
            //         //                 );
            //         //             });
            //         //         });
            //         // });
            //         // strip.strip(|builder| {
            //         //     builder
            //         //         .size(Size::relative(0.1).at_least(width))
            //         //         .size(Size::remainder().at_least(text_edit_width))
            //         //         .horizontal(|mut strip| {
            //         //             strip.cell(|ui| {
            //         //                 ui.label("Description");
            //         //             });
            //         //             strip.cell(|ui| {
            //         //                 ui.add(
            //         //                     TextEdit::singleline(&mut frame.meta.description)
            //         //                         .hint_text("Description"),
            //         //                 );
            //         //             });
            //         //         });
            //         // });
            //     });
            // Grid::new(ui.next_auto_id()).show(ui, |ui| {
            //     ui.label("Version");
            //     // ui.text_edit_singleline(&mut frame.meta.version);
            //     ui.end_row();

            //     ui.label("Name");
            //     ui.text_edit_singleline(&mut frame.meta.name);
            //     ui.end_row();

            //     ui.label("Description");
            //     ui.text_edit_multiline(&mut frame.meta.description);
            //     ui.end_row();

            //     ui.label("Authors");
            //     for author in &mut frame.meta.authors {
            //         ui.text_edit_singleline(author);
            //     }
            //     ui.end_row();

            //     // ui.label("Date");
            //     // ui.text_edit_singleline(&mut frame.meta.date);
            //     // ui.end_row();
            // });
        });
    }

    fn body_data(&mut self, ui: &mut Ui, index: usize) {
        let data_frame = &mut self.frames[index].data;
        if let Some(event) = TableView::new(data_frame, &self.control.settings).show(ui) {
            if let Err(error) = match event {
                Event::AddRow => self.add_row(),
                Event::DeleteRow(row) => self.delete_row(row),
            } {
                error!(%error);
            }
        }
    }

    fn add_row(&mut self) -> PolarsResult<()> {
        let data_frame = &mut self.frames[self.control.index].data;
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

    fn delete_row(&mut self, row: usize) -> PolarsResult<()> {
        let data_frame = &mut self.frames[self.control.index].data;
        let mut lazy_frame = data_frame.clone().lazy();
        lazy_frame = lazy_frame
            .filter(nth(0).neq(lit(row as u32)))
            .with_column(nth(0).cum_count(false) - lit(1));
        *data_frame = lazy_frame.collect()?;
        Ok(())
    }

    fn hash(&self) -> u64 {
        hash(&self.frames)
    }

    fn save(&mut self) -> Result<()> {
        let name = format!("{}.utca.ipc", self.title());
        save(&name, &mut self.frames[self.control.index])?;
        Ok(())
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
        if self.control.settings.editable {
            self.body_meta(ui, self.control.index);
        }
        self.body_data(ui, self.control.index);
    }
}

pub(crate) mod control;

mod names;
mod properties;
mod table;
