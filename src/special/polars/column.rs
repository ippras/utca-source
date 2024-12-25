use anyhow::{Context, anyhow};
use polars::prelude::*;
use std::{borrow::Cow, process::exit};
use thiserror::Error;

// let fatty_acids = entry.destruct("FA");
// // let triples = fatty_acids.explode(["Triples"])?;
// // let triples = triples["Triples"].i8()?;
// let labels = fatty_acids.str("Label");
// let carbons = fatty_acids.u8("Carbons");
// let doubles = fatty_acids.list("Doubles");
// let triples = fatty_acids.list("Triples");
// let tags = entry.f64("TAG");
// let dags1223 = entry.f64("DAG1223");
// let mags2 = entry.f64("MAG2");
// let label = labels.get(index).unwrap();
// let carbons = carbons.get(index).unwrap();
// let doubles = doubles.get_as_series(index).unwrap();
// let triples = triples.get_as_series(index).unwrap();
// let fatty_acid = &mut FattyAcid {
//     carbons,
//     doubles: doubles.i8().unwrap().to_vec_null_aware().left().unwrap(),
//     triples: triples.i8().unwrap().to_vec_null_aware().left().unwrap(),
// };

/// Extension methods for [`Column`]
pub trait ColumnExt {
    fn compositions(&self) -> Compositions;
}

impl ColumnExt for Column {
    fn compositions(&self) -> Compositions {
        Compositions::new(self).expect(r#"Expected "Compositions" column"#)
    }
}

// let labels = fatty_acids.str("Label");
// let carbons = fatty_acids.u8("Carbons");
// let doubles = fatty_acids.list("Doubles");
// let triples = fatty_acids.list("Triples");

/// FattyAcids
pub struct FattyAcids {
    pub carbons: Series,
    pub partition_points: Series,
    pub indices: Series,
}

impl FattyAcids {
    pub fn new(column: &Column) -> PolarsResult<Self> {
        let carbons = column.struct_()?.field_by_name("Carbons")?;
        let partition_points = column.struct_()?.field_by_name("PartitionPoints")?;
        let indices = column.struct_()?.field_by_name("Indices")?;
        Ok(Self {
            carbons,
            partition_points,
            indices,
        })
    }

    // pub fn get(&self, index: usize) -> PolarsResult<Composition> {
    //     let key = self.key(index)?;
    //     let value = self.value(index)?;
    //     Ok(Composition { key, value })
    // }
}

/// Compositions
pub struct Compositions {
    pub key: Series,
    pub value: Series,
}

impl Compositions {
    pub fn new(column: &Column) -> PolarsResult<Self> {
        let key = column.struct_()?.field_by_name("Key")?;
        let value = column.struct_()?.field_by_name("Value")?;
        Ok(Self { key, value })
    }

    pub fn get(&self, index: usize) -> PolarsResult<Composition> {
        let key = self.key(index)?;
        let value = self.value(index)?;
        Ok(Composition { key, value })
    }

    pub fn key(&self, index: usize) -> PolarsResult<Cow<str>> {
        self.key.str_value(index)
    }

    pub fn value(&self, index: usize) -> PolarsResult<Value> {
        let means = self.value.struct_()?.field_by_name("Mean")?;
        let mean = means
            .f64()?
            .get(index)
            .expect("composition value mean is null");
        let standard_deviations = self.value.struct_()?.field_by_name("StandardDeviation")?;
        let standard_deviation = standard_deviations
            .f64()?
            .get(index)
            .expect("composition value standard deviation is null");
        Ok(Value {
            mean,
            standard_deviation,
        })
    }

    pub fn sum(&self) -> PolarsResult<Value> {
        let means = self.value.struct_()?.field_by_name("Mean")?;
        let mean = means
            .f64()?
            .sum()
            .expect("composition value mean sum is null");
        let standard_deviations = self.value.struct_()?.field_by_name("StandardDeviation")?;
        let standard_deviation = standard_deviations
            .f64()?
            .sum()
            .expect("composition value standard deviation sum is null");
        Ok(Value {
            mean,
            standard_deviation,
        })
    }
}

/// Composition
pub struct Composition<'a> {
    pub key: Cow<'a, str>,
    pub value: Value,
}

/// Value
pub struct Value {
    pub mean: f64,
    pub standard_deviation: f64,
}
