use egui::{CursorIcon, Frame, Margin, RichText, Sides, Ui, WidgetText};
use egui_phosphor::regular::X;
use egui_tiles::{TileId, UiResponse};

use super::Pane;
use crate::app::data::Data;

/// Behavior
#[derive(Debug)]
pub(crate) struct Behavior<'a> {
    pub(crate) data: &'a mut Data,
    pub(crate) close: Option<TileId>,
}

impl egui_tiles::Behavior<Pane> for Behavior<'_> {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> WidgetText {
        pane.title().into()
    }

    fn pane_ui(&mut self, ui: &mut Ui, tile_id: TileId, pane: &mut Pane) -> UiResponse {
        Frame::none()
            .inner_margin(Margin::symmetric(4.0, 2.0))
            .show(ui, |ui| {
                let response = Sides::new()
                    .show(
                        ui,
                        |ui| {
                            let response = ui
                                .heading(pane.title())
                                .on_hover_text(format!("{:x}", pane.hash()))
                                .on_hover_cursor(CursorIcon::Grab);
                            pane.header(ui);
                            response
                        },
                        |ui| {
                            ui.visuals_mut().button_frame = false;
                            if ui.button(RichText::new(X).heading()).clicked() {
                                self.close = Some(tile_id);
                            }
                        },
                    )
                    .0;
                pane.content(ui);
                if response.dragged() {
                    UiResponse::DragStarted
                } else {
                    UiResponse::None
                }
            })
            .inner
    }
}
