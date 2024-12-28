use chrono::NaiveDate;
use polars::{io::mmap::MmapBytesReader, prelude::*};
use semver::Version;
use std::collections::BTreeMap;
use thiserror::Error;

pub const DATE_FORMAT: &str = "%Y-%m-%d";

#[derive(Clone, Debug, Default)]
pub struct Metadata {
    pub version: Option<Version>,
    pub name: String,
    pub description: String,
    pub authors: Vec<String>,
    pub date: Option<NaiveDate>,
}

impl TryFrom<&BTreeMap<PlSmallStr, PlSmallStr>> for Metadata {
    type Error = Error;

    fn try_from(value: &BTreeMap<PlSmallStr, PlSmallStr>) -> Result<Self> {
        Ok(Self {
            version: value
                .get("version")
                .map(|version| Version::parse(version))
                .transpose()?,
            name: value
                .get("name")
                .map_or_else(String::new, ToString::to_string),
            description: value
                .get("description")
                .map_or_else(String::new, ToString::to_string),
            authors: value.get("authors").map_or_else(Vec::new, |authors| {
                authors.split(", ").map(ToOwned::to_owned).collect()
            }),
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
        if let Some(version) = value.version {
            metadata.insert("version".into(), version.to_string().into());
        }
        metadata.insert("name".into(), value.name.into());
        metadata.insert("description".into(), value.description.into());
        metadata.insert("authors".into(), value.authors.join(", ").into());
        if let Some(date) = value.date {
            metadata.insert("date".into(), date.format(DATE_FORMAT).to_string().into());
        }
        metadata
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
