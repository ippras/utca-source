use eframe::{APP_KEY, CreationContext, Storage, get_value, set_value};
use egui::{
    Align, Align2, CentralPanel, Color32, Context, FontDefinitions, Id, LayerId, Layout, Memory,
    Order, RichText, ScrollArea, SidePanel, Sides, TextStyle, TopBottomPanel, Ui, Vec2, Visuals,
    menu::bar, util::cache, vec2, warn_if_debug_build,
};
use egui_ext::{DroppedFileExt, HoveredFileExt, LightDarkButton};
use egui_notify::Toasts;
use egui_phosphor::{
    Variant, add_to_fonts,
    regular::{
        ARROWS_CLOCKWISE, CHART_BAR, CLOUD_ARROW_DOWN, FLOPPY_DISK, GRID_FOUR, INFO, PLUS,
        SIDEBAR_SIMPLE, SQUARE_SPLIT_HORIZONTAL, SQUARE_SPLIT_VERTICAL, TABS, TRASH,
    },
};
use egui_tiles::{ContainerKind, Tile, Tree};
use polars::frame::DataFrame;
use serde::{Deserialize, Serialize};
use std::{
    borrow::BorrowMut,
    fmt::Write,
    mem::{replace, take},
    str,
    sync::mpsc::{Receiver, Sender, channel},
    time::Duration,
};
use tracing::{debug, error, info, trace};

use self::{
    data::{Data, File},
    panes::{
        Pane, behavior::Behavior, calculation::Pane as CalculationPane,
        composition::Pane as CompositionPane, configuration::Pane as ConfigurationPane,
    },
    windows::{About, Github},
};
use crate::{
    localization::{UiExt, localize},
    utils::TreeExt,
};

/// IEEE 754-2008
const MAX_PRECISION: usize = 16;

const MARGIN: Vec2 = vec2(4.0, 0.0);
const NOTIFICATIONS_DURATION: Duration = Duration::from_secs(15);

// const DESCRIPTION: &str = "Positional-species and positional-type composition of TAG from mature fruit arils of the Euonymus section species, mol % of total TAG";

const SIZE: f32 = 32.0;

pub(crate) macro icon {
    ($icon:expr, x8) => { RichText::new($icon).size(8.0) },
    ($icon:expr, x16) => { RichText::new($icon).size(16.0) },
    ($icon:expr, x32) => { RichText::new($icon).size(32.0) },
    ($icon:expr, x64) => { RichText::new($icon).size(64.0) }
}

fn custom_style(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = custom_visuals(style.visuals);
    ctx.set_style(style);
}

