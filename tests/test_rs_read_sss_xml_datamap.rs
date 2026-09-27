use triple_s::{Survey, read_datamap};


const PATH_SSS_XML_GENERATION_ONE: &str = "tests/sss-examples/sss_xml_1.0.xml";
const PATH_SSS_XML_GENERATION_TWO_EXAMPLE_1: &str = "tests/sss-examples/sss-2.0/example1.sss";
const PATH_SSS_XML_GENERATION_TWO_EXAMPLE_2: &str = "tests/sss-examples/sss-2.0/example2.sss";


fn example_datamap_xml_generation_one() -> Survey {
    read_datamap(PATH_SSS_XML_GENERATION_ONE).unwrap()
}


fn example_datamap_xml_generation_two_example_one() -> Survey {
    read_datamap(PATH_SSS_XML_GENERATION_TWO_EXAMPLE_1).unwrap()
}


fn example_datamap_xml_generation_two_example_two() -> Survey {
    read_datamap(PATH_SSS_XML_GENERATION_TWO_EXAMPLE_2).unwrap()
}


#[test]
fn test_read_xml_datamap_gen_1() {
    let survey = example_datamap_xml_generation_one();
    assert_eq!(survey.record.content().len(), 8);
}

#[test]
fn test_read_xml_datamap_ver_2_example_1() {
    let survey = example_datamap_xml_generation_two_example_one();
    assert_eq!(survey.record.content().len(), 12);
}

#[test]
fn test_read_xml_datamap_ver_2_example_2() {
    let survey = example_datamap_xml_generation_two_example_two();
    assert_eq!(survey.record.content().len(), 12);
}