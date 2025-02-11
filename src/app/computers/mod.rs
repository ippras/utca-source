pub(super) use self::{
    calculation::{Computed as CalculationComputed, Key as CalculationKey},
    composition::{Computed as CompositionComputed, Key as CompositionKey},
    fatty_acids::{Computed as FattyAcidsComputed, Key as FattyAcidsKey},
};

pub(super) mod calculation;
pub(super) mod composition;
pub(super) mod fatty_acids;