fn custom_visuals<T: BorrowMut<Visuals>>(mut visuals: T) -> T {
    visuals.borrow_mut().collapsing_header_frame = true;
    visuals
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct App {
    // Panels
    left_panel: bool,
    // Panes
    // #[serde(skip)]
    tree: Tree<Pane>,
    // Data
    data: Data,

    // Data channel
    #[serde(skip)]
    channel: (Sender<(String, String)>, Receiver<(String, String)>),

    // Windows
    #[serde(skip)]
    about: About,
    #[serde(skip)]
    github: Github,
    // Notifications
    #[serde(skip)]
    toasts: Toasts,
}

impl Default for App {
    fn default() -> Self {
        Self {
            left_panel: true,
            tree: Tree::empty("central_tree"),
            data: Default::default(),
            channel: channel(),
            toasts: Default::default(),
            about: Default::default(),
            github: Default::default(),
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &CreationContext) -> Self {
        // Customize style of egui.
        let mut fonts = FontDefinitions::default();
        add_to_fonts(&mut fonts, Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);
        custom_style(&cc.egui_ctx);

        // return Default::default();
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        let app = Self::load(cc).unwrap_or_default();
        app.context(&cc.egui_ctx);
        app
    }

    fn load(cc: &CreationContext) -> Option<Self> {
        let storage = cc.storage?;
        let value = get_value(storage, APP_KEY)?;
        Some(value)
    }

    fn context(&self, ctx: &Context) {
        // Data channel
        ctx.data_mut(|data| data.insert_temp(Id::new("Data"), self.channel.0.clone()));
    }
}

// Panels
impl App {
    fn panels(&mut self, ctx: &Context) {
        self.top_panel(ctx);
        self.bottom_panel(ctx);
        self.left_panel(ctx);
        self.central_panel(ctx);
    }

    // Bottom panel
    fn bottom_panel(&mut self, ctx: &Context) {
        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                Sides::new().show(
                    ui,
                    |_| {},
                    |ui| {
                        warn_if_debug_build(ui);
                        ui.label(RichText::new(env!("CARGO_PKG_VERSION")).small());
                        ui.separator();
                    },
                );
            });
        });
    }

    // Central panel
    fn central_panel(&mut self, ctx: &Context) {
        CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.0))
            .show(ctx, |ui| {
                let mut behavior = Behavior {
                    data: &mut self.data,
                    close: None,
                };
                self.tree.ui(&mut behavior, ui);
                if let Some(id) = behavior.close {
                    self.tree.tiles.remove(id);
                }
            });
    }

    // Left panel
    fn left_panel(&mut self, ctx: &Context) {
        SidePanel::left("left_panel")
            .resizable(true)
            .show_animated(ctx, self.left_panel, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    ui.add(&mut self.data);
                });
            });
    }

    // Top panel
    fn top_panel(&mut self, ctx: &Context) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            bar(ui, |ui| {
                ScrollArea::horizontal().show(ui, |ui| {
                    // Left panel
                    ui.toggle_value(&mut self.left_panel, icon!(SIDEBAR_SIMPLE, x32))
                        .on_hover_text(localize!("LeftPanel"));
                    ui.separator();
                    // Light/Dark
                    ui.light_dark_button(SIZE);
                    ui.separator();
                    // Reset
                    if ui
                        .button(icon!(TRASH, x32))
                        .on_hover_text(localize!("reset_application"))
                        .clicked()
                    {
                        *self = Default::default();
                        self.context(ctx);
                    }
                    ui.separator();
                    if ui
                        .button(icon!(ARROWS_CLOCKWISE, x32))
                        .on_hover_text(localize!("reset_gui"))
                        .clicked()
                    {
                        ui.memory_mut(|memory| {
                            memory.caches = replace(memory, Default::default()).caches;
                        });
                        self.context(ctx);
                    }
                    ui.separator();
                    if ui
                        .button(RichText::new(SQUARE_SPLIT_VERTICAL).size(SIZE))
                        .on_hover_text(localize!("vertical"))
                        .clicked()
                    {
                        if let Some(id) = self.tree.root {
                            if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id) {
                                container.set_kind(ContainerKind::Vertical);
                            }
                        }
                    }
                    if ui
                        .button(RichText::new(SQUARE_SPLIT_HORIZONTAL).size(SIZE))
                        .on_hover_text(localize!("horizontal"))
                        .clicked()
                    {
                        if let Some(id) = self.tree.root {
                            if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id) {
                                container.set_kind(ContainerKind::Horizontal);
                            }
                        }
                    }
                    if ui
                        .button(RichText::new(GRID_FOUR).size(SIZE))
                        .on_hover_text(localize!("grid"))
                        .clicked()
                    {
                        if let Some(id) = self.tree.root {
                            if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id) {
                                container.set_kind(ContainerKind::Grid);
                            }
                        }
                    }
                    if ui
                        .button(RichText::new(TABS).size(SIZE))
                        .on_hover_text(localize!("tabs"))
                        .clicked()
                    {
                        if let Some(id) = self.tree.root {
                            if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(id) {
                                container.set_kind(ContainerKind::Tabs);
                            }
                        }
                    }
                    ui.separator();
                    // let mut toggle = |ui: &mut Ui, pane| {
                    //     let tile_id = self.tree.tiles.find_pane(&pane);
                    //     if ui
                    //         .selectable_label(
                    //             tile_id.is_some_and(|tile_id| self.tree.is_visible(tile_id)),
                    //             icon!(pane.icon(), x32),
                    //         )
                    //         .on_hover_text(pane.title())
                    //         .clicked()
                    //     {
                    //         if let Some(id) = tile_id {
                    //             self.tree.tiles.toggle_visibility(id);
                    //         } else {
                    //             self.tree.insert_pane(pane);
                    //         }
                    //     }
                    // };

                    // Configuration
                    if !self.data.is_empty() {
                        let pane = Pane::Configuration(ConfigurationPane::new(self.data.checked()));
                        let button = ui
                            .button(icon!(pane.icon(), x32))
                            .on_hover_text(pane.title());
                        if button.clicked() {
                            self.tree.insert_pane(pane);
                        }
                    }
                    // Create
                    if ui.button(icon!(PLUS, x32)).clicked() {
                        self.data.files.push(Default::default());
                    }
                    // Load
                    if ui.button(icon!(CLOUD_ARROW_DOWN, x32)).clicked() {
                        self.github.toggle();
                    }
                    // Save
                    if ui.button(icon!(FLOPPY_DISK, x32)).clicked() {
                        if let Err(error) = self.data.save() {
                            error!(%error);
                        }
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // About
                        if ui
                            .button(icon!(INFO, x32))
                            .on_hover_text("About window")
                            .clicked()
                        {
                            self.about.open ^= true;
                        }
                        ui.separator();
                        // Locale
                        ui.locale_button().on_hover_text(localize!("language"));
                    });
                });
            });
        });
    }
}

