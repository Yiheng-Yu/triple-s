// use std::str::FromStr;

// ///! Used for creating new instances of survey members from rust
// use anyhow::Result;

// use crate::entities::{
//     CodeFormat, DataSource, Mode, Version
// };

// pub struct SurveyOptions {
//     name: String,
//     mode: Mode,
//     version: Version,
//     title: Option<String>,
//     data_source: DataSource,
// }

// impl Default for SurveyOptions {
//     fn default() -> Self {
//         Self {
//             name: "survey".into(),
//             title: None,
//             mode: Mode::None,
//             version: Version::V1_2,
//             data_source: DataSource::Fixed
//         }
//     }
// }

// impl SurveyOptions {
//     pub fn with_name(mut self, name: impl Into<String>) -> Self {
//         self.name = name.into();
//         self
//     }

//     pub fn with_fixed_data_format(mut self) -> Self {
//         self.data_source = DataSource::Fixed;
//         self
//     }

//     pub fn with_csv_data_format(mut self) -> Self {
//         self.data_source = DataSource::Csv;
//         self
//     }

//     pub fn with_version(mut self, version: Version) -> Self {
//         self.version = version;
//         self
//     }
// }