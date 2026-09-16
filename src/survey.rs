use std::{collections::BTreeMap, fmt::Write};

use rayon::prelude::*;

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

    /// find question via question name, case insensitive
    pub fn get_question(&self, name: &str) -> Option<&Question> {
        self.record
            .content
            .par_iter()
            .find_first(|question| question.name.to_uppercase() == name.to_uppercase())
    }

    pub fn to_sss_classic(&self) -> String {
        let mut output = String::from("SSS\n");
        output.push_str(&self.meta.to_sss_classic());
        append_line(&mut output, "SURVEY");
        if let Some(title) = self.title.as_ref() {
            append_line(&mut output, format!("TITLE {}", quote(title)));
        }
        output.push_str(&self.record.to_sss_classic());
        append_line(&mut output, "END SURVEY");
        append_line(&mut output, "END SSS");
        output
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

impl Meta {
    pub fn to_sss_classic(&self) -> String {
        let mut output = String::new();
        if let Some(value) = &self.version {
            append_line(&mut output, format!("VERSION {}", value));
        }
        for (name, value) in [
            ("DATE", &self.date),
            ("TIME", &self.time),
            ("ORIGIN", &self.origin),
            ("USER", &self.user),
        ] {
            if let Some(value) = value {
                append_line(&mut output, format!("{name} {}", quote(value)));
            }
        }
        for value in &self.note {
            append_line(&mut output, format!("NOTE {}", quote(value)));
        }
        for (name, value) in [("STANDARD", &self.standard), ("STYLE", &self.style)] {
            if let Some(value) = value {
                append_line(&mut output, format!("{name} {}", quote(value)));
            }
        }
        output
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Record {
    pub id: String,
    pub content: Vec<Question>,
}

impl Record {
    pub fn to_sss_classic(&self) -> String {
        let mut output = String::new();
        append_line(&mut output, format!("RECORD {}", self.id));
        for question in self.content.iter() {
            output.push_str(&question.to_sss_classic());
        }
        append_line(&mut output, "END RECORD");
        output
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Question {
    pub index: u64,
    pub name: String,
    pub text: String,
    pub filter: Option<String>,
    pub location: Location,
    pub response: Response,
}

impl Question {
    pub fn to_sss_classic(&self) -> String {
        let mut output = String::new();
        append_line(&mut output, format!("VARIABLE {}", self.index));
        append_line(&mut output, format!("NAME {}", quote(&self.name)));
        append_line(&mut output, format!("LABEL {}", quote(&self.text)));
        append_line(&mut output, self.location.to_sss_classic());
        append_line(&mut output, format!("END VARIABLE {}", self.index));
        output.push_str(&self.response.to_sss_classic());
        output
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    /// Quantity type of response.
    /// min/ max: minimun & maximun value allowed
    /// na_value: default value for empty entries (removed after triple-s 1.1)
    /// na_label: text label for empty entries (removed after triple-s 1.1)
    Quantity {
        min: f64,
        max: f64,
        na_value: Option<f64>,
        na_label: Option<String>,
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

impl Response {
    pub fn is_categorical(&self) -> bool {
        matches!(self, Self::Single { .. } | Self::Multiple { .. })
    }

    pub fn is_quantity(&self) -> bool {
        matches!(self, Self::Quantity { .. })
    }

    /// returns lookup table of code number <-> code label
    /// for single & multiple kinds of responses.
    pub fn codeframe(&self) -> Option<&BTreeMap<u64, Choice>> {
        match self {
            Self::Single { item } | Self::Multiple { item, .. } => Some(item),
            _ => None,
        }
    }

    pub fn to_sss_classic(&self) -> String {
        let mut output = String::new();
        match self {
            Self::Quantity {
                min,
                max,
                na_value,
                na_label,
            } => {
                append_line(&mut output, "TYPE QUANTITY");
                append_line(&mut output, "VALUES");

                // append_line(&mut output, format_range(*min, *max, "TO"));
                match na_value {
                    Some(na_value) => {
                        append_line(&mut output, format_range(*min, *max, "TO"));
                        let na_label = na_label.as_ref().map_or(r#""""#.into(), quote);
                        let na_line = format!("{} {} SPECIAL", na_value, na_label);
                        append_line(&mut output, na_line);
                    }
                    None => {
                        let base = format_range(*min, *max, "TO");
                        let range_line = format!("{} WITH", base);
                        append_line(&mut output, range_line);
                    }
                };
            }
            Self::Character(size) => {
                append_line(&mut output, "TYPE CHARACTER");
                append_line(&mut output, self.size_line(*size));
            }
            Self::Note(value) => {
                append_line(&mut output, "TYPE NOTE");
                append_line(&mut output, format!("LABEL {}", quote(value)));
            }
            Self::Single { item } => {
                append_line(&mut output, "TYPE SINGLE");
                write_choices(&mut output, item);
            }
            Self::Multiple { item, spread } => {
                append_line(&mut output, "TYPE MULTIPLE");
                append_line(&mut output, spread.to_sss_classic());
                write_choices(&mut output, item);
            }
        }
        output
    }

    fn size_line(&self, size: usize) -> String {
        format!("SIZE {size}")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Choice {
    pub code: u64,
    pub label: String,
    pub filter: Option<String>,
    /// (Optional) indicator for special values (e.g. mannotationissing data codes).
    /// Removed from SSS 1.2 onwards.
    pub is_special: bool,
}

impl Choice {
    pub fn new<T>(code: u64, label: T, filter: Option<T>) -> Self
    where
        T: Into<String>,
    {
        Self {
            code,
            label: label.into(),
            filter: filter.map(Into::into),
            is_special: false,
        }
    }

    pub fn to_sss_classic(&self) -> String {
        let base = format!("{} {}", self.code, quote(&self.label));
        if self.is_special {
            format!("{} SPECIAL", base)
        } else {
            base
        }
    }

    pub fn _set_special(mut self, is_special: bool) -> Self {
        self.is_special = is_special;
        self
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

impl Location {
    pub fn to_sss_classic(&self) -> String {
        if self.from == self.to {
            format!("POSITION {}", self.from)
        } else {
            format!("POSITION {} TO {}", self.from, self.to)
        }
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

impl Spread {
    pub fn to_sss_classic(&self) -> String {
        format!("SPREAD {} OF {}", self.num_subfields, self.width)
    }
}

#[inline]
fn quote(value: impl std::fmt::Display) -> String {
    format!(r#""{}""#, value)
}

#[inline]
fn append_line(output: &mut String, content: impl std::fmt::Display) {
    writeln!(output, "{content}").expect("writing to a String shouldn't fail");
}

#[inline]
fn write_choices(output: &mut String, choices: &BTreeMap<u64, Choice>) {
    append_line(output, "VALUES");
    for choice in choices.values() {
        append_line(output, choice.to_sss_classic());
    }
    append_line(output, "END VALUES");
}

#[inline]
fn format_range<T: std::fmt::Display>(from: T, to: T, delimiter: &str) -> String {
    if from.to_string() == to.to_string() {
        from.to_string()
    } else {
        format!("{from} {delimiter} {to}")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_write_spread() {
        let instance = Spread {
            num_subfields: 5,
            width: 10,
        };
        assert_eq!(
            instance.to_sss_classic(),
            format!("SPREAD {} OF {}", 5, 10)
        );
    }

    #[test]
    fn test_write_location() {
        let instance = Location { from: 0, to: 100 };
        assert_eq!(instance.to_sss_classic(), "POSITION 0 TO 100");
    }
}
