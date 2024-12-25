use self::{
    control::Control,
    table::{CalculationTable, Kind},
};
use crate::{
    app::computers::{CalculationComputed, CalculationKey},
    localization::localize,
    utils::polars::{DataFrameExt, ExprExt as _},
};
use control::Fraction;
use egui::{ComboBox, Id, RichText, Ui};
use egui_phosphor::regular::{ARROWS_HORIZONTAL, GEAR, INTERSECT_THREE};
use polars::prelude::*;
use serde::{Deserialize, Serialize};

/// Calculation pane
#[derive(Deserialize, Serialize)]
pub(crate) struct Pane {
    pub(crate) source: DataFrame,
    pub(crate) target: DataFrame,
    pub(crate) control: Control,
}

impl Pane {
    pub const fn new(data_frame: DataFrame) -> Self {
        Self {
            source: data_frame,
            target: DataFrame::empty(),
            control: Control::new(),
        }
    }

    pub(crate) fn header(&mut self, ui: &mut Ui) {
        ui.visuals_mut().button_frame = false;
        let names = &self.source.get_column_names_str()[1..];
        let selected_text = match self.control.index {
            Some(index) => names[index],
            None => "All",
        };
        ComboBox::from_id_salt(ui.next_auto_id())
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                for index in 0..names.len() {
                    ui.selectable_value(&mut self.control.index, Some(index), names[index]);
                }
                ui.selectable_value(&mut self.control.index, None, "All");
            });
        ui.separator();
        ui.toggle_value(
            &mut self.control.settings.resizable,
            RichText::new(ARROWS_HORIZONTAL).heading(),
        )
        .on_hover_text(localize!("resize"));
        ui.toggle_value(&mut self.control.open, RichText::new(GEAR).heading());
        if ui
            .button(RichText::new(INTERSECT_THREE).heading())
            .on_hover_text(localize!("composition"))
            .clicked()
        {
            let mut exprs = vec![col("Index"), col("FA")];
            println!("self.source: {}", self.source);
            println!("self.target: {}", self.target);
            for &name in &self.source.get_column_names_str()[1..] {
                exprs.push(col(name).destruct(["Calculated"]).alias(name));
            }
            ui.data_mut(|data| {
                data.insert_temp(
                    Id::new("Compose"),
                    self.target.clone().lazy().select(exprs).collect().unwrap(),
                )
            });
        }
    }

    pub(crate) fn content(&mut self, ui: &mut Ui) {
        ui.separator();
        self.control.windows(ui);
        self.target = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<CalculationComputed>()
                .get(CalculationKey {
                    data_frame: &self.source,
                    settings: &self.control.settings,
                })
        });
        match self.control.index {
            Some(index) => {
                let data_frame = self
                    .target
                    .clone()
                    .lazy()
                    .select([
                        col("Index"),
                        col("Key").struct_().field_by_name("FA"),
                        col("Values").struct_().field_by_index(index as _),
                    ])
                    .collect()
                    .unwrap();
                CalculationTable::new(data_frame, Kind::Single, &self.control.settings).ui(ui)
            }
            None => {
                let data_frame = self
                    .target
                    .clone()
                    .lazy()
                    .select([
                        col("Index"),
                        col("Key").struct_().field_by_name("FA"),
                        col("Values").struct_().field_by_name("Mean"),
                    ])
                    .collect()
                    .unwrap();
                CalculationTable::new(data_frame, Kind::Mean, &self.control.settings).ui(ui)
            }
        }
        // let key = &self.target["Key"];
        // let values = self.target.destruct("Values");
        // match self.control.index {
        //     Some(index) => CalculationTable::new(
        //         df! {
        //             "FA" => key.as_series().unwrap(),
        //             "Value" => values[index].as_series().unwrap(),
        //         }
        //         .unwrap(),
        //         Kind::Single,
        //         &self.control.settings,
        //     )
        //     .ui(ui),
        //     None => CalculationTable::new(
        //         df! {
        //             "FA" => key.as_series().unwrap(),
        //             "Mean" => values["Mean"].as_series().unwrap(),
        //         }
        //         .unwrap(),
        //         Kind::Mean,
        //         &self.control.settings,
        //     )
        //     .ui(ui),
        // }
    }

    pub(super) fn hash(&self) -> u64 {
        // hash(&self.source)
        0
    }
}

pub(crate) mod control;

mod table;
