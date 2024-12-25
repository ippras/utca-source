use crate::{
    app::panes::calculation::control::{CHRISTIE, Fraction, From, Settings},
    special::polars::{ExprExt as _, FATTY_ACIDS_SCHEMA, SchemaExt},
    utils::polars::{ExprExt as _, destruct},
};
use egui::util::cache::{ComputerMut, FrameCache};
use polars::prelude::*;
use std::hash::{Hash, Hasher};

/// Calculation computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Calculation computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<DataFrame> {
        let values = key.data_frame.select(["Values"])?.unnest(["Values"])?;
        let names = values.get_column_names_str();
        let mut lazy_frame = key.data_frame.clone().lazy().unnest(["Key", "Values"]);
        if key.settings.christie.apply {
            lazy_frame = christie(lazy_frame);
        }
        println!("lazy_frame 555: {}", lazy_frame.clone().collect().unwrap());
        for &name in &names {
            lazy_frame = single(lazy_frame, name, key)?;
        }
        lazy_frame = lazy_frame.select([col("FA"), cols(names)]);
        println!("lazy_frame 999: {}", lazy_frame.clone().collect().unwrap());
        lazy_frame = lazy_frame.select([
            as_struct(vec![col("FA")]).alias("Key"),
            as_struct(vec![
                all().exclude(["FA"]),
                means(key.settings)?.alias("Mean"),
            ])
            .alias("Values"),
        ]);
        println!("lazy_frame 000: {}", lazy_frame.clone().collect().unwrap());
        lazy_frame = lazy_frame.with_row_index("Index", None);
        lazy_frame.collect()
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Calculation key
#[derive(Clone, Copy, Debug)]
pub(crate) struct Key<'a> {
    pub(crate) data_frame: &'a DataFrame,
    pub(crate) settings: &'a Settings,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for column in self.data_frame.get_columns() {
            for value in column.phys_iter() {
                value.hash(state);
            }
        }
        self.settings.hash(state);
    }
}

/// Calculate value
type Value = DataFrame;

/// Extension methods for [`Expr`]
trait ExprExt {
    fn calculated(self) -> Calculated;

    fn christie(self, christie: bool) -> Expr;

    fn experimental(self) -> Experimental;
}

impl ExprExt for Expr {
    fn calculated(self) -> Calculated {
        Calculated(self)
    }

    fn christie(self, christie: bool) -> Expr {
        if christie {
            self * col("Christie")
        } else {
            self
        }
    }

    fn experimental(self) -> Experimental {
        Experimental(self)
    }
}

/// Calculated
struct Calculated(Expr);

impl Calculated {
    // Enrichment factor
    fn ef(&self, name: &str) -> Expr {
        (self.0.clone().struct_().field_by_name(name)
            / self.0.clone().struct_().field_by_name("TAG"))
        .fill_nan(lit(0))
    }

    // Selectivity factor
    fn sf(&self, name: &str) -> Expr {
        self.0.clone().struct_().field_by_name(name)
            * unsaturated(self.0.clone().struct_().field_by_name("TAG"))
            / unsaturated(self.0.clone().struct_().field_by_name(name))
    }
}

fn unsaturated(expr: Expr) -> Expr {
    expr.filter(col("FA").fa().unsaturated()).sum()
}

/// Experimental
struct Experimental(Expr);

impl Experimental {
    fn tag123(&self, settings: &Settings) -> Expr {
        let expr = (lit(4) * self.0.clone().struct_().field_by_name("DAG1223")
            - self.0.clone().struct_().field_by_name("MAG2"))
            / lit(3);
        expr.clip_min_if(settings.unsigned)
            .normalize_if(settings.normalize.theoretical)
    }

    fn dag1223(&self, settings: &Settings) -> Expr {
        let expr = (lit(3) * self.0.clone().struct_().field_by_name("TAG")
            + self.0.clone().struct_().field_by_name("MAG2"))
            / lit(4);
        expr.normalize_if(settings.normalize.theoretical)
    }

    fn mag2(&self, settings: &Settings) -> Expr {
        let expr = lit(4) * self.0.clone().struct_().field_by_name("DAG1223")
            - lit(3) * self.0.clone().struct_().field_by_name("TAG");
        expr.clip_min_if(settings.unsigned)
            .normalize_if(settings.normalize.theoretical)
    }

