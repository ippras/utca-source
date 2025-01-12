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

/// Extension methods for [`Schema`]
pub trait SchemaExt {
    fn names(&self) -> Vec<Expr>;
}

impl SchemaExt for Schema {
    fn names(&self) -> Vec<Expr> {
        self.iter_names_cloned().map(col).collect()
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::special::fatty_acid::fatty_acid;
//     use anyhow::Result;

//     fn c14c16c18() -> PolarsResult<DataFrame> {
//         Ok(df! {
//            "TAG" => df! {
//                 "SN1" => df! {
//                     "FA" => df! {
//                         "Label" => &["M"],
//                         "Carbons" => &[14u8],
//                         "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//                 "SN2" => df! {
//                     "FA" => df! {
//                         "Label" => &["P"],
//                         "Carbons" => &[16u8],
//                         "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//                 "SN3" => df! {
//                     "FA" => df! {
//                         "Label" => &["S"],
//                         "Carbons" => &[18u8],
//                         "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//             }?.into_struct(PlSmallStr::EMPTY),
//         }?)
//     }

//     fn c14c18c16() -> PolarsResult<DataFrame> {
//         Ok(df! {
//            "TAG" => df! {
//                 "SN1" => df! {
//                     "FA" => df! {
//                         "Label" => &["M"],
//                         "Carbons" => &[14u8],
//                         "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//                 "SN2" => df! {
//                     "FA" => df! {
//                         "Label" => &["S"],
//                         "Carbons" => &[18u8],
//                         "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//                 "SN3" => df! {
//                     "FA" => df! {
//                         "Label" => &["P"],
//                         "Carbons" => &[16u8],
//                         "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//             }?.into_struct(PlSmallStr::EMPTY),
//         }?)
//     }

//     fn c16c14c18() -> PolarsResult<DataFrame> {
//         Ok(df! {
//             "TAG" => df! {
//                 "SN1" => df! {
//                     "FA" => df! {
//                         "Label" => &["P"],
//                         "Carbons" => &[16u8],
//                         "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//                 "SN2" => df! {
//                     "FA" => df! {
//                         "Label" => &["M"],
//                         "Carbons" => &[14u8],
//                         "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//                 "SN3" => df! {
//                     "FA" => df! {
//                         "Label" => &["S"],
//                         "Carbons" => &[18u8],
//                         "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//             }?.into_struct(PlSmallStr::EMPTY),
//         }?)
//     }

//     fn c16c18c14() -> PolarsResult<DataFrame> {
//         Ok(df! {
//             "TAG" => df! {
//                 "SN1" => df! {
//                     "FA" => df! {
//                         "Label" => &["P"],
//                         "Carbons" => &[16u8],
//                         "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//                 "SN2" => df! {
//                     "FA" => df! {
//                         "Label" => &["S"],
//                         "Carbons" => &[18u8],
//                         "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//                 "SN3" => df! {
//                     "FA" => df! {
//                         "Label" => &["M"],
//                         "Carbons" => &[14u8],
//                         "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//             }?.into_struct(PlSmallStr::EMPTY),
//         }?)
//     }

//     fn c18c14c16() -> PolarsResult<DataFrame> {
//         Ok(df! {
//            "TAG" => df! {
//                 "SN1" => df! {
//                     "FA" => df! {
//                         "Label" => &["S"],
//                         "Carbons" => &[18u8],
//                         "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//                 "SN2" => df! {
//                     "FA" => df! {
//                         "Label" => &["M"],
//                         "Carbons" => &[14u8],
//                         "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//                 "SN3" => df! {
//                     "FA" => df! {
//                         "Label" => &["P"],
//                         "Carbons" => &[16u8],
//                         "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//             }?.into_struct(PlSmallStr::EMPTY),
//         }?)
//     }

//     fn c18c16c14() -> PolarsResult<DataFrame> {
//         Ok(df! {
//            "TAG" => df! {
//                 "SN1" => df! {
//                     "FA" => df! {
//                         "Label" => &["S"],
//                         "Carbons" => &[18u8],
//                         "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//                 "SN2" => df! {
//                     "FA" => df! {
//                         "Label" => &["P"],
//                         "Carbons" => &[16u8],
//                         "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//                 "SN3" => df! {
//                     "FA" => df! {
//                         "Label" => &["M"],
//                         "Carbons" => &[14u8],
//                         "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//             }?.into_struct(PlSmallStr::EMPTY),
//         }?)
//     }

//     fn su1u2() -> PolarsResult<DataFrame> {
//         Ok(df! {
//             "TAG" => df! {
//                 "SN1" => df! {
//                     "FA" => df! {
//                         "Label" => &["S"],
//                         "Carbons" => &[18u8],
//                         "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//                 "SN2" => df! {
//                     "FA" => df! {
//                         "Label" => &["O"],
//                         "Carbons" => &[18u8],
//                         "Doubles" => &[Series::from_iter([9])],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//                 "SN3" => df! {
//                     "FA" => df! {
//                         "Label" => &["L"],
//                         "Carbons" => &[18u8],
//                         "Doubles" => &[Series::from_iter([9, 12])],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//             }?.into_struct(PlSmallStr::EMPTY),
//         }?)
//     }

//     fn su2u1() -> PolarsResult<DataFrame> {
//         Ok(df! {
//             "TAG" => df! {
//                 "SN1" => df! {
//                     "FA" => df! {
//                         "Label" => &["S"],
//                         "Carbons" => &[18u8],
//                         "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//                 "SN2" => df! {
//                     "FA" => df! {
//                         "Label" => &["L"],
//                         "Carbons" => &[18u8],
//                         "Doubles" => &[Series::from_iter([9, 12])],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//                 "SN3" => df! {
//                     "FA" => df! {
//                         "Label" => &["O"],
//                         "Carbons" => &[18u8],
//                         "Doubles" => &[Series::from_iter([9])],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//             }?.into_struct(PlSmallStr::EMPTY),
//         }?)
//     }

//     fn u1su2() -> PolarsResult<DataFrame> {
//         Ok(df! {
//             "TAG" => df! {
//                 "SN1" => df! {
//                     "FA" => df! {
//                         "Label" => &["O"],
//                         "Carbons" => &[18u8],
//                         "Doubles" => &[Series::from_iter([9])],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//                 "SN2" => df! {
//                     "FA" => df! {
//                         "Label" => &["S"],
//                         "Carbons" => &[18u8],
//                         "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//                 "SN3" => df! {
//                     "FA" => df! {
//                         "Label" => &["L"],
//                         "Carbons" => &[18u8],
//                         "Doubles" => &[Series::from_iter([9, 12])],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//             }?.into_struct(PlSmallStr::EMPTY),
//         }?)
//     }

//     fn u2su1() -> PolarsResult<DataFrame> {
//         Ok(df! {
//             "TAG" => df! {
//                 "SN1" => df! {
//                     "FA" => df! {
//                         "Label" => &["L"],
//                         "Carbons" => &[18u8],
//                         "Doubles" => &[Series::from_iter([9, 12])],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//                 "SN2" => df! {
//                     "FA" => df! {
//                         "Label" => &["S"],
//                         "Carbons" => &[18u8],
//                         "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//                 "SN3" => df! {
//                     "FA" => df! {
//                         "Label" => &["O"],
//                         "Carbons" => &[18u8],
//                         "Doubles" => &[Series::from_iter([9])],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//             }?.into_struct(PlSmallStr::EMPTY),
//         }?)
//     }

//     fn u1u2s() -> PolarsResult<DataFrame> {
//         Ok(df! {
//             "TAG" => df! {
//                 "SN1" => df! {
//                     "FA" => df! {
//                         "Label" => &["O"],
//                         "Carbons" => &[18u8],
//                         "Doubles" => &[Series::from_iter([9])],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//                 "SN2" => df! {
//                     "FA" => df! {
//                         "Label" => &["L"],
//                         "Carbons" => &[18u8],
//                         "Doubles" => &[Series::from_iter([9, 12])],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//                 "SN3" => df! {
//                     "FA" => df! {
//                         "Label" => &["S"],
//                         "Carbons" => &[18u8],
//                         "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//             }?.into_struct(PlSmallStr::EMPTY),
//         }?)
//     }

//     fn u2u1s() -> PolarsResult<DataFrame> {
//         Ok(df! {
//             "TAG" => df! {
//                 "SN1" => df! {
//                     "FA" => df! {
//                         "Label" => &["L"],
//                         "Carbons" => &[18u8],
//                         "Doubles" => &[Series::from_iter([9, 12])],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//                 "SN2" => df! {
//                     "FA" => df! {
//                         "Label" => &["O"],
//                         "Carbons" => &[18u8],
//                         "Doubles" => &[Series::from_iter([9])],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//                 "SN3" => df! {
//                     "FA" => df! {
//                         "Label" => &["S"],
//                         "Carbons" => &[18u8],
//                         "Doubles" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                         "Triples" => &[Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8)],
//                     }?
//                     .into_struct(PlSmallStr::EMPTY),
//                 }?
//                 .into_struct(PlSmallStr::EMPTY),
//             }?.into_struct(PlSmallStr::EMPTY),
//         }?)
//     }

//     #[test]
//     fn mc() -> PolarsResult<()> {
//         assert_eq!(
//             composition(c14c16c18()?, MC)?,
//             df! { "PMC" => &[Series::from_iter([fatty_acid!(14).mass(), fatty_acid!(16).mass(), fatty_acid!(18).mass()])] }?,
//         );
//         assert_eq!(
//             composition(c14c18c16()?, MC)?,
//             df! { "PMC" => &[Series::from_iter([fatty_acid!(14).mass(), fatty_acid!(16).mass(), fatty_acid!(18).mass()])] }?,
//         );
//         assert_eq!(
//             composition(c16c14c18()?, MC)?,
//             df! { "PMC" => &[Series::from_iter([fatty_acid!(14).mass(), fatty_acid!(16).mass(), fatty_acid!(18).mass()])] }?,
//         );
//         assert_eq!(
//             composition(c16c18c14()?, MC)?,
//             df! { "PMC" => &[Series::from_iter([fatty_acid!(14).mass(), fatty_acid!(16).mass(), fatty_acid!(18).mass()])] }?,
//         );
//         assert_eq!(
//             composition(c18c14c16()?, MC)?,
//             df! { "PMC" => &[Series::from_iter([fatty_acid!(14).mass(), fatty_acid!(16).mass(), fatty_acid!(18).mass()])] }?,
//         );
//         assert_eq!(
//             composition(c18c16c14()?, MC)?,
//             df! { "PMC" => &[Series::from_iter([fatty_acid!(14).mass(), fatty_acid!(16).mass(), fatty_acid!(18).mass()])] }?,
//         );
//         Ok(())
//     }

//     #[test]
//     fn pmc() -> PolarsResult<()> {
//         assert_eq!(
//             composition(c14c16c18()?, PMC)?,
//             df! { "PMC" => &[Series::from_iter([fatty_acid!(14).mass(), fatty_acid!(16).mass(), fatty_acid!(18).mass()])] }?,
//         );
//         assert_eq!(
//             composition(c14c18c16()?, PMC)?,
//             df! { "PMC" => &[Series::from_iter([fatty_acid!(14).mass(), fatty_acid!(18).mass(), fatty_acid!(16).mass()])] }?,
//         );
//         assert_eq!(
//             composition(c16c14c18()?, PMC)?,
//             df! { "PMC" => &[Series::from_iter([fatty_acid!(16).mass(), fatty_acid!(14).mass(), fatty_acid!(18).mass()])] }?,
//         );
//         assert_eq!(
//             composition(c16c18c14()?, PMC)?,
//             df! { "PMC" => &[Series::from_iter([fatty_acid!(14).mass(), fatty_acid!(18).mass(), fatty_acid!(16).mass()])] }?,
//         );
//         assert_eq!(
//             composition(c18c14c16()?, PMC)?,
//             df! { "PMC" => &[Series::from_iter([fatty_acid!(16).mass(), fatty_acid!(14).mass(), fatty_acid!(18).mass()])] }?,
//         );
//         assert_eq!(
//             composition(c18c16c14()?, PMC)?,
//             df! { "PMC" => &[Series::from_iter([fatty_acid!(14).mass(), fatty_acid!(16).mass(), fatty_acid!(18).mass()])] }?,
//         );
//         Ok(())
//     }

//     #[test]
//     fn smc() -> PolarsResult<()> {
//         assert_eq!(
//             composition(c14c16c18()?, SMC)?,
//             df! { "SMC" => &[Series::from_iter([fatty_acid!(14).mass(), fatty_acid!(16).mass(), fatty_acid!(18).mass()])] }?,
//         );
//         assert_eq!(
//             composition(c14c18c16()?, SMC)?,
//             df! { "SMC" => &[Series::from_iter([fatty_acid!(14).mass(), fatty_acid!(18).mass(), fatty_acid!(16).mass()])] }?,
//         );
//         assert_eq!(
//             composition(c16c14c18()?, SMC)?,
//             df! { "SMC" => &[Series::from_iter([fatty_acid!(16).mass(), fatty_acid!(14).mass(), fatty_acid!(18).mass()])] }?,
//         );
//         assert_eq!(
//             composition(c16c18c14()?, SMC)?,
//             df! { "SMC" => &[Series::from_iter([fatty_acid!(16).mass(), fatty_acid!(18).mass(), fatty_acid!(14).mass()])] }?,
//         );
//         assert_eq!(
//             composition(c18c14c16()?, SMC)?,
//             df! { "SMC" => &[Series::from_iter([fatty_acid!(18).mass(), fatty_acid!(14).mass(), fatty_acid!(16).mass()])] }?,
//         );
//         assert_eq!(
//             composition(c18c16c14()?, SMC)?,
//             df! { "SMC" => &[Series::from_iter([fatty_acid!(18).mass(), fatty_acid!(16).mass(), fatty_acid!(14).mass()])] }?,
//         );
//         Ok(())
//     }

//     #[test]
//     fn nc() -> PolarsResult<()> {
//         assert_eq!(
//             composition(c14c16c18()?, NC)?,
//             df! { "NC" => &[Series::from_iter([fatty_acid!(14).ecn(), fatty_acid!(16).ecn(), fatty_acid!(18).ecn()])] }?,
//         );
//         assert_eq!(
//             composition(c14c18c16()?, NC)?,
//             df! { "NC" => &[Series::from_iter([fatty_acid!(14).ecn(), fatty_acid!(16).ecn(), fatty_acid!(18).ecn()])] }?,
//         );
//         assert_eq!(
//             composition(c16c14c18()?, NC)?,
//             df! { "NC" => &[Series::from_iter([fatty_acid!(14).ecn(), fatty_acid!(16).ecn(), fatty_acid!(18).ecn()])] }?,
//         );
//         assert_eq!(
//             composition(c16c18c14()?, NC)?,
//             df! { "NC" => &[Series::from_iter([fatty_acid!(14).ecn(), fatty_acid!(16).ecn(), fatty_acid!(18).ecn()])] }?,
//         );
//         assert_eq!(
//             composition(c18c14c16()?, NC)?,
//             df! { "NC" => &[Series::from_iter([fatty_acid!(14).ecn(), fatty_acid!(16).ecn(), fatty_acid!(18).ecn()])] }?,
//         );
//         assert_eq!(
//             composition(c18c16c14()?, NC)?,
//             df! { "NC" => &[Series::from_iter([fatty_acid!(14).ecn(), fatty_acid!(16).ecn(), fatty_acid!(18).ecn()])] }?,
//         );
//         Ok(())
//     }

//     #[test]
//     fn pnc() -> PolarsResult<()> {
//         assert_eq!(
//             composition(c14c16c18()?, PNC)?,
//             df! { "PNC" => &[Series::from_iter([fatty_acid!(14).ecn(), fatty_acid!(16).ecn(), fatty_acid!(18).ecn()])] }?,
//         );
//         assert_eq!(
//             composition(c14c18c16()?, PNC)?,
//             df! { "PNC" => &[Series::from_iter([fatty_acid!(14).ecn(), fatty_acid!(18).ecn(), fatty_acid!(16).ecn()])] }?,
//         );
//         assert_eq!(
//             composition(c16c14c18()?, PNC)?,
//             df! { "PNC" => &[Series::from_iter([fatty_acid!(16).ecn(), fatty_acid!(14).ecn(), fatty_acid!(18).ecn()])] }?,
//         );
//         assert_eq!(
//             composition(c16c18c14()?, PNC)?,
//             df! { "PNC" => &[Series::from_iter([fatty_acid!(14).ecn(), fatty_acid!(18).ecn(), fatty_acid!(16).ecn()])] }?,
//         );
//         assert_eq!(
//             composition(c18c14c16()?, PNC)?,
//             df! { "PNC" => &[Series::from_iter([fatty_acid!(16).ecn(), fatty_acid!(14).ecn(), fatty_acid!(18).ecn()])] }?,
//         );
//         assert_eq!(
//             composition(c18c16c14()?, PNC)?,
//             df! { "PNC" => &[Series::from_iter([fatty_acid!(14).ecn(), fatty_acid!(16).ecn(), fatty_acid!(18).ecn()])] }?,
//         );
//         Ok(())
//     }

//     #[test]
//     fn snc() -> PolarsResult<()> {
//         assert_eq!(
//             composition(c14c16c18()?, SNC)?,
//             df! { "SNC" => &[Series::from_iter([fatty_acid!(14).ecn(), fatty_acid!(16).ecn(), fatty_acid!(18).ecn()])] }?,
//         );
//         assert_eq!(
//             composition(c14c18c16()?, SNC)?,
//             df! { "SNC" => &[Series::from_iter([fatty_acid!(14).ecn(), fatty_acid!(18).ecn(), fatty_acid!(16).ecn()])] }?,
//         );
//         assert_eq!(
//             composition(c16c14c18()?, SNC)?,
//             df! { "SNC" => &[Series::from_iter([fatty_acid!(16).ecn(), fatty_acid!(14).ecn(), fatty_acid!(18).ecn()])] }?,
//         );
//         assert_eq!(
//             composition(c16c18c14()?, SNC)?,
//             df! { "SNC" => &[Series::from_iter([fatty_acid!(16).ecn(), fatty_acid!(18).ecn(), fatty_acid!(14).ecn()])] }?,
//         );
//         assert_eq!(
//             composition(c18c14c16()?, SNC)?,
//             df! { "SNC" => &[Series::from_iter([fatty_acid!(18).ecn(), fatty_acid!(14).ecn(), fatty_acid!(16).ecn()])] }?,
//         );
//         assert_eq!(
//             composition(c18c16c14()?, SNC)?,
//             df! { "SNC" => &[Series::from_iter([fatty_acid!(18).ecn(), fatty_acid!(16).ecn(), fatty_acid!(14).ecn()])] }?,
//         );
//         Ok(())
//     }

//     #[test]
//     fn sc() -> Result<()> {
//         assert_eq!(
//             composition(c14c16c18()?, SC)?,
//             df! { "TAG" => &[Series::from_iter(["M", "P", "S"])] }?,
//         );
//         assert_eq!(
//             composition(c14c18c16()?, SC)?,
//             df! { "TAG" => &[Series::from_iter(["M", "P", "S"])] }?,
//         );
//         assert_eq!(
//             composition(c16c14c18()?, SC)?,
//             df! { "TAG" => &[Series::from_iter(["M", "P", "S"])] }?,
//         );
//         assert_eq!(
//             composition(c16c18c14()?, SC)?,
//             df! { "TAG" => &[Series::from_iter(["M", "P", "S"])] }?,
//         );
//         assert_eq!(
//             composition(c18c14c16()?, SC)?,
//             df! { "TAG" => &[Series::from_iter(["M", "P", "S"])] }?,
//         );
//         assert_eq!(
//             composition(c18c16c14()?, SC)?,
//             df! { "TAG" => &[Series::from_iter(["M", "P", "S"])] }?,
//         );
//         Ok(())
//     }

//     #[test]
//     fn psc() -> Result<()> {
//         assert_eq!(
//             composition(c14c16c18()?, PSC)?,
//             df! { "TAG" => &[Series::from_iter(["M", "P", "S"])] }?,
//         );
//         assert_eq!(
//             composition(c14c18c16()?, PSC)?,
//             df! { "TAG" => &[Series::from_iter(["M", "S", "P"])] }?,
//         );
//         assert_eq!(
//             composition(c16c14c18()?, PSC)?,
//             df! { "TAG" => &[Series::from_iter(["P", "M", "S"])] }?,
//         );
//         assert_eq!(
//             composition(c16c18c14()?, PSC)?,
//             df! { "TAG" => &[Series::from_iter(["M", "S", "P"])] }?,
//         );
//         assert_eq!(
//             composition(c18c14c16()?, PSC)?,
//             df! { "TAG" => &[Series::from_iter(["P", "M", "S"])] }?,
//         );
//         assert_eq!(
//             composition(c18c16c14()?, PSC)?,
//             df! { "TAG" => &[Series::from_iter(["M", "P", "S"])] }?,
//         );
//         Ok(())
//     }

//     #[test]
//     fn ssc() -> Result<()> {
//         assert_eq!(
//             composition(c14c16c18()?, SSC)?,
//             df! { "TAG" => &[Series::from_iter(["M", "P", "S"])] }?,
//         );
//         assert_eq!(
//             composition(c14c18c16()?, SSC)?,
//             df! { "TAG" => &[Series::from_iter(["M", "S", "P"])] }?,
//         );
//         assert_eq!(
//             composition(c16c14c18()?, SSC)?,
//             df! { "TAG" => &[Series::from_iter(["P", "M", "S"])] }?,
//         );
//         assert_eq!(
//             composition(c16c18c14()?, SSC)?,
//             df! { "TAG" => &[Series::from_iter(["P", "S", "M"])] }?,
//         );
//         assert_eq!(
//             composition(c18c14c16()?, SSC)?,
//             df! { "TAG" => &[Series::from_iter(["S", "M", "P"])] }?,
//         );
//         assert_eq!(
//             composition(c18c16c14()?, SSC)?,
//             df! { "TAG" => &[Series::from_iter(["S", "P", "M"])] }?,
//         );
//         Ok(())
//     }

//     #[test]
//     fn tc() -> Result<()> {
//         assert_eq!(
//             composition(su1u2()?, TC)?,
//             df! { "TC" => &[Series::from_iter(["S", "U", "U"])] }?,
//         );
//         assert_eq!(
//             composition(su2u1()?, TC)?,
//             df! { "TC" => &[Series::from_iter(["S", "U", "U"])] }?,
//         );
//         assert_eq!(
//             composition(u1su2()?, TC)?,
//             df! { "TC" => &[Series::from_iter(["S", "U", "U"])] }?,
//         );
//         assert_eq!(
//             composition(u2su1()?, TC)?,
//             df! { "TC" => &[Series::from_iter(["S", "U", "U"])] }?,
//         );
//         assert_eq!(
//             composition(u1u2s()?, TC)?,
//             df! { "TC" => &[Series::from_iter(["S", "U", "U"])] }?,
//         );
//         assert_eq!(
//             composition(u2u1s()?, TC)?,
//             df! { "TC" => &[Series::from_iter(["S", "U", "U"])] }?,
//         );
//         Ok(())
//     }

//     #[test]
//     fn ptc() -> Result<()> {
//         assert_eq!(
//             composition(su1u2()?, PTC)?,
//             df! { "PTC" => &[Series::from_iter(["S", "U", "U"])] }?,
//         );
//         assert_eq!(
//             composition(su2u1()?, PTC)?,
//             df! { "PTC" => &[Series::from_iter(["S", "U", "U"])] }?,
//         );
//         assert_eq!(
//             composition(u1su2()?, PTC)?,
//             df! { "PTC" => &[Series::from_iter(["U", "S", "U"])] }?,
//         );
//         assert_eq!(
//             composition(u2su1()?, PTC)?,
//             df! { "PTC" => &[Series::from_iter(["U", "S", "U"])] }?,
//         );
//         assert_eq!(
//             composition(u1u2s()?, PTC)?,
//             df! { "PTC" => &[Series::from_iter(["S", "U", "U"])] }?,
//         );
//         assert_eq!(
//             composition(u2u1s()?, PTC)?,
//             df! { "PTC" => &[Series::from_iter(["S", "U", "U"])] }?,
//         );
//         Ok(())
//     }

//     #[test]
//     fn stc() -> Result<()> {
//         assert_eq!(
//             composition(su1u2()?, STC)?,
//             df! { "STC" => &[Series::from_iter(["S", "U", "U"])] }?,
//         );
//         assert_eq!(
//             composition(su2u1()?, STC)?,
//             df! { "STC" => &[Series::from_iter(["S", "U", "U"])] }?,
//         );
//         assert_eq!(
//             composition(u1su2()?, STC)?,
//             df! { "STC" => &[Series::from_iter(["U", "S", "U"])] }?,
//         );
//         assert_eq!(
//             composition(u2su1()?, STC)?,
//             df! { "STC" => &[Series::from_iter(["U", "S", "U"])] }?,
//         );
//         assert_eq!(
//             composition(u1u2s()?, STC)?,
//             df! { "STC" => &[Series::from_iter(["U", "U", "S"])] }?,
//         );
//         assert_eq!(
//             composition(u2u1s()?, STC)?,
//             df! { "STC" => &[Series::from_iter(["U", "U", "S"])] }?,
//         );
//         Ok(())
//     }

//     #[test]
//     fn uc() -> Result<()> {
//         assert_eq!(
//             composition(su1u2()?, UC)?,
//             df! { "UC" => &[Series::from_iter([0, 1, 2])] }?,
//         );
//         assert_eq!(
//             composition(su2u1()?, UC)?,
//             df! { "UC" => &[Series::from_iter([0, 1, 2])] }?,
//         );
//         assert_eq!(
//             composition(u1su2()?, UC)?,
//             df! { "UC" => &[Series::from_iter([0, 1, 2])] }?,
//         );
//         assert_eq!(
//             composition(u2su1()?, UC)?,
//             df! { "UC" => &[Series::from_iter([0, 1, 2])] }?,
//         );
//         assert_eq!(
//             composition(u1u2s()?, UC)?,
//             df! { "UC" => &[Series::from_iter([0, 1, 2])] }?,
//         );
//         assert_eq!(
//             composition(u2u1s()?, UC)?,
//             df! { "UC" => &[Series::from_iter([0, 1, 2])] }?,
//         );
//         Ok(())
//     }

//     #[test]
//     fn puc() -> Result<()> {
//         assert_eq!(
//             composition(su1u2()?, PUC)?,
//             df! { "PUC" => &[Series::from_iter([0, 1, 2])] }?,
//         );
//         assert_eq!(
//             composition(su2u1()?, PUC)?,
//             df! { "PUC" => &[Series::from_iter([0, 2, 1])] }?,
//         );
//         assert_eq!(
//             composition(u1su2()?, PUC)?,
//             df! { "PUC" => &[Series::from_iter([1, 0, 2])] }?,
//         );
//         assert_eq!(
//             composition(u2su1()?, PUC)?,
//             df! { "PUC" => &[Series::from_iter([1, 0, 2])] }?,
//         );
//         assert_eq!(
//             composition(u1u2s()?, PUC)?,
//             df! { "PUC" => &[Series::from_iter([0, 2, 1])] }?,
//         );
//         assert_eq!(
//             composition(u2u1s()?, PUC)?,
//             df! { "PUC" => &[Series::from_iter([0, 1, 2])] }?,
//         );
//         Ok(())
//     }

//     #[test]
//     fn suc() -> Result<()> {
//         assert_eq!(
//             composition(su1u2()?, SUC)?,
//             df! { "SUC" => &[Series::from_iter([0, 1, 2])] }?,
//         );
//         assert_eq!(
//             composition(su2u1()?, SUC)?,
//             df! { "SUC" => &[Series::from_iter([0, 2, 1])] }?,
//         );
//         assert_eq!(
//             composition(u1su2()?, SUC)?,
//             df! { "SUC" => &[Series::from_iter([1, 0, 2])] }?,
//         );
//         assert_eq!(
//             composition(u2su1()?, SUC)?,
//             df! { "SUC" => &[Series::from_iter([2, 0, 1])] }?,
//         );
//         assert_eq!(
//             composition(u1u2s()?, SUC)?,
//             df! { "SUC" => &[Series::from_iter([1, 2, 0])] }?,
//         );
//         assert_eq!(
//             composition(u2u1s()?, SUC)?,
//             df! { "SUC" => &[Series::from_iter([2, 1, 0])] }?,
//         );
//         Ok(())
//     }
// }

pub mod column;
pub mod columns;
