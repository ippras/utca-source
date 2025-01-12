use crate::{
    r#const::relative_atomic_mass::{C, H, O},
    special::{
        composition::{
            Composition, Kind, MC, NC, PMC, PNC, PSC, PTC, PUC, SC, SMC, SNC, SSC, STC, SUC, TC, UC,
        },
        fatty_acid::FattyAcid,
    },
    utils::polars::{ColumnExt, DataFrameExt as _, ExprExt as _, SeriesExt},
};
use polars::{
    lazy::dsl::{max_horizontal, min_horizontal},
    prelude::*,
};
use std::{borrow::Borrow, sync::LazyLock};

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
    fn fatty_acids(self) -> FattyAcidDataFrame;
}

impl DataFrameExt for DataFrame {
    fn fatty_acids(self) -> FattyAcidDataFrame {
        FattyAcidDataFrame(self)
    }
}

/// Extension methods for [`Expr`]
pub(crate) trait ExprExt {
    fn fa(self) -> FattyAcidExpr;

    fn sn(self) -> StereospecificNumberExpr;

    fn tag(self) -> TriacylglycerolExpr;
}

impl ExprExt for Expr {
    fn fa(self) -> FattyAcidExpr {
        FattyAcidExpr(self)
    }

    fn sn(self) -> StereospecificNumberExpr {
        StereospecificNumberExpr(self)
    }

