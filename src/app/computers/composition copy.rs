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
use egui::util::cache::{ComputerMut, FrameCache};
use polars::prelude::*;
use std::{
    hash::{Hash, Hasher},
    process::exit,
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
        // Filter
        tags = tags.filter(settings);
        // Compose
        tags.composition(settings)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        match key.settings.method {
            Method::Gunstone => unreachable!(),
            Method::VanderWal => {
                // let mut exprs = vec![col("Index"), col("FA")];
                // for column in &key.data_frame.get_columns()[2..] {
                //     let name = column.name().as_str();
                //     exprs.push(
                //         as_struct(vec![
                //             col(name).struct_().field_by_name("DAG13").alias("DAG13"),
                //             col(name).struct_().field_by_name("MAG2").alias("MAG2"),
                //         ])
                //         .alias(name),
                //     );
                //     println!("1: {}", key.data_frame.a_destruct(&[name]));
                // }
                println!("0: {}", key.data_frame);

                let mut tags = new_cartesian_product(key.data_frame.clone().lazy()).unwrap();
                println!("tags: {:?}", tags.0.clone().collect().unwrap());
                // col("TAG").tag().value()
                let lazy_frame = tags
                    .0
                    .join(
                        key.data_frame.clone().lazy().select([
                            col("Index"),
                            col("FA"),
                            col("PolysciasUnknown.1.utca.ron")
                                .struct_()
                                .field_by_name("DAG13")
                                .alias("SN1"),
                        ]),
                        [
                            col("TAG").tag().sn1().index(),
                            col("TAG").tag().sn1().fa().into(),
                        ],
                        [col("Index"), col("FA")],
                        JoinArgs::new(JoinType::Left),
                    )
                    .select(exprs)
                    .cache()
                    .drop([col("Index"), col("FA")]);
                // let lazy_frame = lazy_frame.join(
                //     key.data_frame.clone().lazy().select([
                //         col("Index"),
                //         col("FA"),
                //         col("PolysciasUnknown.1.utca.ron")
                //             .struct_()
                //             .field_by_name("MAG2")
                //             .alias("SN2"),
                //     ]),
                //     [
                //         col("TAG").tag().sn2().index(),
                //         col("TAG").tag().sn2().fa().into(),
                //     ],
                //     [col("Index"), col("FA")],
                //     JoinArgs::new(JoinType::Left),
                // );
                // let lazy_frame = lazy_frame.join(
                //     key.data_frame.clone().lazy().select([
                //         col("Index"),
                //         col("FA"),
                //         col("PolysciasUnknown.1.utca.ron")
                //             .struct_()
                //             .field_by_name("DAG13")
                //             .alias("SN3"),
                //     ]),
                //     [
                //         col("TAG").tag().sn3().index(),
                //         col("TAG").tag().sn3().fa().into(),
                //     ],
                //     [col("Index"), col("FA")],
                //     JoinArgs::new(JoinType::Left),
                // );
                // .cache()
                // .drop([col("Index"), col("FA")]);
                // .with_column(as_struct(vec![col("Index"), col("FA")]).alias("SN1"))
                // .drop([col("Index"), col("FA")]);
                // let lazy_frame = lazy_frame.unnest([col("SN2")]).left_join(
                //     key.data_frame.clone().lazy().select([
                //         col("Index"),
                //         col("PolysciasUnknown.1.utca.ron")
                //             .struct_()
                //             .field_by_name("DAG13"),
                //     ]),
                //     col("Index"),
                //     col("Index"),
                // );
                // lazy_frame.
                println!("join: {:?}", lazy_frame.clone().collect().unwrap());
                // for column in &key.data_frame.get_columns()[2..] {
                //     // let mut tags =
                //     //     cartesian_product(key.data_frame.clone().lazy(), column.name().as_str())
                //     //         .unwrap();
                //     // Value
                //     // self.0.with_column(col("TAG" ).tag().value())
                //     tags = tags.value();
                //     println!("1: {}", tags.0.clone().collect().unwrap());
                //     // Filter
                //     tags = tags.filter(key.settings);
                //     println!("2: {}", tags.0.clone().collect().unwrap());
                //     // Compose
                //     // tags = tags.composition(key.settings).unwrap();
                //     // println!("3: {}", tags.0.clone().collect().unwrap());
                //     // ┌──────────────────┬──────────┬──────────┬──────────────────────────────────────────────────────┬────────┐
                //     // │ Composition      ┆ Value0   ┆ Value1   ┆ Species                                              ┆ Filter │
                //     // │ ---              ┆ ---      ┆ ---      ┆ ---                                                  ┆ ---    │
                //     // │ struct[2]        ┆ f64      ┆ f64      ┆ list[struct[3]]                                      ┆ bool   │
                //     // ╞══════════════════╪══════════╪══════════╪══════════════════════════════════════════════════════╪════════╡
                //     // │ {"SUU","LiLiBe"} ┆ 0.152838 ┆ 0.00056  ┆ [{"LiLiBe",0.00028,true}, {"BeLiLi",0.00028,true}]   ┆ true   │
                //     // │ {"SSU","OlBeAr"} ┆ 0.005104 ┆ 0.000009 ┆ [{"OlBeAr",0.000004,true}, {"ArBeOl",0.000004,true}] ┆ true   │
                //     // │ {"USU","LiBeGa"} ┆ 0.000241 ┆ 0.000003 ┆ [{"LiBeGa",0.000002,true}, {"GaBeLi",0.000002,true}] ┆ true   │
                //     // │ {"UUU","OlOlOl"} ┆ 0.007231 ┆ 0.000077 ┆ [{"OlOlOl",0.000077,true}]                           ┆ true   │
                //     // │ {"SUU","StLiGa"} ┆ 0.152838 ┆ 0.000611 ┆ [{"StLiGa",0.000305,true}, {"GaLiSt",0.000305,true}] ┆ true   │
                //     // │ …                ┆ …        ┆ …        ┆ …                                                    ┆ …      │
                //     // │ {"USU","LnStGa"} ┆ 0.000241 ┆ 0.000005 ┆ [{"LnStGa",0.000002,true}, {"GaStLn",0.000002,true}] ┆ true   │
                //     // │ {"SUU","LnLiBe"} ┆ 0.152838 ┆ 0.001193 ┆ [{"LnLiBe",0.000596,true}, {"BeLiLn",0.000596,true}] ┆ true   │
                //     // │ {"SUU","LiLiAr"} ┆ 0.152838 ┆ 0.000719 ┆ [{"LiLiAr",0.00036,true}, {"ArLiLi",0.00036,true}]   ┆ true   │
                //     // │ {"SSU","LnPaBe"} ┆ 0.005104 ┆ 0.000045 ┆ [{"LnPaBe",0.000022,true}, {"BePaLn",0.000022,true}] ┆ true   │
                //     // │ {"SSU","StPaLn"} ┆ 0.005104 ┆ 0.000126 ┆ [{"StPaLn",0.000063,true}, {"LnPaSt",0.000063,true}] ┆ true   │
                //     // └──────────────────┴──────────┴──────────┴──────────────────────────────────────────────────────┴────────┘
                // }

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
                //     // Sort
                //     lazy_frame = lazy_frame.compositions().sort(key.settings);
                //     // Meta
                //     lazy_frame = lazy_frame.compositions().meta(key.settings).unwrap();
                //     println!("Meta: {}", lazy_frame.clone().collect().unwrap());
                //     return lazy_frame.collect().unwrap();
                // }
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

fn new_cartesian_product(lazy_frame: LazyFrame) -> PolarsResult<Tags> {
    // Tags with stereospecific number values
    Ok(Tags(
        lazy_frame
            .clone()
            .select([as_struct(vec![col("Index"), col("FA")]).alias("SN1")])
            .cross_join(
                lazy_frame
                    .clone()
                    .select([as_struct(vec![col("Index"), col("FA")]).alias("SN2")]),
                None,
            )
            .cross_join(
                lazy_frame
                    .clone()
                    .select([as_struct(vec![col("Index"), col("FA")]).alias("SN3")]),
                None,
            )
            .select([as_struct(vec![col("SN1"), col("SN2"), col("SN3")]).alias("TAG")]),
    ))
}

fn cartesian_product(lazy_frame: LazyFrame, name: &str) -> PolarsResult<Tags> {
    // Tags with stereospecific number values
    Ok(Tags(
        lazy_frame
            .clone()
            .select([as_struct(vec![
                col("Index"),
                col("FA"),
                col(name).struct_().field_by_name("DAG13").alias("Value"),
            ])
            .alias("SN1")])
            .join(
                lazy_frame.clone().select([as_struct(vec![
                    col("Index"),
                    col("FA"),
                    col(name).struct_().field_by_name("MAG2").alias("Value"),
                ])
                .alias("SN2")]),
                [],
                [],
                JoinArgs::new(JoinType::Cross),
            )
            .join(
                lazy_frame.clone().select([as_struct(vec![
                    col("Index"),
                    col("FA"),
                    col(name).struct_().field_by_name("DAG13").alias("Value"),
                ])
                .alias("SN3")]),
                [],
                [],
                JoinArgs::new(JoinType::Cross),
            )
            .select([as_struct(vec![col("SN1"), col("SN2"), col("SN3")]).alias("TAG")]),
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

/// Tags
struct Tags(LazyFrame);

impl Tags {
    fn composition(self, settings: &Settings) -> PolarsResult<Self> {
        if settings.groups.is_empty() {
            return Ok(self);
        }
        let mut lazy_frame = self.0;
        // Composition
        for (index, group) in settings.groups.iter().enumerate() {
            // lazy_frame = lazy_frame.with_columns([col("TAG").tag().compose(group.composition)?]);
            println!(
                "lazy_frame g0!!!: {}",
                lazy_frame.clone().collect().unwrap()
            );
            // // Temp stereospecific numbers
            // lazy_frame = lazy_frame.with_columns([
            //     col("TAG").struct_().field_by_name("SN1"),
            //     col("TAG").struct_().field_by_name("SN2"),
            //     col("TAG").struct_().field_by_name("SN3"),
            // ]);
            // println!("lazy_frame g0: {}", lazy_frame.clone().collect().unwrap());
            // // Stereospecificity permutation
            // let sort = match group.composition.kind {
            //     Kind::Ecn => sort_by_ecn(),
            //     Kind::Mass => sort_by_mass(),
            //     Kind::Type => sort_by_type(),
            //     Kind::Species => sort_by_species(),
            //     Kind::Unsaturation => sort_by_unsaturation(),
            // };
            // lazy_frame = match group.composition.stereospecificity {
            //     None => lazy_frame.permutation(["SN1", "SN2", "SN3"], sort)?,
            //     Some(Stereospecificity::Positional) => {
            //         lazy_frame.permutation(["SN1", "SN3"], sort)?
            //     }
            //     Some(Stereospecificity::Stereo) => lazy_frame,
            // };
            // println!("lazy_frame g1: {}", lazy_frame.clone().collect().unwrap());
            // lazy_frame = lazy_frame.with_column(
            //     match group.composition.kind {
            //         Kind::Mass => {
            //             let rounded = |expr: Expr| expr.round(settings.precision as _);
            //             match group.composition.stereospecificity {
            //                 None => rounded(col("TAG").tag().mass(*settings.adduct)),
            //                 _ => concat_str(
            //                     [
            //                         lit("["),
            //                         rounded(col("TAG").tag().sn1().fa().mass()),
            //                         lit("|"),
            //                         rounded(col("TAG").tag().sn2().fa().mass()),
            //                         lit("|"),
            //                         rounded(col("TAG").tag().sn3().fa().mass()),
            //                         lit("]"),
            //                         lit(*settings.adduct).round(settings.precision as _),
            //                     ],
            //                     "",
            //                     false,
            //                 ),
            //             }
            //         }
            //         Kind::Ecn => match group.composition.stereospecificity {
            //             None => col("TAG").tag().ecn(),
            //             _ => concat_str(
            //                 [
            //                     lit("["),
            //                     col("TAG").tag().sn1().fa().ecn(),
            //                     lit("|"),
            //                     col("TAG").tag().sn2().fa().ecn(),
            //                     lit("|"),
            //                     col("TAG").tag().sn3().fa().ecn(),
            //                     lit("]"),
            //                 ],
            //                 "",
            //                 false,
            //             ),
            //         },
            //         Kind::Type => concat_str([col("TAG").tag().sn().fa().r#type()], "", false),
            //         Kind::Species => concat_str([col("TAG").tag().sn().fa().label()], "", false),
            //         Kind::Unsaturation => {
            //             concat_str([col("TAG").tag().sn().fa().unsaturation()], "", false)
            //         }
            //     }
            //     .alias(format!("Composition{index}")),
            // );
            let expr = col("TAG").tag().compose(group.composition)?;
            lazy_frame = lazy_frame.with_column(
                match group.composition {
                    MC => expr.list().sum(),
                    PMC => expr,
                    SMC => expr,
                    NC => expr.list().sum(),
                    PNC => expr,
                    SNC => expr,
                    SC => expr.list().join(lit(""), false),
                    PSC => expr.list().join(lit(""), false),
                    SSC => expr.list().join(lit(""), false),
                    TC => expr.list().join(lit(""), false),
                    PTC => expr.list().join(lit(""), false),
                    STC => expr.list().join(lit(""), false),
                    UC => expr.list().sum(),
                    PUC => expr,
                    SUC => expr,
                }
                .alias(format!("Composition{index}")),
            );
            // Value
            let value = format!("Value{index}");
            let compositions = format!(r#"^Composition[0-{index}]$"#);
            lazy_frame = lazy_frame
                .group_by([col(&compositions)])
                .agg([all(), col("Value").sum().alias(&value)])
                .with_column(col(&value).lt_eq(lit(group.filter.value)).alias("Filter"))
                .explode([all().exclude([&compositions, &value, "Filter"])]);
            // Filter by composition
            // if let Some(&filter) = settings.filters.0.get(&group.composition) {
            //     let composition = format!("Composition{index}");
            //     lazy_frame = lazy_frame
            //         .group_by([col(&composition)])
            //         .agg([
            //             all().exclude(["Filter"]),
            //             col("Filter").or(col("Value").sum().gt(lit(filter)).alias("Filter")),
            //         ])
            //         .explode([all().exclude([&composition, "Filter"])]);
            //     println!("lazy_frame IN: {}", lazy_frame.clone().collect().unwrap());
            // }
        }
        // Species
        lazy_frame = lazy_frame
            .with_column(col("TAG").tag().species().alias("Species"))
            .drop(["TAG"]);
        // for (index, group) in settings.groups.iter().enumerate() {
        //     lazy_frame =
        //         lazy_frame.group_by([col(&compositions)]).filter(col("Value").gt_eq(lit(settings.filters.0[&group.composition])));
        // }
        // Values
        // for index in 0..indices.len() {
        //     let value = format!("Value{index}");
        //     let compositions = format!(r#"^Composition[0-{index}]$"#);
        //     lazy_frame = lazy_frame
        //         .group_by([col(&compositions)])
        //         .agg([all(), col("Value").sum().alias(&value)])
        //         .filter(col(&value).gt_eq(lit(settings.groups[index].filter.value)))
        //         .explode([all().exclude([&compositions]).exclude([&value])]);
        //     if let Some(value) = settings.filters.0.get(&settings.groups[index].composition) {
        //         let composition = format!("Composition{index}");
        //         lazy_frame = lazy_frame
        //             .group_by([col(composition)])
        //             .agg([all(), col("Value").sum().alias(&value)]);
        //     }
        // }

        // Nest species
        lazy_frame = lazy_frame
            .with_column(as_struct(vec![col("Species"), col("Value"), col("Filter")]))
            .drop(["Value"]);
        // Group leaves (species)
        lazy_frame = lazy_frame
            .group_by([col(r#"^Composition\d$"#), col(r#"^Value\d$"#)])
            .agg([all().exclude(["Filter"]), col("Filter").all(true)]);
        // println!("lazy_framex: {}", lazy_frame.clone().collect().unwrap());
        // Nest compositions and values
        lazy_frame = lazy_frame.select([
            as_struct(vec![col(r#"^Composition\d$"#)]).alias("Composition"),
            as_struct(vec![col(r#"^Value\d$"#), col("Species"), col("Filter")]).alias("Values"),
        ]);
        // println!("lazy_frame3: {}", lazy_frame.clone().collect().unwrap());
        Ok(Self(lazy_frame))
    }

    fn filter(self, settings: &Settings) -> Self {
        let mut lazy_frame = self.0;
        if !settings.show.nulls {
            lazy_frame = lazy_frame.filter(col("Value").neq(lit(0)));
        }
        Self(lazy_frame)
    }

    fn value(self) -> Self {
        Self(self.0.with_column(col("TAG").tag().value()))
    }
}

/// Compositions
struct Compositions(LazyFrame);

impl Compositions {
    fn meta(self, settings: &Settings) -> PolarsResult<LazyFrame> {
        let lazy_frame = self.0.with_row_index("Index", None);
        let expr = concat_list([all()
            .exclude(["Index", r#"^Composition\d$"#])
            .struct_()
            .field_by_name(&format!("Value{}", settings.groups.len().saturating_sub(1)))])?;
        Ok(lazy_frame
            .with_columns([
                expr.clone().list().mean().alias("Mean"),
                expr.clone().list().std(settings.meta.ddof).alias("Std"),
                expr.clone().list().var(settings.meta.ddof).alias("Var"),
            ])
            .select([
                as_struct(vec![col("Index"), col("Mean"), col("Std"), col("Var")]).alias("Meta"),
                all().exclude(["Index", "Mean", "Std", "Var"]),
            ]))
    }

    fn sort(mut self, settings: &Settings) -> LazyFrame {
        let mut sort_options = SortMultipleOptions::default();
        if let Order::Descending = settings.order {
            sort_options = sort_options
                .with_order_descending(true)
                .with_nulls_last(true);
        }
        self.0 = match settings.sort {
            Sort::Key => self
                .0
                .sort_by_exprs([col(r#"^Composition\d$"#)], sort_options),
            Sort::Value => {
                let value = all()
                    .exclude([r#"^Composition\d$"#])
                    .struct_()
                    .field_by_names([r#"^Value\d$"#]);
                self.0.sort_by_exprs([value], sort_options)
            }
        };
        self.0
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
