use triple_s::{Question, Response, Survey};

const PATH_SSS_CLASSIC: &str = "tests/sss-examples/sss_classic.sss";

fn example_datamap_classic() -> Survey {
    triple_s::read_sss_file(PATH_SSS_CLASSIC).unwrap()
}

#[test]
fn test_read_classic() {
    let survey = example_datamap_classic();
    assert_eq!(survey.record.content.len(), 24);
}

#[test]
fn test_question_from_to_sss_classic() {
    let survey = example_datamap_classic();

    for question in survey.record.content.into_iter() {
        println!("Asserting sss xml question read/write for question: '{}'", question.read().name);
        
        let text = question.read().to_sss_classic().unwrap();
        let parsed = Question::from_string(&text).unwrap();
        assert_eq!(parsed.name, question.read().name);
        assert_eq!(parsed.label, question.read().label);
        assert_eq!(parsed.index, question.read().index);
        assert_eq!(parsed.filter, question.read().filter);
        assert_eq!(parsed.location, question.read().location);
        assert_eq!(parsed.format, question.read().format);
        match (&parsed.response, &question.read().response) {
            (Response::NumericCategory(parsed_), Response::NumericCategory(correct)) => {
                assert_eq!(parsed_.read().is_single, correct.read().is_single);
                assert_eq!(parsed_.read().range, correct.read().range);
                assert_eq!(parsed_.read().spread, correct.read().spread);
                assert_eq!(parsed_.read().values, correct.read().values);
            }
            _ => assert_eq!(parsed.response, question.read().response)
        }
    }
}

// #[test]
// fn test_survey_from_to_string() {
//     let survey = example_datamap_classic();
//     let test = survey.to_sss_classic().unwrap();
//     let parsed = Survey::from_string(&text).unwrap();
//     assert_eq!(parsed, survey)
// }

fn put_into_record(content: &str) -> String {
    format!(
        r#"
        SSS
        VERSION 1.0
        STANDARD NAMES
        SURVEY
        TITLE "Customer Experience Survey - HTML Test"
        RECORD A
        {}
        END RECORD
        END SURVEY
        END SSS
        "#,
        content
    )
}

#[test]
fn test_classic_read_n_write() {
    let survey = example_datamap_classic();

    for question in survey.record.content.iter() {
        let correct_name = question.read().name.clone();
        let sss_string = question.read().to_sss_classic().unwrap();
        let sss_lines = put_into_record(&sss_string);
        let read = triple_s::read_from_string(sss_lines.clone()).unwrap();

        let question_read = read.get_question(&correct_name).unwrap();
        assert_eq!(question_read.read().name, correct_name);
        assert_eq!(question_read.read().location, question.read().location);
        assert_eq!(question_read.read().label, question.read().label);
        assert_eq!(question_read.read().index, question.read().index);

        // have some think about how to compare responses
    }
}
