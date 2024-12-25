use super::general::{DataFrameExt as _, ExprExt as _};
use crate::{
    acylglycerol::Stereospecificity,
    app::panes::settings::composition::Composition,
    fatty_acid::FattyAcid,
    r#const::relative_atomic_mass::{C, H, O},
};
use polars::prelude::*;
use std::sync::LazyLock;

pub static FATTY_ACIDS_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::from_iter([
        Field::new("Label".into(), DataType::String),
        Field::new("Carbons".into(), DataType::UInt8),
        Field::new("Doubles".into(), DataType::List(Box::new(DataType::Int8))),
        Field::new("Triples".into(), DataType::List(Box::new(DataType::Int8))),
    ])
});

pub static DATA_SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::from_iter([
        Field::new(
            "Experimental".into(),
            DataType::Struct(vec![
                Field::new("TAG".into(), DataType::Float64),
                Field::new("DAG1223".into(), DataType::Float64),
                Field::new("MAG2".into(), DataType::Float64),
            ]),
        ),
        Field::new(
            "Theoretical".into(),
            DataType::Struct(vec![
                Field::new("TAG".into(), DataType::Float64),
                Field::new("DAG1223".into(), DataType::Float64),
                Field::new("MAG2".into(), DataType::Float64),
                Field::new("DAG13".into(), DataType::Float64),
                Field::new("DAG13".into(), DataType::Float64),
            ]),
        ),
        Field::new(
            "Calculated".into(),
            DataType::Struct(vec![
                Field::new("TAG".into(), DataType::Float64),
                Field::new("DAG1223".into(), DataType::Float64),
                Field::new("MAG2".into(), DataType::Float64),
            ]),
        ),
        Field::new(
            "EnrichmentFactor".into(),
            DataType::Struct(vec![
                Field::new("MAG2".into(), DataType::Float64),
                Field::new("DAG13".into(), DataType::Float64),
            ]),
        ),
        Field::new(
            "SelectivityFactor".into(),
            DataType::Struct(vec![
                Field::new("MAG2".into(), DataType::Float64),
                Field::new("DAG13".into(), DataType::Float64),
            ]),
        ),
    ])
});

/// Extension methods for [`DataFrame`]
pub trait DataFrameExt {
    fn fatty_acids(self) -> FattyAcidsDataFrame;
}

impl DataFrameExt for DataFrame {
    fn fatty_acids(self) -> FattyAcidsDataFrame {
        FattyAcidsDataFrame(self)
    }
}

/// Extension methods for [`Expr`]
pub(crate) trait ExprExt {
    fn fatty_acid(self) -> FattyAcidExpr;

    fn tag(self) -> TagsExpr;
}

impl ExprExt for Expr {
    fn fatty_acid(self) -> FattyAcidExpr {
        FattyAcidExpr(self)
    }

    fn tag(self) -> TagsExpr {
        TagsExpr(self)
    }
}

/// Extension methods for [`Schema`]
pub trait SchemaExt {
    fn names(&self) -> Vec<Expr>;
}

impl SchemaExt for Schema {
    fn names(&self) -> Vec<Expr> {
        self.iter_names_cloned().map(col).collect()
    }
}

// Fatty acids `DataFrame`
pub struct FattyAcidsDataFrame(DataFrame);

impl FattyAcidsDataFrame {
    pub fn fatty_acid(&self, index: usize) -> Option<FattyAcid> {
        let carbons = self.0.u8("Carbons");
        let doubles = self.0.list("Doubles");
        let triples = self.0.list("Triples");
        let carbons = carbons.get(index)?;
        let doubles = doubles.get_as_series(index)?;
        let triples = triples.get_as_series(index)?;
        Some(FattyAcid {
            carbons,
            doubles: doubles.i8().unwrap().to_vec_null_aware().left()?,
            triples: triples.i8().unwrap().to_vec_null_aware().left()?,
        })
    }

    pub fn label(&self, index: usize) -> Option<&str> {
        self.0.str("Label").get(index)
    }
}

/// Fatty acids `Expr`
pub struct FattyAcidExpr(Expr);

