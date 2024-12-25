use std::borrow::Cow;

use polars::prelude::*;

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
#[derive(Clone)]
struct Compositions(LazyFrame);

impl Compositions {
    // fn meta(self, settings: &Settings) -> PolarsResult<LazyFrame> {
    //     let expr = concat_list([all()
    //         .exclude([r#"^Composition\d$"#])
    //         .struct_()
    //         .field_by_name(&format!("Value{}", settings.groups.len().saturating_sub(1)))])?;
    //     Ok(self
    //         .0
    //         .with_columns([
    //             expr.clone().list().mean().alias("Mean"),
    //             expr.clone()
    //                 .list()
    //                 .std(settings.meta.ddof)
    //                 .alias("StandardDeviation"),
    //         ])
    //         .select([
    //             as_struct(vec![col("Mean"), col("StandardDeviation")]).alias("Statistic"),
    //             all().exclude(["Mean", "StandardDeviation"]),
    //         ]))
    // }
    fn meta(self, ddof: u8) -> PolarsResult<LazyFrame> {
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
        println!(
            "444: {}",
            lazy_frame
                .clone()
                .select([col("LunariaRediviva.1.1.utca.ron")])
                .unnest([col("LunariaRediviva.1.1.utca.ron")])
                // .explode([col("Species")])
                // .select([col("Species")])
                // .unnest([col("Species")])
                .collect()
                .unwrap()
        );
        for index in 0..settings.groups.len() {
            lazy_frame = lazy_frame.with_column(
                as_struct(vec![
                    values(Some(index))?.list().mean().alias("Mean"),
                    values(Some(index))?
                        .list()
                        .std(settings.meta.ddof)
                        .alias("StandardDeviation"),
                ])
                .alias(format!("Value{index}")),
            );
        }
        lazy_frame = lazy_frame.with_column(
            as_struct(vec![
                values(None)?.list().mean().alias("Mean"),
                values(None)?
                    .list()
                    .std(settings.meta.ddof)
                    .alias("StandardDeviation"),
            ])
            .alias("Value"),
        );
        println!("555: {}", lazy_frame.clone().collect().unwrap());
        Ok(lazy_frame)
    }

    fn sort(mut self, settings: &Settings) -> LazyFrame {
        let mut sort_options = SortMultipleOptions::default();
        if let Order::Descending = settings.order {
            sort_options = sort_options
                .with_order_descending(true)
                .with_nulls_last(true);
        }
        self.0 = match settings.sort {
            Sort::Key => self.0.sort_by_exprs(
                [col(r#"^Composition\d$"#).struct_().field_by_name("Key")],
                sort_options,
            ),
            Sort::Value => {
                // let value = all()
                //     .exclude([r#"^Composition\d$"#])
                //     .struct_()
                //     .field_by_names([r#"^Value\d$"#]);
                let value = col(r#"^Composition\d$"#)
                    .struct_()
                    .field_by_name("Value")
                    .struct_()
                    .field_by_name("Mean");
                self.0.sort_by_exprs([value], sort_options)
            }
        };
        self.0
    }
}
