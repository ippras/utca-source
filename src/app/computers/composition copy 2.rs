use std::{
    borrow::Cow,
    hash::{Hash, Hasher},
    process::exit,
};

use egui::util::cache::{ComputerMut, FrameCache};
use polars::{lazy::dsl::sum_horizontal, prelude::*};

use super::tags::Tags;
use crate::{
    app::{
        data::{Entry, FattyAcids},
        panes::settings::composition::{Method, Order, Settings, Sort},
    },
    r#const::relative_atomic_mass::{C, H},
    special::{
        composition::{Kind, MC, NC, PMC, PNC, PSC, PTC, PUC, SC, SMC, SNC, SSC, STC, SUC, TC, UC},
        polars::ExprExt as _,
    },
    utils::polars::{indexed_cols, DataFrameExt, ExprExt as _},
};

/// Composition computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Composition computer
#[derive(Default)]
pub(crate) struct Computer;

// impl Computer {
//     pub(crate) async fn compute(&mut self, key: Key<'_>) -> PolarsResult<Value> {
//         let mut schema = Schema::with_capacity(3);
//         let mut compositions = Vec::new();
//         for index in 0..key.settings.groups.len() {
//             let composition = format!("Composition{index}");
//             compositions.push(col(&composition));
//             schema.insert(composition.into(), DataType::Null);
//         }
//         let mut lazy_frame = DataFrame::empty_with_schema(&schema).lazy();
//         println!("lazy_frame 0: {}", lazy_frame.clone().collect().unwrap());
//         for entry in key.entries {
//             if entry.checked {
//                 println!("entry: {entry:?}");
//                 let other = match key.settings.method {
//                     Method::Gunstone => self.gunstone(&entry.fatty_acids, key.settings)?.0,
//                     Method::VanderWal => {
//                         self.vander_wal(&entry.fatty_acids, key.settings)?
//                             .0
//                             .select([
//                                 // col("Composition"),
//                                 col("Composition").struct_().field_by_names(Some("*")),
//                                 col("Values").alias(&entry.name),
//                             ])
//                     }
//                 };
//                 println!("lazy_frame other: {}", other.clone().collect().unwrap());
//                 lazy_frame = lazy_frame.join(
//                     other,
//                     &compositions,
//                     &compositions,
//                     JoinArgs::new(key.settings.join.into())
//                         .with_coalesce(JoinCoalesce::CoalesceColumns),
//                 );
//             }
//         }
//         match key.settings.method {
//             Method::Gunstone => unreachable!(),
//             Method::VanderWal => {
//                 let mut lazy_frames = key.entries.into_iter().map(|entry| {
//                     self.vander_wal(&entry.fatty_acids, key.settings)
//                         .unwrap()
//                         .0
//                         .select([
//                             // col("Composition"),
//                             col("Composition").struct_().field_by_names(Some("*")),
//                             col("Values").alias(&entry.name),
//                         ])
//                 });
//                 if let Some(mut lazy_frame) = lazy_frames.next() {
//                     // lazy_frame
//                     let compositions = indexed_cols("Composition", 0..key.settings.groups.len());
//                     for other in lazy_frames {
//                         lazy_frame = lazy_frame.join(
//                             other,
//                             &compositions,
//                             &compositions,
//                             JoinArgs::new(key.settings.join.into())
//                                 .with_coalesce(JoinCoalesce::CoalesceColumns),
//                         );
//                     }
//                     // Filter
//                     if !key.settings.show.filtered {
//                         lazy_frame = lazy_frame.filter(
//                             all_horizontal([all()
//                                 .exclude([r#"^Composition\d$"#])
//                                 .struct_()
//                                 .field_by_name("Filter")])?
//                             .not(),
//                         );
//                     }
//                     // Sort
//                     lazy_frame = lazy_frame.compositions().sort(key.settings);
//                     // Meta
//                     lazy_frame = lazy_frame.compositions().meta(key.settings)?;
//                     println!("Meta: {}", lazy_frame.clone().collect().unwrap());
//                     return lazy_frame.collect();
//                 }
//             }
//         }
//         let mut schema = Schema::with_capacity(3);
//         schema.insert("Index".into(), DataType::UInt32);
//         for index in 0..key.settings.groups.len() {
//             schema.insert(format!("Composition{index}").into(), DataType::Null);
//         }
//         Ok(DataFrame::empty_with_schema(&schema))
//     }
// }

