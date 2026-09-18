use std::sync::{Arc, LazyLock};

use anyhow::Result;
use regex::{Regex, regex};

use super::{ Node, Tag};

static RE_PREFIX: LazyLock<Regex> =
    LazyLock::new(|| regex!(r"^(?P<prefix>[A-Z]+)?[\s]?(?P<value>.*)$").to_owned());


impl TryFrom<String> for Node {
    type Error = anyhow::Error;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        match RE_PREFIX.captures(&value) {
            Some(captured) => todo!(),
            None => todo!()
        }
    }
}


impl TryFrom<String> for Tag {
    type Error = anyhow::Error;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        todo!()
    }
}

pub fn read_sss_file_classic<I>(lines: I) -> Result<Node>
where
    I: IntoIterator<Item = Result<String, anyhow::Error>>,
{
    let mut root = Node::new("sss".into());
    
    for (line_number, raw_line) in lines.into_iter().enumerate() {
        // let element: Node = raw_line?.into();
        todo!()
    }

    Ok(root)
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
