use crate::localization::localize;
use egui::Ui;
use egui_phosphor::regular::{CALCULATOR, INTERSECT_THREE, NOTE_PENCIL};
use polars::frame::DataFrame;
use serde::{Deserialize, Serialize};

/// Central pane
#[derive(Deserialize, Serialize)]
pub(crate) enum Pane {
    Configuration(configuration::Pane),
    Calculation(calculation::Pane),
    Composition(composition::Pane),
    Visualization(visualization::CompositionPlot),
}

impl Pane {
    pub(crate) fn calculation(data_frame: DataFrame) -> Self {
        Self::Calculation(calculation::Pane::new(data_frame))
    }

    pub(crate) fn composition(data_frame: DataFrame) -> Self {
        Self::Composition(composition::Pane::new(data_frame))
    }

    pub(crate) fn visualization(data_frame: DataFrame) -> Self {
        Self::Visualization(visualization::CompositionPlot::new(data_frame))
    }

    pub(crate) const fn icon(&self) -> &str {
        match self {
            Self::Configuration(_) => NOTE_PENCIL,
            Self::Calculation(_) => CALCULATOR,
            Self::Composition(_) => INTERSECT_THREE,
            Self::Visualization(_) => INTERSECT_THREE,
        }
    }

    pub(crate) const fn kind(&self) -> Kind {
        match self {
            Self::Configuration(_) => Kind::Configuration,
            Self::Calculation(_) => Kind::Calculation,
            Self::Composition(_) => Kind::Composition,
            Self::Visualization(_) => Kind::Visualization,
        }
    }

    pub(crate) fn title(&self) -> String {
        match self {
            Self::Configuration(_) => localize!("configuration"),
            Self::Calculation(_) => localize!("calculation"),
            Self::Composition(_) => localize!("composition"),
            Self::Visualization(_) => localize!("visualization"),
        }
    }

    pub(crate) fn hash(&self) -> u64 {
        match self {
            Self::Configuration(pane) => pane.hash(),
            Self::Calculation(pane) => pane.hash(),
            Self::Composition(pane) => pane.hash(),
            Self::Visualization(pane) => pane.hash(),
        }
    }

    fn header(&mut self, ui: &mut Ui) {
        match self {
            Self::Configuration(pane) => pane.header(ui),
            Self::Calculation(pane) => pane.header(ui),
            Self::Composition(pane) => pane.header(ui),
            Self::Visualization(pane) => pane.header(ui),
        }
    }

    fn content(&mut self, ui: &mut Ui) {
        match self {
            Self::Configuration(pane) => pane.content(ui),
            Self::Calculation(pane) => pane.content(ui),
            Self::Composition(pane) => pane.content(ui),
            Self::Visualization(pane) => pane.content(ui),
        }
    }
}

impl From<&Pane> for Kind {
    fn from(value: &Pane) -> Self {
        value.kind()
    }
}

impl PartialEq for Pane {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind()
    }
}

/// Central pane kind
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Kind {
    Configuration,
    Calculation,
    Composition,
    Visualization,
}

pub(crate) mod behavior;
pub(crate) mod calculation;
pub(crate) mod composition;
pub(crate) mod configuration;
pub(crate) mod visualization;
// pub(crate) mod settings;
