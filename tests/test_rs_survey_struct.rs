use polars::prelude::*;
use triple_s::{Survey, SurveyData, config::ResponseDisplay, read_triple_s};

const PATH_DATAMAP_V2_0: &str = "tests/sss-examples/sss-2.0/example1.sss";
const PATH_DATAFILE_V2_0: &str = "tests/sss-examples/sss-2.0/example1.asc";

const PATH_STACK_OVERFLOW_DATAMAP: &str = "tests/sss-examples/sss-2.0/stackoverflow_2026.sss";
const PATH_STACK_OVERFLOW_DATA: &str = "tests/sss-examples/sss-2.0/stackoverflow_2026.csv";

fn get_survey_struct() -> Survey {
    read_triple_s(PATH_DATAMAP_V2_0, Some(PATH_DATAFILE_V2_0)).unwrap()
}

fn read_stack_overflow() -> Survey {
    read_triple_s(PATH_STACK_OVERFLOW_DATAMAP, Some(PATH_STACK_OVERFLOW_DATA)).unwrap()
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

#[test]
fn test_question_view_shares_replacement_and_owns_cache() {
    let survey = get_survey_struct();
    let view = survey.get_question_data("Q2").unwrap();
    let replacement = survey.data().to_polars().unwrap().reverse();
    *survey.data_mut() = SurveyData::from_polars(replacement.clone()).unwrap();
    assert_eq!(
        view.to_column().unwrap(),
        replacement.column("Q2").unwrap().clone()
    );
    let cache_path = survey.data().cache().unwrap().to_owned();
    drop(survey);
    assert!(std::path::Path::new(&cache_path).exists());
    assert_eq!(
        view.to_column().unwrap(),
        replacement.column("Q2").unwrap().clone()
    );
    drop(view);
    assert!(!std::path::Path::new(&cache_path).exists());
}

#[test]
fn test_value_display_success() {
    let mut survey = get_survey_struct();
    let question = survey.get_question_data("Q2").unwrap();
    let series_value = question.to_series().unwrap();
    assert_eq!(series_value.dtype(), &DataType::UInt32);

    survey.config_mut().response_display = ResponseDisplay::Label;
    assert_eq!(question.config().response_display, ResponseDisplay::Label);
    let series_label = question.to_series().unwrap();
    assert_eq!(series_label.dtype(), &DataType::String)
}

#[test]
fn test_value_display_label_not_found() {
    let mut survey = get_survey_struct();
    let mut question = survey.get_question_data("Q2").unwrap();

    question
        .datamap_mut()
        .response_mut()
        .with_value_write(0, |item| item.set_value(AnyValue::UInt32(300)));

    survey.config_mut().response_display = ResponseDisplay::Label;
    match question.to_series() {
        Ok(_) => panic!("Error not raised"),
        // correct error
        Err(PolarsError::ComputeError(message)) => {
            assert!(message.contains("contains response values with no matching label"));
        }
        Err(error) => {
            panic!("Wrong type of error raised:\n {:?}", error)
        }
    }
}

#[test]
fn test_read_stack_overflow_survey() {
    let survey = read_stack_overflow();

    let res = survey.crosstab("DevIDEAdmired", "Gender").unwrap();
    println!("Crosstab:");
    println!("{:?}", res);

    println!("Base size:");
    let base = survey
        .get_question_data("Gender")
        .unwrap()
        .value_counts(false, Some("count"))
        .unwrap();
    println!("{:?}", base);
}

#[test]
fn test_value_counts() {
    let survey = get_survey_struct();
    let question = survey.get_question_data("Q3").unwrap();
    let stats = question.value_counts(false, None).unwrap();
    println!("{:?}", stats)
}
