use polars::prelude::*;

use crate::{
    app::panes::composition::control::{Method, Settings},
    special::{
        composition::{MC, NC, PMC, PNC, PSC, PTC, PUC, SC, SMC, SNC, SSC, STC, SUC, TC, UC},
        polars::ExprExt,
    },
};

/// Triacylglycerol
pub struct Triacylglycerol(pub LazyFrame);

impl Triacylglycerol {
    pub fn composition(self, settings: &Settings) -> PolarsResult<LazyFrame> {
        let mut lazy_frame = self.0;
        // Value
        lazy_frame = match settings.method {
            Method::Gunstone => todo!(),
            Method::VanderWal => lazy_frame.with_column(col("TAG").tag().value()),
        };
        // Composition
        for (index, group) in settings.groups.iter().enumerate() {
            let expr = col("TAG").tag().compose(group.composition)?;
            lazy_frame = lazy_frame.with_column(
                match group.composition {
                    MC => col("TAG").tag().mass(settings.adduct).round(1),
                    NC => col("TAG").tag().ecn(),
                    SC => expr.list().join(lit(""), false),
                    PSC => expr.list().join(lit(""), false),
                    SSC => expr.list().join(lit(""), false),
                    TC => expr.list().join(lit(""), false),
                    PTC => expr.list().join(lit(""), false),
                    STC => expr.list().join(lit(""), false),
                    UC => expr.list().sum(),
                    PUC => expr,
                    SUC => expr,
                    _ => unimplemented!(),
                }
                .alias(format!("Composition{index}")),
            );
            // Value
            lazy_frame = lazy_frame.with_column(
                sum("Value")
                    .over([as_struct(vec![col(format!("^Composition[0-{index}]$"))])])
                    .alias(format!("Value{index}")),
            );
        }
        lazy_frame = lazy_frame.drop(["TAG", "Value"]);
        // Group
        lazy_frame = lazy_frame
            .group_by([col(r#"^Composition\d$"#), col(r#"^Value\d$"#)])
            .agg([all()]);
        // Clear
        lazy_frame = lazy_frame.select([
            as_struct(vec![col(r#"^Composition\d$"#)]).alias("Compositions"),
            as_struct(vec![col(r#"^Value\d$"#)]).alias("Values"),
        ]);
        Ok(lazy_frame)
    }
}
