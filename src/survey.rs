use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Clone)]
pub struct Survey {
    pub title: Option<String>,
    pub meta: Meta,
    pub record: Record,
}

impl Survey {
    pub fn empty() -> Self {
        Self {
            title: None,
            meta: Meta::default(),
            record: Record::default(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Meta {
    pub version: Option<String>,
    pub date: Option<String>,
    pub time: Option<String>,
    pub origin: Option<String>,
    pub user: Option<String>,
    pub style: Option<String>,
    pub note: Vec<String>,
    pub standard: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Record {
    pub id: String,
    pub content: Vec<Question>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Question {
    pub name: String,
    pub text: String,
    pub filter: Option<String>,
    pub location: Location,
    pub response: Response,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NumericSpecial {
    pub value: f64,
    pub label: String,
    pub filter: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    Quantity {
        min: f64,
        max: f64,
    },
    QuantitySpecial {
        min: f64,
        max: f64,
        special: Vec<NumericSpecial>,
    },
    Character(usize),
    Note(String),
    Single {
        item: BTreeMap<u64, Choice>,
    },
    Multiple {
        item: BTreeMap<u64, Choice>,
        spread: Spread,
    },
}

impl Default for Response {
    fn default() -> Self {
        Self::Single {
            item: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Choice {
    pub code: u64,
    pub label: String,
    pub filter: Option<String>,
}

impl Choice {
    pub fn new<T>(code: u64, label: impl Into<String>, filter: Option<T>) -> Self
    where
        T: Into<String>,
    {
        Self {
            code,
            label: label.into(),
            filter: filter.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub from: u64,
    pub to: u64,
}

impl Default for Location {
    fn default() -> Self {
        Self { from: 1, to: 1 }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Spread {
    pub num_subfields: u64,
    pub width: u64,
}

impl Default for Spread {
    fn default() -> Self {
        Self {
            num_subfields: 999,
            width: 999,
        }
    }
}
