use crate::localization::localize;
use anyhow::Result;
use egui::{Grid, Label, Response, Sides, Ui, Widget};
use egui_dnd::dnd;
use egui_phosphor::regular::{ARROWS_OUT_CARDINAL, TRASH};
use polars::prelude::*;
use ron::{extensions::Extensions, ser::PrettyConfig};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    fs::write,
    hash::{Hash, Hasher},
    ops::Deref,
    path::Path,
};

/// Data
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Data {
    pub(crate) files: Vec<CheckableFile>,
}

impl Data {
    pub(crate) fn checked(&self) -> Vec<File> {
        self.files
            .iter()
            .filter_map(|checkable| checkable.checked.then_some(checkable.file.clone()))
            .collect()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub(crate) fn push(&mut self, file: File) {
        self.files.push(CheckableFile {
            file,
            checked: true,
        });
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
        dnd(ui, ui.next_auto_id()).show_vec(
            &mut self.files,
            |ui, CheckableFile { checked, file }, handle, state| {
                ui.horizontal(|ui| {
                    Sides::new().show(
                        ui,
                        |ui| {
                            handle.ui(ui, |ui| {
                                let _ = ui.label(ARROWS_OUT_CARDINAL);
                            });
                            ui.checkbox(checked, "");
                            ui.add(Label::new(&file.name).truncate()).on_hover_ui(|ui| {
                                Grid::new(ui.next_auto_id()).show(ui, |ui| {
                                    ui.label("Rows");
                                    ui.label(file.fatty_acids.height().to_string());
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
            },
        );
        if let Some(index) = remove {
            self.files.remove(index);
            ui.ctx().request_repaint();
        }
        response
    }
}

/// Checkable
#[derive(Clone, Debug, Default, Deserialize, Hash, Serialize)]
pub(crate) struct CheckableFile {
    pub(crate) file: File,
    pub(crate) checked: bool,
}

/// File
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) struct File {
    pub(crate) name: String,
    pub(crate) fatty_acids: FattyAcids,
}

impl From<DataFrame> for File {
    fn from(value: DataFrame) -> Self {
        Self {
            fatty_acids: FattyAcids(value),
            ..Default::default()
        }
    }
}

/// Fatty acids
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub(crate) struct FattyAcids(pub(crate) DataFrame);

impl Eq for FattyAcids {}

impl PartialEq for FattyAcids {
    fn eq(&self, other: &Self) -> bool {
        self.0.equals(&other.0)
    }
}

impl FattyAcids {
    pub(crate) fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let value = self.0.select(["FA", "TAG", "DAG1223", "MAG2"]);
        let contents = ron::ser::to_string_pretty(
            &value?,
            PrettyConfig::new().extensions(Extensions::IMPLICIT_SOME | Extensions::UNWRAP_NEWTYPES),
        )?;
        write(path, contents)?;
        Ok(())
    }

    pub(crate) fn add(&mut self) -> PolarsResult<()> {
        self.0 = concat(
            [
                self.0.clone().lazy(),
                df! {
                    "FA" => df! {
                        "Label" => &[""],
                        "Carbons" => &[0u8],
                        "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                        "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                    }?.into_struct(PlSmallStr::EMPTY),
                    "TAG" => &[0.0],
                    "DAG1223" => &[0.0],
                    "MAG2" => &[0.0],
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

    // https://stackoverflow.com/questions/71486019/how-to-drop-row-in-polars-python
    // https://stackoverflow.com/a/71495211/1522758
    pub(crate) fn delete(&mut self, row: usize) -> PolarsResult<()> {
        self.0 = self
            .0
            .slice(0, row)
            .vstack(&self.0.slice((row + 1) as _, usize::MAX))?;
        self.0.as_single_chunk_par();
        Ok(())
    }

    pub(crate) fn set(&mut self, row: usize, column: &str, value: Scalar) -> PolarsResult<()> {
        self.0 = self
            .0
            .clone()
            .lazy()
            .unnest(["FA"])
            .with_row_index("Index", None)
            .with_column(
                when(col("Index").eq(lit(row as i64)))
                    .then(lit(value).alias(column))
                    .otherwise(col(column)),
            )
            .select([
                as_struct(vec![
                    col("Label"),
                    col("Carbons"),
                    col("Doubles"),
                    col("Triples"),
                ])
                .alias("FA"),
                col("TAG"),
                col("DAG1223"),
                col("MAG2"),
            ])
            .collect()?;
        Ok(())
    }

    pub(crate) fn up(&mut self, row: usize) -> PolarsResult<()> {
        if row > 0 {
            self.0 = self
                .0
                .slice(0, row - 1)
                .vstack(&self.0.slice(row as _, 1))?
                .vstack(&self.0.slice((row - 1) as _, 1))?
                .vstack(&self.0.slice((row + 1) as _, usize::MAX))?;
            self.0.as_single_chunk_par();
        }
        Ok(())
    }
}

impl Default for FattyAcids {
    fn default() -> Self {
        Self(DataFrame::empty_with_schema(&Schema::from_iter([
            Field::new(
                "FA".into(),
                DataType::Struct(vec![
                    Field::new("Label".into(), DataType::String),
                    Field::new("Carbons".into(), DataType::UInt8),
                    Field::new("Doubles".into(), DataType::List(Box::new(DataType::Int8))),
                    Field::new("Triples".into(), DataType::List(Box::new(DataType::Int8))),
                ]),
            ),
            Field::new("TAG".into(), DataType::Float64),
            Field::new("DAG1223".into(), DataType::Float64),
            Field::new("MAG2".into(), DataType::Float64),
        ])))
    }
}

impl Deref for FattyAcids {
    type Target = DataFrame;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for FattyAcids {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Hash for FattyAcids {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for fatty_acid in self["FA"].phys_iter() {
            fatty_acid.hash(state);
        }
        for tag in self["TAG"].phys_iter() {
            tag.hash(state);
        }
        for dag1223 in self["DAG1223"].phys_iter() {
            dag1223.hash(state);
        }
        for mag2 in self["MAG2"].phys_iter() {
            mag2.hash(state);
        }
    }
}
