use egui::{Ui, Window};
use egui_phosphor::regular::GEAR;
use polars::frame::DataFrame;
use polars_io::{SerReader as _, ipc::IpcReader};
use serde::{Deserialize, Serialize};
use std::{io::Cursor, sync::LazyLock};
use utca::metadata::{IpcReaderExt as _, Metadata};

pub(crate) static CHRISTIE: LazyLock<(Option<Metadata>, DataFrame)> = LazyLock::new(|| {
    let bytes = include_bytes!("Christie.ipc");
    let mut reader = IpcReader::new(Cursor::new(bytes));
    let metadata = reader.metadata().expect("read metadata Christie.ipc");
    let data_frame = reader.finish().expect("read data Christie.ipc");
    (metadata, data_frame)
});

/// Christie
#[derive(Default, Deserialize, Serialize)]
pub(crate) struct Christie {
    pub(crate) index: Option<usize>,
    pub(crate) open: bool,
}

impl Christie {
    pub(crate) const fn new() -> Self {
        Self {
            index: None,
            open: false,
        }
    }

    pub(crate) fn windows(&mut self, ui: &mut Ui) {
        Window::new(format!("{GEAR} Christie"))
            .id(ui.next_auto_id())
            .default_pos(ui.next_widget_position())
            .open(&mut self.open)
            .show(ui.ctx(), |ui| {
                ui.label("CHRISTIE");
            });
    }
}
