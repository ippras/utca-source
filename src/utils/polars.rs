use polars::prelude::*;
use std::borrow::Cow;

pub fn destruct(names: impl IntoIterator<Item = impl AsRef<str>>) -> Expr {
    let mut names = names.into_iter();
    let Some(name) = names.next() else {
        panic!("destruct names is empty");
    };
    let mut expr = col(PlSmallStr::from(name.as_ref()));
    for name in names {
        expr = expr.struct_().field_by_name(name.as_ref());
    }
    expr
}

/// Extension methods for [`DataFrame`]
pub trait DataFrameExt {
    fn destructs(&self, names: impl IntoIterator<Item = impl AsRef<str>>) -> DataFrame;

    fn try_destructs(
        &self,
        names: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> PolarsResult<DataFrame>;

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

    fn destructs(&self, names: impl IntoIterator<Item = impl AsRef<str>>) -> DataFrame {
        self.try_destructs(names).unwrap()
    }

    fn try_destructs(
        &self,
        names: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> PolarsResult<DataFrame> {
        let mut names = names.into_iter();
        if let Some(name) = names.next() {
            self.try_destructs([name.as_ref()])?.try_destructs(names)
        } else {
            Ok(self.clone())
        }
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
    // fn destruct(&self) -> DataFrame;
}

impl ColumnExt for Column {
    // fn destruct(&self) -> DataFrame {
    //     self.struct_().clone().unnest()
    // }
}

/// Extension methods for [`Expr`]
pub trait ExprExt {
    fn clip_min_if(self, clip: bool) -> Expr;

    fn normalize(self) -> Expr;

    fn normalize_if(self, normalize: bool) -> Expr;

    fn destruct(self, names: impl IntoIterator<Item = impl AsRef<str>>) -> Expr;

    fn suffix(self, suffix: &str) -> Expr;
}

impl ExprExt for Expr {
    fn clip_min_if(self, clip: bool) -> Expr {
        if clip { self.clip_min(lit(0)) } else { self }
    }

    fn normalize(self) -> Expr {
        self.apply(
            |series| {
                let chunked_array = series.f64()?;
                Ok(Some(
                    chunked_array
                        .into_iter()
                        .map(|option| Some(option.unwrap_or_default() / chunked_array.sum()?))
                        .collect::<Float64Chunked>()
                        .into_column(),
                ))
            },
            GetOutput::same_type(),
        )
    }

    fn normalize_if(self, normalize: bool) -> Expr {
        if normalize { self.normalize() } else { self }
    }

    fn destruct(mut self, names: impl IntoIterator<Item = impl AsRef<str>>) -> Expr {
        for name in names {
            self = self.struct_().field_by_name(name.as_ref());
        }
        self
    }

    fn suffix(self, suffix: &str) -> Expr {
        self.name().suffix(suffix)
    }
}

/// Extension methods for [`Schema`]
pub trait SchemaExt {
    fn names(&self) -> Vec<Expr>;
}

impl SchemaExt for Schema {
    fn names(&self) -> Vec<Expr> {
        self.iter_names_cloned().map(col).collect()
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
