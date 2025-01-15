use anyhow::Result;
use metadata::MetaDataFrame;
use std::fs::File;

#[cfg(not(target_arch = "wasm32"))]
pub fn save(name: &str, frame: &mut MetaDataFrame) -> Result<()> {
    let file = File::create(name)?;
    MetaDataFrame::new(frame.meta.clone(), &mut frame.data).write(file)?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn save(name: &str, frame: &mut MetaDataFrame) -> Result<()> {
    use anyhow::anyhow;
    use egui_ext::download;

    let mut bytes = Vec::new();
    MetaDataFrame::new(frame.meta.clone(), &mut frame.data).write(&mut bytes)?;
    download(name, &bytes).map_err(|error| anyhow!(error))
}
