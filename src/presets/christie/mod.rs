use polars::frame::DataFrame;
use std::sync::LazyLock;

pub(crate) static CHRISTIE: LazyLock<DataFrame> = LazyLock::new(|| {
    ron::de::from_str(include_str!("Christie.ron")).expect("deserialize Christie.ron")
});
