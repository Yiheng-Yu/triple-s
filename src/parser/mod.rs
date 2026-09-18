#[cfg(feature = "classic")]
mod classic;

use std::{collections::HashMap, sync::Arc};

#[cfg(feature = "classic")]
pub use classic::read_sss_file_classic;
// use rayon::prelude::*;

/// A xml-like node, used for parsing sss files of different structure
struct Node {
    pub tag: Arc<str>,
    pub attributes: HashMap<Arc<str>, Arc<str>>,
    pub value: Arc<str>,
    pub children: Vec<Box<Node>>,
}

impl Node {
    pub fn new(tag: &str) -> Self {
        Self {
            tag: Arc::from(tag),
            attributes: HashMap::new(),
            value: Arc::from(""),
            children: Vec::new()
        }
    }

    pub fn set_value(&mut self, value: &str) {
        self.value = Arc::from(value);
    }
}

struct SurveyBuilder {
    // name, id, survey meta etc.,
    pub meta: Node,
    pub content: Node
}