impl Computer {
    // let u = 1.0 - s;
    // if s <= 2.0 / 3.0 {
    //     Self {
    //         s,
    //         u,
    //         s3: 0.0,
    //         s2u: (3.0 * s / 2.0).powi(2),
    //         su2: 3.0 * s * (3.0 * u - 1.0) / 2.0,
    //         u3: ((3.0 * u - 1.0) / 2.0).powi(2),
    //     }
    // } else {
    //     Self {
    //         s,
    //         u,
    //         s3: 3.0 * s - 2.0,
    //         s2u: 3.0 * u,
    //         su2: 0.0,
    //         u3: 0.0,
    //     }
    // }
    fn gunstone(&mut self, fatty_acids: &FattyAcids, settings: &Settings) -> PolarsResult<Tags> {
        let lazy_frame = fatty_acids
            .0
            .clone()
            .lazy()
            .filter(col("FA").fa().saturated())
            .with_columns([
                col("TAG.Experimental").sum().alias("S"),
                col("TAG.Experimental").sum().alias("U"),
            ]);
        let s = lazy_frame.clone().collect()?.f64("_Sum").first().unwrap();
        println!("lazy_frame: {}", lazy_frame.clone().collect().unwrap());

        exit(0);
        // // Cartesian product (TAG from FA)
        // let mut tags = fatty_acids.cartesian_product()?;
        // // Filter
        // tags = tags.filter(settings);

        // // let gunstone = Gunstone::new(s);
        // let lazy_frame = key.fatty_acids.0.clone().lazy();
        // // lazy_frame = lazy_frame.select([
        // //     col("Label"),
        // //     col("Formula"),
        // //     col("TAG.Experimental"),
        // //     col("DAG1223.Experimental"),
        // //     col("MAG2.Experimental"),
        // //     col("DAG13.DAG1223.Calculated"),
        // //     col("DAG13.MAG2.Calculated"),
        // // ]);
        // // lazy_frame = lazy_frame.with_columns([s().alias("S"), u().alias("U")]);
        // println!("key.data_frame: {}", lazy_frame.clone().collect().unwrap());
        // lazy_frame.collect().unwrap()
    }

