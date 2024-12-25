use self::{index::Notation, isomerism::Elision};
use crate::r#const::relative_atomic_mass::{C, H, O};
use indexmap::IndexMap;
use itertools::{Either, Itertools};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Formatter},
    ops::Neg,
};

pub const ID: Options = Options {
    separators: Separators {
        c: "c",
        u: "u",
        i: ["", ""],
    },
    notation: Notation::Prefix,
    elision: Elision::Explicit,
};

pub const COMMON: Options = Options {
    separators: Separators {
        c: "",
        u: ":",
        i: ["Δ", ","],
    },
    notation: Notation::Suffix,
    elision: Elision::Implicit,
};

pub macro fatty_acid($c:expr $(; $($i:expr),*)*) {{
    assert!($c > 0);
    #[allow(unused_mut)]
    let mut fatty_acid = Folded::new(vec![0; $c - 1]);
    let mut _count = 0;
    $(
        _count += 1;
        $(
            assert!($i != 0);
            assert!($i < $c);
            let r#i8 = ($i as i8);
            let abs = r#i8.abs();
            let signum = r#i8.signum();
            fatty_acid.0[(abs - 1) as usize] = signum * _count;
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
            .indices
            .values()
            .fold(0, |sum, (unsaturation, _)| match unsaturation {
                Unsaturation::One => sum + 1,
                Unsaturation::Two => sum + 2,
            })
    }
}

impl FattyAcid for &Unfolded {
    fn b(&self) -> u8 {
        self.carbons.saturating_sub(1)
    }

    fn u(&self) -> u8 {
        self.indices
            .values()
            .fold(0, |sum, (unsaturation, _)| match unsaturation {
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
        let mut indices = IndexMap::new();
        for (index, &bound) in self.0.iter().enumerate() {
            let (unsaturation, isomerism) = match bound {
                -2 => (Unsaturation::Two, Isomerism::Trans),
                -1 => (Unsaturation::One, Isomerism::Trans),
                0 => continue,
                1 => (Unsaturation::One, Isomerism::Cis),
                2 => (Unsaturation::Two, Isomerism::Cis),
                _ => unreachable!(),
            };
            indices.insert(index, (unsaturation, isomerism));
        }
        indices.sort_by_cached_key(|_, (unsaturation, _)| *unsaturation);
        Unfolded { carbons, indices }
    }
}

/// Unfolded
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Unfolded {
    pub carbons: u8,
    pub indices: IndexMap<usize, (Unsaturation, Isomerism)>,
}

impl Unfolded {
    /// Fold
    fn fold(&self) -> Folded {
        let mut bounds = vec![0, self.carbons as _];
        for (index, bound) in bounds.iter_mut().enumerate() {
            if let Some(&(unsaturation, isomerism)) = self.indices.get(&index) {
                match unsaturation {
                    Unsaturation::One => *bound = 1,
                    Unsaturation::Two => *bound = 2,
                }
                if isomerism == Isomerism::Trans {
                    *bound = -*bound;
                }
            }
        }
        Folded(bounds)
    }
}

/// Isomerism
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
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
        let point = self
            .fatty_acid
            .indices
            .partition_point(|_, &(unsaturation, _)| unsaturation == Unsaturation::One);
        let doubles = &self.fatty_acid.indices.as_slice()[..point];
        let triples = &self.fatty_acid.indices.as_slice()[point..];
        f.write_str(self.options.separators.u)?;
        fmt::Display::fmt(&doubles.len(), f)?;
        if !triples.is_empty() {
            f.write_str(self.options.separators.u)?;
            fmt::Display::fmt(&triples.len(), f)?;
        }
        if f.alternate() {
            let mut indices = doubles.into_iter().chain(triples);
            if let Some((index, &(_, isomerism))) = indices.next() {
                f.write_str(self.options.separators.i[0])?;
                fmt::Display::fmt(
                    &index::Display::new(
                        index + 1,
                        isomerism::Display::new(isomerism, self.options.elision),
                        self.options.notation,
                    ),
                    f,
                )?;
                for (index, &(_, isomerism)) in indices {
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
    pub u: &'static str,
    pub i: [&'static str; 2],
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
mod test {
    use super::*;

    // #[test]
    // fn isomerism() {
    //     // 3
    //     assert_eq!(
    //         fatty_acid!(18;-9,12,15)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-9t12c15c",
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;9,-12,15)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-9c12t15c",
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;9,12,-15)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-9c12c15t",
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;-9,-12,15)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-9t12t15c",
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;9,-12,-15)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-9c12t15t",
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;-9,12,-15)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-9t12c15t",
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;-9,-12,-15)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-9t12t15t",
    //     );
    //     // 2:1
    //     assert_eq!(
    //         fatty_acid!(18;12,15;-9)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-12c15c-9t",
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;9,15;-12)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-9c15c-12t",
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;9,12;-15)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-9c12c-15t",
    //     );
    //     // 1:2
    // }

    // #[test]
    // fn order() {
    //     // 3
    //     assert_eq!(
    //         fatty_acid!(18;9,12,15)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-9c12c15c",
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;9,15,12)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-9c12c15c",
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;12,9,15)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-9c12c15c",
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;12,15,9)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-9c12c15c",
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;15,9,12)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-9c12c15c",
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;15,12,9)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-9c12c15c",
    //     );
    //     // 2:1
    //     assert_eq!(
    //         fatty_acid!(18;12,15;9)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-12c15c-9c",
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;15,12;9)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-12c15c-9c",
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;9,15;12)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-9c15c-12c",
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;15,9;12)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-9c15c-12c",
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;9,12;15)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-9c12c-15c",
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;12,9;15)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-9c12c-15c",
    //     );
    //     // 1:2
    //     assert_eq!(
    //         fatty_acid!(18;9;12,15)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-9c-12c15c",
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;9;15,12)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-9c-12c15c",
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;12;9,15)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-12c-9c15c",
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;12;15,9)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-12c-9c15c",
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;15;9,12)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-15c-9c12c",
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;15;12,9)
    //             .display(Kind::ColonMinus)
    //             .to_string(),
    //         "18-15c-9c12c",
    //     );
    // }

    // #[test]
    // fn macros() {
    //     // 0
    //     assert_eq!(fatty_acid!(18), new(vec![0; 17]));
    //     // 1
    //     assert_eq!(
    //         fatty_acid!(18;9),
    //         FattyAcid::new(vec![0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]),
    //     );
    //     // 2
    //     assert_eq!(
    //         fatty_acid!(18;9,12),
    //         FattyAcid::new(vec![0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0]),
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;9;12),
    //         FattyAcid::new(vec![0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 2, 0, 0, 0, 0, 0]),
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;;9,12),
    //         FattyAcid::new(vec![0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 2, 0, 0, 0, 0, 0]),
    //     );
    //     // 3
    //     assert_eq!(
    //         fatty_acid!(18;9,12,15),
    //         FattyAcid::new(vec![0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0]),
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;9,12;15),
    //         FattyAcid::new(vec![0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 2, 0, 0]),
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;9;12,15),
    //         FattyAcid::new(vec![0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 2, 0, 0, 2, 0, 0]),
    //     );
    //     assert_eq!(
    //         fatty_acid!(18;;9,12,15),
    //         FattyAcid::new(vec![0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 2, 0, 0, 2, 0, 0]),
    //     );
    // }

    mod errors {
        use super::*;

        #[test]
        #[should_panic(expected = "assertion failed: 0 > 0")]
        fn zero_carbons() {
            fatty_acid!(0);
        }

        #[test]
        #[should_panic(expected = "assertion failed: 0 != 0")]
        fn zero_index() {
            fatty_acid!(18;0);
        }

        #[test]
        #[should_panic(expected = "assertion failed: 18 < 18")]
        fn equal_carbons() {
            fatty_acid!(18;18);
        }

        #[test]
        #[should_panic(expected = "assertion failed: 19 < 18")]
        fn greater_carbons() {
            fatty_acid!(18;19);
        }
    }

    #[test]
    fn common() {
        let fatty_acid = fatty_acid!(18).display(COMMON);
        assert_eq!(fatty_acid.to_string(), "18:0");
        assert_eq!(format!("{fatty_acid:02}"), "18:00");
        assert_eq!(format!("{fatty_acid:#}"), "18:0");
        assert_eq!(format!("{fatty_acid:#02}"), "18:00");
        let fatty_acid = &fatty_acid!(18;9).display(COMMON);
        assert_eq!(fatty_acid.to_string(), "18:1");
        assert_eq!(format!("{fatty_acid:02}"), "18:01");
        assert_eq!(format!("{fatty_acid:#}"), "18:1Δ9");
        assert_eq!(format!("{fatty_acid:#02}"), "18:01Δ09");
        let fatty_acid = fatty_acid!(18;9,12).display(COMMON);
        assert_eq!(fatty_acid.to_string(), "18:2");
        assert_eq!(format!("{fatty_acid:02}"), "18:02");
        assert_eq!(format!("{fatty_acid:#}"), "18:2Δ9,12");
        assert_eq!(format!("{fatty_acid:#02}"), "18:02Δ09,12");
        // Triple
        let fatty_acid = fatty_acid!(18;9;12).display(COMMON);
        assert_eq!(fatty_acid.to_string(), "18:1:1");
        assert_eq!(format!("{fatty_acid:02}"), "18:01:01");
        assert_eq!(format!("{fatty_acid:#}"), "18:1:1Δ9,12");
        assert_eq!(format!("{fatty_acid:#02}"), "18:01:01Δ09,12");
        // Isomerism
        let fatty_acid = fatty_acid!(18;-9,-12,-15).display(COMMON);
        assert_eq!(fatty_acid.to_string(), "18:3");
        assert_eq!(format!("{fatty_acid:02}"), "18:03");
        assert_eq!(format!("{fatty_acid:#}"), "18:3Δ9t,12t,15t");
        assert_eq!(format!("{fatty_acid:#02}"), "18:03Δ09t,12t,15t");
    }

    #[test]
    fn id() {
        let fatty_acid = fatty_acid!(18).display(ID);
        assert_eq!(fatty_acid.to_string(), "c18u0");
        assert_eq!(format!("{fatty_acid:02}"), "c18u00");
        assert_eq!(format!("{fatty_acid:#}"), "c18u0");
        assert_eq!(format!("{fatty_acid:#02}"), "c18u00");
        let fatty_acid = fatty_acid!(18;9).display(ID);
        assert_eq!(fatty_acid.to_string(), "c18u1");
        assert_eq!(format!("{fatty_acid:02}"), "c18u01");
        assert_eq!(format!("{fatty_acid:#}"), "c18u1c9");
        assert_eq!(format!("{fatty_acid:#02}"), "c18u01c09");
        let fatty_acid = fatty_acid!(18;9,12).display(ID);
        assert_eq!(fatty_acid.to_string(), "c18u2");
        assert_eq!(format!("{fatty_acid:02}"), "c18u02");
        assert_eq!(format!("{fatty_acid:#}"), "c18u2c9c12");
        assert_eq!(format!("{fatty_acid:#02}"), "c18u02c09c12");
        // Triple
        let fatty_acid = fatty_acid!(18;9;12).display(ID);
        assert_eq!(fatty_acid.to_string(), "c18u1u1");
        assert_eq!(format!("{fatty_acid:02}"), "c18u01u01");
        assert_eq!(format!("{fatty_acid:#}"), "c18u1u1c9c12");
        assert_eq!(format!("{fatty_acid:#02}"), "c18u01u01c09c12");
        // Isomerism
        let fatty_acid = fatty_acid!(18;-9,-12,-15).display(ID);
        assert_eq!(fatty_acid.to_string(), "c18u3");
        assert_eq!(format!("{fatty_acid:02}"), "c18u03");
        assert_eq!(format!("{fatty_acid:#}"), "c18u3t9t12t15");
        assert_eq!(format!("{fatty_acid:#02}"), "c18u03t09t12t15");
    }
}
