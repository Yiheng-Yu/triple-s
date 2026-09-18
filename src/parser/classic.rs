use std::sync::{Arc, LazyLock};

use anyhow::Result;
use regex::{Regex, regex};

use super::Node;
use crate::entities::{Survey, Text};

static RE_PREFIX: LazyLock<Regex> =
    LazyLock::new(|| regex!(r"^(?P<prefix>[A-Z]+)?[\s]?(?P<value>.*)$").to_owned());

/// represents one single raw line in a .sss classic file
struct Line<'a>(&'a str);

impl<'a> From<&'a str> for Line<'a> {
    fn from(value: &'a str) -> Self {
        Self(value)
    }
}


pub fn read_sss_file_classic<I, E>(lines: I) -> Result<Survey>
where
    I: IntoIterator<Item = Result<String, E>>,
    E: std::error::Error,
{
    let mut survey = Survey::empty();
    // let mut current = None;
    for (line_number, raw_line) in lines.into_iter().enumerate() {
        todo!()
    }

    Ok(survey)
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_re_prefix_success() {
        let examples = [
            ("1 TO 5", "", "1 TO 5"),
            (r#"POSITION 1 TO 5"#, "POSITION", "1 TO 5"),
            (r#"TYPE"#, "TYPE", ""),
            (r#"999.9"#, "", "999.9"),
            ("", "", ""),
        ];
        for (text, corr_prefix, corr_value) in examples.into_iter() {
            let captured = RE_PREFIX
                .captures(text)
                .expect(format!("Error capturing text: '{}'", text).as_str());

            assert_eq!(
                captured
                    .name("prefix")
                    .map(|s| s.as_str())
                    .unwrap_or_default(),
                corr_prefix
            );
            assert_eq!(
                captured
                    .name("value")
                    .map(|s| s.as_str())
                    .unwrap_or_default(),
                corr_value
            );
        }
    }
}