    // 1,3-sn 2-sn 1,2,3-sn
    // PSC:
    // [abc] = 2*[a_{13}]*[_b2]*[c_{13}]
    // [aab] = 2*[a_{13}]*[a_2]*[b13]
    // [aba] = [a13]^2*[b2]
    // `2*[a_{13}]` - потому что зеркальные ([abc]=[cba], [aab]=[baa]).
    // SSC: [abc] = [a_{13}]*[b_2]*[c_{13}]
    fn vander_wal(&mut self, fatty_acids: &FattyAcids, settings: &Settings) -> PolarsResult<Tags> {
        // Cartesian product (TAG from FA)
        let mut tags = fatty_acids.cartesian_product()?;
        println!("tags: {:?}", tags.0.clone().collect().unwrap());
        // Value
        tags = tags.value();
        // Compose
        tags.composition(settings)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        match key.settings.method {
            Method::Gunstone => unreachable!(),
            Method::VanderWal => {
                let mut schema = Schema::with_capacity(3);
                let mut compositions = Vec::new();
                for (index, group) in key.settings.groups.iter().enumerate() {
                    let composition = format!("Composition{index}");
                    compositions.push(col(&composition));
                    match group.composition.kind {
                        Kind::Ecn => schema.insert(composition.into(), DataType::UInt32),
                        Kind::Mass => schema.insert(composition.into(), DataType::Float64),
                        Kind::Species => schema.insert(composition.into(), DataType::String),
                        Kind::Type => schema.insert(composition.into(), DataType::String),
                        Kind::Unsaturation => todo!(),
                    };
                }
                schema.insert(
                    "Composition".into(),
                    DataType::List(Box::new(DataType::String)),
                );
                compositions.push(col("Composition"));
                let mut lazy_frame = DataFrame::empty_with_schema(&schema).lazy();
                println!("UnJoin: {}", key.data_frame);
                for column in &key.data_frame.clone().get_columns()[2..] {
                    let tags = vander_wal(
                        key.data_frame.clone().lazy().select([
                            col("Index"),
                            col("FA"),
                            col(column.name().as_str()).alias("Values"),
                        ]),
                        key.settings,
                    )
                    .unwrap();
                    println!(
                        "tags: {}",
                        tags.0
                            .clone()
                            .lazy()
                            .lazy()
                            .select([
                                col("Compositions").struct_().field_by_names(["*"]),
                                col("Values").alias(column.name().as_str()),
                            ])
                            .collect()
                            .unwrap()
                    );
                    lazy_frame = lazy_frame.join(
                        tags.0.lazy().select([
                            col("Compositions").struct_().field_by_names(["*"]),
                            col("Values").alias(column.name().as_str()),
                        ]),
                        &compositions,
                        &compositions,
                        JoinArgs::new(JoinType::Full).with_coalesce(JoinCoalesce::CoalesceColumns),
                    );
                    println!("0: {}", lazy_frame.clone().collect().unwrap());
                    // exit(0);
                    // ┌──────────────┬──────────────┬────────────────────────────┬──────────────────────────────┐
                    // │ Composition0 ┆ Composition1 ┆ Composition                ┆ LunariaRediviva.1.1.utca.ron │
                    // │ ---          ┆ ---          ┆ ---                        ┆ ---                          │
                    // │ f64          ┆ str          ┆ list[str]                  ┆ struct[3]                    │
                    // ╞══════════════╪══════════════╪════════════════════════════╪══════════════════════════════╡
                    // │ 740.7        ┆ MyrMyrMyr    ┆ ["MyrMyrMyr", "MyrMyrMyr"] ┆ {0.0,0.0,[0.0, 0.0]}         │
                    // │ 768.7        ┆ MyrMyrPam    ┆ ["MyrMyrPam", "MyrMyrPam"] ┆ {8.8314e-10,0.0,[0.0, 0.0]}  │
                    // │ 766.7        ┆ MyrMyrP7x    ┆ ["MyrMyrP7x", "MyrMyrP7x"] ┆ {0.0,0.0,[0.0, 0.0]}         │
                    // │ 766.7        ┆ MyrMyrP9x    ┆ ["MyrMyrP9x", "MyrMyrP9x"] ┆ {0.0,0.0,[0.0, 0.0]}         │
                    // │ 762.7        ┆ MyrMyrRug    ┆ ["MyrMyrRug", "MyrMyrRug"] ┆ {0.0,0.0,[0.0, 0.0]}         │
                    // │ …            ┆ …            ┆ …                          ┆ …                            │
                    // │ 1129.1       ┆ BehNerNer    ┆ ["NerNerBeh", "NerNerBeh"] ┆ {0.0,0.0,[0.0, 0.0]}         │
                    // │ 1127.1       ┆ EruNerNer    ┆ ["NerNerEru", "NerNerEru"] ┆ {0.0,0.0,[0.0, 0.0]}         │
                    // │ 1127.1       ┆ E15NerNer    ┆ ["NerNerE15", "NerNerE15"] ┆ {0.0,0.0,[0.0, 0.0]}         │
                    // │ 1125.1       ┆ DdeNerNer    ┆ ["NerNerDde", "NerNerDde"] ┆ {0.0,0.0,[0.0, 0.0]}         │
                    // │ 1155.1       ┆ NerNerNer    ┆ ["NerNerNer", "NerNerNer"] ┆ {0.0,0.0,[0.0, 0.0]}         │
                    // └──────────────┴──────────────┴────────────────────────────┴──────────────────────────────┘
                }
                println!("Join: {}", lazy_frame.clone().collect().unwrap());

                // let lazy_frame = key.data_frame.clone().lazy().clone().select(exprs);
                // println!("2: {}", lazy_frame.collect().unwrap());
                // .select([fatty_acid("DAG13").alias("SN1")]);
                // let mut lazy_frames = key.entries.into_iter().map(|entry| {
                //     self.vander_wal(&entry.fatty_acids, key.settings)
                //         .unwrap()
                //         .0
                //         .select([
                //             // col("Composition"),
                //             col("Composition").struct_().field_by_names(Some("*")),
                //             col("Values").alias(&entry.name),
                //         ])
                // });
                // if let Some(mut lazy_frame) = lazy_frames.next() {
                //     let compositions = indexed_cols("Composition", 0..key.settings.groups.len());
                //     for other in lazy_frames {
                //         lazy_frame = lazy_frame.join(
                //             other,
                //             &compositions,
                //             &compositions,
                //             JoinArgs::new(key.settings.join.into())
                //                 .with_coalesce(JoinCoalesce::CoalesceColumns),
                //         );
                //     }
                //     // Filter
                //     if !key.settings.show.filtered {
                //         lazy_frame = lazy_frame.filter(
                //             all_horizontal([all()
                //                 .exclude([r#"^Composition\d$"#])
                //                 .struct_()
                //                 .field_by_name("Filter")])
                //             .unwrap()
                //             .not(),
                //         );
                //     }

                // Meta
                lazy_frame = lazy_frame.compositions().meta(key.settings).unwrap();
                println!("Meta: {}", lazy_frame.clone().collect().unwrap());
                // Filter
                let mut compositions = lazy_frame.compositions().filter(key.settings);
                println!("Filter: {}", compositions.0.clone().collect().unwrap());
                // Sort
                compositions = compositions.sort(key.settings);
                println!("Sort: {}", compositions.0.clone().collect().unwrap());
                // Restruct
                lazy_frame = compositions.restruct(key.settings.groups.len());
                return lazy_frame.collect().unwrap();
            }
        }
        let mut schema = Schema::with_capacity(3);
        schema.insert("Index".into(), DataType::UInt32);
        for index in 0..key.settings.groups.len() {
            schema.insert(format!("Composition{index}").into(), DataType::Null);
        }
        DataFrame::empty_with_schema(&schema)
    }
}

