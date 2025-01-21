use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub(crate) struct State {
    pub(crate) open_settings_window: bool,
    pub(crate) reset_table_state: bool,
    pub(crate) view: View,
}

impl State {
    pub(crate) const fn new() -> Self {
        Self {
            open_settings_window: false,
            reset_table_state: false,
            view: View::Table,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub(crate) enum View {
    Plot,
    #[default]
    Table,
}

impl View {
    pub(crate) fn toggle(&mut self) {
        *self = match self {
            Self::Plot => Self::Table,
            Self::Table => Self::Plot,
        };
    }
}
