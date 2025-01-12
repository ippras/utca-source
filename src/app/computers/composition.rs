use super::compositions::LazyFrameExt as _;
use crate::{
    app::panes::composition::control::{Filter, Group, Method, Order, Settings, Sort},
    r#const::relative_atomic_mass::{C, H},
    special::composition::{
        Kind, MC, NC, PMC, PNC, PSC, PTC, PUC, SC, SMC, SNC, SSC, STC, SUC, TC, UC,
    },
    utils::polars::{DataFrameExt as _, ExprExt as _},
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::{
    prelude::*,
    triacylglycerol::polars::expr::{
        ExprExt as _, Options as StereospecificOptions, mass::Mass as _,
    },
};
use polars::{
    lazy::dsl::{max_horizontal, min_horizontal, sum_horizontal},
    prelude::*,
};
use std::{
    borrow::Cow,
    collections::VecDeque,
    convert::identity,
    hash::{Hash, Hasher},
    process::exit,
};
use utca::metadata::MetaDataFrame;

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
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        match *key.index {
            Some(index) => {
                let frame = &key.frames[index];
                let mut lazy_frame = frame.data.clone().lazy();
                lazy_frame = lazy_frame.select([
                    col("Index"),
                    col("Label"),
                    col("FattyAcid"),
                    col("Calculated")
                        .struct_()
                        .field_by_names(["Monoacylglycerol2", "Diacylglycerol13"]),
                ]);
                lazy_frame = vander_wal(lazy_frame, key.settings)?;
                // Filter
                lazy_frame = filter(lazy_frame, key.settings);
                // Sort
                lazy_frame = sort(lazy_frame, key.settings);
                // Index
                lazy_frame = lazy_frame.with_row_index("Index", None);
                lazy_frame.collect()
            }
            None => {
                unimplemented!()
            }
        }
    }

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
    fn gunstone(&mut self, data_frame: &DataFrame, settings: &Settings) -> PolarsResult<LazyFrame> {
        let lazy_frame = data_frame
            .clone()
            .lazy()
            .filter(col("FattyAcid").fa().is_saturated())
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

    // fn vander_wal(&mut self, data_frame: &DataFrame, settings: &Settings) -> PolarsResult<Tags> {
    //     Ok(())
    // }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
        // match key.settings.method {
        //     Method::Gunstone => unreachable!(),
        //     Method::VanderWal => {
        //         assert!(!key.settings.groups.is_empty());
        //         // let mut compositions = Vec::new();
        //         // for index in 0..key.settings.groups.len() {
        //         //     compositions.push(col(format!("Composition{index}")));
        //         // }
        //         // let mut lazy_frame: Option<LazyFrame> = None;
        //         // for column in &key.data_frame.clone().get_columns()[2..] {
        //         //     let composition = vander_wal(
        //         //         key.data_frame.clone().lazy().select([
        //         //             col("Index"),
        //         //             col("FA"),
        //         //             col(column.name().as_str()).alias("Values"),
        //         //         ]),
        //         //         key.settings,
        //         //     )
        //         //     .unwrap();
        //         //     let next = composition.select([
        //         //         col("Compositions").struct_().field_by_names(["*"]),
        //         //         col("Values").alias(column.name().as_str()),
        //         //     ]);
        //         //     lazy_frame = Some(if let Some(current) = lazy_frame {
        //         //         current.join(
        //         //             next,
        //         //             &compositions,
        //         //             &compositions,
        //         //             JoinArgs::new(JoinType::Full)
        //         //                 .with_coalesce(JoinCoalesce::CoalesceColumns),
        //         //         )
        //         //     } else {
        //         //         next
        //         //     });
        //         // }
        //         // let Some(mut lazy_frame) = lazy_frame else {
        //         //     return DataFrame::empty();
        //         // };
        //         // // Meta
        //         // let mut compositions = lazy_frame.compositions().meta(key.settings).unwrap();
        //         // // Filter
        //         // compositions = compositions.filter(key.settings);
        //         // // Sort
        //         // compositions = compositions.sort(key.settings);
        //         // // Restruct
        //         // lazy_frame = compositions.restruct(key.settings);
        //         // lazy_frame.collect().unwrap()
        //     }
        // }
    }
}

