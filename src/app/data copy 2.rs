use crate::{
    localization::localize,
    utils::{ExprExt, VecExt},
};
use anyhow::Result;
use egui::{Label, Response, RichText, Sense, Sides, Ui, Widget};
use egui_dnd::dnd;
use egui_phosphor::regular::{ARROWS_OUT_CARDINAL, ARROW_FAT_LINE_UP, TRASH};
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

// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub(crate) struct Bundle {
//     pub(crate) entries: Vec<Data>,
//     // pub(crate) triacylglycerols: DataFrame,
// }

// #[derive(Clone, Debug, Deserialize, Serialize)]
// pub(crate) struct Data {
//     pub(crate) fatty_acids: DataFrame,
//     // pub(crate) triacylglycerols: DataFrame,
// }

// impl Data {
//     pub(crate) const fn new(fatty_acids: DataFrame) -> Self {
//         Self { fatty_acids }
//     }

//     pub(crate) fn save(&self, path: impl AsRef<Path>) -> Result<()> {
//         let value = self.fatty_acids.select(["FA", "TAG", "DAG1223", "MAG2"]);
//         let contents = ron::ser::to_string_pretty(
//             &value?,
//             PrettyConfig::new().extensions(Extensions::IMPLICIT_SOME),
//         )?;
//         write(path, contents)?;
//         Ok(())
//     }

//     pub(crate) fn add(&mut self) -> PolarsResult<()> {
//         self.fatty_acids = concat(
//             [
//                 self.fatty_acids.clone().lazy(),
//                 df! {
//                     "FA" => df! {
//                         "Label" => &[""],
//                         "Carbons" => &[0u8],
//                         // "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         // "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Doubles" => &[Series::new_empty("", &DataType::Int8)],
//                         "Triples" => &[Series::new_empty("", &DataType::Int8)],
//                     }?.into_struct(""),
//                     "TAG" => &[0.0],
//                     "DAG1223" => &[0.0],
//                     "MAG2" => &[0.0],
//                 }?
//                 .lazy(),
//             ],
//             UnionArgs {
//                 rechunk: true,
//                 diagonal: true,
//                 ..Default::default()
//             },
//         )?
//         .collect()?;
//         Ok(())
//     }

//     // https://stackoverflow.com/questions/71486019/how-to-drop-row-in-polars-python
//     // https://stackoverflow.com/a/71495211/1522758
//     pub(crate) fn delete(&mut self, row: usize) -> PolarsResult<()> {
//         self.fatty_acids = self
//             .fatty_acids
//             .slice(0, row)
//             .vstack(&self.fatty_acids.slice((row + 1) as _, usize::MAX))?;
//         self.fatty_acids.as_single_chunk_par();
//         Ok(())
//     }

//     pub(crate) fn set(
//         &mut self,
//         row: usize,
//         mut column: &str,
//         value: LiteralValue,
//     ) -> PolarsResult<()> {
//         self.fatty_acids = self
//             .fatty_acids
//             .clone()
//             .lazy()
//             .with_row_index("Index", None)
//             .with_column(
//                 when(col("Index").eq(lit(row as i64)))
//                     .then({
//                         if let Some((prefix, suffix)) = column.split_once('.') {
//                             column = prefix;
//                             let field = if let LiteralValue::Binary(binary) = value {
//                                 lit(Series::from_any_values(
//                                     "",
//                                     &[AnyValue::List(Series::from_iter(binary.r#as()))],
//                                     false,
//                                 )?)
//                             } else {
//                                 lit(value)
//                             };
//                             col(prefix)
//                                 .r#struct()
//                                 .with_fields(vec![field.alias(suffix)])?
//                         } else {
//                             lit(value).alias(column)
//                         }
//                     })
//                     .otherwise(col(column)),
//             )
//             .drop(["Index"])
//             .collect()?;
//         Ok(())
//     }

//     pub(crate) fn up(&mut self, row: usize) -> PolarsResult<()> {
//         if row > 0 {
//             self.fatty_acids = self
//                 .fatty_acids
//                 .slice(0, row - 1)
//                 .vstack(&self.fatty_acids.slice(row as _, 1))?
//                 .vstack(&self.fatty_acids.slice((row - 1) as _, 1))?
//                 .vstack(&self.fatty_acids.slice((row + 1) as _, usize::MAX))?;
//             self.fatty_acids.as_single_chunk_par();
//         }
//         Ok(())
//     }
// }

// impl Default for Data {
//     fn default() -> Self {
//         Self {
//             fatty_acids: DataFrame::empty_with_schema(&Schema::from_iter([
//                 Field::new(
//                     "FA".into(),
//                     DataType::Struct(vec![
//                         Field::new("Label".into(), DataType::String),
//                         Field::new("Carbons".into(), DataType::UInt8),
//                         Field::new("Doubles".into(), DataType::List(Box::new(DataType::Int8))),
//                         Field::new("Triples".into(), DataType::List(Box::new(DataType::Int8))),
//                     ]),
//                 ),
//                 Field::new("TAG".into(), DataType::Float64),
//                 Field::new("DAG1223".into(), DataType::Float64),
//                 Field::new("MAG2".into(), DataType::Float64),
//             ])),
//         }
//     }
// }

// impl Display for Data {
//     fn fmt(&self, f: &mut Formatter) -> fmt::Result {
//         Display::fmt(&self.fatty_acids, f)
//     }
// }

