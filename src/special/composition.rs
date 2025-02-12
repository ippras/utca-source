use lipid::triacylglycerol::Stereospecificity;
use serde::{Deserialize, Serialize};

pub const MC: Composition = Composition {
    stereospecificity: None,
    kind: Kind::Mass,
};
pub const PMC: Composition = Composition {
    stereospecificity: Some(Stereospecificity::Positional),
    kind: Kind::Mass,
};
pub const SMC: Composition = Composition {
    stereospecificity: Some(Stereospecificity::Stereo),
    kind: Kind::Mass,
};
pub const ECNC: Composition = Composition {
    stereospecificity: None,
    kind: Kind::Ecn,
};
pub const PECNC: Composition = Composition {
    stereospecificity: Some(Stereospecificity::Positional),
    kind: Kind::Ecn,
};
pub const SECNC: Composition = Composition {
    stereospecificity: Some(Stereospecificity::Stereo),
    kind: Kind::Ecn,
};
pub const SC: Composition = Composition {
    stereospecificity: None,
    kind: Kind::Species,
};
pub const PSC: Composition = Composition {
    stereospecificity: Some(Stereospecificity::Positional),
    kind: Kind::Species,
};
pub const SSC: Composition = Composition {
    stereospecificity: Some(Stereospecificity::Stereo),
    kind: Kind::Species,
};
pub const TC: Composition = Composition {
    stereospecificity: None,
    kind: Kind::Type,
};
pub const PTC: Composition = Composition {
    stereospecificity: Some(Stereospecificity::Positional),
    kind: Kind::Type,
};
pub const STC: Composition = Composition {
    stereospecificity: Some(Stereospecificity::Stereo),
    kind: Kind::Type,
};
pub const UC: Composition = Composition {
    stereospecificity: None,
    kind: Kind::Unsaturation,
};
pub const PUC: Composition = Composition {
    stereospecificity: Some(Stereospecificity::Positional),
    kind: Kind::Unsaturation,
};
pub const SUC: Composition = Composition {
    stereospecificity: Some(Stereospecificity::Stereo),
    kind: Kind::Unsaturation,
};

/// Composition
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Composition {
    pub kind: Kind,
    pub stereospecificity: Option<Stereospecificity>,
}

impl Composition {
    pub const fn new() -> Self {
        Self {
            stereospecificity: Some(Stereospecificity::Positional),
            kind: Kind::Species,
        }
    }
}

impl Default for Composition {
    fn default() -> Self {
        Self::new()
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