/// Composition key
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Key<'a> {
    pub(crate) frames: &'a [MetaDataFrame],
    pub(crate) index: &'a Option<usize>,
    pub(crate) settings: &'a Settings,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.frames.hash(state);
        self.index.hash(state);
        self.settings.hash(state);
    }
}

/// Composition value
type Value = DataFrame;

// 1,3-sn 2-sn 1,2,3-sn
// PSC:
// [abc] = 2*[a_{13}]*[_b2]*[c_{13}]
// [aab] = 2*[a_{13}]*[a_2]*[b13]
// [aba] = [a13]^2*[b2]
// `2*[a_{13}]` - потому что зеркальные ([abc]=[cba], [aab]=[baa]).
// SSC: [abc] = [a_{13}]*[b_2]*[c_{13}]
fn vander_wal(mut lazy_frame: LazyFrame, settings: &Settings) -> PolarsResult<LazyFrame> {
    // Cartesian product (TAG from FA)
    lazy_frame = cartesian_product(lazy_frame)?;
    // Compose
    lazy_frame = compose(lazy_frame, settings)?;
    Ok(lazy_frame)
}

fn cartesian_product(mut lazy_frame: LazyFrame) -> PolarsResult<LazyFrame> {
    lazy_frame = lazy_frame
        .clone()
        .select([as_struct(vec![
            col("Label"),
            col("FattyAcid"),
            col("Diacylglycerol13").alias("Value"),
        ])
        .alias("StereospecificNumber1")])
        .cross_join(
            lazy_frame.clone().select([as_struct(vec![
                col("Label"),
                col("FattyAcid"),
                col("Monoacylglycerol2").alias("Value"),
            ])
            .alias("StereospecificNumber2")]),
            None,
        )
        .cross_join(
            lazy_frame.clone().select([as_struct(vec![
                col("Label"),
                col("FattyAcid"),
                col("Diacylglycerol13").alias("Value"),
            ])
            .alias("StereospecificNumber3")]),
            None,
        );
    // Restruct
    lazy_frame = lazy_frame.select([
        as_struct(vec![
            col("StereospecificNumber1")
                .struct_()
                .field_by_name("Label")
                .alias("StereospecificNumber1"),
            col("StereospecificNumber2")
                .struct_()
                .field_by_name("Label")
                .alias("StereospecificNumber2"),
            col("StereospecificNumber3")
                .struct_()
                .field_by_name("Label")
                .alias("StereospecificNumber3"),
        ])
        .alias("Label"),
        as_struct(vec![
            col("StereospecificNumber1")
                .struct_()
                .field_by_name("FattyAcid")
                .alias("StereospecificNumber1"),
            col("StereospecificNumber2")
                .struct_()
                .field_by_name("FattyAcid")
                .alias("StereospecificNumber2"),
            col("StereospecificNumber3")
                .struct_()
                .field_by_name("FattyAcid")
                .alias("StereospecificNumber3"),
        ])
        .alias("FattyAcid"),
        col("StereospecificNumber1")
            .struct_()
            .field_by_name("Value")
            * col("StereospecificNumber2")
                .struct_()
                .field_by_name("Value")
            * col("StereospecificNumber3")
                .struct_()
                .field_by_name("Value"),
    ]);
    Ok(lazy_frame)
}

