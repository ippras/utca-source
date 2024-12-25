use self::{index::Notation, isomerism::Elision};
use crate::r#const::relative_atomic_mass::{C, H, O};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt::{self, Formatter},
};

pub const ID: Options = Options {
    separators: Separators {
        c: "c",
        u: ["a", "b"],
        i: ["", ""],
    },
    notation: Notation::Prefix,
    elision: Elision::Explicit,
};

pub const COMMON: Options = Options {
    separators: Separators {
        c: "",
        u: [":", ":"],
        i: ["Δ", ","],
    },
    notation: Notation::Suffix,
    elision: Elision::Implicit,
};

// 18d1c9d2c12c15
pub macro fatty_acid($c:expr $(; $($i:expr),*)*) {{
    assert!($c > 0);
    #[allow(unused_mut)]
    let mut fatty_acid = Folded::new(vec![0; $c - 1]);
    let mut count = 0;
    $(
        count += 1;
        $(
            assert!($i != 0);
            assert!($i < $c);
            let r#i8 = ($i as i8);
            let abs = r#i8.abs();
            let signum = r#i8.signum();
            fatty_acid.0[(abs - 1) as usize] = signum * count;
        )*
    )*
    fatty_acid
}}

/// Fatty acid
pub trait FattyAcid {
    /// Carbon
    fn c(&self) -> u8 {
        self.b() + 1
    }

    /// Hydrogen
    ///
    /// `H = 2C - 2U`
    fn h(&self) -> u8 {
        2 * self.c() - 2 * self.u()
    }

    /// Fatty acid ECN (Equivalent carbon number)
    ///
    /// `ECN = C - 2U`
    fn ecn(&self) -> u8 {
        self.c() - 2 * self.u()
    }

    /// Mass
    fn mass(&self) -> f64 {
        self.c() as f64 * C + self.h() as f64 * H + 2. * O
    }

    /// Saturated
    fn s(&self) -> bool {
        self.u() == 0
    }

    /// Bounds
    fn b(&self) -> u8;

    /// Unsaturated bounds
    fn u(&self) -> u8;
}

impl FattyAcid for Folded {
    fn b(&self) -> u8 {
        self.0.len() as _
    }

    fn u(&self) -> u8 {
        self.unfold()
            .unsaturations
            .keys()
            .fold(0, |sum, unsaturation| match unsaturation {
                Unsaturation::One => sum + 1,
                Unsaturation::Two => sum + 2,
            })
    }
}

/// Folded
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Folded(Vec<i8>);

impl Folded {
    pub const fn new(bounds: Vec<i8>) -> Self {
        Self(bounds)
    }

    /// Unfold
    fn unfold(&self) -> Unfolded {
        let carbons = self.0.len() as u8 + 1;
        let mut unsaturations = BTreeMap::new();
        for (index, &bound) in self.0.iter().enumerate() {
            let (unsaturation, isomerism) = match bound {
                -2 => (Unsaturation::Two, Isomerism::Trans),
                -1 => (Unsaturation::One, Isomerism::Trans),
                0 => continue,
                1 => (Unsaturation::One, Isomerism::Cis),
                2 => (Unsaturation::Two, Isomerism::Cis),
                _ => unreachable!(),
            };
            unsaturations
                .entry(unsaturation)
                .or_insert(BTreeMap::new())
                .insert(index, isomerism);
        }
        Unfolded {
            carbons,
            unsaturations,
        }
    }
}

/// Unfolded
#[derive(Clone, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Unfolded {
    pub carbons: u8,
    pub unsaturations: BTreeMap<Unsaturation, BTreeMap<usize, Isomerism>>,
}

/// Isomerism
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Isomerism {
    Cis,
    Trans,
}

/// Unsaturation
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Unsaturation {
    One,
    Two,
}

/// Display with options
pub trait DisplayWithOptions {
    fn display(self, options: Options) -> Display<Self>
    where
        Self: Sized + FattyAcid;
}

impl<T: FattyAcid> DisplayWithOptions for T {
    fn display(self, options: Options) -> Display<T> {
        Display::new(self, options)
    }
}

/// Fatty acid display
#[derive(Clone, Debug)]
pub struct Display<T> {
    fatty_acid: T,
    options: Options,
}

impl<T> Display<T> {
    pub fn new(fatty_acid: T, options: Options) -> Self {
        Self {
            fatty_acid,
            options,
        }
    }
}

impl fmt::Display for Display<Folded> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        fmt::Display::fmt(&Display::new(&self.fatty_acid.unfold(), self.options), f)
    }
}

impl fmt::Display for Display<&Folded> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        fmt::Display::fmt(&Display::new(&self.fatty_acid.unfold(), self.options), f)
    }
}

