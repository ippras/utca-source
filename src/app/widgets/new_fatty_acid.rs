use egui::{Align, DragValue, Grid, InnerResponse, Layout, Ui, Widget, vec2};
use lipid::fatty_acid::{
    FattyAcid, FattyAcidExt as _, Isomerism, Unsaturated, Unsaturation,
    display::{COMMON, DisplayWithOptions},
};
use polars::prelude::*;
use std::cmp::Ordering;

/// Fatty acid widget
pub(crate) struct FattyAcidWidget<'a> {
    pub(crate) value: Box<dyn Fn() -> PolarsResult<Option<FattyAcid>> + 'a>,
    pub(crate) editable: bool,
    pub(crate) hover: bool,
}

impl<'a> FattyAcidWidget<'a> {
    pub(crate) fn new(value: impl Fn() -> PolarsResult<Option<FattyAcid>> + 'a) -> Self {
        Self {
            value: Box::new(value),
            editable: false,
            hover: false,
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
}

impl FattyAcidWidget<'_> {
    pub(crate) fn ui(self, ui: &mut Ui) -> PolarsResult<InnerResponse<Option<FattyAcid>>> {
        let fatty_acid = (self.value)()?;
        let text = match &fatty_acid {
            Some(fatty_acid) => &format!("{:#}", fatty_acid.display(COMMON)),
            None => "",
        };
        let mut inner = None;
        let mut response = if self.editable {
            ui.add_sized(
                vec2(ui.available_width(), ui.style().spacing.interact_size.y),
                |ui: &mut Ui| {
                    ui.menu_button(text, |ui| {
                        let mut fatty_acid = fatty_acid.unwrap_or_default();
                        Grid::new(ui.next_auto_id()).show(ui, |ui| {
                            // Carbons
                            ui.label("Carbons");
                            if DragValue::new(&mut fatty_acid.carbons).ui(ui).changed() {
                                inner = Some(fatty_acid.clone());
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
                                    inner = Some(fatty_acid.clone());
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
                                    inner = Some(fatty_acid.clone());
                                }
                            });
                        });
                    })
                    .response
                },
            )
        } else {
            ui.label(text)
        };
        if self.hover {
            response = response.on_hover_text(text);
        }
        Ok(InnerResponse::new(inner, response))
    }
}
