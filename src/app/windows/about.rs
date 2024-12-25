use egui::{Context, Id, Label, RichText, Sense, Window};
use egui_phosphor::regular::{COPYRIGHT, GITHUB_LOGO, GLOBE, INFO, WARNING};

use crate::app::icon;

/// About
#[derive(Debug, Default)]
pub(crate) struct About {
    pub(crate) open: bool,
}

impl About {
    pub(crate) fn window(&mut self, ctx: &Context) {
        Window::new(format!("{INFO} About"))
            .open(&mut self.open)
            .show(ctx, |ui| {
                // let color = ui.visuals().text_color();
                // let mut text = LayoutJob::default();
                // text.append(
                //     &format!(
                //         "UTCA {version}\n\
                //         Ultimate TAG Calculation Application\n\
                //         © 2023 Giorgi Kazakov & Roman Sidorov"
                //     ),
                //     0.0,
                //     TextFormat {
                //         color,
                //         ..Default::default()
                //     },
                // );
                // ctx.frame_nr()
                ui.vertical_centered(|ui| {
                    let version = env!("CARGO_PKG_VERSION");
                    ui.label(format!("UTCA {version}"));
                    ui.label("Ultimate TAG Calculation Application");
                    ui.label(COPYRIGHT);
                    ui.add(Label::new("Giorgi Kazakov").sense(Sense::click()));
                    let id = Id::new("counter");
                    let counter =
                        ui.data_mut(|data| data.get_temp::<usize>(id).unwrap_or_default());
                    let mut response = ui.add(Label::new("Roman Sidorov").sense(Sense::click()));
                    if counter > 42 {
                        response = response.on_hover_text("♥ лучший котик в мире");
                    }
                    if response.clicked() {
                        ui.data_mut(|data| data.insert_temp(id, counter + 1));
                    } else if ui.input(|input| input.pointer.any_click()) {
                        ui.data_mut(|data| data.remove::<usize>(id));
                    }
                    ui.label("2024");
                    ui.separator();
                    ui.collapsing(RichText::new("Links").heading(), |ui| {
                        ui.horizontal(|ui| {
                            ui.label(icon!(GLOBE, x32)).on_hover_text("web");
                            ui.hyperlink_to(
                                "https://ippras.github.io/utca",
                                "https://ippras.github.io/utca",
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(icon!(GITHUB_LOGO, x32))
                                .on_hover_text("github.com");
                            ui.hyperlink_to(
                                "https://github.com/ippras/utca",
                                "https://github.com/ippras/utca",
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(icon!(WARNING, x32))
                                .on_hover_text("report an issue");
                            ui.hyperlink_to(
                                "https://github.com/ippras/utca/issues",
                                "https://github.com/ippras/utca/issues",
                            );
                        });
                    });
                    ui.collapsing(RichText::new("Dedications").heading(), |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Giorgi Kazakov:");
                            ui.label("Моим родителям, Тане и Володе, посвящается.");
                        });
                    });
                });
            });
    }
}
