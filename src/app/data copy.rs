use anyhow::Result;
use polars::{functions::concat_df_diagonal, prelude::*};
use ron::{extensions::Extensions, ser::PrettyConfig};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display, Formatter},
    fs::write,
    hash::{Hash, Hasher},
    path::Path,
};

use crate::utils::ExprExt;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Bundle {
    pub(crate) entries: Vec<Data>,
    // pub(crate) triacylglycerols: DataFrame,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Data {
    pub(crate) fatty_acids: DataFrame,
    // pub(crate) triacylglycerols: DataFrame,
}

impl Data {
    pub(crate) const fn new(fatty_acids: DataFrame) -> Self {
        Self { fatty_acids }
    }

    pub(crate) fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let value = self.fatty_acids.select(["FA", "TAG", "DAG1223", "MAG2"]);
        let contents = ron::ser::to_string_pretty(
            &value?,
            PrettyConfig::new().extensions(Extensions::IMPLICIT_SOME),
        )?;
        write(path, contents)?;
        Ok(())
    }

    pub(crate) fn add(&mut self) -> PolarsResult<()> {
        self.fatty_acids = concat(
            [
                self.fatty_acids.clone().lazy(),
                df! {
                    "FA" => df! {
                        "Label" => &[""],
                        "Carbons" => &[0u8],
                        // "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                        // "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                        "Doubles" => &[Series::new_empty("", &DataType::Int8)],
                        "Triples" => &[Series::new_empty("", &DataType::Int8)],
                    }?.into_struct(""),
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
        self.fatty_acids = self
            .fatty_acids
            .slice(0, row)
            .vstack(&self.fatty_acids.slice((row + 1) as _, usize::MAX))?;
        self.fatty_acids.as_single_chunk_par();
        Ok(())
    }

    pub(crate) fn set(
        &mut self,
        row: usize,
        column: &str,
        value: LiteralValue,
    ) -> PolarsResult<()> {
        let lazy_frame = self
            .fatty_acids
            .clone()
            .lazy()
            .with_row_index("Index", None)
            .with_column(
                
                when(col("Index").eq(lit(row as i64)))
                    .then({
                        println!("set: {row}, {column}, {value:?}");
                        if let Some((prefix, suffix)) = column.split_once('.') {
                            println!("split_once: {prefix}, {suffix}");
                            // println!("{}", self.fatty_acids);
                            // let field = col(prefix).r#struct().field_by_name(suffix);
                            // if let "Doubles" | "Triples" = suffix {
                            //     field;
                            // }
                            // col(prefix)
                            //     .r#struct()
                            //     .with_fields(vec![lit(value).alias("suffix")])?
                            //     .alias("FA")
                            // as_struct(vec![
                            //     lit(value).alias(suffix),
                            //     col(prefix),
                            //     // col(prefix).r#struct().field_by_names(),
                            // ]).alias("FACK")
                            // col(prefix).alias("FACK")
                            col(prefix)
                                .r#struct()
                                .with_fields(vec![lit(value).alias(suffix)])?
                                .alias("FA")
                            // lit(88).alias(column)
                        } else {
                            lit(value).alias(column)
                        }
                        // match column {
                        //     "FA.Label" | "FA.Carbons" | "TAG" | "DAG1223" | "MAG2" => lit(value),
                        //     "FA.Doubles" | "FA.Triples" => lit(Series::from_any_values(
                        //         "",
                        //         &[AnyValue::to_any_value(value)?],
                        //         false,
                        //     )?),
                        //     _ => unreachable!(),
                        // }

                        // if let LiteralValue::Binary(binary) = value {
                        //     lit(Series::from_any_values(
                        //         "",
                        //         &[AnyValue::List(Series::new("", binary))],
                        //         false,
                        //     )?)
                        // }
                    })
                    .otherwise(col(column)),
            );
        println!("GGGG{}", lazy_frame.clone().collect()?);
        self.fatty_acids = lazy_frame.drop(["Index"]).collect()?;
        // println!("self.data_frame: {}", data);
        Ok(())
    }

    pub(crate) fn up(&mut self, row: usize) -> PolarsResult<()> {
        if row > 0 {
            self.fatty_acids = self
                .fatty_acids
                .slice(0, row - 1)
                .vstack(&self.fatty_acids.slice(row as _, 1))?
                .vstack(&self.fatty_acids.slice((row - 1) as _, 1))?
                .vstack(&self.fatty_acids.slice((row + 1) as _, usize::MAX))?;
            self.fatty_acids.as_single_chunk_par();
        }
        Ok(())
    }
}

impl Display for Data {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.fatty_acids, f)
    }
}

impl Default for Data {
    fn default() -> Self {
        Self {
            fatty_acids: DataFrame::empty_with_schema(&Schema::from_iter([
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
            ])),
        }
    }
}

impl Hash for Data {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for column in self.fatty_acids.get_columns() {
            for label in column.iter() {
                label.hash(state);
            }
        }
    }
}
