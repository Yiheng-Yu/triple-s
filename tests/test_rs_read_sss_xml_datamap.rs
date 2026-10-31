use triple_s::{Datamap, read_datamap};

const PATH_SSS_XML_GENERATION_ONE: &str = "tests/sss-examples/sss_xml_1.0.xml";
const PATH_SSS_XML_GENERATION_TWO_EXAMPLE_1: &str = "tests/sss-examples/sss-2.0/example1.sss";
const PATH_SSS_XML_GENERATION_TWO_EXAMPLE_2: &str = "tests/sss-examples/sss-2.0/example2.sss";

fn example_datamap_xml_generation_one() -> Datamap {
    read_datamap(PATH_SSS_XML_GENERATION_ONE).unwrap()
}

fn example_datamap_xml_generation_two_example_one() -> Datamap {
    read_datamap(PATH_SSS_XML_GENERATION_TWO_EXAMPLE_1).unwrap()
}

fn example_datamap_xml_generation_two_example_two() -> Datamap {
    read_datamap(PATH_SSS_XML_GENERATION_TWO_EXAMPLE_2).unwrap()
}

#[test]
fn test_read_xml_datamap_gen_1() {
    let datamap = example_datamap_xml_generation_one();
    assert_eq!(datamap.record().content().len(), 8);
}

#[test]
fn test_read_xml_datamap_ver_2_example_1() {
    let datamap = example_datamap_xml_generation_two_example_one();
    assert_eq!(datamap.record().content().len(), 12);
}

#[test]
fn test_read_xml_datamap_ver_2_example_2() {
    let datamap = example_datamap_xml_generation_two_example_two();
    assert_eq!(datamap.record().content().len(), 12);
}

#[test]
fn test_read_xml_replaces_invalid_utf8_in_text() {
    use std::io::Write;

    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<sss version="2.0"><survey><title>Café INVALID &amp; survey</title>
<record ident="A"><variable ident="1" type="single">
<name>Q1</name><label>Question INVALID 中文<text mode="analysis">Variant INVALID</text></label>
<position start="1" finish="1"/><values>
<value code="1">Answer INVALID<text mode="analysis">Code INVALID</text></value>
</values></variable></record></survey></sss>"#;
    let mut invalid = Vec::new();
    for (index, part) in xml.split("INVALID").enumerate() {
        if index > 0 {
            invalid.push(0xff);
        }
        invalid.extend_from_slice(part.as_bytes());
    }
    let mut file = tempfile::Builder::new().suffix(".sss").tempfile().unwrap();
    file.write_all(&invalid).unwrap();
    let datamap = read_datamap(file.path()).unwrap();
    assert_eq!(datamap.title().as_deref(), Some("Café � & survey"));
    let record = datamap.record();
    let questions = record.content();
    let question = questions[0].read();
    assert_eq!(question.label().value(), "Question � 中文");
    assert_eq!(question.label().variants()[0].value(), "Variant �");
    let response = question.response();
    let values = response.values();
    let code = values[0].read();
    assert_eq!(code.label().value(), "Answer �");
    assert_eq!(code.label().variants()[0].value(), "Code �");
}
