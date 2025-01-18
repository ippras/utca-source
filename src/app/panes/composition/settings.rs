use crate::{
    app::{MAX_PRECISION, text::Text},
    r#const::relative_atomic_mass::{H, LI, NA, NH4},
    localize,
    special::composition::{Composition, MC, NC, PSC, PTC, PUC, SC, SSC, STC, SUC, TC, UC},
};
use egui::{
    ComboBox, DragValue, Grid, Id, Key, KeyboardShortcut, Modifiers, PopupCloseBehavior, RichText,
    Slider, SliderClamping, Ui, emath::Float,
};
use egui_ext::LabeledSeparator;
use egui_phosphor::regular::{FUNNEL, FUNNEL_X, MINUS, PLUS, TRASH};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    hash::{Hash, Hasher},
};

/// Composition settings bundle
#[derive(Default, Deserialize, Serialize)]
pub(crate) struct Bundle {
    pub(crate) confirmed: Settings,
    pub(crate) unconfirmed: Settings,
}

impl Bundle {
    pub(crate) fn new(index: Option<usize>) -> Self {
        Self {
            confirmed: Settings::new(index),
            unconfirmed: Settings::new(index),
        }
    }
}

/// Composition settings
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) index: Option<usize>,

    pub(crate) percent: bool,
    pub(crate) precision: usize,
    pub(crate) resizable: bool,
    pub(crate) sticky_columns: usize,

    pub(crate) adduct: f64,
    pub(crate) method: Method,
    pub(crate) groups: VecDeque<Group>,
    pub(crate) show: Show,
    pub(crate) sort: Sort,
    pub(crate) order: Order,
    pub(crate) join: Join,

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
            adduct: 0.0,
            method: Method::VanderWal,
            groups: VecDeque::new(),
            show: Show::new(),
            sort: Sort::Value,
            order: Order::Descending,
            join: Join::Left,
            ddof: 1,
        }
    }
}