// Windows
impl App {
    fn windows(&mut self, ctx: &Context) {
        self.about.window(ctx);
        self.github.window(ctx);
    }
}

// Notifications
impl App {
    fn notifications(&mut self, ctx: &Context) {
        self.toasts.show(ctx);
    }
}

// Copy/Paste, Drag&Drop
impl App {
    fn calculate(&mut self, ctx: &Context) {
        if let Some(data_frame) =
            ctx.data_mut(|data| data.remove_temp::<DataFrame>(Id::new("Calculate")))
        {
            self.tree.insert_pane(Pane::calculation(data_frame));
        }
    }

    fn compose(&mut self, ctx: &Context) {
        if let Some(data_frame) =
            ctx.data_mut(|data| data.remove_temp::<DataFrame>(Id::new("Compose")))
        {
            self.tree.insert_pane(Pane::composition(data_frame));
        }
    }

    fn drag_and_drop(&mut self, ctx: &Context) {
        // Preview hovering files
        if let Some(text) = ctx.input(|input| {
            (!input.raw.hovered_files.is_empty()).then(|| {
                let mut text = String::from("Dropping files:");
                for file in &input.raw.hovered_files {
                    write!(text, "\n{}", file.display()).ok();
                }
                text
            })
        }) {
            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));
            let screen_rect = ctx.screen_rect();
            painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                screen_rect.center(),
                Align2::CENTER_CENTER,
                text,
                TextStyle::Heading.resolve(&ctx.style()),
                Color32::WHITE,
            );
        }
        // Parse dropped files
        if let Some(dropped_files) = ctx.input(|input| {
            (!input.raw.dropped_files.is_empty()).then_some(input.raw.dropped_files.clone())
        }) {
            info!(?dropped_files);
            for dropped in dropped_files {
                trace!(?dropped);
                let content = match dropped.content() {
                    Ok(content) => content,
                    Err(error) => {
                        error!(%error);
                        self.toasts
                            .error(format!("{}: {error}", dropped.display()))
                            .closable(true)
                            .duration(Some(NOTIFICATIONS_DURATION));
                        continue;
                    }
                };
                trace!(content);
                self.channel
                    .0
                    .send((dropped.name().to_owned(), content))
                    .ok();
            }
        }
    }

    fn parse(&mut self, ctx: &Context) {
        for (name, content) in self.channel.1.try_iter() {
            trace!(name, content);
            match ron::de::from_str(&content) {
                Ok(fatty_acids) => {
                    trace!(?fatty_acids);
                    self.data.push(File { name, fatty_acids });
                    ctx.request_repaint();
                }
                Err(error) => error!(%error),
            };
        }
    }

    // fn paste(&mut self, ctx: &Context) {
    //     if !ctx.memory(|memory| memory.focused().is_some()) {
    //         ctx.input(|input| {
    //             for event in &input.raw.events {
    //                 if let Event::Paste(paste) = event {
    //                     if let Err(error) = self.parse(paste) {
    //                         error!(?error);
    //                         self.toasts
    //                             .error(error.to_string().chars().take(64).collect::<String>())
    //                             .set_duration(Some(Duration::from_secs(5)))
    //                             .set_closable(true);
    //                     }
    //                 }
    //             }
    //         });
    //     }
    // }

    // fn parse(&mut self, paste: &str) -> Result<()> {
    //     use crate::parsers::whitespace::Parser;
    //     let parsed = Parser::parse(paste)?;
    //     debug!(?parsed);
    //     for parsed in parsed {
    //         // self.docks.central.tabs.input.add(match parsed {
    //         //     Parsed::All(label, (c, n), tag, dag, mag) => FattyAcid {
    //         //         label,
    //         //         formula: ether!(c as usize, n as usize),
    //         //         values: [tag, dag, mag],
    //         //     },
    //         //     // Parsed::String(label) => Row { label, ..default() },
    //         //     // Parsed::Integers(_) => Row { label, ..default() },
    //         //     // Parsed::Float(tag) => Row { label, ..default() },
    //         //     _ => unimplemented!(),
    //         // })?;
    //         // self.config.push_row(Row {
    //         //     acylglycerols,
    //         //     label:  parsed.,
    //         //     ether: todo!(),
    //         //     // ..default()
    //         // })?;
    //     }
    //     // let mut rows = Vec::new();
    //     // for row in paste.split('\n') {
    //     //     let mut columns = [0.0; COUNT];
    //     //     for (j, column) in row.split('\t').enumerate() {
    //     //         ensure!(j < COUNT, "Invalid shape, columns: {COUNT} {j}");
    //     //         columns[j] = column.replace(',', ".").parse()?;
    //     //     }
    //     //     rows.push(columns);
    //     // }
    //     // for acylglycerols in rows {
    //     //     self.config.push_row(Row {
    //     //         acylglycerol: acylglycerols,
    //     //         ..default()
    //     //     })?;
    //     // }
    //     Ok(())
    // }

    // fn export(&self) -> Result<(), impl Debug> {
    //     let content = to_string(&TomlParsed {
    //         name: self.context.state.entry().meta.name.clone(),
    //         fatty_acids: self.context.state.entry().fatty_acids(),
    //     })
    //     .unwrap();
    //     self.file_dialog
    //         .save(
    //             &format!("{}.toml", self.context.state.entry().meta.name),
    //             content,
    //         )
    //         .unwrap();
    //     Ok::<_, ()>(())
    // }

    // fn import(&mut self) -> Result<(), impl Debug> {
    //     self.file_dialog.load()
    // }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn Storage) {
        set_value(storage, APP_KEY, self);
        // set_value(storage, APP_KEY, &Self::default());
    }

    /// Called each time the UI needs repainting, which may be many times per
    /// second.
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.calculate(ctx);
        self.compose(ctx);
        // Pre update
        self.panels(ctx);
        self.windows(ctx);
        self.notifications(ctx);
        // Post update
        self.drag_and_drop(ctx);
        self.parse(ctx);
    }
}

mod computers;
mod data;
mod panes;
mod text;
mod widgets;
mod windows;
