use crate::{
    app::text::Text,
    special::composition::{
        Composition, ECNC, MC, PECNC, PMC, PSC, PTC, PUC, SC, SECNC, SMC, SSC, STC, SUC, TC, UC,
    },
};
use egui::{Response, Slider, SliderClamping, TextStyle, Ui, Widget, emath::Float as _};
use egui_ext::LabeledSeparator as _;
use egui_extras::{Column, TableBuilder};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{FUNNEL, FUNNEL_X, HASH};
use indexmap::IndexMap;
use lipid::triacylglycerol::Stereospecificity;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

const DEFAULT: [bool; 3] = [false; 3];

/// Filter
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Filter {
    pub key: IndexMap<AnyValue<'static>, [bool; 3]>,
    pub value: f64,
}

impl Filter {
    pub fn new() -> Self {
        Self {
            key: IndexMap::new(),
            value: 0.0,
        }
    }
}

impl Eq for Filter {}

impl Hash for Filter {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.as_slice().hash(state);
        self.value.ord().hash(state);
    }
}

impl PartialEq for Filter {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value.ord() == other.value.ord()
    }
}

/// Filter widget
pub struct FilterWidget<'a> {
    filter: &'a mut Filter,
    composition: &'a Composition,
    data_frame: &'a DataFrame,
    percent: bool,
}

impl<'a> FilterWidget<'a> {
    pub fn new(
        filter: &'a mut Filter,
        composition: &'a Composition,
        data_frame: &'a DataFrame,
    ) -> Self {
        Self {
            filter,
            composition,
            data_frame,
            percent: false,
        }
    }

    pub fn percent(mut self, percent: bool) -> Self {
        self.percent = percent;
        self
    }
}

impl Widget for FilterWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let title = if *self.filter == Default::default() {
            FUNNEL_X
        } else {
            ui.visuals_mut().widgets.inactive = ui.visuals().widgets.active;
            FUNNEL
        };
        ui.menu_button(title, |ui| -> PolarsResult<()> {
            ui.heading(format!(
                "{} {}",
                self.composition.text(),
                ui.localize("settings-filter?case=lower"),
            ));
            let column = match *self.composition {
                ECNC | PECNC | SECNC => &self.data_frame["EquivalentCarbonNumber"],
                MC | PMC | SMC => &self.data_frame["Mass"],
                SC | PSC | SSC => &self.data_frame["Species"],
                TC | PTC | STC => &self.data_frame["Type"],
                UC | PUC | SUC => &self.data_frame["Unsaturation"],
            };
            let column = column.unique()?.sort(Default::default())?;
            let series = column.as_materialized_series();
            // Key
            ui.labeled_separator("Key");
            let max_scroll_height = ui.spacing().combo_height;
            let height = TextStyle::Body
                .resolve(ui.style())
                .size
                .max(ui.spacing().interact_size.y);
            TableBuilder::new(ui)
                .column(Column::auto().auto_size_this_frame(true))
                .columns(Column::remainder(), 3)
                .max_scroll_height(max_scroll_height)
                .vscroll(true)
                .header(height, |mut header| {
                    header.col(|ui| {
                        ui.heading(HASH);
                    });
                    header.col(|ui| {
                        ui.heading("SN-1");
                    });
                    header.col(|ui| {
                        ui.heading("SN-2");
                    });
                    header.col(|ui| {
                        ui.heading("SN-3");
                    });
                })
                .body(|body| {
                    body.rows(height, column.len(), |mut row| {
                        let index = row.index();
                        row.col(|ui| {
                            ui.label(index.to_string());
                        });
                        // match self.composition.stereospecificity {
                        //     None => todo!(),
                        //     Some(Stereospecificity::Positional) => todo!(),
                        //     Some(Stereospecificity::Stereo) => todo!(),
                        // }
                        row.col(|ui| {
                            ui.add(StereospecificNumberWidget {
                                number: 0,
                                filter: self.filter,
                                index,
                                series,
                            });
                        });
                        row.col(|ui| {
                            ui.add(StereospecificNumberWidget {
                                number: 1,
                                filter: self.filter,
                                index,
                                series,
                            });
                        });
                        row.col(|ui| {
                            ui.add(StereospecificNumberWidget {
                                number: 2,
                                filter: self.filter,
                                index,
                                series,
                            });
                        });
                    });
                });
            // Value
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Value");
                ui.add(
                    Slider::new(&mut self.filter.value, 0.0..=1.0)
                        .clamping(SliderClamping::Always)
                        .logarithmic(true)
                        .custom_formatter(|mut value, _| {
                            if self.percent {
                                value *= 100.0;
                            }
                            AnyValue::Float64(value).to_string()
                        })
                        .custom_parser(|value| {
                            let mut parsed = value.parse::<f64>().ok()?;
                            if self.percent {
                                parsed /= 100.0;
                            }
                            Some(parsed)
                        }),
                );
            });
            Ok(())
        })
        .response
    }
}

struct StereospecificNumberWidget<'a> {
    number: usize,
    filter: &'a mut Filter,
    index: usize,
    series: &'a Series,
}

impl<'a> Widget for StereospecificNumberWidget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let key = self.series.get(self.index).unwrap();
        let text = key.str_value();
        let value = self
            .filter
            .key
            .entry(key.clone().into_static())
            .or_insert(DEFAULT);
        let response = ui.toggle_value(&mut value[self.number], text);
        response.context_menu(|ui| {
            if ui.button(format!("{FUNNEL} Select all")).clicked() {
                for key in self.series.iter() {
                    let av_values: Vec<_> = key._iter_struct_av().collect();
                    let value = self.filter.key.entry(key.into_static()).or_default();
                    value[self.number] = true;
                }
                ui.close_menu();
            }
            if ui.button(format!("{FUNNEL_X} Unselect all")).clicked() {
                for value in self.filter.key.values_mut() {
                    value[self.number] = false;
                }
                ui.close_menu();
            }
        });
        response
    }
}

/// Extension methods for [`Response`]
trait ResponseExt {
    fn or_union(&mut self, other: Response);

    fn unwrap_and_union(self, other: Response) -> Response;
}

impl ResponseExt for Option<Response> {
    fn or_union(&mut self, other: Response) {
        *self = Some(self.take().unwrap_and_union(other));
    }

    fn unwrap_and_union(self, other: Response) -> Response {
        match self {
            Some(outer_response) => outer_response | other,
            None => other,
        }
    }
}