impl Settings {
    pub(crate) fn show(&mut self, ui: &mut Ui, data_frame: &DataFrame) {
        Grid::new("composition").show(ui, |ui| {
            // Sticky
            ui.label(localize!("sticky"));
            ui.add(Slider::new(
                &mut self.sticky_columns,
                0..=self.groups.len() * 2 + 1,
            ));
            ui.end_row();

            // Precision
            ui.label(localize!("precision"));
            ui.add(Slider::new(&mut self.precision, 0..=MAX_PRECISION));
            ui.end_row();

            // Percent
            ui.label(localize!("percent"));
            ui.checkbox(&mut self.percent, "");
            ui.end_row();

            ui.separator();
            ui.separator();
            ui.end_row();

            // Method
            ui.label(localize!("method"));
            if ui.input_mut(|input| {
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::G))
            }) {
                self.method = Method::Gunstone;
            }
            if ui.input_mut(|input| {
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::W))
            }) {
                self.method = Method::VanderWal;
            }
            ComboBox::from_id_salt("method")
                .selected_text(self.method.text())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.method,
                        Method::Gunstone,
                        Method::Gunstone.text(),
                    )
                    .on_hover_text(Method::Gunstone.hover_text());
                    ui.selectable_value(
                        &mut self.method,
                        Method::VanderWal,
                        Method::VanderWal.text(),
                    )
                    .on_hover_text(Method::VanderWal.hover_text());
                })
                .response
                .on_hover_text(self.method.hover_text());
            ui.end_row();

            ui.label(localize!("adduct"));
            ui.horizontal(|ui| {
                let adduct = &mut self.adduct;
                ui.add(
                    DragValue::new(adduct)
                        .range(0.0..=f64::MAX)
                        .speed(1.0 / 10f64.powi(self.precision as _)),
                )
                .on_hover_text(format!("{adduct}"));
                ComboBox::from_id_salt(ui.next_auto_id())
                    .selected_text(match *adduct {
                        H => "H",
                        NH4 => "NH4",
                        NA => "Na",
                        LI => "Li",
                        _ => "",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(adduct, 0.0, "None");
                        ui.selectable_value(adduct, H, "H");
                        ui.selectable_value(adduct, NH4, "NH4");
                        ui.selectable_value(adduct, NA, "Na");
                        ui.selectable_value(adduct, LI, "Li");
                    });
            });
            ui.end_row();

            // View
            ui.separator();
            ui.labeled_separator(RichText::new(localize!("view")).heading());
            ui.end_row();

            ui.label(localize!("nulls")).on_hover_text("Show nulls");
            ui.checkbox(&mut self.show.nulls, "");
            ui.end_row();

            ui.label(localize!("filtered"))
                .on_hover_text("Show filtered");
            ui.checkbox(&mut self.show.filtered, "");
            ui.end_row();

            // Compose
            ui.label(localize!("compose"));
            if ui.button(PLUS).clicked() {
                self.groups.push_front(Group::new());
            }
            ui.end_row();
            let mut index = 0;
            self.groups.retain_mut(|group| {
                let mut keep = true;
                ui.label("");
                ui.horizontal(|ui| {
                    // Delete
                    keep = !ui.button(MINUS).clicked();
                    ComboBox::from_id_salt(ui.next_auto_id())
                        .selected_text(group.composition.text())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut group.composition, NC, NC.text())
                                .on_hover_text(NC.hover_text());
                            ui.separator();
                            ui.selectable_value(&mut group.composition, MC, MC.text())
                                .on_hover_text(MC.hover_text());
                            ui.separator();
                            ui.selectable_value(&mut group.composition, UC, UC.text())
                                .on_hover_text(UC.hover_text());
                            ui.selectable_value(&mut group.composition, PUC, PUC.text())
                                .on_hover_text(PUC.hover_text());
                            ui.selectable_value(&mut group.composition, SUC, SUC.text())
                                .on_hover_text(SUC.hover_text());
                            ui.separator();
                            ui.selectable_value(&mut group.composition, TC, TC.text())
                                .on_hover_text(TC.hover_text());
                            ui.selectable_value(&mut group.composition, PTC, PTC.text())
                                .on_hover_text(PTC.hover_text());
                            ui.selectable_value(&mut group.composition, STC, STC.text())
                                .on_hover_text(STC.hover_text());
                            ui.separator();
                            ui.selectable_value(&mut group.composition, SC, SC.text())
                                .on_hover_text(SC.hover_text());
                            ui.selectable_value(&mut group.composition, PSC, PSC.text())
                                .on_hover_text(PSC.hover_text());
                            ui.selectable_value(&mut group.composition, SSC, SSC.text())
                                .on_hover_text(SSC.hover_text());
                        })
                        .response
                        .on_hover_text(group.composition.hover_text());
                    // Filter
                    let title = if group.filter == Default::default() {
                        FUNNEL
                    } else {
                        ui.visuals_mut().widgets.inactive = ui.visuals().widgets.active;
                        FUNNEL_X
                    };
                    // ui.interact(
                    //     egui::Rect::EVERYTHING,
                    //     egui::Id::new("Right click menu"),
                    //     egui::Sense::hover(),
                    // ).context_menu_opened();
                    // .context_menu(|ui|
                    //     {
                    //                                 if ui.button(format!("test")).clicked() {
                    //                                 }
                    //     }
                    ui.menu_button(title, |ui| {
                        ui.label(format!(
                            "{} {}",
                            group.composition.text(),
                            localize!("filter"),
                        ));
                        // Key
                        let mut is_open = false;
                        let t = AnyValue::List(Series::new(PlSmallStr::EMPTY, &group.filter.key));
                        ui.horizontal(|ui| {
                            let id_salt = "FattyAcidsFilter";
                            ComboBox::from_id_salt(id_salt)
                                // .height(ui.available_height())
                                .selected_text(group.filter.key.len().to_string())
                                .height(ui.available_height())
                                .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                                .show_ui(ui, |ui| -> PolarsResult<()> {
                                    // println!("data_frame: {data_frame}");
                                    let key = data_frame[index + 1]
                                        .struct_()
                                        .unwrap()
                                        .field_by_name("Key")
                                        .unwrap();
                                    // println!("composition: {:?}", key.str_value(0));
                                    for index in 0..key.len() {
                                        if let Ok(key) = key.str_value(index) {
                                            let key = key.to_string();
                                            let contains = group.filter.key.contains(&key);
                                            let mut selected = contains;
                                            ui.toggle_value(&mut selected, &key);
                                            if selected && !contains {
                                                group.filter.key.push(key);
                                            } else if !selected && contains {
                                                group.filter.remove(&key);
                                            }
                                        }
                                    }
                                    Ok(())
                                })
                                .response
                                .on_hover_text(t.str_value());
                            let id = ui.make_persistent_id(Id::new(id_salt));
                            is_open = ComboBox::is_open(ui.ctx(), id);
                            if ui.button(TRASH).clicked() {
                                group.filter.key = Vec::new();
                            }
                        });
                        if is_open {
                            ui.add_space(100.0 - ui.spacing().interact_size.y);
                        }
                        // Value
                        ui.add(
                            Slider::new(&mut group.filter.value, 0.0..=1.0)
                                .clamping(SliderClamping::Always)
                                .logarithmic(true)
                                .custom_formatter(|mut value, _| {
                                    if self.percent {
                                        value *= 100.0;
                                    }
                                    AnyValue::Float64(value).to_string()
                                })
                                .custom_parser(|value| {
                                    let mut parsed = value.parse::<f64>().ok()?;
                                    if self.percent {
                                        parsed /= 100.0;
                                    }
                                    Some(parsed)
                                }),
                        );
                    });
                });
                ui.end_row();
                index += 1;
                keep
            });
            if self.groups.is_empty() {
                self.groups.push_back(Group::new());
            }

            // // Join
            // ui.label(localize!("join"));
            // ComboBox::from_id_salt("join")
            //     .selected_text(self.join.text())
            //     .show_ui(ui, |ui| {
            //         ui.selectable_value(&mut self.join, Join::Left, Join::Left.text())
            //             .on_hover_text(Join::Left.hover_text());
            //         ui.selectable_value(&mut self.join, Join::And, Join::And.text())
            //             .on_hover_text(Join::And.hover_text());
            //         ui.selectable_value(&mut self.join, Join::Or, Join::Or.text())
            //             .on_hover_text(Join::Or.hover_text());
            //     })
            //     .response
            //     .on_hover_text(self.join.hover_text());
            // ui.end_row();

            ui.separator();
            ui.labeled_separator(RichText::new(localize!("sort")).heading());
            ui.end_row();

            // Sort
            ui.label(localize!("sort"));
            ComboBox::from_id_salt("sort")
                .selected_text(self.sort.text())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.sort, Sort::Key, Sort::Key.text())
                        .on_hover_text(Sort::Key.hover_text());
                    ui.selectable_value(&mut self.sort, Sort::Value, Sort::Value.text())
                        .on_hover_text(Sort::Value.hover_text());
                })
                .response
                .on_hover_text(self.sort.hover_text());
            ui.end_row();
            // Order
            ui.label(localize!("order"));
            ComboBox::from_id_salt("order")
                .selected_text(self.order.text())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.order, Order::Ascending, Order::Ascending.text())
                        .on_hover_text(Order::Ascending.hover_text());
                    ui.selectable_value(
                        &mut self.order,
                        Order::Descending,
                        Order::Descending.text(),
                    )
                    .on_hover_text(Order::Descending.hover_text());
                })
                .response
                .on_hover_text(self.order.hover_text());
            ui.end_row();

            // Statistic
            ui.separator();
            ui.labeled_separator(RichText::new(localize!("statistic")).heading());
            ui.end_row();

            // https://numpy.org/devdocs/reference/generated/numpy.std.html
            ui.label(localize!("ddof"));
            ui.add(Slider::new(&mut self.ddof, 0..=2));
            ui.end_row();

            ui.separator();
            ui.separator();
        });
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Hash for Settings {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.percent.hash(state);
        self.precision.hash(state);
        self.resizable.hash(state);
        self.sticky_columns.hash(state);
        self.adduct.ord().hash(state);
        self.method.hash(state);
        self.groups.hash(state);
        self.show.hash(state);
        self.sort.hash(state);
        self.order.hash(state);
        self.join.hash(state);
        self.ddof.hash(state);
    }
}

