#[cfg(test)]
mod test {
    use super::{metadata::Metadata, *};
    use anyhow::Result;
    use chrono::NaiveDate;
    use maplit::btreemap;
    use metadata::{DATE_FORMAT, IpcReaderExt as _};
    use polars::prelude::*;
    use semver::Version;
    use std::{fs::File, path::Path, sync::LazyLock};

    static AUTHORS: LazyLock<Vec<String>> =
        LazyLock::new(|| vec!["Giorgi Kazakov".into(), "Roman Sidorov".into()]);

    #[test]
    fn convert_ron_to_ipc() -> Result<()> {
        let path = Path::new(
            r#"C:\Users\9\git\ippras\utca-configs\LunariaRediviva\2024-01-24\LunariaRediviva.3.3.utca.ron"#,
        );
        let file = File::open(path)?;
        let mut data_frame: DataFrame = ron::de::from_reader(file)?;
        println!("data_frame: {data_frame}");
        let metadata = Metadata {
            version: Some(Version::new(3, 3, 0)),
            name: "Lunaria rediviva".into(),
            description: String::new(),
            authors: AUTHORS.clone(),
            date: Some(NaiveDate::parse_from_str("2024-01-24", DATE_FORMAT)?),
        };
        write(path.with_extension("ipc"), &mut data_frame, Some(metadata))?;
        Ok(())
    }

    #[test]
    fn read_ipc() -> Result<()> {
        let path = "Lunaria rediviva 1.1.0.utca.ipc";
        // let path = "Christie.0.1.0.ipc";
        // let path = Path::new(
        //     r#"C:\Users\9\git\ippras\utca-configs\LunariaRediviva\2024-01-24\LunariaRediviva.1.1.utca.ipc"#,
        // );
        let (data_frame, metadata) = read(path)?;
        println!("metadata: {metadata:#?}");
        println!("data_frame: {data_frame}");
        Ok(())
    }

    #[test]
    fn convert_fatty_acid() -> Result<()> {
        let path = Path::new(
            r#"C:\Users\9\git\ippras\utca-configs\LunariaRediviva\2024-01-24\LunariaRediviva.1.1.utca.ron"#,
        );
        let (mut data_frame, metadata) = read(path)?;
        println!("data_frame: {data_frame}");
        let mut lazy_frame = data_frame.lazy();
        lazy_frame = lazy_frame
            .with_columns([as_struct(vec![
                col("FA").struct_().field_by_name("Carbons"),
                col("FA")
                    .struct_()
                    .field_by_name("Doubles")
                    .list()
                    .eval(
                        as_struct(vec![
                            col("").cast(DataType::UInt8).alias("Index"),
                            col("").sign().cast(DataType::Int8).alias("Isomerism"),
                            lit(1).cast(DataType::UInt8).alias("Unsaturation"),
                        ]),
                        true,
                    )
                    .alias("Unsaturated"),
            ])
            .alias("FattyAcid")])
            .select([col("FattyAcid"), col("Christie")]);
        data_frame = lazy_frame.with_row_index("Index", None).collect().unwrap();
        println!("data_frame: {data_frame}");
        write(path.with_file_name("out.ipc"), &mut data_frame, metadata)?;
        Ok(())
    }

    fn read(path: impl AsRef<Path>) -> Result<(DataFrame, Option<Metadata>)> {
        let file = File::open(path)?;
        let mut reader = IpcReader::new(file);
        let metadata = reader.metadata()?;
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
}

pub mod metadata;