impl FattyAcidExpr {
    /// Carbons count
    pub fn c(&self) -> Expr {
        self.0.clone().r#struct().field_by_name("Carbons")
    }

    /// Hydrogens count
    pub fn h(&self) -> Expr {
        lit(2) * self.c() - lit(2) * self.d() - lit(4) * self.t()
    }

    /// Double bounds count
    pub fn d(&self) -> Expr {
        self.0
            .clone()
            .r#struct()
            .field_by_name("Doubles")
            .list()
            .len()
    }

    /// Triple bounds count
    pub fn t(&self) -> Expr {
        self.0
            .clone()
            .r#struct()
            .field_by_name("Triples")
            .list()
            .len()
    }

    /// Fatty acid ECN (Equivalent carbon number)
    ///
    /// `ECN = CN - 2DB`
    pub fn ecn(self) -> Expr {
        // lit(2) * c(expr) - lit(2) * d(expr) - lit(4) * t(expr)
        // c(&self) - lit(2) * d(&self) - lit(4) * t(&self)
        self.c() - lit(2) * self.d() - lit(4) * self.t()
        // h(&self) - lit(C)
    }

    pub fn mass(self) -> Expr {
        // TODO: c(&self) * lit(C) + h(&self) * lit(H) + lit(2) * lit(O)
        self.c() * lit(C) + self.h() * lit(H) + lit(2. * O)
    }

    pub fn saturated(self) -> Expr {
        self.unsaturation().eq(lit(0))
    }

    pub fn species(self) -> Expr {
        self.0.r#struct().field_by_name("Label")
    }

    pub fn r#type(self) -> Expr {
        ternary_expr(self.saturated(), lit("S"), lit("U"))
    }

    pub fn unsaturated(self) -> Expr {
        self.saturated().not()
    }

    pub fn unsaturation(self) -> Expr {
        self.d() + lit(2) * self.t()
    }
}

/// Tags
pub struct TagsExpr(Expr);

