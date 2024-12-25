use serde::{Deserialize, Serialize};
use std::{
    array::IntoIter,
    fmt::{Display, Formatter, Result, Write},
    hash::Hash,
    ops::{Add, Deref, DerefMut},
    slice::Iter,
};

/// Acylglycerol
#[derive(Clone, Copy, Debug)]
pub enum Acylglycerol<T> {
    Mono(Mag<T>),
    Di(Dag<T>),
    Tri(Tag<T>),
}

// impl<T> Deref for Acylglycerol<T> {
//     type Target = [T];

//     fn deref(&self) -> &Self::Target {
//         match self {
//             Self::Mono(mag) => from_ref(&mag.0),
//             Self::Di(dag) => &dag.0,
//             Self::Tri(tag) => &tag.0,
//         }
//     }
// }

/// Monoacylglycerol
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Mag<T>(pub T);

/// Diacylglycerol
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Dag<T>(pub [T; 2]);

/// Triacylglycerol
#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
pub struct Tag<T>(pub [T; 3]);

impl<T> Tag<T> {
    pub fn compose(mut self, stereospecificity: Option<Stereospecificity>) -> Tag<T>
    where
        T: Ord,
    {
        if let None = stereospecificity {
            self.sort();
        } else if let Some(Stereospecificity::Positional) = stereospecificity {
            if self[0] > self[2] {
                self.swap(0, 2);
            }
        }
        self
    }

    pub fn filter(self, f: impl FnMut(&T) -> bool) -> impl Iterator<Item = T> {
        self.0.into_iter().filter(f)
    }

    pub fn map<U>(self, f: impl FnMut(T) -> U) -> Tag<U> {
        Tag(self.0.map(f))
    }
}

impl<T: Add<Output = T> + Copy> Tag<T> {
    pub fn sum(self) -> T {
        self[0] + self[1] + self[2]
    }
}

impl<T: Display> Display for Tag<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.write_char('[')?;
        Display::fmt(&self.0[0], f)?;
        if f.alternate() {
            f.write_char('|')?;
        }
        Display::fmt(&self.0[1], f)?;
        if f.alternate() {
            f.write_char('|')?;
        }
        Display::fmt(&self.0[2], f)?;
        f.write_char(']')
    }
}

impl<T> Deref for Tag<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Tag<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> IntoIterator for Tag<T> {
    type Item = T;

    type IntoIter = IntoIter<T, 3>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Tag<T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// Stereospecificity
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Stereospecificity {
    Positional,
    Stereo,
}

/// Count
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(super) enum Count {
    Mono,
    Di,
    Tri,
}

impl Display for Count {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Self::Di => f.write_str("di"),
            Self::Mono => f.write_str("mono"),
            Self::Tri => f.write_str("tri"),
        }
    }
}

/// Stereochemical number
#[derive(Clone, Copy, Debug, Hash)]
pub(super) enum Sn {
    One,
    Two,
    Three,
}

impl Sn {
    pub(super) fn text(self) -> &'static str {
        match self {
            Self::One => "1",
            Self::Two => "2",
            Self::Three => "3",
        }
    }
}
