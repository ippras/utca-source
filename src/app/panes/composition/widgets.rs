use crate::{
    app::widgets::FloatWidget,
    localization::localize,
    utils::polars::{ColumnExt, SeriesExt},
};
use egui::{Align2, Color32, Grid, Response, RichText, ScrollArea, TextStyle, Ui, Widget};
use egui_phosphor::regular::LIST;
use polars::prelude::*;
use std::{convert::identity, iter::zip};

/// Cell widget
pub(crate) struct Cell<'a> {
    pub(crate) row: usize,
    pub(crate) column: &'a Column,
    pub(crate) percent: bool,
    pub(crate) precision: usize,
}

impl Widget for Cell<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let r#struct = self.column.struct_().unwrap();
        let fields = r#struct.fields_as_series();
        let values = &fields[0..fields.len() - 2];
        let species = r#struct.field_by_name("Species").unwrap();
        let filter = r#struct.field_by_name("Filter").unwrap();
        let values = values
            .into_iter()
            .map(|field| (field.name(), field.f64().unwrap().get(self.row)));
        let filter = filter.bool().unwrap().get(self.row).is_some_and(identity);
        ui.add(Species {
            species: species.list().unwrap().get_as_series(self.row),
            percent: self.percent,
        });
        let value = values.clone().next_back().and_then(|(_, value)| value);
        let mut response = ui.add_enabled(
            !filter,
            FloatWidget::new(|| Ok(value))
                .percent(self.percent)
                .precision(Some(self.precision))
                .disable(true),
        );
        if value.is_some() {
            response = response
                .on_hover_ui(|ui| {
                    ui.add(Values {
                        values: values.clone(),
                        percent: self.percent,
                    });
                })
                .on_disabled_hover_ui(|ui| {
                    ui.add(Values {
                        values,
                        percent: self.percent,
                    });
                });
        }
        response
    }
}

/// Values
struct Values<T> {
    pub(crate) values: T,
    pub(crate) percent: bool,
}

impl<'a, T: Iterator<Item = (&'a PlSmallStr, Option<f64>)>> Widget for Values<T> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.heading(localize!("values"));
        Grid::new(ui.next_auto_id())
            .show(ui, |ui| {
                for (name, value) in self.values {
                    ui.label(name.to_string());
                    ui.add(FloatWidget::new(|| Ok(value)).percent(self.percent));
                    ui.end_row();
                }
            })
            .response
    }
}

/// Species widget
struct Species {
    pub(crate) species: Option<Series>,
    pub(crate) percent: bool,
}

impl Widget for Species {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.visuals_mut().button_frame = false;
        let len = self
            .species
            .as_ref()
            .map(|series| series.len())
            .unwrap_or_default();
        let response = ui
            .menu_button(LIST, |ui| {
                ui.heading(localize!("species"));
                ScrollArea::vertical().show(ui, |ui| {
                    Grid::new(ui.next_auto_id()).show(ui, |ui| {
                        if let Some(series) = self.species {
                            let fields = series.r#struct().fields_as_series();
                            let species = fields[0].str().unwrap();
                            let value = fields[1].f64().unwrap();
                            for (species, value) in zip(species, value) {
                                let text = if let Some(species) = species {
                                    RichText::new(species)
                                } else {
                                    RichText::new("None").color(Color32::RED)
                                };
                                ui.label(text);
                                ui.add(FloatWidget::new(|| Ok(value)).percent(self.percent));
                                ui.end_row();
                            }
                        }
                    });
                });
            })
            .response;
        ui.painter().text(
            response.rect.center(),
            Align2::LEFT_BOTTOM,
            len.to_string(),
            TextStyle::Small.resolve(ui.style()),
            ui.visuals().text_color(),
            // ui.visuals().widgets.active.text_color(),
        );
        response.on_hover_text(len.to_string())
    }
}
