use polars::prelude::*;
use std::{
    borrow::{Borrow, Cow},
    fmt::Display,
    iter::Step,
    ops::Range,
};

pub fn r#struct(name: &str) -> StructNameSpace {
    col(name).r#struct()
}

/// Select multiple columns by name and index.
pub fn indexed_cols<T: Display + Step>(name: &str, range: Range<T>) -> Vec<Expr> {
    range.map(|index| col(format!("{name}{index}"))).collect()
}

/// Extension methods for [`DataFrame`]
pub trait DataFrameExt {
    fn destruct(&self, name: &str) -> DataFrame;

    fn try_destruct(&self, name: &str) -> PolarsResult<DataFrame>;

    fn f64(&self, name: &str) -> &Float64Chunked;

    fn list(&self, name: &str) -> &ListChunked;

    fn str(&self, name: &str) -> &StringChunked;

    fn string(&self, name: &str, index: usize) -> Cow<'_, str>;

    fn u8(&self, name: &str) -> &UInt8Chunked;

    fn u32(&self, name: &str) -> &UInt32Chunked;
}

impl DataFrameExt for DataFrame {
    fn f64(&self, name: &str) -> &Float64Chunked {
        self[name].f64().unwrap()
    }

    fn list(&self, name: &str) -> &ListChunked {
        self[name].list().unwrap()
    }

    fn str(&self, name: &str) -> &StringChunked {
        self[name].str().unwrap()
    }

    fn string(&self, name: &str, index: usize) -> Cow<'_, str> {
        self[name].get(index).unwrap().str_value()
    }

    fn u8(&self, name: &str) -> &UInt8Chunked {
        self[name].u8().unwrap()
    }

    fn u32(&self, name: &str) -> &UInt32Chunked {
        self[name].u32().unwrap()
    }

    fn destruct(&self, name: &str) -> DataFrame {
        self.try_destruct(name).unwrap()
    }

    fn try_destruct(&self, name: &str) -> PolarsResult<DataFrame> {
        self.select([PlSmallStr::from_str(name)])?.unnest([name])
    }
}

/// Extension methods for [`Column`]
pub trait ColumnExt {
    fn destruct(&self) -> DataFrame;

    fn r#struct(&self) -> &StructChunked;

    fn try_struct(&self) -> PolarsResult<&StructChunked>;
}

impl ColumnExt for Column {
    fn destruct(&self) -> DataFrame {
        self.r#struct().clone().unnest()
    }

    fn r#struct(&self) -> &StructChunked {
        self.try_struct().unwrap()
    }

    fn try_struct(&self) -> PolarsResult<&StructChunked> {
        self.struct_()
    }
}

/// Extension methods for [`Expr`]
pub trait ExprExt {
    fn normalize(self) -> Expr;

    fn r#struct(self) -> StructNameSpace;

    fn destruct(self, names: impl IntoIterator<Item = impl AsRef<str>>) -> Expr;

    fn suffix(self, suffix: &str) -> Expr;
}

impl ExprExt for Expr {
    fn normalize(self) -> Expr {
        self.apply(
            |series| {
                let chunked_array = series.f64()?;
                Ok(Some(Column::Series(
                    chunked_array
                        .into_iter()
                        .map(|option| Some(option? / chunked_array.sum()?))
                        .collect(),
                )))
            },
            GetOutput::same_type(),
        )
    }

    fn r#struct(self) -> StructNameSpace {
        self.struct_()
    }

    fn destruct(mut self, names: impl IntoIterator<Item = impl AsRef<str>>) -> Expr {
        for name in names {
            self = self.r#struct().field_by_name(name.as_ref());
        }
        self
    }

    fn suffix(self, suffix: &str) -> Expr {
        self.name().suffix(suffix)
    }
}

/// Extension methods for [`Series`]
pub trait SeriesExt {
    fn r#struct(&self) -> &StructChunked;

    fn try_struct(&self) -> PolarsResult<&StructChunked>;

    fn field_by_name(&self, name: &str) -> Series;
}

impl SeriesExt for Series {
    fn r#struct(&self) -> &StructChunked {
        self.try_struct().unwrap()
    }

    fn try_struct(&self) -> PolarsResult<&StructChunked> {
        self.struct_()
    }

    fn field_by_name(&self, name: &str) -> Series {
        self.try_struct().unwrap().field_by_name(name).unwrap()
    }
}

/// Extension methods for [`StructChunked`]
pub trait StructChunkedExt {
    fn field(&self, name: &str) -> Series;

    fn try_field(&self, name: &str) -> PolarsResult<Series>;
}

impl StructChunkedExt for StructChunked {
    fn field(&self, name: &str) -> Series {
        self.try_field(name).unwrap()
    }

    fn try_field(&self, name: &str) -> PolarsResult<Series> {
        self.field_by_name(name)
    }
}
