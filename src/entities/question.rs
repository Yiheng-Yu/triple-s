#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

use crate::{
    entities::{CodeFormat, Text, Usage}, prelude::DeepCopy, utils::ArcLock,
};

#[cfg_attr(feature = "pyo3", pyclass(module = "triple_s", name = "Question"))]
#[derive(Debug)]
pub struct Question {
    pub index: u64,
    pub name: ArcLock<String>,
    pub label: ArcLock<Text>,
    pub filter: ArcLock<Option<String>>,
    pub format: ArcLock<CodeFormat>,
    pub usage: ArcLock<Usage>,
    pub location: ArcLock<Location>,
    // pub response: Response,
}

impl DeepCopy for Question {
    type Output = Self;
    fn deepcopy(&self) -> Self {
        Self {
            index: self.index.clone(),
            name: self.name.deepcopy(),
            label: self.label.deepcopy(),
            filter: self.filter.deepcopy(),
            format: self.format.deepcopy(),
            usage: self.usage.deepcopy(),
            location: self.location.deepcopy(),
        }
    }
}

/// Defines the part of the data record that is allocated to holding the value of the variable.
#[cfg_attr(feature = "pyo3", pyclass(module = "triple_s", name = "Location", get_all, set_all, from_py_object))]
#[derive(Debug, Clone, PartialEq, Default, Eq)]
pub struct Location {
    pub from: u64,
    pub to: u64,
}

impl DeepCopy for Location {
    type Output = Location;
    fn deepcopy(&self) -> Self::Output {
        Self {
            from: self.from.clone(),
            to: self.to.clone()
        }
    }
}
// impl Position {
//     pub fn to_sss_classic(&self) -> String {
//         if self.to != self.from {
//             format!("POSITION {} TO {}", self.from, self.to)
//         } else {
//             format!("POSITION {}", self.from)
//         }
//     }
// }