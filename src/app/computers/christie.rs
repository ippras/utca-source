use polars::prelude::*;

/// Extension methods for [`Expr`]
pub trait ExprExt {
    fn christie(self, christie: bool) -> Expr;
}

impl ExprExt for Expr {
    fn christie(self, christie: bool) -> Expr {
        if christie {
            self * col("Christie")
        } else {
            self
        }
    }
}
