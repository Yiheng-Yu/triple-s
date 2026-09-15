use triple_s::{Survey, read_sss};

const PATH_SSS_CLASSIC: &str = "tests/sss_classic.sss";

#[test]
fn read_sss_classic() {
    let survey = read_sss(PATH_SSS_CLASSIC).unwrap();
    assert_eq!(
        survey.title.unwrap(),
        "Customer Experience Survey - HTML Test"
    );
}

#[test]
fn survey_writer_is_available_on_the_public_model() {
    let survey = Survey::empty();
    assert!(survey.to_sss_classic().starts_with("SSS\n"));
}
