use chrono::NaiveDate;
use polars::{io::mmap::MmapBytesReader, prelude::*};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    hash::{Hash, Hasher},
    io::Write,
};
use thiserror::Error;

pub const DATE_FORMAT: &str = "%Y-%m-%d";

/// Metadata
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Metadata {
    pub name: String,
    pub description: String,
    pub authors: Vec<String>,
    pub version: Option<Version>,
    pub date: Option<NaiveDate>,
}

impl Metadata {
    pub fn title(&self) -> String {
        match &self.version {
            Some(version) => format!("{} {version}", self.name),
            None => self.name.to_owned(),
        }
    }
}

impl TryFrom<&BTreeMap<PlSmallStr, PlSmallStr>> for Metadata {
    type Error = Error;

    fn try_from(value: &BTreeMap<PlSmallStr, PlSmallStr>) -> Result<Self> {
        Ok(Self {
            name: value
                .get("name")
                .map_or_else(String::new, ToString::to_string),
            description: value
                .get("description")
                .map_or_else(String::new, ToString::to_string),
            authors: value.get("authors").map_or_else(Vec::new, |authors| {
                authors.split(",").map(ToOwned::to_owned).collect()
            }),
            version: value
                .get("version")
                .map(|version| Version::parse(version))
                .transpose()?,
            date: value
                .get("date")
                .map(|date| NaiveDate::parse_from_str(date, DATE_FORMAT))
                .transpose()?,
        })
    }
}

impl From<Metadata> for BTreeMap<PlSmallStr, PlSmallStr> {
    fn from(value: Metadata) -> Self {
        let mut metadata = BTreeMap::new();
        metadata.insert("name".into(), value.name.into());
        metadata.insert("description".into(), value.description.into());
        metadata.insert("authors".into(), value.authors.join(",").into());
        if let Some(version) = value.version {
            metadata.insert("version".into(), version.to_string().into());
        }
        if let Some(date) = value.date {
            metadata.insert("date".into(), date.format(DATE_FORMAT).to_string().into());
        }
        metadata
    }
}

/// MetaDataFrame
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct MetaDataFrame {
    pub meta: Metadata,
    pub data: DataFrame,
}

impl MetaDataFrame {
    pub fn new(meta: Metadata, data: DataFrame) -> Self {
        Self { meta, data }
    }

    pub fn read(reader: impl MmapBytesReader) -> Result<Self> {
        let mut reader = IpcReader::new(reader);
        let meta = reader.metadata()?.unwrap_or_default();
        let data = reader.finish()?;
        Ok(Self { meta, data })
    }
}

impl Hash for MetaDataFrame {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.meta.hash(state);
        for series in self.data.iter() {
            for value in series.iter() {
                value.hash(state);
            }
        }
    }
}

/// Extension methods for [`IpcReader`]
pub trait IpcReaderExt {
    fn metadata(&mut self) -> Result<Option<Metadata>>;
}

impl<R: MmapBytesReader> IpcReaderExt for IpcReader<R> {
    fn metadata(&mut self) -> Result<Option<Metadata>> {
        let Some(metadata) = self.custom_metadata()? else {
            return Ok(None);
        };
        let metadata = Metadata::try_from(&*metadata)?;
        Ok(Some(metadata))
    }
}

/// Extension methods for [`IpcWriter`]
pub trait IpcWriterExt {
    fn metadata(&mut self, metadata: Metadata);
}

impl<W: Write> IpcWriterExt for IpcWriter<W> {
    fn metadata(&mut self, metadata: Metadata) {
        self.set_custom_schema_metadata(Arc::new(metadata.into()));
    }
}

/// Result
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Error
#[derive(Error, Debug)]
pub enum Error {
    #[error(r#"metadata date "{0}""#)]
    Date(#[from] chrono::ParseError),
    #[error(r#"metadata version "{0}""#)]
    Version(#[from] semver::Error),
    #[error(r#"polars metadata "{0}""#)]
    Polars(#[from] PolarsError),
}
