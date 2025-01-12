use crate::special::fatty_acid::FattyAcid;
use egui::{DragValue, TextEdit, Ui, style::Widgets};
use egui_phosphor::regular::{MINUS, PLUS};

/// Fatty acid widget
pub(crate) struct FattyAcidWidget<'a> {
    pub(crate) label: &'a mut String,
    pub(crate) fatty_acid: &'a mut FattyAcid,
}

impl<'a> FattyAcidWidget<'a> {
    pub(crate) fn new(label: &'a mut String, fatty_acid: &'a mut FattyAcid) -> Self {
        Self { label, fatty_acid }
    }
}

impl FattyAcidWidget<'_> {
    pub(crate) fn ui(self, ui: &mut Ui) -> Option<Change> {
        ui.visuals_mut().widgets = if ui.style().visuals.dark_mode {
            Widgets::dark()
        } else {
            Widgets::light()
        };
        let mut change = None;
        // Label
        ui.horizontal(|ui| {
            ui.label("Label");
            if TextEdit::singleline(self.label)
                .hint_text("C")
                .desired_width(ui.available_width())
                .show(ui)
                .response
                .changed()
            {
                change = Some(Change::Label);
            }
        });
        // Carbons
        ui.horizontal(|ui| {
            ui.label("Carbons");
            if ui
                .add(DragValue::new(&mut self.fatty_acid.carbons))
                .changed()
            {
                change = Some(Change::Carbons);
            }
        });
        // Doubles
        ui.horizontal(|ui| {
            ui.label("Doubles");
            if !self.fatty_acid.doubles.is_empty() {
                if ui.button(MINUS).clicked() {
                    self.fatty_acid.doubles.pop();
                    change = Some(Change::Doubles);
                }
            }
            let end = self.fatty_acid.b();
            for bound in &mut self.fatty_acid.doubles {
                if ui.add(DragValue::new(bound).range(0..=end)).changed() {
                    change = Some(Change::Doubles);
                }
            }
            if ui.button(PLUS).clicked() {
                if self.fatty_acid.b() > self.fatty_acid.u() {
                    self.fatty_acid.doubles.push(0);
                    change = Some(Change::Doubles);
                }
            }
        });
        // Triples
        ui.horizontal(|ui| {
            let mut response = ui.label("Triples");
            if !self.fatty_acid.triples.is_empty() {
                if ui.button(MINUS).clicked() {
                    self.fatty_acid.triples.pop();
                    response.mark_changed();
                }
            }
            let end = self.fatty_acid.b();
            for bound in &mut self.fatty_acid.triples {
                response |= ui.add(DragValue::new(bound).range(0..=end));
            }
            if ui.button(PLUS).clicked() {
                if self.fatty_acid.b() > self.fatty_acid.u() {
                    self.fatty_acid.triples.push(0);
                    response.mark_changed();
                }
            }
            if response.changed() {
                change = Some(Change::Triples);
            }
        });
        change
    }
}

pub(crate) enum Change {
    Label,
    Carbons,
    Doubles,
    Triples,
}