impl TagsExpr {
    // Triacylglycerol species
    pub fn species(self) -> Expr {
        concat_str(
            [self
                .0
                .r#struct()
                .field_by_names([r#"^SN[1-3]$"#])
                .r#struct()
                .field_by_name("FA")
                .fatty_acid()
                .species()],
            "",
            true,
        )
    }

    // Triacylglycerol value
    pub fn value(self) -> Expr {
        // self.0
        //     .r#struct()
        //     .field_by_names([r#"^SN[1-3]$"#])
        //     .r#struct()
        //     .field_by_name("Value")
        //     .product()
        self.0
            .clone()
            .r#struct()
            .field_by_name("SN1")
            .r#struct()
            .field_by_name("Value")
            * self
                .0
                .clone()
                .r#struct()
                .field_by_name("SN2")
                .r#struct()
                .field_by_name("Value")
            * self
                .0
                .r#struct()
                .field_by_name("SN3")
                .r#struct()
                .field_by_name("Value")
    }

    // Sort FA in SN.
    pub fn sort(self, composition: Composition) -> PolarsResult<Expr> {
        let expr = match composition.kind {
            Kind::Ecn => sort_by_ecn(),
            Kind::Mass => sort_by_mass(),
            Kind::Species => sort_by_species(),
            Kind::Type => sort_by_type(),
            Kind::Unsaturation => sort_by_unsaturation(),
        };
        if let Some(Stereospecificity::Stereo) = composition.stereospecificity {
            return Ok(self.0);
        }
        let list = if composition.stereospecificity.is_none() {
            concat_list([
                self.0.clone().r#struct().field_by_name("SN1"),
                self.0.clone().r#struct().field_by_name("SN2"),
                self.0.clone().r#struct().field_by_name("SN3"),
            ])
        } else {
            concat_list([
                self.0.clone().r#struct().field_by_name("SN1"),
                self.0.clone().r#struct().field_by_name("SN3"),
            ])
        }?
        .list()
        .eval(expr, true);
        Ok(if stereospecificity.is_none() {
            as_struct(vec![
                list.clone().list().get(lit(0), false).alias("SN1"),
                list.clone().list().get(lit(1), false).alias("SN2"),
                list.list().get(lit(2), false).alias("SN3"),
            ])
        } else {
            as_struct(vec![
                list.clone().list().get(lit(0), false).alias("SN1"),
                self.0.r#struct().field_by_name("SN2"),
                list.list().get(lit(1), false).alias("SN3"),
            ])
        })
    }

    // pub fn non_stereospecific(self) -> Expr {
    // }
}

fn sort_by_ecn() -> Expr {
    col("").sort_by(
        [col("").r#struct().field_by_name("FA").fatty_acid().ecn()],
        Default::default(),
    )
}

fn sort_by_mass() -> Expr {
    col("").sort_by(
        [col("").r#struct().field_by_name("FA").fatty_acid().mass()],
        Default::default(),
    )
}

fn sort_by_type() -> Expr {
    col("").sort_by(
        [col("")
            .r#struct()
            .field_by_name("FA")
            .fatty_acid()
            .saturated()],
        Default::default(),
    )
}

fn sort_by_species() -> Expr {
    col("").sort_by(
        [
            col("")
                .r#struct()
                .field_by_name("FA")
                .r#struct()
                .field_by_name("Carbons"),
            col("")
                .r#struct()
                .field_by_name("FA")
                .r#struct()
                .field_by_name("Doubles")
                .list()
                .len(),
            col("")
                .r#struct()
                .field_by_name("FA")
                .r#struct()
                .field_by_name("Triples")
                .list()
                .len(),
            col("")
                .r#struct()
                .field_by_name("FA")
                .r#struct()
                .field_by_name("Label"),
            col("").r#struct().field_by_name("Index"),
        ],
        Default::default(),
    )
}

fn sort_by_unsaturation() -> Expr {
    col("").sort_by(
        [col("")
            .r#struct()
            .field_by_name("FA")
            .fatty_acid()
            .unsaturation()],
        Default::default(),
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;

    fn sort_by_type() -> Expr {
        col("").sort_by(
            [col("")
                .r#struct()
                .field_by_name("FA")
                .fatty_acid()
                .saturated()],
            SortMultipleOptions::default().with_order_reversed(),
        )
    }

    fn sort_by_species() -> Expr {
        col("").sort_by(
            [
                col("")
                    .r#struct()
                    .field_by_name("FA")
                    .r#struct()
                    .field_by_name("Carbons"),
                col("")
                    .r#struct()
                    .field_by_name("FA")
                    .r#struct()
                    .field_by_name("Doubles")
                    .list()
                    .len(),
                col("")
                    .r#struct()
                    .field_by_name("FA")
                    .r#struct()
                    .field_by_name("Triples")
                    .list()
                    .len(),
                col("")
                    .r#struct()
                    .field_by_name("FA")
                    .r#struct()
                    .field_by_name("Label"),
            ],
            SortMultipleOptions::default(),
        )
    }

    fn psm() -> Result<DataFrame> {
        Ok(df! {
            "TAG" => df! {
                "SN1" => df! {
                    "FA" => df! {
                        "Label" => &["P"],
                        "Carbons" => &[16u8],
                        "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                        "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                    }?
                    .into_struct(PlSmallStr::EMPTY),
                }?
                .into_struct(PlSmallStr::EMPTY),
                "SN2" => df! {
                    "FA" => df! {
                        "Label" => &["S"],
                        "Carbons" => &[18u8],
                        "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                        "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                    }?
                    .into_struct(PlSmallStr::EMPTY),
                }?
                .into_struct(PlSmallStr::EMPTY),
                "SN3" => df! {
                    "FA" => df! {
                        "Label" => &["M"],
                        "Carbons" => &[14u8],
                        "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                        "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                    }?
                    .into_struct(PlSmallStr::EMPTY),
                }?
                .into_struct(PlSmallStr::EMPTY),
            }?.into_struct(PlSmallStr::EMPTY),
        }?)
    }

    fn msp() -> Result<DataFrame> {
        Ok(df! {
           "TAG" => df! {
                "SN1" => df! {
                    "FA" => df! {
                        "Label" => &["M"],
                        "Carbons" => &[14u8],
                        "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                        "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                    }?
                    .into_struct(PlSmallStr::EMPTY),
                }?
                .into_struct(PlSmallStr::EMPTY),
                "SN2" => df! {
                    "FA" => df! {
                        "Label" => &["S"],
                        "Carbons" => &[18u8],
                        "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                        "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                    }?
                    .into_struct(PlSmallStr::EMPTY),
                }?
                .into_struct(PlSmallStr::EMPTY),
                "SN3" => df! {
                    "FA" => df! {
                        "Label" => &["P"],
                        "Carbons" => &[16u8],
                        "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                        "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                    }?
                    .into_struct(PlSmallStr::EMPTY),
                }?
                .into_struct(PlSmallStr::EMPTY),
            }?.into_struct(PlSmallStr::EMPTY),
        }?)
    }

    fn mps() -> Result<DataFrame> {
        Ok(df! {
           "TAG" => df! {
                "SN1" => df! {
                    "FA" => df! {
                        "Label" => &["M"],
                        "Carbons" => &[14u8],
                        "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                        "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                    }?
                    .into_struct(PlSmallStr::EMPTY),
                }?
                .into_struct(PlSmallStr::EMPTY),
                "SN2" => df! {
                    "FA" => df! {
                        "Label" => &["P"],
                        "Carbons" => &[16u8],
                        "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                        "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                    }?
                    .into_struct(PlSmallStr::EMPTY),
                }?
                .into_struct(PlSmallStr::EMPTY),
                "SN3" => df! {
                    "FA" => df! {
                        "Label" => &["S"],
                        "Carbons" => &[18u8],
                        "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                        "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                    }?
                    .into_struct(PlSmallStr::EMPTY),
                }?
                .into_struct(PlSmallStr::EMPTY),
            }?.into_struct(PlSmallStr::EMPTY),
        }?)
    }

    #[test]
    fn species() -> Result<()> {
        let source = psm()?;
        let target = source
            .lazy()
            .select([col("TAG").tag().sort(sort_by_species(), None)?.alias("TAG")])
            .collect()?;
        assert_eq!(target, mps()?);
        // positional
        let source = psm()?;
        let target = source
            .lazy()
            .select([col("TAG")
                .tag()
                .sort(sort_by_species(), Some(Stereospecificity::Positional))?
                .alias("TAG")])
            .collect()?;
        assert_eq!(target, msp()?);
        Ok(())
    }

    #[test]
    fn r#type() -> Result<()> {
        std::env::set_var("POLARS_FMT_STR_LEN", "256");

        let a = df! {
            "Label" => &["A"],
            "Carbons" => &[14u8],
            "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
            "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
        }?
        .into_struct(PlSmallStr::EMPTY);
        let b = df! {
            "Label" => &["B"],
            "Carbons" => &[15u8],
            "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
            "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
        }?
        .into_struct(PlSmallStr::EMPTY);
        let c = df! {
            "Label" => &["C"],
            "Carbons" => &[16u8],
            "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
            "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
        }?
        .into_struct(PlSmallStr::EMPTY);
        let sn1 = df! {
           "FA" => c,
        }?
        .into_struct(PlSmallStr::EMPTY);
        let sn2 = df! {
           "FA" => b,
        }?
        .into_struct(PlSmallStr::EMPTY);
        let sn3 = df! {
           "FA" => a,
        }?
        .into_struct(PlSmallStr::EMPTY);
        let tag = df! {
            "SN1" => sn1,
            "SN2" => sn2,
            "SN3" => sn3,
        }?
        .into_struct(PlSmallStr::EMPTY);
        let data_frame = df! {
            "TAG" => tag,
        }?;
        println!("data_frame: {data_frame}");
        let lazy_frame = data_frame
            .lazy()
            // .with_column(col("TAG").tag().positional(sort_by_species())?)
            .with_column(col("TAG").tag().sort(sort_by_type(), None)?)
            .collect()
            .unwrap();
        println!("lazy_frame: {lazy_frame}");
        Ok(())
    }
}
