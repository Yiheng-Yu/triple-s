// //! Configuration options for reading/ exporting/ intereacting with objects

// use pyo3::prelude::*;
// #[cfg(feature="pyo3")]
// use strum::{EnumString, EnumIter};
// use crate::prelude::*;

// #[derive(Debug, Clone)]
// pub enum Config {
//     Libarary(TableKind)
// }

// /// Specifies the DataFrame library used for Python output, (default `Polars`).
// #[cfg_attr(feature = "pyo3", pyclass(from_py_object))]
// #[derive(Default, Debug, Clone, EnumString, EnumIter)]
// pub enum TableKind {
//     /// Polars dataframe
//     #[default]
//     #[strum(serialize = "polars")]
//     Polars,

//     /// Pandas dataframe
//     #[strum(serialize = "pandas")]
//     Pandas
// }

// impl Flag for TableKind {
//     fn to_string(&self) -> String {
//         match self {
//             Self::Polars => "polars".into(),
//             Self::Pandas => "pandas".into(),
//         }
//     }
// }
