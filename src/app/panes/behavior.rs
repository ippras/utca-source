use super::Pane;
use egui::{Frame, Margin, RichText, Sides, Ui, WidgetText};
use egui_phosphor::regular::X;
use egui_tiles::{TileId, UiResponse};

/// Behavior
#[derive(Debug)]
pub(crate) struct Behavior {
    pub(crate) close: Option<TileId>,
}

impl egui_tiles::Behavior<Pane> for Behavior {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> WidgetText {
        pane.title().into()
    }

    fn pane_ui(&mut self, ui: &mut Ui, tile_id: TileId, pane: &mut Pane) -> UiResponse {
        Frame::none()
            .inner_margin(Margin::symmetric(4.0, 6.0))
            .show(ui, |ui| {
                let response = Sides::new()
                    .show(
                        ui,
                        |ui| pane.header(ui),
                        |ui| {
                            ui.visuals_mut().button_frame = false;
                            if ui.button(RichText::new(X).heading()).clicked() {
                                self.close = Some(tile_id);
                            }
                        },
                    )
                    .0;
                pane.body(ui);
                if response.dragged() {
                    UiResponse::DragStarted
                } else {
                    UiResponse::None
                }
            })
            .inner
    }
}