fn vander_wal(lazy_frame: LazyFrame, settings: &Settings) -> PolarsResult<Tags> {
    // Cartesian product (TAG from FA)
    let mut tags = cartesian_product(lazy_frame)?;
    // Value
    tags = tags.value();
    // Compose
    tags = tags.composition(settings)?;
    Ok(tags)
}

fn cartesian_product(lazy_frame: LazyFrame) -> PolarsResult<Tags> {
    // Tags with stereospecific number values
    Ok(Tags(
        lazy_frame
            .clone()
            .select([as_struct(vec![
                col("Index"),
                col("FA"),
                col("Values")
                    .struct_()
                    .field_by_name("DAG13")
                    .alias("Value"),
            ])
            .alias("SN1")])
            .cross_join(
                lazy_frame.clone().select([as_struct(vec![
                    col("Index"),
                    col("FA"),
                    col("Values").struct_().field_by_name("MAG2").alias("Value"),
                ])
                .alias("SN2")]),
                None,
            )
            .cross_join(
                lazy_frame.clone().select([as_struct(vec![
                    col("Index"),
                    col("FA"),
                    col("Values")
                        .struct_()
                        .field_by_name("DAG13")
                        .alias("Value"),
                ])
                .alias("SN3")]),
                None,
            )
            .select([as_struct(vec![col("SN1"), col("SN2"), col("SN3")]).alias("TAG")])
            .cache(),
    ))
}

