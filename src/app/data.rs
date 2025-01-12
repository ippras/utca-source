use crate::localization::localize;
use anyhow::Result;
use egui::{Grid, Label, Response, Sides, Ui, Widget};
use egui_dnd::dnd;
use egui_phosphor::regular::{ARROWS_OUT_CARDINAL, TRASH};
use polars::prelude::*;
use ron::{extensions::Extensions, ser::PrettyConfig};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
    fs::write,
    hash::{Hash, Hasher},
    io::Read,
    iter::zip,
    ops::Deref,
    path::Path,
};
use utca::metadata::{IpcReaderExt as _, MetaDataFrame, Metadata};

/// Data
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Data {
    pub(crate) frames: Vec<MetaDataFrame>,
    pub(crate) checked: Vec<bool>,
}

impl Data {
    pub(crate) fn checked(&self) -> Vec<MetaDataFrame> {
        zip(&self.frames, &self.checked)
            .filter_map(|(frame, checked)| checked.then_some(frame.clone()))
            .collect()
    }

    pub(crate) fn is_empty(&self) -> bool {
        assert_eq!(self.frames.len(), self.checked.len());
        self.frames.is_empty()
    }

    pub(crate) fn push(&mut self, frame: MetaDataFrame) {
        self.frames.push(frame);
        self.checked.push(false);
    }

    pub(crate) fn remove(&mut self, index: usize) {
        self.frames.remove(index);
        self.checked.remove(index);
    }

    pub(crate) fn save(&self) -> Result<()> {
        // for (index, entry) in self.checked() {
        //     entry.fatty_acids.save(format!("{index}.utca.ron"))?;
        // }
        Ok(())
    }
}

impl Widget for &mut Data {
    fn ui(self, ui: &mut Ui) -> Response {
        let response = ui.heading(localize!("files"));
        let mut remove = None;
        dnd(ui, ui.next_auto_id()).show_vec(&mut self.frames, |ui, frame, handle, state| {
            ui.horizontal(|ui| {
                Sides::new().show(
                    ui,
                    |ui| {
                        handle.ui(ui, |ui| {
                            let _ = ui.label(ARROWS_OUT_CARDINAL);
                        });
                        ui.checkbox(&mut self.checked[state.index], "");
                        let text = if let Some(version) = &frame.meta.version {
                            &format!("{} {version}", frame.meta.name)
                        } else {
                            &frame.meta.name
                        };
                        ui.add(Label::new(text).truncate()).on_hover_ui(|ui| {
                            Grid::new(ui.next_auto_id()).show(ui, |ui| {
                                ui.label("Rows");
                                ui.label(frame.data.height().to_string());
                            });
                        });
                    },
                    |ui| {
                        if ui.button(TRASH).clicked() {
                            remove = Some(state.index);
                        }
                    },
                );
            });
        });
        if let Some(index) = remove {
            self.remove(index);
            ui.ctx().request_repaint();
        }
        response
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub(crate) enum Format {
    #[default]
    Bin,
    Parquet,
    Ron,
}

pub(crate) fn save(path: impl AsRef<Path>, format: Format, data_frame: DataFrame) -> Result<()> {
    println!("data_frame: {:#?}", data_frame.schema());
    match format {
        Format::Bin => {
            let contents = bincode::serialize(&data_frame)?;
            write(path, contents)?;
        }
        Format::Parquet => {
            // let mut file = File::create(path)?;
            // ParquetWriter::new(&mut file).finish(&mut data_frame)?;
        }
        Format::Ron => {
            let file = std::fs::File::create(path)?;
            ron::ser::to_writer_pretty(
                file,
                &data_frame,
                PrettyConfig::default().extensions(Extensions::IMPLICIT_SOME),
            )?;
            // let contents = ron::ser::to_string_pretty(
            //     &data_frame,
            //     PrettyConfig::default().extensions(Extensions::IMPLICIT_SOME),
            // )?;
            // write(path, contents)?;
        }
    }
    Ok(())
}
