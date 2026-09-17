use triple_s::{Survey, read_sss_file};

const PATH_SSS_XML_GENERATION_ONE: &str = "tests/sss-examples/sss_xml_1.0.xml";
const PATH_SSS_XML_GENERATION_TWO: &str = "tests/sss-examples/SSS XML 2.0 examples/example1.sss";

fn example_datamap_xml_generation_one() -> Survey {
    read_sss_file(PATH_SSS_XML_GENERATION_ONE).unwrap()
}

fn example_datamap_xml_generation_two() -> Survey {
    read_sss_file(PATH_SSS_XML_GENERATION_TWO).unwrap()
}

// #[test]
// fn test_read_xml_datamap_gen_1() {
//     let _survey = example_datamap_xml_generation_one();
// }

#[test]
fn test_read_xml_datamap_gen_2() {
    let survey = example_datamap_xml_generation_two();
}

// #[test]
// fn test_read_xml_datamap_gen_3() {
//     let survey = example_datamap_xml_generation_three();
// }