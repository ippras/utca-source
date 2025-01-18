use crate::app::panes::composition::settings::{Order, Settings, Sort};
use polars::prelude::*;
use std::borrow::Cow;

/// Extension methods for [`LazyFrame`]
pub trait LazyFrameExt: Sized {
    fn compositions(self) -> Compositions;
}

impl LazyFrameExt for LazyFrame {
    fn compositions(self) -> Compositions {
        Compositions(self)
    }
}

/// Compositions
pub struct Compositions(pub LazyFrame);

impl Compositions {
    pub fn meta(self, settings: &Settings) -> PolarsResult<Self> {
        let values = |index| {
            concat_list([all()
                .exclude([r#"^Composition\d$"#])
                .exclude([r#"^Value\d$"#])
                .struct_()
                .field_by_name(&match index {
                    Some(index) => Cow::Owned(format!("Value{index}")),
                    None => Cow::Borrowed("Value"),
                })])
        };
        let mut lazy_frame = self.0;
        for index in 0..settings.groups.len() {
            lazy_frame = lazy_frame.with_column(
                as_struct(vec![
                    values(Some(index))?.list().mean().alias("Mean"),
                    values(Some(index))?
                        .list()
                        .std(settings.ddof)
                        .alias("StandardDeviation"),
                ])
                .alias(format!("Value{index}")),
            );
        }
        // Group
        lazy_frame = lazy_frame.select([
            as_struct(vec![col(r#"^Composition\d$"#)]).alias("Compositions"),
            as_struct(vec![col(r#"^Value\d$"#)]).alias("Values"),
        ]);
        Ok(Self(lazy_frame))
    }

    pub fn filter(mut self, settings: &Settings) -> Self {
        if !settings.show.filtered {
            let mut predicate = lit(true);
            for (index, group) in settings.groups.iter().enumerate() {
                predicate = predicate.and(
                    col("Values")
                        .struct_()
                        .field_by_name(&format!("Value{index}"))
                        .struct_()
                        .field_by_name("Mean")
                        .gt(lit(group.filter.value)),
                );
            }
            self.0 = self.0.filter(predicate);
        }
        self
    }

    pub fn sort(mut self, settings: &Settings) -> Self {
        let mut sort_options = SortMultipleOptions::default();
        if let Order::Descending = settings.order {
            sort_options = sort_options
                .with_order_descending(true)
                .with_nulls_last(true);
        }
        self.0 = match settings.sort {
            Sort::Key => self.0.sort_by_exprs(
                [col("Compositions")
                    .struct_()
                    .field_by_name(r#"^Composition\d$"#)],
                sort_options,
            ),
            Sort::Value => self.0.sort_by_exprs(
                [col("Values")
                    .struct_()
                    .field_by_name(r#"^Value\d$"#)
                    .struct_()
                    .field_by_name("Mean")],
                sort_options,
            ),
        };
        self
    }

    pub fn restruct(self, settings: &Settings) -> LazyFrame {
        let mut expr = Vec::new();
        for index in 0..settings.groups.len() {
            expr.push(
                as_struct(vec![
                    col("Compositions")
                        .struct_()
                        .field_by_name(&format!("Composition{index}"))
                        .alias("Key"),
                    col("Values")
                        .struct_()
                        .field_by_name(&format!("Value{index}"))
                        .alias("Value"),
                ])
                .alias(format!("Composition{index}")),
            );
        }
        self.0.select(expr)
    }
}
