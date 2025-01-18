use super::State;
use crate::{app::MAX_PRECISION, localize};
use egui::{ComboBox, Grid, Key, KeyboardShortcut, Modifiers, RichText, Slider, Ui};
use egui_phosphor::regular::BROWSERS;
use serde::{Deserialize, Serialize};

/// Calculation settings
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) index: Option<usize>,

    pub(crate) percent: bool,
    pub(crate) precision: usize,
    pub(crate) resizable: bool,
    pub(crate) sticky_columns: usize,
    pub(crate) truncate: bool,

    pub(crate) fraction: Fraction,
    pub(crate) from: From,
    pub(crate) normalize: Normalize,
    pub(crate) unsigned: bool,
    pub(crate) christie: bool,
    pub(crate) factors: bool,

    pub(crate) ddof: u8,
}

impl Settings {
    pub(crate) const fn new(index: Option<usize>) -> Self {
        Self {
            index,
            percent: true,
            precision: 1,
            resizable: false,
            sticky_columns: 0,
            truncate: false,
            fraction: Fraction::AsIs,
            from: From::Mag2,
            normalize: Normalize::new(),
            unsigned: true,
            christie: false,
            factors: true,
            ddof: 1,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Settings {
    pub(crate) fn show(&mut self, ui: &mut Ui, state: &mut State) {
        Grid::new("calculation").show(ui, |ui| {
            // Sticky
            ui.label(localize!("sticky"));
            ui.add(Slider::new(&mut self.sticky_columns, 0..=14));
            ui.end_row();

            // Precision
            ui.label(localize!("precision"));
            ui.add(Slider::new(&mut self.precision, 0..=MAX_PRECISION));
            ui.end_row();

            // Percent
            ui.label(localize!("percent"));
            ui.checkbox(&mut self.percent, "");
            ui.end_row();

            // Truncate
            ui.label(localize!("truncate"));
            ui.checkbox(&mut self.truncate, "");
            ui.end_row();

            ui.separator();
            ui.separator();
            ui.end_row();

            // Fraction
            ui.label(localize!("fraction"));
            let fraction = &mut self.fraction;
            ComboBox::from_id_salt("fraction")
                .selected_text(fraction.text())
                .show_ui(ui, |ui| {
                    ui.selectable_value(fraction, Fraction::AsIs, Fraction::AsIs.text())
                        .on_hover_text(Fraction::AsIs.hover_text());
                    ui.selectable_value(fraction, Fraction::Fraction, Fraction::Fraction.text())
                        .on_hover_text(Fraction::Fraction.hover_text());
                })
                .response
                .on_hover_text(fraction.hover_text());
            ui.end_row();

            // Calculate
            ui.label(localize!("from"))
                .on_hover_text(localize!("from.description"));
            if ui.input_mut(|input| {
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::Num1))
            }) {
                self.from = From::Dag1223;
            }
            if ui.input_mut(|input| {
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::Num2))
            }) {
                self.from = From::Mag2;
            }
            ComboBox::from_id_salt("1,3")
                .selected_text(self.from.text())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.from, From::Dag1223, From::Dag1223.text())
                        .on_hover_text(From::Dag1223.hover_text());
                    ui.selectable_value(&mut self.from, From::Mag2, From::Mag2.text())
                        .on_hover_text(From::Mag2.hover_text());
                })
                .response
                .on_hover_text(self.from.hover_text());
            ui.end_row();

            // Signed
            ui.label(localize!("unsigned"));
            ui.checkbox(&mut self.unsigned, localize!("theoretical"));
            ui.end_row();

            // Normalize
            ui.label(localize!("normalize"));
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.normalize.experimental, localize!("experimental"));
                ui.checkbox(&mut self.normalize.theoretical, localize!("theoretical"));
            });
            ui.end_row();

            // Factors
            ui.label(localize!("factors"));
            ui.checkbox(&mut self.factors, "");
            ui.end_row();

            // Christie
            ui.label(localize!("christie"));
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.christie, "");
                ui.toggle_value(
                    &mut state.open_christie_window,
                    RichText::new(BROWSERS).heading(),
                );
            });
            ui.end_row();

            ui.separator();
            ui.separator();
            ui.end_row();

            // ui.label(localize!("merge"));
            // ui.checkbox(&mut self.merge, "");
            // ComboBox::from_id_salt("show")
            //     .selected_text(self.show.text())
            //     .show_ui(ui, |ui| {
            //         ui.selectable_value(&mut self.show, Show::Separate, Show::Separate.text())
            //             .on_hover_text(Show::Separate.hover_text());
            //         ui.selectable_value(&mut self.show, Show::Join, Show::Join.text())
            //             .on_hover_text(Show::Join.hover_text());
            //     })
            //     .response
            //     .on_hover_text(self.show.hover_text());
            // ui.end_row();

            // https://numpy.org/devdocs/reference/generated/numpy.std.html
            ui.label(localize!("ddof"));
            ui.add(Slider::new(&mut self.ddof, 0..=2));
            ui.end_row();
        });
    }
}

/// Fraction
///
/// [wikipedia.org](https://en.wikipedia.org/wiki/Mole_fraction)
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum Fraction {
    AsIs,
    Fraction,
}

impl Fraction {
    pub(crate) fn text(self) -> String {
        match self {
            Self::AsIs => localize!("as_is"),
            Self::Fraction => "Pchelkin".to_owned(),
        }
    }

    // col(name) / (col(name) * col("FA").fa().mass() / lit(10)).sum()
    pub(crate) fn hover_text(self) -> &'static str {
        match self {
            Self::AsIs => "S / ∑ S",
            Self::Fraction => "S / ∑(S * M)",
        }
    }
}

/// From
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum From {
    Dag1223,
    Mag2,
}

impl From {
    pub(crate) fn text(self) -> String {
        match self {
            Self::Dag1223 => localize!("from_dag"),
            Self::Mag2 => localize!("from_mag"),
        }
    }

    pub(crate) fn hover_text(self) -> String {
        match self {
            Self::Dag1223 => localize!("from_dag.description"),
            Self::Mag2 => localize!("from_mag.description"),
        }
    }
}

/// Normalize
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Normalize {
    pub(crate) experimental: bool,
    pub(crate) theoretical: bool,
}

impl Normalize {
    pub(crate) const fn new() -> Self {
        Self {
            experimental: true,
            theoretical: true,
        }
    }
}

impl Default for Normalize {
    fn default() -> Self {
        Self::new()
    }
}