fn compose(mut lazy_frame: LazyFrame, settings: &Settings) -> PolarsResult<LazyFrame> {
    // lazy_frame = lazy_frame.with_columns([
    //     col("FattyAcid")
    //         .tag()
    //         .mass(lit(settings.adduct))
    //         .round(1)
    //         .alias("MC"),
    //     col("FattyAcid")
    //         .tag()
    //         .map(|expr| expr.fa().ecn())
    //         .alias("NC"),
    //     col("FattyAcid")
    //         .tag()
    //         .map(|expr| expr.fa().unsaturated().sum())
    //         .alias("UC"),
    //     col("FattyAcid")
    //         .tag()
    //         .non_stereospecific(
    //             |expr| expr.fa().is_saturated(),
    //             StereospecificOptions::default().map(true),
    //         )?
    //         .alias("TC"),
    //     col("FattyAcid")
    //         .tag()
    //         .positional(
    //             |expr| expr.fa().is_saturated(),
    //             StereospecificOptions::default().map(true),
    //         )
    //         .alias("PTC"),
    //     col("FattyAcid")
    //         .tag()
    //         .map(|expr| expr.fa().is_saturated())
    //         .alias("STC"),
    //     col("Label")
    //         .tag()
    //         .non_stereospecific(identity, StereospecificOptions::default())?
    //         .alias("SC"),
    //     col("Label")
    //         .tag()
    //         .positional(identity, StereospecificOptions::default())
    //         .alias("PSC"),
    //     col("Label").alias("SSC"),
    // ]);
    let mut groups = settings.groups.clone();
    if groups.is_empty() {
        groups.push_back(Group {
            composition: SSC,
            filter: Filter::new(),
        });
    }
    // Composition
    for (index, group) in groups.iter().enumerate() {
        lazy_frame = lazy_frame.with_column(
            match group.composition {
                MC => col("FattyAcid").tag().mass(lit(settings.adduct)).round(1),
                NC => col("FattyAcid").tag().ecn(),
                UC => col("FattyAcid").tag().unsaturation(),
                TC => col("FattyAcid")
                    .tag()
                    .non_stereospecific(
                        |expr| expr.fa().is_saturated(),
                        StereospecificOptions::default().map(true),
                    )?
                    .alias("TC"),
                PTC => col("FattyAcid")
                    .tag()
                    .positional(
                        |expr| expr.fa().is_saturated(),
                        StereospecificOptions::default().map(true),
                    )
                    .alias("PTC"),
                STC => col("FattyAcid").tag().map(|expr| expr.fa().is_saturated()),
                SC => col("Label")
                    .tag()
                    .non_stereospecific(identity, StereospecificOptions::default())?
                    .alias("SC"),
                PSC => col("Label")
                    .tag()
                    .positional(identity, StereospecificOptions::default())
                    .alias("PSC"),
                SSC => col("Label").alias("SSC"),
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
    // Group
    lazy_frame = lazy_frame
        .group_by([col(r#"^Composition\d$"#), col(r#"^Value\d$"#)])
        .agg([as_struct(vec![col("Label"), col("FattyAcid"), col("Value")]).alias("Species")]);
    // Restruct
    let mut exprs = Vec::new();
    for index in 0..settings.groups.len() {
        exprs.push(
            as_struct(vec![
                col(&format!("Composition{index}")).alias("Key"),
                col(&format!("Value{index}")).alias("Value"),
            ])
            .alias(format!("Composition{index}")),
        );
    }
    exprs.push(col("Species"));
    Ok(lazy_frame.select(exprs))
}

fn filter(mut lazy_frame: LazyFrame, settings: &Settings) -> LazyFrame {
    if !settings.show.filtered {
        let mut predicate = lit(true);
        for (index, group) in settings.groups.iter().enumerate() {
            predicate = predicate.and(
                col(&format!("Composition{index}"))
                    .struct_()
                    .field_by_name("Value")
                    .gt(lit(group.filter.value)),
            );
        }
        lazy_frame = lazy_frame.filter(predicate);
    }
    lazy_frame
}

fn sort(mut lazy_frame: LazyFrame, settings: &Settings) -> LazyFrame {
    let mut sort_options = SortMultipleOptions::default();
    if let Order::Descending = settings.order {
        sort_options = sort_options
            .with_order_descending(true)
            .with_nulls_last(true);
    }
    lazy_frame = match settings.sort {
        Sort::Key => lazy_frame.sort_by_exprs(
            [col(r#"^Composition\d$"#).struct_().field_by_name("Key")],
            sort_options,
        ),
        Sort::Value => lazy_frame.sort_by_exprs(
            [col(r#"^Composition\d$"#).struct_().field_by_name("Value")],
            sort_options,
        ),
    };
    lazy_frame
}

// /// Extension methods for [`DataFrame`]
// trait DataFrameExt {
//     fn cartesian_product(&self) -> PolarsResult<Triacylglycerol>;
// }

// impl DataFrameExt for DataFrame {
//     // TODO https://github.com/pola-rs/polars/issues/18587
//     fn cartesian_product(&self) -> PolarsResult<Triacylglycerol> {
//         let lazy_frame = self.clone().lazy().with_row_index("Index", None);
//         // Fatty acid with stereospecific number value
//         let fatty_acid = |name| {
//             as_struct(vec![
//                 col("Index"),
//                 col("FA"),
//                 col("Target")
//                     .struct_()
//                     .field_by_name("Calculated")
//                     .struct_()
//                     .field_by_name(name)
//                     .alias("Value"),
//             ])
//         };
//         Ok(Triacylglycerol(
//             lazy_frame
//                 .clone()
//                 .select([fatty_acid("DAG13").alias("SN1")])
//                 .cross_join(
//                     lazy_frame.clone().select([fatty_acid("MAG2").alias("SN2")]),
//                     None,
//                 )
//                 .cross_join(lazy_frame.select([fatty_acid("DAG13").alias("SN3")]), None)
//                 .select([as_struct(vec![col("SN1"), col("SN2"), col("SN3")]).alias("TAG")]),
//         ))
//     }
//     // fn cartesian_product(&self) -> PolarsResult<Tags> {
//     //     let lazy_frame = self.0.clone().lazy().with_row_index("Index", None);
//     //     Ok(Tags(
//     //         lazy_frame
//     //             .clone()
//     //             .select([fatty_acid("DAG13.Calculated")?.alias("SN1")])
//     //             .cross_join(
//     //                 lazy_frame
//     //                     .clone()
//     //                     .select([fatty_acid("MAG2.Calculated")?.alias("SN2")]),
//     //                 None,
//     //             )
//     //             .cross_join(
//     //                 lazy_frame.select([fatty_acid("DAG13.Calculated")?.alias("SN3")]),
//     //                 None,
//     //             )
//     //             .select([as_struct(vec![col("SN1"), col("SN2"), col("SN3")]).alias("TAG")]),
//     //     ))
//     // }
// }

// fn sort_by_ecn() -> Expr {
//     col("").sort_by(
//         [col("").struct_().field_by_name("FA").fa().ecn()],
//         Default::default(),
//     )
// }

// fn sort_by_mass() -> Expr {
//     col("").sort_by(
//         [col("").struct_().field_by_name("FA").fa().mass()],
//         Default::default(),
//     )
// }

// fn sort_by_type() -> Expr {
//     col("").sort_by(
//         [col("").struct_().field_by_name("FA").fa().saturated()],
//         Default::default(),
//     )
// }

// fn sort_by_species() -> Expr {
//     col("").sort_by(
//         [
//             col("")
//                 .struct_()
//                 .field_by_name("FA")
//                 .struct_()
//                 .field_by_name("Carbons"),
//             col("")
//                 .struct_()
//                 .field_by_name("FA")
//                 .struct_()
//                 .field_by_name("Doubles")
//                 .list()
//                 .len(),
//             col("")
//                 .struct_()
//                 .field_by_name("FA")
//                 .struct_()
//                 .field_by_name("Triples")
//                 .list()
//                 .len(),
//             col("")
//                 .struct_()
//                 .field_by_name("FA")
//                 .struct_()
//                 .field_by_name("Label"),
//             col("").struct_().field_by_name("Index"),
//         ],
//         Default::default(),
//     )
// }

// fn sort_by_unsaturation() -> Expr {
//     col("").sort_by(
//         [col("").struct_().field_by_name("FA").fa().unsaturation()],
//         Default::default(),
//     )
// }

// // Triacylglycerol species
// fn species() -> Expr {
//     concat_str(
//         [col("TAG")
//             .struct_()
//             // .field_by_name("FA")
//             // .struct_()
//             .field_by_names([r#"^SN[1-3]$"#])
//             .fa()
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
