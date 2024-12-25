pub(crate) mod atoms {
    use molecule::atom::isotopes::*;

    pub(crate) const C: C = C::Twelve;
    pub(crate) const H: H = H::One;
    pub(crate) const O: O = O::Sixteen;
}

#[rustfmt::skip]
pub(crate) mod relative_atomic_mass {
    use molecule::atom::isotopes::*;

    // Twelve
    // Thirteen
    // Fourteen
    pub(crate) const C: f64 = C::Twelve.relative_atomic_mass().value;
    // One
    // Two
    // Three
    pub(crate) const H: f64 = H::One.relative_atomic_mass().value;
    pub(crate) const LI: f64 = Li::Seven.relative_atomic_mass().value;
    // Fourteen
    // Fifteen
    pub(crate) const N: f64 = N::Fourteen.relative_atomic_mass().value;
    pub(crate) const NA: f64 = Na.relative_atomic_mass().value;
    pub(crate) const O: f64 = O::Sixteen.relative_atomic_mass().value;

    pub(crate) const CH2: f64 = C + 2.0 * H;
    pub(crate) const NH4: f64 = N + 4.0 * H;
}

// pub(crate) const OLEIC: Counter = counter! {
//     C::Twelve => 18,
//     H::One => 34,
// };
// pub(crate) const ELAIDIC: Counter = counter! {
//     C::Twelve => 18,
//     H::One => 34,
// };
// pub(crate) const LINOLEIC: Counter = counter! {
//     C::Twelve => 18,
//     H::One => 32,
// };

pub const R: f64 = 8.314_462_618_153_24;

// Triglyceride Property Calculator
// <https://lipidlibrary.shinyapps.io/Triglyceride_Property_Calculator>
pub mod polymorphism {
    pub mod alpha {
        pub const H: f64 = 2.7;
        pub const H0: f64 = -31.95;
        pub const H_XY: f64 = -13.28;

        pub const S: f64 = 6.79;
        pub const S0: f64 = -19.09;
        pub const S_XY: f64 = -36.7;

        pub const K: f64 = 4.39;
        pub const K_X: f64 = K;
        pub const K_Y: f64 = K;

        pub const X0: f64 = 1.25;
    }

    pub mod beta_prime {
        pub const H: f64 = 3.86;
        pub const H0: f64 = -35.86;
        pub const H_XY: f64 = -19.35;

        pub const S: f64 = 10.13;
        pub const S0: f64 = -39.59;
        pub const S_XY: f64 = -52.51;

        pub const K: f64 = 1.99;
        pub const K_X: f64 = K;
        pub const K_Y: f64 = K;

        pub const X0: f64 = 2.46;
    }

    pub mod beta {
        pub const H: f64 = 3.89;
        pub const H0: f64 = -17.16;
        pub const H_XY: f64 = -22.29;
        pub const H_ODD: f64 = 2.29;

        pub const S: f64 = 9.83;
        pub const S0: f64 = 31.04;
        pub const S_XY: f64 = -64.58;

        pub const K: f64 = 2.88;
        pub const K_X: f64 = K;
        pub const K_Y: f64 = K;

        pub const X0: f64 = 0.77;
    }
}
