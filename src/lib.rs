#[cfg(test)]
mod test {
    use anyhow::{Error, Result};
    use chrono::{DateTime, Local};
    use maplit::btreemap;
    use polars::prelude::*;
    use semver::Version;
    use std::{collections::BTreeMap, fs::File, path::Path};
    use url::form_urlencoded::parse;

    #[test]
    fn convert_ron_to_ipc() -> Result<()> {
        let path = "Christie.ron";
        let file = File::open(path)?;
        let mut data_frame: DataFrame = ron::de::from_reader(file)?;
        println!("data_frame: {data_frame}");
        let metadata = btreemap! {
            "Name".into() => "Name".into(),
            "CreatedBy".into() => "Created by".into(),
            "Version".into() => "0.0.1".into(),
        };
        write("Christie.ipc", &mut data_frame, metadata.try_into().ok())?;
        Ok(())
    }

    #[test]
    fn read_christie() -> Result<()> {
        // let path = "data_frame.utca.ipc";
        let (mut data_frame, mut metadata) = read("Christie.ipc")?;
        println!("data_frame: {data_frame}");
        println!("metadata: {metadata:?}");
        if let Some(metadata) = &mut metadata {
            metadata.version = Some(Version::parse("0.1.0")?);
            metadata.name = "Christie".into();
            metadata.description = "".to_owned();
            metadata.authors = vec!["Giorgi Kazakov".into(), "Roman Sidorov".into()];
        } else {
            metadata = Some(Metadata {
                version: Some(Version::parse("0.1.0")?),
                name: "Christie".into(),
                description: "".to_owned(),
                authors: vec!["Giorgi Kazakov".into(), "Roman Sidorov".into()],
                timestamp: Some(Local::now()),
            })
        }
        write("Christie.ipc", &mut data_frame, metadata)?;
        Ok(())
    }

    fn read(path: impl AsRef<Path>) -> Result<(DataFrame, Option<Metadata>)> {
        let file = File::open(path)?;
        let mut reader = IpcReader::new(file);
        let metadata = if let Some(metadata) = reader.custom_metadata()? {
            println!("metadata: {metadata:?}");
            Some(Metadata::try_from(&*metadata)?)
        } else {
            None
        };
        let data_frame = reader.finish()?;
        Ok((data_frame, metadata))
    }

    fn write(
        path: impl AsRef<Path>,
        data_frame: &mut DataFrame,
        metadata: Option<Metadata>,
    ) -> Result<()> {
        let mut file = File::create(path)?;
        let mut writer = IpcWriter::new(&mut file);
        if let Some(metadata) = metadata {
            writer.set_custom_schema_metadata(Arc::new(metadata.into()));
        }
        writer.finish(data_frame)?;
        Ok(())
    }

    #[derive(Clone, Debug, Default)]
    pub struct Metadata {
        pub version: Option<Version>,
        pub name: String,
        pub description: String,
        pub authors: Vec<String>,
        pub timestamp: Option<DateTime<Local>>,
    }

    impl TryFrom<BTreeMap<PlSmallStr, PlSmallStr>> for Metadata {
        type Error = Error;

        fn try_from(value: BTreeMap<PlSmallStr, PlSmallStr>) -> Result<Self> {
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
                timestamp: value.get("timestamp"),
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
            if let Some(timestamp) = value.timestamp {
                metadata.insert("timestamp".into(), timestamp.to_string().into());
            }
            metadata
        }
    }
}
