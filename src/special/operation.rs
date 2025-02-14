use lipid::triacylglycerol::Stereospecificity;
use serde::{Deserialize, Serialize};

pub const MA: Union = Union {
    kind: Kind::Mass,
    operation: Operation::Agregation,
};
pub const MC: Union = Union {
    kind: Kind::Mass,
    operation: Operation::Composition {
        stereospecificity: None,
    },
};
pub const PMC: Union = Union {
    kind: Kind::Mass,
    operation: Operation::Composition {
        stereospecificity: Some(Stereospecificity::Positional),
    },
};
pub const SMC: Union = Union {
    kind: Kind::Mass,
    operation: Operation::Composition {
        stereospecificity: Some(Stereospecificity::Stereo),
    },
};
pub const EA: Union = Union {
    kind: Kind::Ecn,
    operation: Operation::Composition {
        stereospecificity: None,
    },
};
pub const EC: Union = Union {
    kind: Kind::Ecn,
    operation: Operation::Composition {
        stereospecificity: None,
    },
};
pub const PEC: Union = Union {
    kind: Kind::Ecn,
    operation: Operation::Composition {
        stereospecificity: Some(Stereospecificity::Positional),
    },
};
pub const SEC: Union = Union {
    kind: Kind::Ecn,
    operation: Operation::Composition {
        stereospecificity: Some(Stereospecificity::Stereo),
    },
};
pub const SC: Union = Union {
    kind: Kind::Species,
    operation: Operation::Composition {
        stereospecificity: None,
    },
};
pub const PSC: Union = Union {
    kind: Kind::Species,
    operation: Operation::Composition {
        stereospecificity: Some(Stereospecificity::Positional),
    },
};
pub const SSC: Union = Union {
    kind: Kind::Species,
    operation: Operation::Composition {
        stereospecificity: Some(Stereospecificity::Stereo),
    },
};
pub const TC: Union = Union {
    kind: Kind::Type,
    operation: Operation::Composition {
        stereospecificity: None,
    },
};
pub const PTC: Union = Union {
    kind: Kind::Type,
    operation: Operation::Composition {
        stereospecificity: Some(Stereospecificity::Positional),
    },
};
pub const STC: Union = Union {
    kind: Kind::Type,
    operation: Operation::Composition {
        stereospecificity: Some(Stereospecificity::Stereo),
    },
};
pub const UC: Union = Union {
    kind: Kind::Unsaturation,
    operation: Operation::Composition {
        stereospecificity: None,
    },
};
pub const PUC: Union = Union {
    kind: Kind::Unsaturation,
    operation: Operation::Composition {
        stereospecificity: Some(Stereospecificity::Positional),
    },
};
pub const SUC: Union = Union {
    kind: Kind::Unsaturation,
    operation: Operation::Composition {
        stereospecificity: Some(Stereospecificity::Stereo),
    },
};

// pub const UA: Union = Union {
//     kind: Kind::Unsaturation,
//     operation: Operation::Agregation,
// };

/// Union
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Union {
    pub kind: Kind,
    pub operation: Operation,
}

// /// Operation
// #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
// pub enum Operation {
//     Composition {
//         stereospecificity: Option<Stereospecificity>,
//     },
//     Agregation,
// }

/// Composition kind
///
/// * `None` stereospecific composition
/// * `Some(Permutation{ positional: true })`: positional specific permutation
/// * `Some(Permutation{ positional: false })`: non-stereospecific permutation
/// * `Some(Agregation)`: non-stereospecific agregation
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Composition {
    Ecn(Operation<Agregation>),
    Mass(Operation<Agregation>),
    Unsaturation(Operation<Agregation>),
    Species(Operation<Permutation>),
    Type(Operation<Permutation>),
}

/// Operation
///
/// * `None` the operation is not used, stereospecific composition
/// * `Some(Permutation{ positional: true })`: positional specific permutation
/// * `Some(Permutation{ positional: false })`: non-stereospecific permutation
/// * `Some(Agregation)`: non-stereospecific agregation
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Operation<T>(Option<T>);

/// Agregation
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Agregation;

/// Permutation
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Permutation {
    pub positional: bool,
}

// Composition
// 1. Ecn(Option<Agregation>)
// • EA
// 2. Mass(Option<Agregation>)
// • MA
// 3. Unsaturation(Option<Agregation>)
// • UA
// 4. Species(Permutation)
// • SC
// • PSC
// • SSC
// 5. Type(Permutation)
// • TC
// • PTC
// • STC

// type Permutation = Option<Stereospecificity>;
// Operation:
// Permutation {
//     stereospecificity: Option<Stereospecificity>,
// }
// * Agregation
