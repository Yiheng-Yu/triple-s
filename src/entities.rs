use std::{collections::BTreeMap, fmt::Write, str::FromStr};

use strum_macros::EnumString;
use rayon::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumString)]
pub enum Version {
    #[strum(serialize = "1.0")]
    V1_0,
    #[strum(serialize = "1.1")]
    V1_1,
    #[strum(serialize = "1.2")]
    V1_2,
    #[strum(serialize = "2.0")]
    V2_0,
    #[strum(serialize = "3.0")]
    V3_0,
}

impl Version {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::V1_0 => "1.0",
            Self::V1_1 => "1.1",
            Self::V1_2 => "1.2",
            Self::V2_0 => "2.0",
            Self::V3_0 => "3.0",
        }
    }
}


#[derive(Debug, Default, Clone, PartialEq)]
pub struct Survey {

}

// #[derive(Debug, Clone, PartialEq)]
// pub struct Spread {
//     /// The number of subfields within the overall field that is defined by the <position> element.
//     pub num_subfields: u64,
//     /// Denotes the width of each subfield.
//     pub width: Option<u64>,
// }


#[derive(Debug, Default, Clone, PartialEq)]
pub struct Text {
    /// the default value
    pub value: String,

    /// variants
    #[cfg(feature="xml")]
    pub variant: Vec<TextVariant>,
}

impl Text {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            value: content.into(),
            variant: Vec::new(),
        }
    }

    /// Will modify existing variant if they have the same `mode` and `language` attribute
    /// or append to the current `self.variant` if not
    #[cfg(feature="xml")]
    pub fn set_variant(&mut self, variant: TextVariant) {
        for item in self.variant.iter_mut() {
            if item.has_same_attribute(&variant) {
                item.value = variant.value.clone();
                return;
            }
        }
        self.variant.push(variant)
    }

    pub fn to_sss_classic(&self) -> String {
        self.value.clone()
    }
}

/// From sss xml 2.0 onwards supports multi-language
///
/// For sss xml 2.0:
/// ```text
/// The text for a survey <title>, variable <label> or values
/// <value> element may contain any number of specialised text elements.
/// ```
///
/// Example sss xml 2.0 text, note that one entry can contain both `label` and `language` attribute
/// ```xml
/// <label>Age
///     <text xml:lang="en-GB" mode="interview">How old are you?</text>
///     <text xml:lang="fr" mode="interview">Quel est votre âge?</text>
///     <text xml:lang="en-GB" mode="analysis">Age of respondent</text>
///     <text xml:lang="fr" mode="analysis">Âge de répondant</text>
/// </label>
/// ```
///
/// -> TODO: Impl this AFTER the sss xml website when up again
#[cfg(feature="xml")]
#[derive(Debug, Clone, PartialEq)]
pub struct TextVariant {
    pub value: String,
    pub language: Language,
    pub mode: Mode,
}

impl TextVariant {
    pub fn new<S>(value: S, mode: Mode, language: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            value: value.into(),
            mode,
            language: Language::Named(language.into()),
        }
    }

    pub fn with_language(
        content: impl Into<String>,
        language_name: impl Into<String>,
    ) -> Self {
        Self {
            value: content.into(),
            language: Language::Named(language_name.into()),
            mode: Mode::None,
        }
    }

    /// set `language` to `Language::Name(value)`
    pub fn set_language(&mut self, new: impl Into<String>) {
        self.language = Language::Named(new.into());
    }

    /// reset `language` back to `Default`
    pub fn reset_language(&mut self) {
        self.language = Language::None;
    }

    /// check if instance of `other` has the same attributes as `self`
    pub fn has_same_attribute(&self, other: &Self) -> bool {
        (self.mode == other.mode) & (self.language == other.language)
    }
}


#[cfg(feature="xml")]
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Language {
    /// undefined
    #[default]
    None,
    /// Language as defined by W3C XML version 1.0 specification
    Named(String),
}

impl Language {
    pub fn named(name: impl Into<String>) -> Self {
        Self::Named(name.into())
    }
}

/// Introduced in SSS Xml 2.0.
///
/// Official documentation:
///
/// ```text
/// (Optional) The use of specialised texts for interviewing and/or analysis within the
/// specification must be signalled by a modes="mode_identifier_list"
/// attribute on the initial <sss> element.   
/// Two explicit modes are available: “interview” and “analysis”. In the absence of
/// a mode specification, the appropriate text is assumed to be used in both
/// modes.
/// ```
///
/// Variants:
/// - None: default value when no 'mode' value was defined.
/// - Analysis: mode 'analysis'
/// - Interview: mode 'interview'
#[derive(Debug, PartialEq, Clone, EnumString)]
#[strum(ascii_case_insensitive)]
#[derive(Default)]
pub enum Mode {
    #[default]
    None,
    #[strum(serialize = "analysis")]
    Analysis,
    #[strum(serialize = "interview")]
    Interview,
}

// #[derive(Debug, Clone, PartialEq)]
// pub struct Choice {
//     pub code: u64,
//     /// text label
//     pub label: Text,
//     /// Response filter.
//     pub filter: Option<String>,
//     pub score: Option<f64>,
//     /// (Optional) indicator for special values (e.g. mannotationissing data codes).
//     /// Removed from SSS 1.2 onwards.
//     pub is_special: bool,
// }