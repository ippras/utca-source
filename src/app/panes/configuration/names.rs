use crate::localize;
use egui::{Grid, Response, Ui, Widget};
use lipid::fatty_acid::{
    FattyAcid,
    display::{DisplayWithOptions, ID},
};

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
            let id = self.fatty_acid.display(ID);
            ui.label(localize!("abbreviation"));
            ui.label(localize!(&format!("{id}.abbreviation")));
            ui.end_row();

            ui.label(localize!("common_name"));
            ui.label(localize!(&format!("{id}.common_name")));
            ui.end_row();

            ui.label(localize!("systematic_name"));
            ui.label(localize!(&format!("{id}.systematic_name")));
            ui.end_row();
        });
        response
    }
}
