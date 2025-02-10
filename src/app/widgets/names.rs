use crate::{localize, try_localize};
use egui::{Grid, Response, Ui, Widget};
use lipid::fatty_acid::{
    FattyAcid,
    display::{DisplayWithOptions as _, ID},
};

/// Names widget
pub(crate) struct NamesWidget<'a> {
    fatty_acid: &'a FattyAcid,
}

impl<'a> NamesWidget<'a> {
    pub(crate) fn new(fatty_acid: &'a FattyAcid) -> Self {
        Self { fatty_acid }
    }
}

impl Widget for NamesWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.heading(localize!("names"));
        Grid::new(ui.next_auto_id())
            .show(ui, |ui| {
                let id = self.fatty_acid.display(ID);
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

                if let Some(systematic_name) = try_localize!(&format!("{id:#}.systematic_name")) {
                    ui.label(localize!("systematic_name"));
                    ui.label(systematic_name);
                    ui.end_row();
                }
            })
            .response
    }
}