/// Composition key
#[derive(Clone, Copy, Debug, PartialEq)]
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

/// Composition value
type Value = DataFrame;

impl FattyAcids {
    // TODO https://github.com/pola-rs/polars/issues/18587
    fn cartesian_product(&self) -> PolarsResult<Tags> {
        let lazy_frame = self.0.clone().lazy().with_row_index("Index", None);
        // Fatty acid with stereospecific number value
        let fatty_acid = |name| {
            as_struct(vec![
                col("Index"),
                col("FA"),
                col("Target")
                    .struct_()
                    .field_by_name("Calculated")
                    .struct_()
                    .field_by_name(name)
                    .alias("Value"),
            ])
        };
        Ok(Tags(
            lazy_frame
                .clone()
                .select([fatty_acid("DAG13").alias("SN1")])
                .cross_join(
                    lazy_frame.clone().select([fatty_acid("MAG2").alias("SN2")]),
                    None,
                )
                .cross_join(lazy_frame.select([fatty_acid("DAG13").alias("SN3")]), None)
                .select([as_struct(vec![col("SN1"), col("SN2"), col("SN3")]).alias("TAG")]),
        ))
    }
    // fn cartesian_product(&self) -> PolarsResult<Tags> {
    //     let lazy_frame = self.0.clone().lazy().with_row_index("Index", None);
    //     Ok(Tags(
    //         lazy_frame
    //             .clone()
    //             .select([fatty_acid("DAG13.Calculated")?.alias("SN1")])
    //             .cross_join(
    //                 lazy_frame
    //                     .clone()
    //                     .select([fatty_acid("MAG2.Calculated")?.alias("SN2")]),
    //                 None,
    //             )
    //             .cross_join(
    //                 lazy_frame.select([fatty_acid("DAG13.Calculated")?.alias("SN3")]),
    //                 None,
    //             )
    //             .select([as_struct(vec![col("SN1"), col("SN2"), col("SN3")]).alias("TAG")]),
    //     ))
    // }
}