    // 3 * TAG123 =  2 * DAG13 + MAG2
    // DAG13 = (3 * TAG123 - MAG2) / 2
    fn dag13_from_dag1223(&self, settings: &Settings) -> Expr {
        let expr = lit(3) * self.0.clone().struct_().field_by_name("TAG")
            - lit(2) * self.0.clone().struct_().field_by_name("DAG1223");
        expr.clip_min_if(settings.unsigned)
            .normalize_if(settings.normalize.theoretical)
    }

    // 2 * DAG13 = 3 * TAG123 - MAG2 (стр. 116)
    fn dag13_from_mag2(&self, settings: &Settings) -> Expr {
        let expr = (lit(3) * self.0.clone().struct_().field_by_name("TAG")
            - self.0.clone().struct_().field_by_name("MAG2"))
            / lit(2);
        expr.clip_min_if(settings.unsigned)
            .normalize_if(settings.normalize.theoretical)
    }
}

fn christie(lazy_frame: LazyFrame) -> LazyFrame {
    lazy_frame
        .unnest(["FA"])
        .join(
            CHRISTIE.clone().lazy().unnest(["FA"]),
            &[col("Carbons"), col("Doubles"), col("Triples")],
            &[col("Carbons"), col("Doubles"), col("Triples")],
            JoinArgs::new(JoinType::Left),
        )
        .with_columns([
            as_struct(vec![
                col("Carbons"),
                col("Doubles"),
                col("Triples"),
                col("Label"),
            ])
            .alias("FA"),
            col("Christie").fill_null(lit(1)),
        ])
        .drop([col("Carbons"), col("Doubles"), col("Triples"), col("Label")])
}

fn single(mut lazy_frame: LazyFrame, column: &str, key: Key) -> PolarsResult<LazyFrame> {
    let fraction = match key.settings.fraction {
        Fraction::AsIs => as_is,
        Fraction::ToMole => to_mole,
        Fraction::ToMass => to_mass,
        Fraction::Pchelkin => fraction,
    };
    lazy_frame = lazy_frame.with_column(
        as_struct(vec![
            fraction([column, "TAG"])
                .fill_null(lit(0.0))
                .christie(key.settings.christie.apply)
                .normalize_if(key.settings.normalize.experimental),
            fraction([column, "DAG1223"])
                .fill_null(lit(0.0))
                .christie(key.settings.christie.apply)
                .normalize_if(key.settings.normalize.experimental),
            fraction([column, "MAG2"])
                .fill_null(lit(0.0))
                .christie(key.settings.christie.apply)
                .normalize_if(key.settings.normalize.experimental),
        ])
        .alias("Experimental"),
    );
    // Theoretical
    lazy_frame = lazy_frame.with_column(
        as_struct(vec![
            col("Experimental")
                .experimental()
                .tag123(key.settings)
                .alias("TAG"),
            col("Experimental")
                .experimental()
                .dag1223(key.settings)
                .alias("DAG1223"),
            col("Experimental")
                .experimental()
                .mag2(key.settings)
                .alias("MAG2"),
            as_struct(vec![
                col("Experimental")
                    .experimental()
                    .dag13_from_dag1223(key.settings)
                    .alias("DAG1223"),
                col("Experimental")
                    .experimental()
                    .dag13_from_mag2(key.settings)
                    .alias("MAG2"),
            ])
            .alias("DAG13"),
        ])
        .alias("Theoretical"),
    );
    // Calculated
    lazy_frame = lazy_frame.with_column(
        as_struct(vec![
            col("Experimental")
                .struct_()
                .field_by_names(["TAG", "DAG1223", "MAG2"]),
            col("Theoretical")
                .struct_()
                .field_by_name("DAG13")
                .struct_()
                .field_by_name(match key.settings.from {
                    From::Dag1223 => "DAG1223",
                    From::Mag2 => "MAG2",
                })
                .alias("DAG13"),
        ])
        .alias("Calculated"),
    );
    // Enrichment factor
    lazy_frame = lazy_frame.with_column(
        as_struct(vec![
            col("Calculated").calculated().ef("MAG2").alias("MAG2"),
            col("Calculated").calculated().ef("DAG13").alias("DAG13"),
        ])
        .alias("EF"),
    );
    // Selectivity factor
    lazy_frame = lazy_frame.with_column(
        as_struct(vec![
            col("Calculated").calculated().sf("MAG2").alias("MAG2"),
            col("Calculated").calculated().sf("DAG13").alias("DAG13"),
        ])
        .alias("SF"),
    );
    println!("lazy_frame 8: {}", lazy_frame.clone().collect().unwrap());
    lazy_frame = lazy_frame.with_column(
        as_struct(vec![
            col("Experimental"),
            col("Theoretical"),
            col("Calculated"),
            as_struct(vec![col("EF"), col("SF")]).alias("Factors"),
        ])
        .alias(column),
    );
    println!("lazy_frame 9: {}", lazy_frame.clone().collect().unwrap());
    Ok(lazy_frame)
}

