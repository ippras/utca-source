use crate::{
    localization::{localize, lowercase},
    special::fatty_acid::FattyAcid,
};
use egui::{Grid, Response, Ui, Widget};

/// Names
pub(crate) struct Names<'a> {
    pub(crate) fatty_acid: &'a FattyAcid,
}

impl<'a> Names<'a> {
    pub(crate) fn new(fatty_acid: &'a mut FattyAcid) -> Self {
        Self { fatty_acid }
    }
}

impl Widget for Names<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let response = ui.heading(localize!("names"));
        Grid::new(ui.next_auto_id()).show(ui, |ui| {
            let id = self.fatty_acid.id();
            if let Some(abbreviation) = lowercase!(&format!("fa_{id:#02}.abbreviation")) {
                ui.label(localize!("abbreviation"));
                ui.label(abbreviation);
                ui.end_row();
            }
            if let Some(common_name) = lowercase!(&format!("fa_{id:#02}.common_name")) {
                ui.label(localize!("common_name"));
                ui.label(common_name);
                ui.end_row();
            }
            if let Some(systematic_name) = lowercase!(&format!("fa_{id:#02}.systematic_name")) {
                ui.label(localize!("systematic_name"));
                ui.label(systematic_name);
            }
        });
        response
    }
}
