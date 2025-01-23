use constcat::concat;
use egui::{
    Align, Context, Id, Layout, Response, SelectableLabel, TextEdit, Ui, Widget, Window,
    mutex::Mutex, util::undoer::Undoer,
};
use egui_phosphor::regular::{EYE, GEAR};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const ID_SOURCE: &str = "Settings";

/// Settings window
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct SettingsWindow {
    pub(crate) open: bool,
}

impl SettingsWindow {
    pub(crate) const fn new() -> Self {
        Self { open: false }
    }

    const fn title(&self) -> &'static str {
        concat!(GEAR, " Settings")
    }

    pub(crate) fn show(&mut self, ctx: &Context) {
        Window::new(self.title())
            .open(&mut self.open)
            .resizable([true, false])
            .show(ctx, settings);
    }
}

impl Default for SettingsWindow {
    fn default() -> Self {
        Self::new()
    }
}

fn settings(ui: &mut Ui) -> Response {
    let id_salt = Id::new(ID_SOURCE);
    let id = ui.make_persistent_id(id_salt);
    // Load
    let mut state = SettingsState::load(ui.ctx(), id).unwrap_or_default();
    let mut show = ui.data_mut(|data| data.get_temp::<String>(id).unwrap_or(false));
    let response = ui
        .horizontal(|ui| {
            ui.label("Token");
            ui.add(TextEdit::singleline(&mut state.token).password(!state.password));
            let response = ui
                .add(SelectableLabel::new(state.password, EYE))
                .on_hover_text("Show/hide token");
            if response.clicked() {
                state.password = !state.password;
            }
            response
        })
        .inner;
    // Store
    state.store(ui.ctx(), id);
    response
}

/// State
pub trait State: Sized {
    fn load(ctx: &Context, id: Id) -> Option<Self>;

    fn store(self, ctx: &Context, id: Id);

    fn reset(ctx: &Context, id: Id);
}

/// Settings state
#[derive(Clone, Default, Deserialize, Serialize)]
pub(crate) struct SettingsState {
    pub(crate) token: String,
    pub(crate) password: bool,

    /// Wrapped in Arc for cheaper clones.
    #[serde(skip)]
    pub(crate) undoer: Arc<Mutex<SettingsUndoer>>,
}

impl State for SettingsState {
    fn load(ctx: &Context, id: Id) -> Option<Self> {
        ctx.data_mut(|data| data.get_persisted(id))
    }

    fn store(self, ctx: &Context, id: Id) {
        ctx.data_mut(|data| data.insert_persisted(id, self));
    }

    fn reset(ctx: &Context, id: Id) {
        ctx.data_mut(|data| {
            data.remove::<Self>(id);
        });
    }
}

/// Settings undoer
pub(crate) type SettingsUndoer = Undoer<(String, bool)>;

impl SettingsState {
    // pub(crate) fn undoer(&self) -> SettingsUndoer {
    //     self.undoer.lock().clone()
    // }

    // #[allow(clippy::needless_pass_by_ref_mut)] // Intentionally hide interiority of mutability
    // pub(crate) fn set_undoer(&mut self, undoer: SettingsUndoer) {
    //     *self.undoer.lock() = undoer;
    // }

    // pub(crate) fn clear_undoer(&mut self) {
    //     self.set_undoer(SettingsUndoer::default());
    // }
}

pub fn password(password: &mut String) -> impl Widget {
    move |ui: &mut Ui| {
        let id_salt = Id::new(ID_SOURCE).with("Token");
        let id = ui.make_persistent_id(id_salt);
        let mut show = ui.data_mut(|data| data.get_temp::<bool>(id).unwrap_or(false));
        let response = ui
            .with_layout(Layout::right_to_left(Align::Center), |ui| {
                let response = ui
                    .add(SelectableLabel::new(show, "👁"))
                    .on_hover_text("Show/hide token");
                if response.clicked() {
                    show = !show;
                }
                ui.add_sized(
                    ui.available_size(),
                    TextEdit::singleline(password).password(!show),
                );
            })
            .response;
        ui.data_mut(|data| data.insert_temp(id, show));
        response
    }
}
// pub fn password(password: &mut String) -> impl Widget {
//     move |ui: &mut Ui| {
//         let id_salt = Id::new(ID_SOURCE).with("Token");
//         let id = ui.make_persistent_id(id_salt);
//         let mut show = ui.data_mut(|data| data.get_temp::<bool>(id).unwrap_or(false));
//         let response = ui
//             .with_layout(Layout::right_to_left(Align::Center), |ui| {
//                 let response = ui
//                     .add(SelectableLabel::new(show, "👁"))
//                     .on_hover_text("Show/hide token");
//                 if response.clicked() {
//                     show = !show;
//                 }
//                 ui.add_sized(
//                     ui.available_size(),
//                     TextEdit::singleline(password).password(!show),
//                 );
//             })
//             .response;
//         ui.data_mut(|data| data.insert_temp(id, show));
//         response
//     }
// }
