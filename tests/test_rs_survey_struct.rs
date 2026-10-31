use triple_s::{Survey, read_triple_s};

const PATH_DATAMAP_V2_0: &str = "tests/sss-examples/sss-2.0/example1.sss";
const PATH_DATAFILE_V2_0: &str = "tests/sss-examples/sss-2.0/example1.asc";

fn get_survey_struct() -> Survey {
    read_triple_s(PATH_DATAMAP_V2_0, Some(PATH_DATAFILE_V2_0)).unwrap()
}

#[test]
fn test_survey_read_success() {
    let survey = get_survey_struct();

    assert_eq!(survey.has_data(), true);
    assert_eq!(
        survey.title().as_deref(),
        Some("Historic House Exit Survey<br/>First Wave")
    );
    assert_eq!(survey.name().as_deref(), Some("SP5201-1"));
    assert_eq!(survey.num_response().unwrap(), 3)
}