/// Join
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum Join {
    Left,
    And,
    Or,
}

impl Join {
    pub(crate) fn text(self) -> String {
        match self {
            Self::Left => localize!("left"),
            Self::And => localize!("and"),
            Self::Or => localize!("or"),
        }
    }

    pub(crate) fn hover_text(self) -> String {
        match self {
            Self::Left => localize!("left.description"),
            Self::And => localize!("and.description"),
            Self::Or => localize!("or.description"),
        }
    }
}

impl From<Join> for JoinType {
    fn from(value: Join) -> Self {
        match value {
            Join::Left => JoinType::Left,
            Join::And => JoinType::Inner,
            Join::Or => JoinType::Full,
        }
    }
}

/// Method
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum Method {
    Gunstone,
    VanderWal,
}

impl Method {
    pub(crate) fn text(&self) -> String {
        match self {
            Self::Gunstone => localize!("gunstone"),
            Self::VanderWal => localize!("vander_wal"),
        }
    }

    pub(crate) fn hover_text(&self) -> String {
        match self {
            Self::Gunstone => localize!("gunstone.description"),
            Self::VanderWal => localize!("vander_wal.description"),
        }
    }
}

/// Filters
#[derive(Clone, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Show {
    pub(crate) nulls: bool,
    pub(crate) filtered: bool,
}

