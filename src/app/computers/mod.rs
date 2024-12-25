pub(super) use self::{
    calculation::{
        Computed as CalculationComputed, Computer as CalculationComputer, Key as CalculationKey,
    },
    composition::{
        Computed as CompositionComputed, Computer as CompositionComputer, Key as CompositionKey,
    },
    visualization::{Computed as VisualizationComputed, Key as VisualizationKey},
};

pub(super) mod calculation;
pub(super) mod composition;
pub(super) mod visualization;

pub(crate) mod compositions;
pub(crate) mod tags;
