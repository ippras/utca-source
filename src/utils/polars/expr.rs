use super::{column, hash, normalize};
use polars::prelude::*;

/// Extension methods for [`Expr`]
pub trait ExprExt {
    fn clip_min_if(self, clip: bool) -> Expr;

    fn destruct(self, names: impl IntoIterator<Item = impl AsRef<str>>) -> Expr;

    fn hash(self) -> Expr;

    fn normalize(self) -> Expr;

    fn normalize_if(self, normalize: bool) -> Expr;
}

impl ExprExt for Expr {
    fn clip_min_if(self, clip: bool) -> Expr {
        if clip { self.clip_min(lit(0)) } else { self }
    }

    fn destruct(mut self, names: impl IntoIterator<Item = impl AsRef<str>>) -> Expr {
        for name in names {
            self = self.struct_().field_by_name(name.as_ref());
        }
        self
    }

    /// Hash column, type [`u64`], name "Hash"
    fn hash(self) -> Expr {
        self.apply(column(hash), GetOutput::from_type(DataType::UInt64))
            .alias("Hash")
    }

    /// Normalize column, type [`f64`], the same name
    /// TODO all into float types
    fn normalize(self) -> Expr {
        // self.apply(
        //     |series| {
        //         let chunked_array = series.f64()?;
        //         let sum = chunked_array.sum();
        //         Ok(Some(
        //             chunked_array
        //                 .iter()
        //                 .map(|option| Some(option.unwrap_or_default() / sum?))
        //                 .collect::<Float64Chunked>()
        //                 .into_column(),
        //         ))
        //     },
        //     GetOutput::same_type(),
        // )
        self.apply(column(normalize), GetOutput::same_type())
    }

    fn normalize_if(self, normalize: bool) -> Expr {
        if normalize { self.normalize() } else { self }
    }
}