    fn tag(self) -> TriacylglycerolExpr {
        TriacylglycerolExpr(self)
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

// Fatty acid `DataFrame`
pub struct FattyAcidDataFrame(DataFrame);

impl FattyAcidDataFrame {
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

/// Fatty acids [`Expr`]
#[derive(Clone)]
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

    /// Species
    pub fn label(self) -> Expr {
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

impl From<FattyAcidExpr> for Expr {
    fn from(value: FattyAcidExpr) -> Self {
        value.0
    }
}

/// Triacylglycerol [`Expr`]
pub struct TriacylglycerolExpr(Expr);

impl TriacylglycerolExpr {
    pub fn sn(self) -> StereospecificNumberExpr {
        StereospecificNumberExpr(self.0.clone().r#struct().field_by_name("*"))
    }

    pub fn sn1(&self) -> StereospecificNumberExpr {
        StereospecificNumberExpr(self.0.clone().r#struct().field_by_name("SN1"))
    }

    pub fn sn2(&self) -> StereospecificNumberExpr {
        StereospecificNumberExpr(self.0.clone().r#struct().field_by_name("SN2"))
    }

    pub fn sn3(&self) -> StereospecificNumberExpr {
        StereospecificNumberExpr(self.0.clone().r#struct().field_by_name("SN3"))
    }

    pub fn ecn(self) -> Expr {
        self.sn1().fa().ecn() + self.sn2().fa().ecn() + self.sn3().fa().ecn()
    }

    /// C3H2 + SN1 + SN2 + SN3 + ADDUCT
    pub fn mass(self, adduct: f64) -> Expr {
        lit(3) * lit(C)
            + lit(2) * lit(H)
            + self.sn1().fa().mass()
            + self.sn2().fa().mass()
            + self.sn3().fa().mass()
            + lit(adduct)
    }

    // Triacylglycerol species
    pub fn label(self) -> Expr {
        concat_str([self.sn().fa().label()], "", true)
    }

    // Compose
    pub fn compose(self, composition: Composition) -> PolarsResult<Expr> {
        Ok(match composition {
            MC => concat_list([
                self.sn1().fa().mass(),
                self.sn2().fa().mass(),
                self.sn3().fa().mass(),
            ])?
            .list()
            .sort(Default::default())
            .alias("MC"),
            PMC => concat_list([
                min_horizontal([self.sn1().fa().mass(), self.sn3().fa().mass()])?,
                self.sn2().fa().mass(),
                max_horizontal([self.sn1().fa().mass(), self.sn3().fa().mass()])?,
            ])?
            .alias("PMC"),
            SMC => concat_list([
                self.sn1().fa().mass(),
                self.sn2().fa().mass(),
                self.sn3().fa().mass(),
            ])?
            .alias("SMC"),
            NC => concat_list([
                self.sn1().fa().ecn(),
                self.sn2().fa().ecn(),
                self.sn3().fa().ecn(),
            ])?
            .list()
            .sort(Default::default())
            .alias("NC"),
            PNC => concat_list([
                min_horizontal([self.sn1().fa().ecn(), self.sn3().fa().ecn()])?,
                self.sn2().fa().ecn(),
                max_horizontal([self.sn1().fa().ecn(), self.sn3().fa().ecn()])?,
            ])?
            .alias("PNC"),
            SNC => concat_list([
                self.sn1().fa().ecn(),
                self.sn2().fa().ecn(),
                self.sn3().fa().ecn(),
            ])?
            .alias("SNC"),
            SC => concat_list([self.sn1().fa(), self.sn2().fa(), self.sn3().fa()])?
                .list()
                .eval(sort_by_species(), true)
                .map_list(
                    |column| {
                        Ok(Some(
                            column
                                .list()?
                                .apply_to_inner(&|series| series.r#struct().field_by_name("Label"))?
                                .into_series()
                                .into_column(),
                        ))
                    },
                    GetOutput::from_type(DataType::List(Box::new(DataType::String))),
                )
                .alias("SC"),
            PSC => {
                let sn13 = concat_list([self.sn1().fa(), self.sn3().fa()])?
                    .list()
                    .eval(sort_by_species(), true);
                concat_list([
                    sn13.clone().list().get(lit(0), false).fa().label(),
                    self.sn2().fa().label(),
                    sn13.list().get(lit(1), false).fa().label(),
                ])?
                .alias("PSC")
            }
            SSC => concat_list([
                self.sn1().fa().label(),
                self.sn2().fa().label(),
                self.sn3().fa().label(),
            ])?
            .alias("SSC"),
            TC => concat_list([
                self.sn1().fa().r#type(),
                self.sn2().fa().r#type(),
                self.sn3().fa().r#type(),
            ])?
            .list()
            .sort(SortOptions::default())
            .alias("TC"),
            PTC => concat_list([
                min_horizontal([self.sn1().fa().r#type(), self.sn3().fa().r#type()])?,
                self.sn2().fa().r#type(),
                max_horizontal([self.sn1().fa().r#type(), self.sn3().fa().r#type()])?,
            ])?
            .alias("PTC"),
            STC => concat_list([
                self.sn1().fa().r#type(),
                self.sn2().fa().r#type(),
                self.sn3().fa().r#type(),
            ])?
            .alias("STC"),
            UC => concat_list([
                self.sn1().fa().unsaturation(),
                self.sn2().fa().unsaturation(),
                self.sn3().fa().unsaturation(),
            ])?
            .list()
            .sort(Default::default())
            .alias("UC"),
            PUC => concat_list([
                min_horizontal([
                    self.sn1().fa().unsaturation(),
                    self.sn3().fa().unsaturation(),
                ])?,
                self.sn2().fa().unsaturation(),
                max_horizontal([
                    self.sn3().fa().unsaturation(),
                    self.sn1().fa().unsaturation(),
                ])?,
            ])?
            .alias("PUC"),
            SUC => concat_list([
                self.sn1().fa().unsaturation(),
                self.sn2().fa().unsaturation(),
                self.sn3().fa().unsaturation(),
            ])?
            .alias("SUC"),
        })
    }

    // /// Permutate (permutation)
    // pub fn permutate(self, composition: Composition) -> PolarsResult<Expr> {
    //     // let substitution = self.substitute(composition.kind);
    //     // Ok(match composition.stereospecificity {
    //     //     None => {}
    //     //     Some(Stereospecificity::Positional) => {}
    //     //     Some(Stereospecificity::Stereo) => self.0,
    //     // })
    //     Ok(match composition {
    //         MC => concat_list([self.sn1(), self.sn2(), self.sn3()])?
    //             .list()
    //             .eval(
    //                 col("").sort_by([col("").sn().fa().mass()], Default::default()),
    //                 true,
    //             )
    //             .list()
    //             .to_struct(ListToStructArgs::FixedWidth(
    //                 ["SN1".into(), "SN2".into(), "SN3".into()].into(),
    //             )),
    //         NC => concat_list([
    //             min_horizontal([self.sn1().fa().ecn(), self.sn3().fa().ecn()])?,
    //             self.sn2().fa().ecn(),
    //             max_horizontal([self.sn1().fa().ecn(), self.sn3().fa().ecn()])?,
    //         ])?
    //         .list()
    //         .to_struct(ListToStructArgs::FixedWidth(
    //             ["SN1".into(), "SN2".into(), "SN3".into()].into(),
    //         )),
    //         SC => concat_list([self.sn1().fa(), self.sn2().fa(), self.sn3().fa()])?
    //             .list()
    //             .eval(sort_by_species(), true)
    //             .list()
    //             .to_struct(ListToStructArgs::FixedWidth(
    //                 ["SN1".into(), "SN2".into(), "SN3".into()].into(),
    //             )),
    //         PMC => concat_list([
    //             min_horizontal([self.sn1().fa().mass(), self.sn3().fa().mass()])?,
    //             self.sn2().fa().mass(),
    //             max_horizontal([self.sn1().fa().mass(), self.sn3().fa().mass()])?,
    //         ])?,
    //         PNC => concat_list([
    //             min_horizontal([self.sn1().fa().ecn(), self.sn3().fa().ecn()])?,
    //             self.sn2().fa().ecn(),
    //             max_horizontal([self.sn1().fa().ecn(), self.sn3().fa().ecn()])?,
    //         ])?,
    //         PSC => {
    //             let sn13 = concat_list([self.sn1().fa(), self.sn3().fa()])?
    //                 .list()
    //                 .eval(sort_by_species(), true);
    //             concat_list([
    //                 sn13.clone().list().get(lit(0), false).fa().species(),
    //                 self.sn2().fa().species(),
    //                 sn13.list().get(lit(1), false).fa().species(),
    //             ])?
    //         }
    //         PTC => concat_list([
    //             max_horizontal([self.sn1().fa().saturated(), self.sn3().fa().saturated()])?,
    //             self.sn2().fa().saturated(),
    //             min_horizontal([self.sn1().fa().saturated(), self.sn3().fa().saturated()])?,
    //         ])?,
    //         PUC => concat_list([
    //             min_horizontal([
    //                 self.sn1().fa().unsaturation(),
    //                 self.sn3().fa().unsaturation(),
    //             ])?,
    //             self.sn2().fa().unsaturation(),
    //             max_horizontal([
    //                 self.sn1().fa().unsaturation(),
    //                 self.sn3().fa().unsaturation(),
    //             ])?,
    //         ])?,
    //         SMC | SNC | SSC | STC | SUC => self.0,
    //     })
    // }

    // // Substitute (substitution)
    // pub fn substitute(self, kind: Kind) -> Expr {
    //     match kind {
    //         Kind::Ecn => as_struct(vec![
    //             self.sn1().fa().ecn(),
    //             self.sn2().fa().ecn(),
    //             self.sn3().fa().ecn(),
    //         ]),
    //         Kind::Mass => as_struct(vec![
    //             self.sn1().fa().mass(),
    //             self.sn2().fa().mass(),
    //             self.sn3().fa().mass(),
    //         ]),
    //         Kind::Species => as_struct(vec![
    //             as_struct(vec![
    //                 self.sn1().fa().c(),
    //                 self.sn1().fa().d(),
    //                 self.sn1().fa().t(),
    //             ]),
    //             as_struct(vec![
    //                 self.sn2().fa().c(),
    //                 self.sn2().fa().d(),
    //                 self.sn2().fa().t(),
    //             ]),
    //             as_struct(vec![
    //                 self.sn3().fa().c(),
    //                 self.sn3().fa().d(),
    //                 self.sn3().fa().t(),
    //             ]),
    //         ]),
    //         Kind::Type => as_struct(vec![
    //             self.sn1().fa().saturated(),
    //             self.sn2().fa().saturated(),
    //             self.sn3().fa().saturated(),
    //         ]),
    //         Kind::Unsaturation => as_struct(vec![
    //             self.sn1().fa().unsaturation(),
    //             self.sn1().fa().unsaturation(),
    //             self.sn1().fa().unsaturation(),
    //         ]),
    //     }
    // }

    // pub fn composition(expr: Expr, composition: Composition) -> PolarsResult<Expr> {
    //     expr.map(
    //         |column| {
    //             let tag = TriacylglycerolColumn(column);
    //             let sn1 = tag.sn1()?;
    //             let sn3 = tag.sn3()?;
    //             let sn1c = sn1.fa()?.c("FA")?;
    //             // MC => {}
    //             // PMC => {}
    //             // SMC => {}
    //             // NC => {}
    //             // PNC => {}
    //             // SNC => {}
    //             // SC => {}
    //             // PSC => {}
    //             // SSC => {}
    //             // TC => {}
    //             // PTC => {}
    //             // STC => {}
    //             // UC => {}
    //             // PUC => {}
    //             // SUC => {}
    //             match composition.kind {
    //                 Kind::Ecn => {
    //                     sn1.field_by_name("FA");
    //                 }
    //                 Kind::Mass => {}
    //                 Kind::Species => {}
    //                 Kind::Type => {}
    //                 Kind::Unsaturation => {}
    //             };
    //             let out: StructChunked = sn1
    //                 .into_iter()
    //                 .zip(sn3)
    //                 .map(|(sn1, sn3)| match (sn1, sn3) {
    //                     (Some(sn1), Some(sn3)) => Some(a.len() as i32 + b),
    //                     _ => None,
    //                 })
    //                 .collect();
    //             Ok(Some(
    //                 column
    //                     .list()?
    //                     .apply_to_inner(&|series| series.r#struct().field_by_name("Label"))?
    //                     .into_series()
    //                     .into_column(),
    //             ))
    //         },
    //         GetOutput::from_type(DataType::List(Box::new(DataType::String))),
    //     );
    //     Ok(lit(0))
    // }

    // pub fn composition(self, composition: Composition) -> PolarsResult<Expr> {
    //     let expr = match composition.kind {
    //         Kind::Ecn => sort_by_ecn(),
    //         Kind::Mass => sort_by_mass(),
    //         Kind::Species => sort_by_species(),
    //         Kind::Type => sort_by_type(),
    //         Kind::Unsaturation => sort_by_unsaturation(),
    //     };
    //     if let Some(Stereospecificity::Stereo) = composition.stereospecificity {
    //         return Ok(self.0);
    //     }
    //     let list = if composition.stereospecificity.is_none() {
    //         concat_list([
    //             self.0.clone().r#struct().field_by_name("SN1"),
    //             self.0.clone().r#struct().field_by_name("SN2"),
    //             self.0.clone().r#struct().field_by_name("SN3"),
    //         ])
    //     } else {
    //         concat_list([
    //             self.0.clone().r#struct().field_by_name("SN1"),
    //             self.0.clone().r#struct().field_by_name("SN3"),
    //         ])
    //     }?
    //     .list()
    //     .eval(expr, true);
    //     Ok(if composition.stereospecificity.is_none() {
    //         as_struct(vec![
    //             list.clone().list().get(lit(0), false).alias("SN1"),
    //             list.clone().list().get(lit(1), false).alias("SN2"),
    //             list.list().get(lit(2), false).alias("SN3"),
    //         ])
    //     } else {
    //         as_struct(vec![
    //             list.clone().list().get(lit(0), false).alias("SN1"),
    //             self.0.r#struct().field_by_name("SN2"),
    //             list.list().get(lit(1), false).alias("SN3"),
    //         ])
    //     })
    // }

    // Triacylglycerol value
    pub fn value(self) -> Expr {
        self.sn1().value() * self.sn2().value() * self.sn3().value()
    }
}

// Stereospecific number [`Expr`]
#[derive(Clone)]
pub struct StereospecificNumberExpr(Expr);

impl StereospecificNumberExpr {
    pub fn index(self) -> Expr {
        self.0.r#struct().field_by_name("Index")
    }

    // Fatty acid
    pub fn fa(self) -> FattyAcidExpr {
        self.0.r#struct().field_by_name("FA").fa()
    }

    pub fn substitute(self, kind: Kind) -> Expr {
        match kind {
            Kind::Ecn => self.fa().ecn(),
            Kind::Mass => self.fa().mass(),
            Kind::Species => as_struct(vec![
                self.clone().fa().c(),
                self.clone().fa().d(),
                self.fa().t(),
            ]),
            Kind::Type => self.fa().saturated(),
            Kind::Unsaturation => self.fa().unsaturation(),
        }
    }

    pub fn value(self) -> Expr {
        self.0.r#struct().field_by_name("Value")
    }
}

impl From<StereospecificNumberExpr> for Expr {
    fn from(value: StereospecificNumberExpr) -> Self {
        value.0
    }
}

fn sort_by_species() -> Expr {
    col("").sort_by(
        [col("").fa().c(), col("").fa().d(), col("").fa().t()],
        Default::default(),
    )
}

// Triacylglycerol [`Column`]
#[derive(Clone)]
pub struct TriacylglycerolColumn(Column);

impl TriacylglycerolColumn {
    pub fn sn1(&self) -> PolarsResult<StereospecificNumberSeries> {
        Ok(StereospecificNumberSeries(
            self.0.struct_()?.field_by_name("SN1")?,
        ))
    }

    pub fn sn2(&self) -> PolarsResult<StereospecificNumberSeries> {
        Ok(StereospecificNumberSeries(
            self.0.struct_()?.field_by_name("SN2")?,
        ))
    }

    pub fn sn3(&self) -> PolarsResult<StereospecificNumberSeries> {
        Ok(StereospecificNumberSeries(
            self.0.struct_()?.field_by_name("SN3")?,
        ))
    }
    // pub fn sn(&self, sn: Sn) -> PolarsResult<StereospecificNumberSeries> {
    //     let r#struct = self.0.struct_()?;
    //     Ok(StereospecificNumberSeries(match sn {
    //         Sn::One => r#struct.field_by_name("SN1")?,
    //         Sn::Two => r#struct.field_by_name("SN2")?,
    //         Sn::Three => r#struct.field_by_name("SN3")?,
    //     }))
    // }
}

// Stereospecific number [`ChunkedArray`]
#[derive(Clone)]
pub struct StereospecificNumberSeries(Series);

impl StereospecificNumberSeries {
    pub fn fa(&self) -> PolarsResult<FattyAcidSeries> {
        Ok(FattyAcidSeries(self.0.struct_()?.field_by_name("FA")?))
    }
}

// Fatty acid [`ChunkedArray`]
#[derive(Clone)]
pub struct FattyAcidSeries(Series);

impl StereospecificNumberSeries {
    // /// Carbons
    // pub fn c(&self) -> PolarsResult<Series> {
    //     Ok(self.0.struct_()?.field_by_name("Carbons")?.u8()?)
    // }

    // /// Carbons count
    // pub fn c(&self) -> Expr {
    //     self.0.clone().r#struct().field_by_name("Carbons")
    // }

    // /// Hydrogens count
    // pub fn h(&self) -> Expr {
    //     lit(2) * self.c() - lit(2) * self.d() - lit(4) * self.t()
    // }

    // /// Double bounds count
    // pub fn d(&self) -> Expr {
    //     self.0
    //         .clone()
    //         .r#struct()
    //         .field_by_name("Doubles")
    //         .list()
    //         .len()
    // }

    // /// Triple bounds count
    // pub fn t(&self) -> Expr {
    //     self.0
    //         .clone()
    //         .r#struct()
    //         .field_by_name("Triples")
    //         .list()
    //         .len()
    // }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::special::fatty_acid::fatty_acid;
    use anyhow::Result;

    #[test]
    fn temp() -> Result<()> {
        // std::env::set_var("POLARS_FMT_STR_LEN", "256");
        // // assert_eq!(
        // //     composition(s14s16s18()?, SC)?,
        // //     df! { "TAG" => &[Series::from_iter(["M", "P", "S"])] }?,
        // // );

        // // let tag = c18c16c14()?.lazy().select([concat_list([
        // //     col("TAG").tag().sn1().fa(),
        // //     col("TAG").tag().sn2().fa(),
        // //     col("TAG").tag().sn3().fa(),
        // // ])?
        // // .list()
        // // .eval(sort_by_species(), true)
        // // .map(
        // //     |column| {
        // //         // let column = column.r#struct().field_by_name("Label")?.into_column();
        // //         // let list = column.list()?;
        // //         let series = column
        // //             .list()?
        // //             .apply_to_inner(&|series| series.r#struct().field_by_name("Label"))?
        // //             .into_series();
        // //         // let s_cos: Series = column
        // //         //     .list()?
        // //         //     .into_iter()
        // //         //     .map(|option| {
        // //         //         Series::from_any_values(
        // //         //             PlSmallStr::EMPTY,
        // //         //             &[AnyValue::List(Series::from_iter(binary.r#as()))],
        // //         //             false,
        // //         //         )
        // //         //         // option.map(|angle|  angle.)
        // //         //     })
        // //         //     .collect();
        // //         // let t = Series::from_iter(column
        // //         //     .list()?
        // //         //     .into_iter())
        // //         //     // .map(|series| {
        // //         //     //     series
        // //         //     //         .map(|series| series.r#struct().field_by_name("Label"))
        // //         //     //         .transpose()
        // //         //     // })
        // //         //     ;
        // //         println!("get: {series:?}");
        // //         //     .collect::<PolarsResult<Series>>()?;
        // //         // let get = column.get(0).unwrap();
        // //         // println!("get: {get}");
        // //         // Ok(Some(
        // //         //     Series::new("name".into(), ["column".to_string()]).into_column(),
        // //         // ))
        // //         Ok(Some(series.into_column()))
        // //     },
        // //     GetOutput::from_type(DataType::List(Box::new(DataType::String))),
        // // )]);

        // // let tag = c14b16a18()?.lazy().select([concat_list([
        // //     col("TAG").tag().sn1(),
        // //     col("TAG").tag().sn2(),
        // //     col("TAG").tag().sn3(),
        // // ])?
        // // .list()
        // // .eval(
        // //     col("").sort_by(
        // //         [
        // //             col("").sn().fa().c(),
        // //             col("").sn().fa().d(),
        // //             col("").sn().fa().t(),
        // //             col("").sn().fa().label(),
        // //         ],
        // //         Default::default(),
        // //     ),
        // //     true,
        // // )]);
        // let tag = a18b16c14()?
        //     .lazy()
        //     .select([concat_list([lit(0), lit(1), lit(2)])?
        //         // .sort_by(by, sort_options)
        //         .list()
        //         .eval(
        //             col("").sort_by([col("TAG").tag().sn().fa().mass()], Default::default()),
        //             true,
        //         )
        //         .list()
        //         .to_struct(ListToStructArgs::FixedWidth(Arc::new([
        //             "SN1".into(),
        //             "SN2".into(),
        //             "SN3".into(),
        //         ])))
        //         .alias("Tag")]);

        // // .map(
        // //     |column| {
        // //         // let column = column.r#struct().field_by_name("Label")?.into_column();
        // //         // let list = column.list()?;
        // //         let series = column
        // //             .list()?
        // //             .apply_to_inner(&|series| series.r#struct().field_by_name("Label"))?
        // //             .into_series();
        // //         // let s_cos: Series = column
        // //         //     .list()?
        // //         //     .into_iter()
        // //         //     .map(|option| {
        // //         //         Series::from_any_values(
        // //         //             PlSmallStr::EMPTY,
        // //         //             &[AnyValue::List(Series::from_iter(binary.r#as()))],
        // //         //             false,
        // //         //         )
        // //         //         // option.map(|angle|  angle.)
        // //         //     })
        // //         //     .collect();
        // //         // let t = Series::from_iter(column
        // //         //     .list()?
        // //         //     .into_iter())
        // //         //     // .map(|series| {
        // //         //     //     series
        // //         //     //         .map(|series| series.r#struct().field_by_name("Label"))
        // //         //     //         .transpose()
        // //         //     // })
        // //         //     ;
        // //         println!("get: {series:?}");
        // //         //     .collect::<PolarsResult<Series>>()?;
        // //         // let get = column.get(0).unwrap();
        // //         // println!("get: {get}");
        // //         // Ok(Some(
        // //         //     Series::new("name".into(), ["column".to_string()]).into_column(),
        // //         // ))
        // //         Ok(Some(series.into_column()))
        // //     },
        // //     GetOutput::from_type(DataType::List(Box::new(DataType::String))),
        // // )
        // println!("tag: {}", tag.collect().unwrap());
        Ok(())
    }

    fn composition(tag: DataFrame, composition: Composition) -> PolarsResult<DataFrame> {
        tag.lazy()
            .select([col("TAG").tag().compose(composition)?])
            .collect()
    }

    fn c14c16c18() -> PolarsResult<DataFrame> {
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

    fn c14c18c16() -> PolarsResult<DataFrame> {
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

    fn c16c14c18() -> PolarsResult<DataFrame> {
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
                        "Label" => &["M"],
                        "Carbons" => &[14u8],
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

    fn c16c18c14() -> PolarsResult<DataFrame> {
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

    fn c18c14c16() -> PolarsResult<DataFrame> {
        Ok(df! {
           "TAG" => df! {
                "SN1" => df! {
                    "FA" => df! {
                        "Label" => &["S"],
                        "Carbons" => &[18u8],
                        "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                        "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                    }?
                    .into_struct(PlSmallStr::EMPTY),
                }?
                .into_struct(PlSmallStr::EMPTY),
                "SN2" => df! {
                    "FA" => df! {
                        "Label" => &["M"],
                        "Carbons" => &[14u8],
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

    fn c18c16c14() -> PolarsResult<DataFrame> {
        Ok(df! {
           "TAG" => df! {
                "SN1" => df! {
                    "FA" => df! {
                        "Label" => &["S"],
                        "Carbons" => &[18u8],
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

    fn su1u2() -> PolarsResult<DataFrame> {
        Ok(df! {
            "TAG" => df! {
                "SN1" => df! {
                    "FA" => df! {
                        "Label" => &["S"],
                        "Carbons" => &[18u8],
                        "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                        "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                    }?
                    .into_struct(PlSmallStr::EMPTY),
                }?
                .into_struct(PlSmallStr::EMPTY),
                "SN2" => df! {
                    "FA" => df! {
                        "Label" => &["O"],
                        "Carbons" => &[18u8],
                        "Doubles" => &[Series::from_iter([9])],
                        "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                    }?
                    .into_struct(PlSmallStr::EMPTY),
                }?
                .into_struct(PlSmallStr::EMPTY),
                "SN3" => df! {
                    "FA" => df! {
                        "Label" => &["L"],
                        "Carbons" => &[18u8],
                        "Doubles" => &[Series::from_iter([9, 12])],
                        "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                    }?
                    .into_struct(PlSmallStr::EMPTY),
                }?
                .into_struct(PlSmallStr::EMPTY),
            }?.into_struct(PlSmallStr::EMPTY),
        }?)
    }

    fn su2u1() -> PolarsResult<DataFrame> {
        Ok(df! {
            "TAG" => df! {
                "SN1" => df! {
                    "FA" => df! {
                        "Label" => &["S"],
                        "Carbons" => &[18u8],
                        "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                        "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                    }?
                    .into_struct(PlSmallStr::EMPTY),
                }?
                .into_struct(PlSmallStr::EMPTY),
                "SN2" => df! {
                    "FA" => df! {
                        "Label" => &["L"],
                        "Carbons" => &[18u8],
                        "Doubles" => &[Series::from_iter([9, 12])],
                        "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                    }?
                    .into_struct(PlSmallStr::EMPTY),
                }?
                .into_struct(PlSmallStr::EMPTY),
                "SN3" => df! {
                    "FA" => df! {
                        "Label" => &["O"],
                        "Carbons" => &[18u8],
                        "Doubles" => &[Series::from_iter([9])],
                        "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                    }?
                    .into_struct(PlSmallStr::EMPTY),
                }?
                .into_struct(PlSmallStr::EMPTY),
            }?.into_struct(PlSmallStr::EMPTY),
        }?)
    }

    fn u1su2() -> PolarsResult<DataFrame> {
        Ok(df! {
            "TAG" => df! {
                "SN1" => df! {
                    "FA" => df! {
                        "Label" => &["O"],
                        "Carbons" => &[18u8],
                        "Doubles" => &[Series::from_iter([9])],
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
                        "Label" => &["L"],
                        "Carbons" => &[18u8],
                        "Doubles" => &[Series::from_iter([9, 12])],
                        "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                    }?
                    .into_struct(PlSmallStr::EMPTY),
                }?
                .into_struct(PlSmallStr::EMPTY),
            }?.into_struct(PlSmallStr::EMPTY),
        }?)
    }

    fn u2su1() -> PolarsResult<DataFrame> {
        Ok(df! {
            "TAG" => df! {
                "SN1" => df! {
                    "FA" => df! {
                        "Label" => &["L"],
                        "Carbons" => &[18u8],
                        "Doubles" => &[Series::from_iter([9, 12])],
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
                        "Label" => &["O"],
                        "Carbons" => &[18u8],
                        "Doubles" => &[Series::from_iter([9])],
                        "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                    }?
                    .into_struct(PlSmallStr::EMPTY),
                }?
                .into_struct(PlSmallStr::EMPTY),
            }?.into_struct(PlSmallStr::EMPTY),
        }?)
    }

    fn u1u2s() -> PolarsResult<DataFrame> {
        Ok(df! {
            "TAG" => df! {
                "SN1" => df! {
                    "FA" => df! {
                        "Label" => &["O"],
                        "Carbons" => &[18u8],
                        "Doubles" => &[Series::from_iter([9])],
                        "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                    }?
                    .into_struct(PlSmallStr::EMPTY),
                }?
                .into_struct(PlSmallStr::EMPTY),
                "SN2" => df! {
                    "FA" => df! {
                        "Label" => &["L"],
                        "Carbons" => &[18u8],
                        "Doubles" => &[Series::from_iter([9, 12])],
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

    fn u2u1s() -> PolarsResult<DataFrame> {
        Ok(df! {
            "TAG" => df! {
                "SN1" => df! {
                    "FA" => df! {
                        "Label" => &["L"],
                        "Carbons" => &[18u8],
                        "Doubles" => &[Series::from_iter([9, 12])],
                        "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
                    }?
                    .into_struct(PlSmallStr::EMPTY),
                }?
                .into_struct(PlSmallStr::EMPTY),
                "SN2" => df! {
                    "FA" => df! {
                        "Label" => &["O"],
                        "Carbons" => &[18u8],
                        "Doubles" => &[Series::from_iter([9])],
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
    fn mc() -> PolarsResult<()> {
        assert_eq!(
            composition(c14c16c18()?, MC)?,
            df! { "PMC" => &[Series::from_iter([fatty_acid!(14).mass(), fatty_acid!(16).mass(), fatty_acid!(18).mass()])] }?,
        );
        assert_eq!(
            composition(c14c18c16()?, MC)?,
            df! { "PMC" => &[Series::from_iter([fatty_acid!(14).mass(), fatty_acid!(16).mass(), fatty_acid!(18).mass()])] }?,
        );
        assert_eq!(
            composition(c16c14c18()?, MC)?,
            df! { "PMC" => &[Series::from_iter([fatty_acid!(14).mass(), fatty_acid!(16).mass(), fatty_acid!(18).mass()])] }?,
        );
        assert_eq!(
            composition(c16c18c14()?, MC)?,
            df! { "PMC" => &[Series::from_iter([fatty_acid!(14).mass(), fatty_acid!(16).mass(), fatty_acid!(18).mass()])] }?,
        );
        assert_eq!(
            composition(c18c14c16()?, MC)?,
            df! { "PMC" => &[Series::from_iter([fatty_acid!(14).mass(), fatty_acid!(16).mass(), fatty_acid!(18).mass()])] }?,
        );
        assert_eq!(
            composition(c18c16c14()?, MC)?,
            df! { "PMC" => &[Series::from_iter([fatty_acid!(14).mass(), fatty_acid!(16).mass(), fatty_acid!(18).mass()])] }?,
        );
        Ok(())
    }

    #[test]
    fn pmc() -> PolarsResult<()> {
        assert_eq!(
            composition(c14c16c18()?, PMC)?,
            df! { "PMC" => &[Series::from_iter([fatty_acid!(14).mass(), fatty_acid!(16).mass(), fatty_acid!(18).mass()])] }?,
        );
        assert_eq!(
            composition(c14c18c16()?, PMC)?,
            df! { "PMC" => &[Series::from_iter([fatty_acid!(14).mass(), fatty_acid!(18).mass(), fatty_acid!(16).mass()])] }?,
        );
        assert_eq!(
            composition(c16c14c18()?, PMC)?,
            df! { "PMC" => &[Series::from_iter([fatty_acid!(16).mass(), fatty_acid!(14).mass(), fatty_acid!(18).mass()])] }?,
        );
        assert_eq!(
            composition(c16c18c14()?, PMC)?,
            df! { "PMC" => &[Series::from_iter([fatty_acid!(14).mass(), fatty_acid!(18).mass(), fatty_acid!(16).mass()])] }?,
        );
        assert_eq!(
            composition(c18c14c16()?, PMC)?,
            df! { "PMC" => &[Series::from_iter([fatty_acid!(16).mass(), fatty_acid!(14).mass(), fatty_acid!(18).mass()])] }?,
        );
        assert_eq!(
            composition(c18c16c14()?, PMC)?,
            df! { "PMC" => &[Series::from_iter([fatty_acid!(14).mass(), fatty_acid!(16).mass(), fatty_acid!(18).mass()])] }?,
        );
        Ok(())
    }

    #[test]
    fn smc() -> PolarsResult<()> {
        assert_eq!(
            composition(c14c16c18()?, SMC)?,
            df! { "SMC" => &[Series::from_iter([fatty_acid!(14).mass(), fatty_acid!(16).mass(), fatty_acid!(18).mass()])] }?,
        );
        assert_eq!(
            composition(c14c18c16()?, SMC)?,
            df! { "SMC" => &[Series::from_iter([fatty_acid!(14).mass(), fatty_acid!(18).mass(), fatty_acid!(16).mass()])] }?,
        );
        assert_eq!(
            composition(c16c14c18()?, SMC)?,
            df! { "SMC" => &[Series::from_iter([fatty_acid!(16).mass(), fatty_acid!(14).mass(), fatty_acid!(18).mass()])] }?,
        );
        assert_eq!(
            composition(c16c18c14()?, SMC)?,
            df! { "SMC" => &[Series::from_iter([fatty_acid!(16).mass(), fatty_acid!(18).mass(), fatty_acid!(14).mass()])] }?,
        );
        assert_eq!(
            composition(c18c14c16()?, SMC)?,
            df! { "SMC" => &[Series::from_iter([fatty_acid!(18).mass(), fatty_acid!(14).mass(), fatty_acid!(16).mass()])] }?,
        );
        assert_eq!(
            composition(c18c16c14()?, SMC)?,
            df! { "SMC" => &[Series::from_iter([fatty_acid!(18).mass(), fatty_acid!(16).mass(), fatty_acid!(14).mass()])] }?,
        );
        Ok(())
    }

    #[test]
    fn nc() -> PolarsResult<()> {
        assert_eq!(
            composition(c14c16c18()?, NC)?,
            df! { "NC" => &[Series::from_iter([fatty_acid!(14).ecn(), fatty_acid!(16).ecn(), fatty_acid!(18).ecn()])] }?,
        );
        assert_eq!(
            composition(c14c18c16()?, NC)?,
            df! { "NC" => &[Series::from_iter([fatty_acid!(14).ecn(), fatty_acid!(16).ecn(), fatty_acid!(18).ecn()])] }?,
        );
        assert_eq!(
            composition(c16c14c18()?, NC)?,
            df! { "NC" => &[Series::from_iter([fatty_acid!(14).ecn(), fatty_acid!(16).ecn(), fatty_acid!(18).ecn()])] }?,
        );
        assert_eq!(
            composition(c16c18c14()?, NC)?,
            df! { "NC" => &[Series::from_iter([fatty_acid!(14).ecn(), fatty_acid!(16).ecn(), fatty_acid!(18).ecn()])] }?,
        );
        assert_eq!(
            composition(c18c14c16()?, NC)?,
            df! { "NC" => &[Series::from_iter([fatty_acid!(14).ecn(), fatty_acid!(16).ecn(), fatty_acid!(18).ecn()])] }?,
        );
        assert_eq!(
            composition(c18c16c14()?, NC)?,
            df! { "NC" => &[Series::from_iter([fatty_acid!(14).ecn(), fatty_acid!(16).ecn(), fatty_acid!(18).ecn()])] }?,
        );
        Ok(())
    }

    #[test]
    fn pnc() -> PolarsResult<()> {
        assert_eq!(
            composition(c14c16c18()?, PNC)?,
            df! { "PNC" => &[Series::from_iter([fatty_acid!(14).ecn(), fatty_acid!(16).ecn(), fatty_acid!(18).ecn()])] }?,
        );
        assert_eq!(
            composition(c14c18c16()?, PNC)?,
            df! { "PNC" => &[Series::from_iter([fatty_acid!(14).ecn(), fatty_acid!(18).ecn(), fatty_acid!(16).ecn()])] }?,
        );
        assert_eq!(
            composition(c16c14c18()?, PNC)?,
            df! { "PNC" => &[Series::from_iter([fatty_acid!(16).ecn(), fatty_acid!(14).ecn(), fatty_acid!(18).ecn()])] }?,
        );
        assert_eq!(
            composition(c16c18c14()?, PNC)?,
            df! { "PNC" => &[Series::from_iter([fatty_acid!(14).ecn(), fatty_acid!(18).ecn(), fatty_acid!(16).ecn()])] }?,
        );
        assert_eq!(
            composition(c18c14c16()?, PNC)?,
            df! { "PNC" => &[Series::from_iter([fatty_acid!(16).ecn(), fatty_acid!(14).ecn(), fatty_acid!(18).ecn()])] }?,
        );
        assert_eq!(
            composition(c18c16c14()?, PNC)?,
            df! { "PNC" => &[Series::from_iter([fatty_acid!(14).ecn(), fatty_acid!(16).ecn(), fatty_acid!(18).ecn()])] }?,
        );
        Ok(())
    }

    #[test]
    fn snc() -> PolarsResult<()> {
        assert_eq!(
            composition(c14c16c18()?, SNC)?,
            df! { "SNC" => &[Series::from_iter([fatty_acid!(14).ecn(), fatty_acid!(16).ecn(), fatty_acid!(18).ecn()])] }?,
        );
        assert_eq!(
            composition(c14c18c16()?, SNC)?,
            df! { "SNC" => &[Series::from_iter([fatty_acid!(14).ecn(), fatty_acid!(18).ecn(), fatty_acid!(16).ecn()])] }?,
        );
        assert_eq!(
            composition(c16c14c18()?, SNC)?,
            df! { "SNC" => &[Series::from_iter([fatty_acid!(16).ecn(), fatty_acid!(14).ecn(), fatty_acid!(18).ecn()])] }?,
        );
        assert_eq!(
            composition(c16c18c14()?, SNC)?,
            df! { "SNC" => &[Series::from_iter([fatty_acid!(16).ecn(), fatty_acid!(18).ecn(), fatty_acid!(14).ecn()])] }?,
        );
        assert_eq!(
            composition(c18c14c16()?, SNC)?,
            df! { "SNC" => &[Series::from_iter([fatty_acid!(18).ecn(), fatty_acid!(14).ecn(), fatty_acid!(16).ecn()])] }?,
        );
        assert_eq!(
            composition(c18c16c14()?, SNC)?,
            df! { "SNC" => &[Series::from_iter([fatty_acid!(18).ecn(), fatty_acid!(16).ecn(), fatty_acid!(14).ecn()])] }?,
        );
        Ok(())
    }

    #[test]
    fn sc() -> Result<()> {
        assert_eq!(
            composition(c14c16c18()?, SC)?,
            df! { "TAG" => &[Series::from_iter(["M", "P", "S"])] }?,
        );
        assert_eq!(
            composition(c14c18c16()?, SC)?,
            df! { "TAG" => &[Series::from_iter(["M", "P", "S"])] }?,
        );
        assert_eq!(
            composition(c16c14c18()?, SC)?,
            df! { "TAG" => &[Series::from_iter(["M", "P", "S"])] }?,
        );
        assert_eq!(
            composition(c16c18c14()?, SC)?,
            df! { "TAG" => &[Series::from_iter(["M", "P", "S"])] }?,
        );
        assert_eq!(
            composition(c18c14c16()?, SC)?,
            df! { "TAG" => &[Series::from_iter(["M", "P", "S"])] }?,
        );
        assert_eq!(
            composition(c18c16c14()?, SC)?,
            df! { "TAG" => &[Series::from_iter(["M", "P", "S"])] }?,
        );
        Ok(())
    }

    #[test]
    fn psc() -> Result<()> {
        assert_eq!(
            composition(c14c16c18()?, PSC)?,
            df! { "TAG" => &[Series::from_iter(["M", "P", "S"])] }?,
        );
        assert_eq!(
            composition(c14c18c16()?, PSC)?,
            df! { "TAG" => &[Series::from_iter(["M", "S", "P"])] }?,
        );
        assert_eq!(
            composition(c16c14c18()?, PSC)?,
            df! { "TAG" => &[Series::from_iter(["P", "M", "S"])] }?,
        );
        assert_eq!(
            composition(c16c18c14()?, PSC)?,
            df! { "TAG" => &[Series::from_iter(["M", "S", "P"])] }?,
        );
        assert_eq!(
            composition(c18c14c16()?, PSC)?,
            df! { "TAG" => &[Series::from_iter(["P", "M", "S"])] }?,
        );
        assert_eq!(
            composition(c18c16c14()?, PSC)?,
            df! { "TAG" => &[Series::from_iter(["M", "P", "S"])] }?,
        );
        Ok(())
    }

    #[test]
    fn ssc() -> Result<()> {
        assert_eq!(
            composition(c14c16c18()?, SSC)?,
            df! { "TAG" => &[Series::from_iter(["M", "P", "S"])] }?,
        );
        assert_eq!(
            composition(c14c18c16()?, SSC)?,
            df! { "TAG" => &[Series::from_iter(["M", "S", "P"])] }?,
        );
        assert_eq!(
            composition(c16c14c18()?, SSC)?,
            df! { "TAG" => &[Series::from_iter(["P", "M", "S"])] }?,
        );
        assert_eq!(
            composition(c16c18c14()?, SSC)?,
            df! { "TAG" => &[Series::from_iter(["P", "S", "M"])] }?,
        );
        assert_eq!(
            composition(c18c14c16()?, SSC)?,
            df! { "TAG" => &[Series::from_iter(["S", "M", "P"])] }?,
        );
        assert_eq!(
            composition(c18c16c14()?, SSC)?,
            df! { "TAG" => &[Series::from_iter(["S", "P", "M"])] }?,
        );
        Ok(())
    }

    #[test]
    fn tc() -> Result<()> {
        assert_eq!(
            composition(su1u2()?, TC)?,
            df! { "TC" => &[Series::from_iter(["S", "U", "U"])] }?,
        );
        assert_eq!(
            composition(su2u1()?, TC)?,
            df! { "TC" => &[Series::from_iter(["S", "U", "U"])] }?,
        );
        assert_eq!(
            composition(u1su2()?, TC)?,
            df! { "TC" => &[Series::from_iter(["S", "U", "U"])] }?,
        );
        assert_eq!(
            composition(u2su1()?, TC)?,
            df! { "TC" => &[Series::from_iter(["S", "U", "U"])] }?,
        );
        assert_eq!(
            composition(u1u2s()?, TC)?,
            df! { "TC" => &[Series::from_iter(["S", "U", "U"])] }?,
        );
        assert_eq!(
            composition(u2u1s()?, TC)?,
            df! { "TC" => &[Series::from_iter(["S", "U", "U"])] }?,
        );
        Ok(())
    }

    #[test]
    fn ptc() -> Result<()> {
        assert_eq!(
            composition(su1u2()?, PTC)?,
            df! { "PTC" => &[Series::from_iter(["S", "U", "U"])] }?,
        );
        assert_eq!(
            composition(su2u1()?, PTC)?,
            df! { "PTC" => &[Series::from_iter(["S", "U", "U"])] }?,
        );
        assert_eq!(
            composition(u1su2()?, PTC)?,
            df! { "PTC" => &[Series::from_iter(["U", "S", "U"])] }?,
        );
        assert_eq!(
            composition(u2su1()?, PTC)?,
            df! { "PTC" => &[Series::from_iter(["U", "S", "U"])] }?,
        );
        assert_eq!(
            composition(u1u2s()?, PTC)?,
            df! { "PTC" => &[Series::from_iter(["S", "U", "U"])] }?,
        );
        assert_eq!(
            composition(u2u1s()?, PTC)?,
            df! { "PTC" => &[Series::from_iter(["S", "U", "U"])] }?,
        );
        Ok(())
    }

    #[test]
    fn stc() -> Result<()> {
        assert_eq!(
            composition(su1u2()?, STC)?,
            df! { "STC" => &[Series::from_iter(["S", "U", "U"])] }?,
        );
        assert_eq!(
            composition(su2u1()?, STC)?,
            df! { "STC" => &[Series::from_iter(["S", "U", "U"])] }?,
        );
        assert_eq!(
            composition(u1su2()?, STC)?,
            df! { "STC" => &[Series::from_iter(["U", "S", "U"])] }?,
        );
        assert_eq!(
            composition(u2su1()?, STC)?,
            df! { "STC" => &[Series::from_iter(["U", "S", "U"])] }?,
        );
        assert_eq!(
            composition(u1u2s()?, STC)?,
            df! { "STC" => &[Series::from_iter(["U", "U", "S"])] }?,
        );
        assert_eq!(
            composition(u2u1s()?, STC)?,
            df! { "STC" => &[Series::from_iter(["U", "U", "S"])] }?,
        );
        Ok(())
    }

    #[test]
    fn uc() -> Result<()> {
        assert_eq!(
            composition(su1u2()?, UC)?,
            df! { "UC" => &[Series::from_iter([0, 1, 2])] }?,
        );
        assert_eq!(
            composition(su2u1()?, UC)?,
            df! { "UC" => &[Series::from_iter([0, 1, 2])] }?,
        );
        assert_eq!(
            composition(u1su2()?, UC)?,
            df! { "UC" => &[Series::from_iter([0, 1, 2])] }?,
        );
        assert_eq!(
            composition(u2su1()?, UC)?,
            df! { "UC" => &[Series::from_iter([0, 1, 2])] }?,
        );
        assert_eq!(
            composition(u1u2s()?, UC)?,
            df! { "UC" => &[Series::from_iter([0, 1, 2])] }?,
        );
        assert_eq!(
            composition(u2u1s()?, UC)?,
            df! { "UC" => &[Series::from_iter([0, 1, 2])] }?,
        );
        Ok(())
    }

    #[test]
    fn puc() -> Result<()> {
        assert_eq!(
            composition(su1u2()?, PUC)?,
            df! { "PUC" => &[Series::from_iter([0, 1, 2])] }?,
        );
        assert_eq!(
            composition(su2u1()?, PUC)?,
            df! { "PUC" => &[Series::from_iter([0, 2, 1])] }?,
        );
        assert_eq!(
            composition(u1su2()?, PUC)?,
            df! { "PUC" => &[Series::from_iter([1, 0, 2])] }?,
        );
        assert_eq!(
            composition(u2su1()?, PUC)?,
            df! { "PUC" => &[Series::from_iter([1, 0, 2])] }?,
        );
        assert_eq!(
            composition(u1u2s()?, PUC)?,
            df! { "PUC" => &[Series::from_iter([0, 2, 1])] }?,
        );
        assert_eq!(
            composition(u2u1s()?, PUC)?,
            df! { "PUC" => &[Series::from_iter([0, 1, 2])] }?,
        );
        Ok(())
    }

    #[test]
    fn suc() -> Result<()> {
        assert_eq!(
            composition(su1u2()?, SUC)?,
            df! { "SUC" => &[Series::from_iter([0, 1, 2])] }?,
        );
        assert_eq!(
            composition(su2u1()?, SUC)?,
            df! { "SUC" => &[Series::from_iter([0, 2, 1])] }?,
        );
        assert_eq!(
            composition(u1su2()?, SUC)?,
            df! { "SUC" => &[Series::from_iter([1, 0, 2])] }?,
        );
        assert_eq!(
            composition(u2su1()?, SUC)?,
            df! { "SUC" => &[Series::from_iter([2, 0, 1])] }?,
        );
        assert_eq!(
            composition(u1u2s()?, SUC)?,
            df! { "SUC" => &[Series::from_iter([1, 2, 0])] }?,
        );
        assert_eq!(
            composition(u2u1s()?, SUC)?,
            df! { "SUC" => &[Series::from_iter([2, 1, 0])] }?,
        );
        Ok(())
    }
}

// pub mod lazy_frame;
pub mod column;
pub mod columns;
pub mod data_frame;
