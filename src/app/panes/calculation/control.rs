use crate::{
    app::{MAX_PRECISION, widgets::FloatValue},
    localization::localize,
    special::fatty_acid::{COMMON, DisplayWithOptions, FattyAcid},
    utils::polars::DataFrameExt,
};
use egui::{
    ComboBox, Grid, Key, KeyboardShortcut, Modifiers, RichText, ScrollArea, Slider, Ui, Window,
};
use egui_phosphor::regular::{BROWSERS, GEAR, MATH_OPERATIONS};
use polars::frame::DataFrame;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

static FILE: &str = include_str!("../../../../christie.ron");
pub(crate) static CHRISTIE: LazyLock<DataFrame> =
    LazyLock::new(|| ron::de::from_str(FILE).expect("deserialize CHRISTIE"));

/// Calculation control
#[derive(Default, Deserialize, Serialize)]
pub(crate) struct Control {
    pub(crate) settings: Settings,
    pub(crate) index: Option<usize>,
    pub(crate) open: bool,
}

impl Control {
    pub(crate) const fn new() -> Self {
        Self {
            settings: Settings::new(),
            index: None,
            open: false,
        }
    }

    pub(crate) fn windows(&mut self, ui: &mut Ui) {
        // Settings
        Window::new(format!("{GEAR} Calculation settings"))
            .id(ui.next_auto_id())
            .default_pos(ui.next_widget_position())
            .open(&mut self.open)
            .show(ui.ctx(), |ui| {
                self.settings.ui(ui);
            });
        // Christie
        Window::new(format!("{MATH_OPERATIONS} Christie"))
            .default_pos(ui.next_widget_position())
            .id(ui.auto_id_with("christie"))
            .open(&mut self.settings.christie.open)
            .show(ui.ctx(), |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    Grid::new(ui.next_auto_id()).show(ui, |ui| {
                        let fatty_acids = CHRISTIE["FA"].struct_().unwrap();
                        let carbons = fatty_acids.field_by_name("Carbons").unwrap();
                        let doubles = fatty_acids.field_by_name("Doubles").unwrap();
                        let triples = fatty_acids.field_by_name("Triples").unwrap();
                        for index in 0..CHRISTIE.height() {
                            let carbons = carbons.u8().unwrap().get(index).unwrap();
                            let doubles = doubles.list().unwrap().get_as_series(index).unwrap();
                            let triples = triples.list().unwrap().get_as_series(index).unwrap();
                            let fatty_acid = FattyAcid {
                                carbons,
                                doubles: doubles.i8().unwrap().to_vec_null_aware().left().unwrap(),
                                triples: triples.i8().unwrap().to_vec_null_aware().left().unwrap(),
                            };
                            ui.label(format!("{:#}", fatty_acid.display(COMMON)));
                            let value = CHRISTIE.f64("CHRISTIE").get(index);
                            ui.add(FloatValue::new(value));
                            ui.end_row();
                        }
                    });
                });
            });
    }
}

/// Calculation settings
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) percent: bool,
    pub(crate) precision: usize,
    pub(crate) resizable: bool,
    pub(crate) sticky_columns: usize,
    pub(crate) truncate: bool,

    pub(crate) fraction: Fraction,
    pub(crate) from: From,
    pub(crate) normalize: Normalize,
    pub(crate) unsigned: bool,
    pub(crate) christie: Christie,

    pub(crate) ddof: u8,
}

impl Settings {
    pub(crate) const fn new() -> Self {
        Self {
            percent: true,
            precision: 1,
            resizable: false,
            sticky_columns: 0,
            truncate: false,
            fraction: Fraction::AsIs,
            from: From::Mag2,
            normalize: Normalize::new(),
            unsigned: true,
            christie: Christie::new(),
            ddof: 1,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

impl Settings {
    pub(crate) fn ui(&mut self, ui: &mut Ui) {
        // ui.visuals_mut().collapsing_header_frame = true;
        // ui.collapsing(RichText::new(localize!("settings")).heading(), |ui| {
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
                    ui.selectable_value(fraction, Fraction::ToMole, Fraction::ToMole.text())
                        .on_hover_text(Fraction::ToMole.hover_text());
                    ui.selectable_value(fraction, Fraction::ToMass, Fraction::ToMass.text())
                        .on_hover_text(Fraction::ToMass.hover_text());
                    ui.selectable_value(fraction, Fraction::Pchelkin, Fraction::Pchelkin.text())
                        .on_hover_text(Fraction::Pchelkin.hover_text());
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

            // Christie
            ui.label(localize!("christie"));
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.christie.apply, "");
                ui.toggle_value(&mut self.christie.open, RichText::new(BROWSERS).heading());
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

/// Christie
#[derive(Clone, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Christie {
    pub(crate) apply: bool,
    pub(crate) open: bool,
}

impl Christie {
    pub(crate) const fn new() -> Self {
        Self {
            apply: false,
            open: false,
        }
    }
}

/// Fraction
///
/// [wikipedia.org](https://en.wikipedia.org/wiki/Mole_fraction)
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum Fraction {
    AsIs,
    ToMole,
    ToMass,
    Pchelkin,
}

impl Fraction {
    pub(crate) fn text(self) -> String {
        match self {
            Self::AsIs => localize!("as_is"),
            Self::ToMole => localize!("to_mole_fraction"),
            Self::ToMass => localize!("to_mass_fraction"),
            Self::Pchelkin => "Pchelkin".to_owned(),
        }
    }

    pub(crate) fn hover_text(self) -> &'static str {
        match self {
            Self::AsIs => "S / ∑ S",
            Self::ToMole => "S / M / ∑(S / M)",
            Self::ToMass => "S * M / ∑(S * M)",
            Self::Pchelkin => "Pchelkin",
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
