use crate::special::acylglycerol::Stereospecificity;
use serde::{Deserialize, Serialize};

pub const MC: Composition = Composition::Mass;
pub const NC: Composition = Composition::Ecn;
pub const UC: Composition = Composition::Unsaturation;
pub const TC: Composition = Composition::Type(None);
pub const PTC: Composition = Composition::Type(Some(Stereospecificity::Positional));
pub const STC: Composition = Composition::Type(Some(Stereospecificity::Stereo));
pub const SC: Composition = Composition::Species(None);
pub const PSC: Composition = Composition::Species(Some(Stereospecificity::Positional));
pub const SSC: Composition = Composition::Species(Some(Stereospecificity::Stereo));

/// Composition
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Composition {
    Ecn,
    Mass,
    Species(Option<Stereospecificity>),
    Type(Option<Stereospecificity>),
    Unsaturation,
}

impl Composition {
    pub const fn new() -> Self {
        SSC
    }
}

impl From<Composition> for Kind {
    fn from(value: Composition) -> Self {
        match value {
            Composition::Ecn => Self::Ecn,
            Composition::Mass => Self::Mass,
            Composition::Species(_) => Self::Species,
            Composition::Type(_) => Self::Type,
            Composition::Unsaturation => Self::Unsaturation,
        }
    }
}

/// Composition kind
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Kind {
    Ecn,
    Mass,
    Species,
    Type,
    Unsaturation,
}
