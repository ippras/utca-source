use self::table::TableView;
use crate::app::presets::CHRISTIE;
use ahash::RandomState;
use egui::{Context, Ui, Window, util::hash};
use egui_phosphor::regular::GEAR;
use metadata::{IpcReaderExt as _, Metadata};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    hash::{BuildHasher, Hash as _, Hasher as _},
    io::Cursor,
    sync::LazyLock,
};

/// Christie pane
#[derive(Default, Deserialize, Serialize)]
pub(crate) struct Pane {
    // pub(crate) data_frame: DataFrame,
    // pub(crate) control: Control,
}

impl Pane {
    pub(crate) const fn new() -> Self {
        Self {
            // data_frame: (*CHRISTIE).1.clone(),
            // control: Control::new(),
        }
    }

    pub(crate) fn content(&mut self, ui: &mut Ui) {
        ui.separator();
        // self.control.windows(ui);
        TableView::new(&CHRISTIE.data).ui(ui);
        // if let Err(error) = self.delete_row(row) {
        //     error!(%error);
        // }
    }

    pub(super) fn hash(&self) -> u64 {
        let hash_builder = RandomState::with_seeds(1, 2, 3, 4);
        let mut hasher = hash_builder.build_hasher();
        CHRISTIE.meta.hash(&mut hasher);
        hasher.finish()

        // let state = ahash::RandomState::with_seeds(1, 2, 3, 4);
        // state.hash_one(x)
        // hash(&CHRISTIE.1)
    }
}

mod table;