/// Compositions
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
    fn meta(self, settings: &Settings) -> PolarsResult<LazyFrame> {
        let values = |index| {
            concat_list([all()
                .exclude([r#"^Composition\d?$"#])
                .exclude([r#"^Value\d?$"#])
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
                        .std(settings.meta.ddof)
                        .alias("StandardDeviation"),
                ])
                .alias(format!("Value{index}")),
            );
        }
        // TODO: Mean and StandardDeviation calculates for all SSC values? separate if for each
        // pl.col("a").list.eval(pl.element() * pl.int_range(pl.len()))
        lazy_frame = lazy_frame.with_column(
            as_struct(vec![
                concat_list([values(None)?])?.list().mean().alias("Mean"),
                concat_list([values(None)?])?
                    .list()
                    .std(settings.meta.ddof)
                    .alias("StandardDeviation"),
                // sum_horizontal([values(None)?.explode()])?.alias("StandardDeviation"),
            ])
            .alias("Value"),
        );
        // Group
        lazy_frame = lazy_frame.select([
            as_struct(vec![col(r#"^Composition\d?$"#)]).alias("Compositions"),
            as_struct(vec![col(r#"^Value\d?$"#)]).alias("Values"),
        ]);
        Ok(lazy_frame)
    }

    fn filter(mut self, settings: &Settings) -> Self {
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

    fn sort(mut self, settings: &Settings) -> Self {
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
                    .field_by_names([r#"^Composition\d$"#])],
                sort_options,
            ),
            Sort::Value => self.0.sort_by_exprs(
                [col("Values")
                    .struct_()
                    .field_by_names([r#"^Value\d$"#])
                    .struct_()
                    .field_by_name("Mean")],
                sort_options,
            ),
        };
        self
    }

    fn restruct(self, len: usize) -> LazyFrame {
        let mut expr = Vec::new();
        for index in 0..len {
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
        expr.push(as_struct(vec![
            col("Compositions")
                .struct_()
                .field_by_name("Composition")
                .alias("Key"),
            col("Values").struct_().field_by_name("Value"),
        ]));
        self.0.select(expr)
    }
}

/// Extension methods for [`LazyFrame`]
trait LazyFrameExt: Sized {
    fn compositions(self) -> Compositions;

    fn permutation<const N: usize>(self, names: [&str; N], sort: Expr) -> PolarsResult<Self>;
}

impl LazyFrameExt for LazyFrame {
    fn compositions(self) -> Compositions {
        Compositions(self)
    }

    fn permutation<const N: usize>(self, names: [&str; N], sort: Expr) -> PolarsResult<Self> {
        const NAME: &str = "_KEY";

        let mut lazy_frame = self.with_column(
            concat_list(names.map(col))?
                .list()
                .eval(sort, true)
                .alias(NAME),
        );
        for index in 0..N {
            lazy_frame = lazy_frame.with_column(
                col(NAME)
                    .list()
                    .get(lit(index as u32), false)
                    .alias(names[index]),
            );
        }
        Ok(lazy_frame.drop([NAME]))
    }
}

fn sort_by_ecn() -> Expr {
    col("").sort_by(
        [col("").struct_().field_by_name("FA").fa().ecn()],
        Default::default(),
    )
}

fn sort_by_mass() -> Expr {
    col("").sort_by(
        [col("").struct_().field_by_name("FA").fa().mass()],
        Default::default(),
    )
}

fn sort_by_type() -> Expr {
    col("").sort_by(
        [col("").struct_().field_by_name("FA").fa().saturated()],
        Default::default(),
    )
}

fn sort_by_species() -> Expr {
    col("").sort_by(
        [
            col("")
                .struct_()
                .field_by_name("FA")
                .struct_()
                .field_by_name("Carbons"),
            col("")
                .struct_()
                .field_by_name("FA")
                .struct_()
                .field_by_name("Doubles")
                .list()
                .len(),
            col("")
                .struct_()
                .field_by_name("FA")
                .struct_()
                .field_by_name("Triples")
                .list()
                .len(),
            col("")
                .struct_()
                .field_by_name("FA")
                .struct_()
                .field_by_name("Label"),
            col("").struct_().field_by_name("Index"),
        ],
        Default::default(),
    )
}

fn sort_by_unsaturation() -> Expr {
    col("").sort_by(
        [col("").struct_().field_by_name("FA").fa().unsaturation()],
        Default::default(),
    )
}

// // Triacylglycerol species
// fn species() -> Expr {
//     concat_str(
//         [col("TAG")
//             .struct_()
//             // .field_by_name("FA")
//             // .struct_()
//             .field_by_names([r#"^SN[1-3]$"#])
//             .fatty_acid()
//             .species()],
//         "",
//         true,
//     )
// }

// // Triacylglycerol value
// fn value() -> Expr {
//     col("TAG")
//         .struct_()
//         .field_by_name("SN1")
//         .struct_()
//         .field_by_name("Value")
//         * col("TAG")
//             .struct_()
//             .field_by_name("SN2")
//             .struct_()
//             .field_by_name("Value")
//         * col("TAG")
//             .struct_()
//             .field_by_name("SN3")
//             .struct_()
//             .field_by_name("Value")
// }

// impl Composer {
//     fn gunstone(&mut self, key: Key) -> Tree<Meta, Data> {
//         let Key { context } = key;
//         let tags123 = &context
//             .state
//             .entry()
//             .data
//             .calculated
//             .tags123
//             .experimental
//             .normalized;
//         let tags1 = discriminated(&context, Sn::One);
//         let tags2 = discriminated(&context, Sn::Two);
//         let tags3 = discriminated(&context, Sn::Three);
//         let s = zip(tags123, &context.state.entry().meta.formulas)
//             .filter_map(|(value, formula)| match formula.saturation() {
//                 Saturated => Some(value),
//                 Unsaturated => None,
//             })
//             .sum();
//         let gunstone = Gunstone::new(s);
//         let ungrouped = repeat(0..context.state.entry().len())
//             .take(3)
//             .multi_cartesian_product()
//             .map(|indices| {
//                 let tag = Tag([indices[0], indices[1], indices[2]])
//                     .compose(context.settings.composition.tree.leafs.stereospecificity);
//                 let value = gunstone.factor(context.r#type(tag))
//                     * tags1[indices[0]]
//                     * tags2[indices[1]]
//                     * tags3[indices[2]];
//                 (tag, value.into())
//             })
//             .into_grouping_map()
//             .sum();
//         Tree::from(ungrouped.group_by_key(key))
//     }

//     // 1,3-sn 2-sn 1,2,3-sn
//     // [abc] = 2*[a13]*[b2]*[c13]
//     // [aab] = 2*[a13]*[a2]*[b13]
//     // [aba] = [a13]^2*[b2]
//     // [abc] = [a13]*[b2]*[c13]
//     // `2*[a13]` - потому что зеркальные ([abc]=[cba], [aab]=[baa]).
//     fn vander_wal(&mut self, key: Key) -> Tree<Meta, Data> {
//         let Key { context } = key;
//         let dags13 = &context
//             .state
//             .entry()
//             .data
//             .calculated
//             .dags13
//             .value(context.settings.calculation.from)
//             .normalized;
//         let mags2 = &context
//             .state
//             .entry()
//             .data
//             .calculated
//             .mags2
//             .value()
//             .normalized;
//         let ungrouped = repeat(0..context.state.entry().len())
//             .take(3)
//             .multi_cartesian_product()
//             .map(|indices| {
//                 let tag = Tag([indices[0], indices[1], indices[2]])
//                     .compose(context.settings.composition.tree.leafs.stereospecificity);
//                 let value = dags13[indices[0]] * mags2[indices[1]] * dags13[indices[2]];
//                 (tag, value.into())
//             })
//             .into_grouping_map()
//             .sum();
//         Tree::from(ungrouped.group_by_key(key))
//     }
// }

/// Gunstone
struct Gunstone {
    s: f64,
    u: f64,
    s3: f64,
    s2u: f64,
    su2: f64,
    u3: f64,
}

impl Gunstone {
    fn new(s: f64) -> Self {
        let u = 1.0 - s;
        if s <= 2.0 / 3.0 {
            Self {
                s,
                u,
                s3: 0.0,
                s2u: (3.0 * s / 2.0).powi(2),
                su2: 3.0 * s * (3.0 * u - 1.0) / 2.0,
                u3: ((3.0 * u - 1.0) / 2.0).powi(2),
            }
        } else {
            Self {
                s,
                u,
                s3: 3.0 * s - 2.0,
                s2u: 3.0 * u,
                su2: 0.0,
                u3: 0.0,
            }
        }
    }

    // fn factor(&self, r#type: Tag<Saturation>) -> f64 {
    //     match r#type.into() {
    //         S3 => self.s3 / self.s.powi(3),                    // [SSS]
    //         S2U => self.s2u / (self.s.powi(2) * self.u) / 3.0, // [SSU], [USS], [SUS]
    //         SU2 => self.su2 / (self.s * self.u.powi(2)) / 3.0, // [SUU], [USU], [UUS]
    //         U3 => self.u3 / self.u.powi(3),                    // [UUU]
    //     }
    // }
}

// fn discriminated(context: &Context, sn: Sn) -> Vec<f64> {
//     context
//         .state
//         .entry()
//         .data
//         .calculated
//         .tags123
//         .experimental
//         .normalized
//         .iter()
//         .enumerate()
//         .map(move |(index, &value)| {
//             let discrimination = &context.settings.composition.discrimination;
//             match sn {
//                 Sn::.sn1.One => discrimination.get(&index),
//                 Sn::.sn2.Two => discrimination.get(&index),
//                 Sn::.sn3.Three => discrimination.get(&index),
//             }
//             .map_or(value, |&f| f * value)
//         })
//         .normalized()
// }