fn means(settings: &Settings) -> PolarsResult<Expr> {
    Ok(as_struct(vec![
        as_struct(vec![
            mean(&["Experimental", "TAG"], settings.ddof)?,
            mean(&["Experimental", "DAG1223"], settings.ddof)?,
            mean(&["Experimental", "MAG2"], settings.ddof)?,
        ])
        .alias("Experimental"),
        as_struct(vec![
            mean(&["Theoretical", "TAG"], settings.ddof)?,
            mean(&["Theoretical", "DAG1223"], settings.ddof)?,
            mean(&["Theoretical", "MAG2"], settings.ddof)?,
            as_struct(vec![
                mean(&["Theoretical", "DAG13", "DAG1223"], settings.ddof)?,
                mean(&["Theoretical", "DAG13", "MAG2"], settings.ddof)?,
            ])
            .alias("DAG13"),
        ])
        .alias("Theoretical"),
        as_struct(vec![
            as_struct(vec![
                mean(&["Factors", "EF", "MAG2"], settings.ddof)?,
                mean(&["Factors", "EF", "DAG13"], settings.ddof)?,
            ])
            .alias("EF"),
            as_struct(vec![
                mean(&["Factors", "SF", "MAG2"], settings.ddof)?,
                mean(&["Factors", "SF", "DAG13"], settings.ddof)?,
            ])
            .alias("SF"),
        ])
        .alias("Factors"),
    ]))
}

fn mean(names: &[&str], ddof: u8) -> PolarsResult<Expr> {
    Ok(as_struct(vec![
        concat_list([all().exclude(["FA"]).destruct(names)])?
            .list()
            .mean()
            .alias("Mean"),
        concat_list([all().exclude(["FA"]).destruct(names)])?
            .list()
            .std(ddof)
            .alias("Std"),
    ])
    .alias(names[names.len() - 1]))
}

fn as_is(names: [&str; 2]) -> Expr {
    destruct(names)
}

// n = m / M
fn to_mole(names: [&str; 2]) -> Expr {
    destruct(names) / col("FA").fa().mass()
}

// m = n * M
fn to_mass(names: [&str; 2]) -> Expr {
    destruct(names) * col("FA").fa().mass()
}

// Pchelkin fraction
fn fraction(names: [&str; 2]) -> Expr {
    // col(name) / (col(name) * col("FA").fa().mass() / lit(10)).sum()
    destruct(names) / to_mass(names).sum()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() -> PolarsResult<()> {
        let data_frame = df! {
            "A" => &[
                0208042.,
                0302117.,
                2420978.,
                0085359.,
                0195625.,
                2545783.,
                0031482.,
                4819586.,
                0012823.,
            ],
            "B" => &[
                0042194.,
                0145011.,
                0599666.,
                0025799.,
                0074037.,
                0595393.,
                0007738.,
                1158289.,
                0005070.,
            ],
            "M" => &[
                294.462,
                270.442,
                292.446,
                322.414,
                298.494,
                296.478,
                326.546,
                294.462,
                292.446,
            ],
        }?;
        let lazy_frame = data_frame.lazy().with_columns([
            (col("A") / (col("A") * col("M")).sum())
                .round(6)
                .alias("_N___GLC_Peak_Area__Free_1,2-DAGs"),
            (col("B") / (col("B") * col("M")).sum())
                .round(6)
                .alias("_N___GLC_Peak_Area__Total_TAGs"),
        ]);
        let data_frame = lazy_frame.collect()?;
        assert_eq!(
            data_frame["_N___GLC_Peak_Area__Free_1,2-DAGs"],
            Series::from_iter([
                0.000067, 0.000097, 0.000775, 0.000027, 0.000063, 0.000815, 0.000010, 0.001542,
                0.000004,
            ])
            .into_column(),
        );
        // [
        //     0.000067, 0.000097, 0.000775, 0.000027, 0.000063, 0.000815, 0.000010, 0.001542,
        //     0.000004,
        // ]
        // println!("data_frame: {}", );
        Ok(())
    }
}
