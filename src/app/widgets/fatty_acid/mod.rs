use crate::{app::ResultExt, localize, try_localize};
use egui::{Align, DragValue, Grid, InnerResponse, Layout, Ui, Widget, style::Widgets, vec2};
use lipid::fatty_acid::{
    FattyAcid, FattyAcidExt as _, Isomerism, Unsaturated, Unsaturation,
    display::{COMMON, DisplayWithOptions, ID},
};
use polars::prelude::*;
use std::{cmp::Ordering, mem::replace};

/// Fatty acid widget
pub(crate) struct FattyAcidWidget<'a> {
    pub(crate) value: Box<dyn Fn() -> PolarsResult<Option<FattyAcid>> + 'a>,
    pub(crate) editable: bool,
    pub(crate) hover: bool,
    pub(crate) names: bool,
}

impl<'a> FattyAcidWidget<'a> {
    pub(crate) fn new(value: impl Fn() -> PolarsResult<Option<FattyAcid>> + 'a) -> Self {
        Self {
            value: Box::new(value),
            editable: false,
            hover: false,
            names: false,
        }
    }

    pub(crate) fn editable(self, editable: bool) -> Self {
        Self { editable, ..self }
    }

    pub(crate) fn hover(self) -> Self {
        Self {
            hover: true,
            ..self
        }
    }

    pub(crate) fn names(self, names: bool) -> Self {
        Self { names, ..self }
    }
}

impl FattyAcidWidget<'_> {
    pub(crate) fn try_show(self, ui: &mut Ui) -> InnerResponse<PolarsResult<Option<FattyAcid>>> {
        let mut inner = (self.value)();
        let Ok(Some(mut fatty_acid)) = replace(&mut inner, Ok(None)) else {
            // Null
            let response = ui.label(AnyValue::Null.to_string());
            return InnerResponse::new(inner, response);
        };
        let text = &format!("{:#}", (&fatty_acid).display(COMMON));
        let mut response = if self.editable {
            // Writable
            ui.add_sized(
                vec2(ui.available_width(), ui.spacing().interact_size.y),
                |ui: &mut Ui| {
                    ui.menu_button(text, |ui| {
                        let widgets = if ui.visuals().dark_mode {
                            Widgets::dark()
                        } else {
                            Widgets::light()
                        };
                        ui.visuals_mut().widgets.inactive.weak_bg_fill =
                            widgets.active.weak_bg_fill;
                        // let mut fatty_acid = fatty_acid.clone();
                        Grid::new(ui.next_auto_id()).show(ui, |ui| {
                            // Carbons
                            ui.label("Carbons");
                            if ui.add(DragValue::new(&mut fatty_acid.carbons)).changed() {
                                inner = Ok(Some(fatty_acid.clone()));
                            }
                            ui.end_row();

                            // Unsaturated
                            ui.collapsing("Unsaturated", |ui| {
                                let bounds = fatty_acid.bounds();
                                let mut changed = false;
                                for unsaturated in &mut fatty_acid.unsaturated {
                                    Grid::new(ui.next_auto_id()).show(ui, |ui| {
                                        // Index
                                        changed |= DragValue::new(
                                            unsaturated.index.get_or_insert_default(),
                                        )
                                        .range(0..=bounds)
                                        .custom_formatter(|value, _| {
                                            if value != 0.0 {
                                                value.to_string()
                                            } else {
                                                "*".to_owned()
                                            }
                                        })
                                        .clamp_existing_to_range(true)
                                        .update_while_editing(false)
                                        .ui(ui)
                                        .changed();
                                        // Isomerism
                                        let text = match &unsaturated.isomerism {
                                            Some(Isomerism::Cis) => "C",
                                            Some(Isomerism::Trans) => "T",
                                            None => "*",
                                        };
                                        if ui.button(text).clicked() {
                                            unsaturated.isomerism = match unsaturated.isomerism {
                                                None => Some(Isomerism::Cis),
                                                Some(Isomerism::Cis) => Some(Isomerism::Trans),
                                                Some(Isomerism::Trans) => None,
                                            };
                                            changed = true;
                                        }
                                        // Unsaturation
                                        let text = match &unsaturated.unsaturation {
                                            Some(Unsaturation::One) => "1",
                                            Some(Unsaturation::Two) => "2",
                                            None => "*",
                                        };
                                        if ui.button(text).clicked() {
                                            unsaturated.unsaturation = match unsaturated
                                                .unsaturation
                                            {
                                                None => Some(Unsaturation::One),
                                                Some(Unsaturation::One) => Some(Unsaturation::Two),
                                                Some(Unsaturation::Two) => None,
                                            };
                                            changed = true;
                                        }
                                    });
                                    ui.end_row();
                                }
                                if changed {
                                    fatty_acid.unsaturated.sort_by_cached_key(|unsaturated| {
                                        (
                                            unsaturated.index,
                                            unsaturated.isomerism,
                                            unsaturated.unsaturation,
                                        )
                                    });
                                    inner = Ok(Some(fatty_acid.clone()));
                                }
                            });
                            let mut unsaturated = fatty_acid.unsaturated.len();
                            ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                                if ui
                                    .add(
                                        DragValue::new(&mut unsaturated)
                                            .range(0..=fatty_acid.carbons)
                                            .clamp_existing_to_range(true),
                                    )
                                    .changed()
                                {
                                    loop {
                                        match unsaturated.cmp(&fatty_acid.unsaturated.len()) {
                                            Ordering::Less => {
                                                fatty_acid.unsaturated.pop();
                                            }
                                            Ordering::Equal => break,
                                            Ordering::Greater => {
                                                fatty_acid.unsaturated.push(Unsaturated {
                                                    index: Some(0),
                                                    isomerism: Some(Isomerism::Cis),
                                                    unsaturation: Some(Unsaturation::One),
                                                });
                                            }
                                        }
                                    }
                                    inner = Ok(Some(fatty_acid.clone()));
                                }
                            });
                        });
                    })
                    .response
                },
            )
        } else {
            // Readable
            ui.label(text)
        };
        // Hover
        if self.hover {
            response = response.on_hover_text(text);
            if self.names {
                response = response.on_hover_ui(|ui| {
                    ui.heading(localize!("names"));
                    Grid::new(ui.next_auto_id()).show(ui, |ui| {
                        let id = fatty_acid.display(ID);
                        if let Some(abbreviation) = try_localize!(&format!("{id:#}.abbreviation")) {
                            ui.label(localize!("abbreviation"));
                            ui.label(abbreviation);
                            ui.end_row();
                        }

                        if let Some(common_name) = try_localize!(&format!("{id:#}.common_name")) {
                            ui.label(localize!("common_name"));
                            ui.label(common_name);
                            ui.end_row();
                        }

                        if let Some(systematic_name) =
                            try_localize!(&format!("{id:#}.systematic_name"))
                        {
                            ui.label(localize!("systematic_name"));
                            ui.label(systematic_name);
                            ui.end_row();
                        }
                    });
                });
            }
        }
        InnerResponse::new(inner, response)
    }

    pub(crate) fn show(self, ui: &mut Ui) -> InnerResponse<Option<FattyAcid>> {
        let InnerResponse { inner, response } = self.try_show(ui);
        let inner = inner.context(ui.ctx()).flatten();
        InnerResponse::new(inner, response)
    }
}

impl Widget for FattyAcidWidget<'_> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        self.show(ui).response
    }
}
