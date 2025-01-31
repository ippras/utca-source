use self::{
    calculation::Pane as CalculationPane, composition::Pane as CompositionPane,
    configuration::Pane as ConfigurationPane,
};
use crate::localize;
use egui::{Response, Sense, Ui, Vec2, vec2};
use metadata::MetaDataFrame;
use serde::{Deserialize, Serialize};

const MARGIN: Vec2 = vec2(4.0, 2.0);

/// Central pane
#[derive(Deserialize, Serialize)]
pub(crate) enum Pane {
    Christie(christie::Pane),
    Configuration(configuration::Pane),
    Calculation(calculation::Pane),
    Composition(composition::Pane),
}

impl Pane {
    pub(crate) fn christie() -> Self {
        Self::Christie(christie::Pane::new())
    }

    pub(crate) fn calculation(frames: Vec<MetaDataFrame>, index: usize) -> Self {
        Self::Calculation(calculation::Pane::new(frames, index))
    }

    pub(crate) fn composition(frames: Vec<MetaDataFrame>, index: Option<usize>) -> Self {
        Self::Composition(composition::Pane::new(frames, index))
    }

    pub(crate) const fn icon(&self) -> &str {
        match self {
            Self::Christie(_) => "",
            Self::Configuration(_) => ConfigurationPane::icon(),
            Self::Calculation(_) => CalculationPane::icon(),
            Self::Composition(_) => CompositionPane::icon(),
        }
    }

    pub(crate) const fn kind(&self) -> Kind {
        match self {
            Self::Christie(_) => Kind::Christie,
            Self::Configuration(_) => Kind::Configuration,
            Self::Calculation(_) => Kind::Calculation,
            Self::Composition(_) => Kind::Composition,
        }
    }

    pub(crate) fn title(&self) -> String {
        match self {
            Self::Christie(_) => localize!("christie"),
            Self::Configuration(pane) => pane.title(),
            Self::Calculation(pane) => pane.title(),
            Self::Composition(pane) => pane.title(),
        }
    }

    // pub(crate) fn hash(&self) -> u64 {
    //     match self {
    //         Self::Christie(pane) => pane.hash(),
    //         Self::Configuration(pane) => pane.hash(),
    //         Self::Calculation(pane) => pane.hash(),
    //         Self::Composition(pane) => pane.hash(),
    //         Self::Visualization(pane) => pane.hash(),
    //     }
    // }

    fn header(&mut self, ui: &mut Ui) -> Response {
        match self {
            Self::Christie(pane) => {
                ui.allocate_response(Default::default(), Sense::hover())
                // pane.header(ui)
            }
            Self::Configuration(pane) => pane.header(ui),
            Self::Calculation(pane) => pane.header(ui),
            Self::Composition(pane) => pane.header(ui),
        }
    }

    fn content(&mut self, ui: &mut Ui) {
        match self {
            Self::Christie(pane) => pane.content(ui),
            Self::Configuration(pane) => pane.body(ui),
            Self::Calculation(pane) => pane.body(ui),
            Self::Composition(pane) => pane.body(ui),
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

/// Pane delegate
pub(crate) trait PaneDelegate {
    fn header(&mut self, ui: &mut Ui) -> Response;

    fn body(&mut self, ui: &mut Ui);
}

/// Central pane kind
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Kind {
    Christie,
    Configuration,
    Calculation,
    Composition,
}

pub(crate) mod behavior;
pub(crate) mod calculation;
pub(crate) mod christie;
pub(crate) mod composition;
pub(crate) mod configuration;
