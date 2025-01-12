use polars::frame::DataFrame;
use polars_io::{SerReader as _, ipc::IpcReader};
use std::{io::Cursor, sync::LazyLock};
use utca::metadata::{IpcReaderExt as _, Metadata};

pub(crate) static CHRISTIE: LazyLock<(Option<Metadata>, DataFrame)> = LazyLock::new(|| {
    let bytes = include_bytes!("Christie.ipc");
    let mut reader = IpcReader::new(Cursor::new(bytes));
    let metadata = reader.metadata().expect("read metadata Christie.ipc");
    let data_frame = reader.finish().expect("read data Christie.ipc");
    (metadata, data_frame)
});
