use triple_s::{Survey, Mode, Version, read_sss_file};

#[allow(dead_code)]
const PATH_SSS_XML_GENERATION_ONE: &str = "tests/sss-examples/sss_xml_1.0.xml";
const PATH_SSS_XML_GENERATION_TWO: &str = "tests/sss-examples/sss-2.0/example2.sss";

#[allow(dead_code)]
fn example_datamap_xml_generation_one() -> Survey {
    read_sss_file(PATH_SSS_XML_GENERATION_ONE).unwrap()
}

#[allow(dead_code)]
fn example_datamap_xml_generation_two() -> Survey {
    read_sss_file(PATH_SSS_XML_GENERATION_TWO).unwrap()
}

// #[allow(dead_code)]
// #[test]
// fn test_read_xml_datamap_gen_1() {
//     let _survey = example_datamap_xml_generation_one();
// }

#[test]
fn test_read_xml_datamap_ver_2_example_2() {
    let survey = example_datamap_xml_generation_two();
    assert_eq!(survey.meta.version, Version::V2_0);
    assert_eq!(survey.meta.language_options, vec!["en-GB", "en-US", "fr"]);
    assert_eq!(survey.title.as_ref().unwrap(), "Historic House Exit Survey<br/>First Wave");
    assert_eq!(survey.meta.mode_options, vec![Mode::Interview, Mode::Analysis]);
}

// #[test]
// fn test_read_xml_datamap_gen_3() {
//     let survey = example_datamap_xml_generation_three();
// }