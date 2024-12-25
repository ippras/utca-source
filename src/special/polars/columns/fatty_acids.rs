use crate::{
    r#const::relative_atomic_mass::{C, H, O},
    special::new_fatty_acid::{Isomerism, Unfolded, Unsaturation},
};
use indexmap::IndexMap;
use itertools::izip;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    convert::identity,
    fmt::{self, Display, Formatter, Write},
    iter::zip,
};

/// Extension methods for [`Column`]
pub trait ColumnExt {
    fn fa(&self) -> FattyAcids;
}

impl ColumnExt for Column {
    fn fa(&self) -> FattyAcids {
        FattyAcids::new(self).expect(r#"Expected "FattyAcids" column"#)
    }
}

/// Fatty acids
#[derive(Clone)]
pub struct FattyAcids {
    carbons: Series,
    doubles: Series,
    labels: Series,
}

impl FattyAcids {
    pub fn new(column: &Column) -> PolarsResult<Self> {
        let carbons = column.struct_()?.field_by_name("Carbons")?;
        let doubles = column.struct_()?.field_by_name("Doubles")?;
        let labels = column.struct_()?.field_by_name("Label")?;
        Ok(Self {
            carbons,
            doubles,
            labels,
        })
    }

    pub fn get(&self, index: usize) -> PolarsResult<(String, Unfolded)> {
        let label = self.labels.str()?.get(index).unwrap().to_owned();
        let carbons = self.carbons.u8()?.get(index).unwrap();
        let doubles = self.doubles.list()?.get_as_series(index).unwrap();
        let doubles = doubles.i8()?.iter().filter_map(identity);
        let mut indices = IndexMap::new();
        for double in doubles {
            let isomerism = if double.is_negative() {
                Isomerism::Trans
            } else {
                Isomerism::Cis
            };
            indices.insert(index, (Unsaturation::Two, isomerism));
        }
        indices.sort_by_cached_key(|_, (unsaturation, _)| *unsaturation);
        Ok((label, Unfolded { carbons, indices }))
    }

    // pub fn iter(&self) -> PolarsResult<impl Iterator<Item = FattyAcid> + '_> {
    //     Ok(
    //         izip!(self.carbons.u8()?, self.doubles.list()?, self.labels.str()?).filter_map(
    //             |(carbons, indices, bounds, label)| {
    //                 Some(FattyAcid {
    //                     carbons: carbons?,
    //                     doubles: doubles?.i8().unwrap().to_vec_null_aware().left()?,
    //                     label: label?.to_owned(),
    //                 })
    //             },
    //         ),
    //     )
    // }
}