impl fmt::Display for Display<&Unfolded> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self.options.separators.c)?;
        fmt::Display::fmt(&self.fatty_acid.carbons, f)?;
        let double = &self.fatty_acid.unsaturations.get(&Unsaturation::One);
        if let Some(indices) = double {
            f.write_str(self.options.separators.u[0])?;
            fmt::Display::fmt(&indices.len(), f)?;
        }
        let triple = &self.fatty_acid.unsaturations.get(&Unsaturation::Two);
        if let Some(indices) = triple {
            if double.is_none() {
                f.write_str(self.options.separators.u[0])?;
            }
            f.write_str(self.options.separators.u[1])?;
            fmt::Display::fmt(&indices.len(), f)?;
        }
        if f.alternate() {
            f.write_str(self.options.separators.i[0])?;
            let mut indices = self.fatty_acid.unsaturations.values().flatten();
            if let Some((index, &isomerism)) = indices.next() {
                fmt::Display::fmt(
                    &index::Display::new(
                        index + 1,
                        isomerism::Display::new(isomerism, self.options.elision),
                        self.options.notation,
                    ),
                    f,
                )?;
                for (index, &isomerism) in indices {
                    f.write_str(self.options.separators.i[1])?;
                    fmt::Display::fmt(
                        &index::Display::new(
                            index + 1,
                            isomerism::Display::new(isomerism, self.options.elision),
                            self.options.notation,
                        ),
                        f,
                    )?;
                }
            }
        }
        Ok(())
    }
}

/// Display options
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Options {
    pub separators: Separators,
    pub notation: Notation,
    pub elision: Elision,
}

/// Separators
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Separators {
    pub c: &'static str,
    pub u: [&'static str; 2],
    pub i: [&'static str; 2],
}

impl Separators {
    fn u(&self, unsaturation: Unsaturation) -> &str {
        match unsaturation {
            Unsaturation::One => self.u[0],
            Unsaturation::Two => self.u[1],
        }
    }
}

mod index {
    use super::isomerism;
    use serde::{Deserialize, Serialize};
    use std::fmt::{self, Formatter};

    /// Index display
    pub(super) struct Display {
        index: usize,
        isomerism: isomerism::Display,
        notation: Notation,
    }

    impl Display {
        pub(super) fn new(index: usize, isomerism: isomerism::Display, notation: Notation) -> Self {
            Self {
                index,
                isomerism,
                notation,
            }
        }
    }

    impl fmt::Display for Display {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            match self.notation {
                Notation::Prefix => {
                    fmt::Display::fmt(&self.isomerism, f)?;
                    fmt::Display::fmt(&self.index, f)
                }
                Notation::Suffix => {
                    fmt::Display::fmt(&self.index, f)?;
                    fmt::Display::fmt(&self.isomerism, f)
                }
            }
        }
    }

    /// Isomerism notation
    #[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
    pub enum Notation {
        Prefix,
        Suffix,
    }
}

// C:D:TΔI,I,I
mod isomerism {
    use super::Isomerism;
    use serde::{Deserialize, Serialize};
    use std::fmt::{self, Formatter, Write};

    /// Display isomerism
    pub(super) struct Display {
        pub(super) isomerism: Isomerism,
        pub(super) elision: Elision,
    }

    impl Display {
        pub(super) fn new(isomerism: Isomerism, elision: Elision) -> Self {
            Self { isomerism, elision }
        }
    }

    impl fmt::Display for Display {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            match self.isomerism {
                Isomerism::Cis => {
                    if self.elision == Elision::Explicit {
                        f.write_char('c')?;
                    }
                }
                Isomerism::Trans => {
                    f.write_char('t')?;
                }
            }
            Ok(())
        }
    }

    /// Isomerism elision
    #[derive(
        Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize,
    )]
    pub enum Elision {
        Explicit,
        #[default]
        Implicit,
    }
}

#[cfg(test)]
mod test1 {
    use super::*;

    #[test]
    fn temp() {
        let fatty_acid = fatty_acid!(16;;9,12,15);
        println!("fa: {}", fatty_acid.display(COMMON));
        let fatty_acid = fatty_acid!(16;9,12;15);
        println!("fa: {:#}", fatty_acid.display(COMMON));
        let fatty_acid = fatty_acid!(16;9,12,-15);
        println!("fa: {:#}", fatty_acid.display(COMMON));
        let fatty_acid = fatty_acid!(16;9,12;15);
        println!("fa: {:#}", fatty_acid.display(ID));
        let fatty_acid = fatty_acid!(16;9,12,-15);
        println!("fa: {:#}", fatty_acid.display(ID));
        // println!("fa: {:?}", fatty_acid.indices().collect::<Vec<_>>());
        // let fa = TempFattyAcid::new(vec![1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, -2, 1, 1, 1, 1]);
        // println!("fa: {:?}", fa.indices().collect::<Vec<_>>());
    }

