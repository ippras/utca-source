pub(super) use self::{
    calculation::{Computed as CalculationComputed, Key as CalculationKey},
    composition::{Computed as CompositionComputed, Key as CompositionKey},
    // visualization::{Computed as VisualizationComputed, Key as VisualizationKey},
};

pub(super) mod calculation;
pub(super) mod composition;
pub(super) mod visualization;

pub(crate) mod compositions;
