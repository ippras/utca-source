#[cfg(not(target_arch = "wasm32"))]
pub(crate) use self::native::save;
#[cfg(target_arch = "wasm32")]
pub(crate) use self::web::save;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use anyhow::Result;
    use polars::prelude::*;
    use std::fs::File;
    use utca::metadata::{IpcWriterExt as _, MetaDataFrame};

    pub(crate) fn save(name: &str, frame: &mut MetaDataFrame) -> Result<()> {
        let file = File::create(name)?;
        let mut writer = IpcWriter::new(file);
        writer.metadata(frame.meta.clone());
        writer.finish(&mut frame.data)?;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use anyhow::Result;
    use eframe::{
        wasm_bindgen::{JsCast, JsValue, prelude::*},
        web_sys::{Blob, File, FilePropertyBag, HtmlAnchorElement, Url, window},
    };
    use gloo_utils::error::JsError;
    use js_sys::{Array, ArrayBuffer, Uint8Array};
    use polars::prelude::*;
    use utca::metadata::{IpcWriterExt as _, MetaDataFrame};

    pub(crate) fn save(name: &str, frame: &mut MetaDataFrame) -> Result<()> {
        let mut bytes = Vec::new();
        let mut writer = IpcWriter::new(&mut bytes);
        writer.metadata(frame.meta.clone());
        writer.finish(&mut frame.data)?;
        Ok(download(name, &bytes)?)
    }

    /// * https://stackoverflow.com/questions/3665115/how-to-create-a-file-in-memory-for-user-to-download-but-not-through-server
    /// * https://stackoverflow.com/questions/69556755/web-sysurlcreate-object-url-with-blobblob-not-formatting-binary-data-co
    /// * https://github.com/emilk/egui/discussions/3571
    fn download(name: &str, bytes: &[u8]) -> Result<(), JsError> {
        let window = window().expect("window not found");
        let document = window.document().expect("document not found");
        let body = document.body().expect("body not found");

        let output: HtmlAnchorElement = document.create_element("a")?.dyn_into()?;
        output.style().set_property("display", "none")?;
        output.set_href(&file(name, bytes)?);
        output.set_download(name);
        output.click();
        Ok(())
    }

    fn file(name: &str, bytes: &[u8]) -> Result<String, JsError> {
        let bytes = Uint8Array::from(bytes);
        let array = Array::new();
        array.push(&bytes.buffer());
        let file = File::new_with_blob_sequence_and_options(
            &array,
            name,
            FilePropertyBag::new().type_("application/octet-stream"),
        )?;
        Ok(Url::create_object_url_with_blob(&file)?)
    }
}