    // #[test]
    // fn test() {
    //     // let s = "00000000100100100";
    //     // 0 - COOH
    //     // 1 - C-COOH
    //     // 2 - C=COOH
    //     let s = "11211211211111111";
    //     let len = s.len();
    //     let t = BigUint::from_str_radix(s, RADIX).unwrap();
    //     println!("t: {:x}", t);
    //     println!("t: {:x?}", t.to_radix_le(RADIX));
    //     let le = t.to_radix_le(RADIX);
    //     let doubles = le.iter().filter(|&n| n / 2 == 1).count();
    //     println!("doubles: {doubles}");
    //     let triples = le.iter().filter(|&n| n / 2 == 2).count();
    //     println!("triples: {triples}");
    //     println!("t: {:0>len$}", t.to_str_radix(RADIX));

    //     // for i in Iter::new(0b_1000_0000_1100) {
    //     //     println!("i: {i}");
    //     // }
    //     // for i in Iter::new(u64::MAX) {
    //     //     println!("i: {i}");
    //     // }

    //     // 18d1c9d2c12c15
    //     // 18d01c09
    //     // 18d01c12
    //     // 18d02c09c12
    //     // 18d02c09c12
    //     // 18d1c9d2c12c15
    //     println!("{:?}", "18:02:01-".cmp("18:02-"));
    //     println!("{:?}", "18:02:01".cmp("18:02:01"));
    //     println!("{:?}", "18:02-09,12".cmp("18:02-12,15"));

    //     let zero = NewFattyAcid { layers: [0, 0] };
    //     println!("c: {}", zero.c());
    //     println!("u: {}", zero.u());
    //     println!("d: {}", zero.doubles().count_ones());
    //     println!("t: {}", zero.triples().count_ones());

    //     let saturated = NewFattyAcid::saturated(17);
    //     println!(
    //         "saturated: {:b} {:b}",
    //         saturated.layers[0], saturated.layers[1]
    //     );
    //     let unsaturated = saturated.d(8).d(11).t(14);
    //     // {value:.*}
    //     println!(
    //         "unsaturated: {:0w$b}\n           : {:0w$b}",
    //         unsaturated.layers[0],
    //         unsaturated.layers[1],
    //         w = unsaturated.c() as _,
    //     );

    //     let mut layers = [(1 << 17) - 1, 0];
    //     let index = 8;
    //     layers[0] ^= 1 << index;
    //     layers[1] ^= 1 << index;
    //     let g = NewFattyAcid { layers };
    //     println!("layers: {:b} {:b}", g.layers[0], g.layers[1]);
    //     println!("c: {}", g.c());
    //     println!("u: {}", g.u());
    //     println!("d: {}", g.doubles().count_ones());
    //     println!("t: {}", g.triples().count_ones());
    //     for index in Iter::new(g.doubles()) {
    //         let index = index + 1;
    //         println!("index: {index}");
    //     }
    // }
}

// /// Isomerism kind
// #[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
// pub enum Kind {
//     #[default]
//     CisTrans,
//     PlusMinus,
// }

// /// Elision
// #[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
// pub enum Elision {
//     Explicit,
//     #[default]
//     Implicit,
// }

// /// Bound
// #[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
// enum Bound {
//     #[default]
//     Single,
//     Double(Isomerism),
//     Triple(Isomerism),
// }

// /// Bound
// #[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
// pub struct Bound(i8);
// impl Display for Bound {
//     fn fmt(&self, f: &mut Formatter) -> fmt::Result {
//         let value = self.0.abs();
//         let isomerism = if self.0 < 0 { "t" } else { "c" };
//         write!(f, "{value}{isomerism}")
//     }
// }

// /// Bound
// #[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
// pub enum Bound {
//     #[default]
//     Single = 0,
//     Double = 1,
//     Triple = 2,
// }

// impl Bound {
//     fn n(n: i8) -> Self {
//         Self { n, index: 0 }
//     }
// }

// impl Display for Bound {
//     fn fmt(&self, f: &mut Formatter) -> fmt::Result {
//         match self {
//             Self::Single => f.write_str("1"),
//             Self::Double => f.write_str("2"),
//             Self::Triple => f.write_str("3"),
//         }
//     }
// }

// impl TryFrom<u8> for Bound {
//     type Error = u8;
//     fn try_from(value: u8) -> Result<Self, Self::Error> {
//         match value {
//             1 => Ok(Self::Single),
//             2 => Ok(Self::Double),
//             3 => Ok(Self::Triple),
//             value => Err(value),
//         }
//     }
// }