// impl Hash for Data {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         for fatty_acid in self.fatty_acids["FA"].iter() {
//             fatty_acid.hash(state);
//         }
//         for tag in self.fatty_acids["TAG"].iter() {
//             tag.hash(state);
//         }
//         for dag1223 in self.fatty_acids["DAG1223"].iter() {
//             dag1223.hash(state);
//         }
//         for mag2 in self.fatty_acids["MAG2"].iter() {
//             mag2.hash(state);
//         }
//     }
// }

// pub(crate) fn deserialize(content: &str) -> Result<Entry, SpannedError> {
//     let data_frame = ron::de::from_str(&content)?;
//     Ok(Entry {
//         fatty_acids: FattyAcids(data_frame),
//         ..Default::default()
//     })
// }

/// Data
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Data {
    pub(crate) entries: Vec<Entry>,
}

impl Data {
    pub(crate) fn checked(&self) -> impl Iterator<Item = (usize, &Entry)> {
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.checked)
    }

    pub(crate) fn save(&self) -> Result<()> {
        for (index, entry) in self.checked() {
            if entry.checked {
                entry.fatty_acids.save(format!("{index}.utca.ron"))?;
            }
        }
        Ok(())
    }
}

impl Widget for &mut Data {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.visuals_mut().collapsing_header_frame = true;
        ui.collapsing(RichText::new(localize!("entries")).heading(), |ui| {
            // let mut up = None;
            let mut remove = None;
            dnd(ui, ui.next_auto_id()).show_vec(&mut self.entries, |ui, entry, handle, state| {
                ui.horizontal(|ui| {
                    // ui.checkbox(&mut entry.checked, "");
                    // ui.add(Label::new(&entry.name).truncate())
                    //     .on_hover_text(format!("{:?}", entry.fatty_acids.shape()));
                    // let amount = ui.available_width()
                    //     - ui.spacing().interact_size.y
                    //     - 2.0 * ui.spacing().button_padding.x;
                    // ui.add_space(amount);
                    // if ui.button(TRASH).clicked() {
                    //     remove = Some(state.index);
                    // }
                    Sides::new().show(
                        ui,
                        |ui| {
                            handle.ui(ui, |ui| {
                                let _ = ui.label(ARROWS_OUT_CARDINAL);
                            });
                            ui.checkbox(&mut entry.checked, "");
                            ui.add(Label::new(&entry.name).truncate())
                                .on_hover_text(format!("{:?}", entry.fatty_acids.shape()));
                        },
                        |ui| {
                            if ui.button(TRASH).clicked() {
                                remove = Some(state.index);
                            }
                        },
                    );
                });
            });

            // let mut checked = Vec::new();
            // for (index, entry) in self.entries.iter_mut().enumerate() {
            //     ui.vertical(|ui| {
            //         Sides::new().show(
            //             ui,
            //             |ui| {
            //                 if ui.button(ARROW_FAT_LINE_UP).clicked() {
            //                     up = Some(index);
            //                 }
            //                 ui.checkbox(&mut entry.checked, "");
            //                 ui.add(Label::new(&entry.name).truncate())
            //                     .on_hover_text(format!("{:?}", entry.fatty_acids.shape()));
            //             },
            //             |ui| {
            //                 if ui.button(TRASH).clicked() {
            //                     remove = Some(index);
            //                 }
            //             },
            //         );
            //         checked.push(&mut entry.checked);
            //     });
            // }

            if let Some(index) = remove {
                self.entries.remove(index);
                ui.ctx().request_repaint();
            }
            // if let Some(index) = up {
            //     self.entries.swap(index, index.saturating_sub(1));
            //     ui.ctx().request_repaint();
            // }
        });
        ui.allocate_response(Default::default(), Sense::hover())
        // dnd(ui, Id::new("dnd").with("files")).show_vec(
        //     &mut self.data,
        //     |ui, item, handle, state| {
        //         ui.horizontal(|ui| {
        //             handle.ui(ui, |ui| {
        //                 let _ = ui.button(if state.dragged { "👊" } else { "✋" });
        //             });
        //             ui.radio_value(&mut context.state.index, state.index, "");
        //             ui.add(Label::new(&item.meta.name).truncate(true));
        //             if ui.button("🗑").clicked() {
        //                 remove = Some(state.index);
        //             }
        //         });
        //     },
        // );
        // if let Some(index) = remove {
        //     context.state.entries.remove(index);
        //     if index <= context.state.index {
        //         context.state.index = context.state.index.saturating_sub(1);
        //     }
        //     if context.state.entries.is_empty() {
        //         context.state.entries.push(Default::default());
        //     }
        // }
    }
}

/// Entry
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) struct Entry {
    pub(crate) name: String,
    pub(crate) checked: bool,
    pub(crate) fatty_acids: FattyAcids,
    // pub(crate) triacylglycerols: DataFrame,
}

impl From<DataFrame> for Entry {
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

    pub(crate) fn set(
        &mut self,
        row: usize,
        mut column: &str,
        value: LiteralValue,
    ) -> PolarsResult<()> {
        self.0 = self
            .0
            .clone()
            .lazy()
            .with_row_index("Index", None)
            .with_column(
                when(col("Index").eq(lit(row as i64)))
                    .then({
                        if let Some((prefix, suffix)) = column.split_once('.') {
                            column = prefix;
                            let field = if let LiteralValue::Binary(binary) = value {
                                lit(Series::from_any_values(
                                    PlSmallStr::EMPTY,
                                    &[AnyValue::List(Series::from_iter(binary.r#as()))],
                                    false,
                                )?)
                            } else {
                                lit(value)
                            };
                            col(prefix)
                                .r#struct()
                                .with_fields(vec![field.alias(suffix)])?
                        } else {
                            lit(value).alias(column)
                        }
                    })
                    .otherwise(col(column)),
            )
            .drop(["Index"])
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