impl Show {
    pub(crate) const fn new() -> Self {
        Self {
            nulls: false,
            filtered: false,
        }
    }
}

/// Filter
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Filter {
    pub(crate) key: Vec<String>,
    pub(crate) value: f64,
}

impl Filter {
    pub(crate) const fn new() -> Self {
        Self {
            key: Vec::new(),
            value: 0.0,
        }
    }

    fn remove(&mut self, target: &String) -> Option<String> {
        let position = self.key.iter().position(|source| source == target)?;
        Some(self.key.remove(position))
    }
}

impl Eq for Filter {}

impl Hash for Filter {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.ord().hash(state);
    }
}

impl PartialEq for Filter {
    fn eq(&self, other: &Self) -> bool {
        self.value.ord() == other.value.ord()
    }
}

/// Sort
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum Sort {
    Key,
    Value,
}

impl Sort {
    pub(crate) fn text(self) -> String {
        match self {
            Self::Key => localize!("key"),
            Self::Value => localize!("value"),
        }
    }

    pub(crate) fn hover_text(self) -> String {
        match self {
            Self::Key => localize!("key.description"),
            Self::Value => localize!("value.description"),
        }
    }
}

/// Order
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum Order {
    Ascending,
    Descending,
}

impl Order {
    pub(crate) fn text(self) -> String {
        match self {
            Self::Ascending => localize!("ascending"),
            Self::Descending => localize!("descending"),
        }
    }

    pub(crate) fn hover_text(self) -> String {
        match self {
            Self::Ascending => localize!("ascending.description"),
            Self::Descending => localize!("descending.description"),
        }
    }
}

/// Group
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) struct Group {
    pub(crate) composition: Composition,
    pub(crate) filter: Filter,
}

impl Group {
    pub(crate) const fn new() -> Self {
        Self {
            composition: Composition::new(),
            filter: Filter::new(),
        }
    }
}
