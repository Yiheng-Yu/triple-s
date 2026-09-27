use triple_s::{Question, Response, Survey, EventLog};

const PATH_SSS_CLASSIC: &str = "tests/sss-examples/sss_classic.sss";

fn example_datamap_classic() -> Survey {
    triple_s::read_datamap(PATH_SSS_CLASSIC).unwrap()
}

#[test]
fn test_read_classic() {
    let survey = example_datamap_classic();
    assert_eq!(survey.record.content().len(), 24);
}

#[test]
fn test_question_from_to_sss_classic() {
    let survey = example_datamap_classic();

    for question in survey.record.content().iter() {
        let question = question.read();
        let text = question.to_sss_classic().unwrap();
        let parsed = Question::from_string(&text).unwrap();
        assert_eq!(*parsed.name(), *question.name());
        assert_eq!(parsed.label(), question.label());
        assert_eq!(parsed.index(), question.index());
        assert_eq!(*parsed.filter(), *question.filter());
        assert_eq!(parsed.location(), question.location());
        assert_eq!(*parsed.format(), *question.format());
        assert_eq!(*parsed.question_type(), *question.question_type());
        assert_eq!(parsed.spread(), question.spread());

        let parsed_response: &Response = parsed.response();
        let expected_response: &Response = question.response();
        assert_eq!(*parsed_response.range(), *expected_response.range());

        let parsed_values = parsed_response.values();
        let expected_values = expected_response.values();
        assert_eq!(parsed_values.len(), expected_values.len());
        for (parsed_code, expected_code) in parsed_values.iter().zip(expected_values.iter()) {
            let parsed_code = parsed_code.read();
            let expected_code = expected_code.read();
            assert_eq!(*parsed_code.value(), *expected_code.value());
            assert_eq!(*parsed_code.label(), *expected_code.label());
            assert_eq!(*parsed_code.filter(), *expected_code.filter());
            assert_eq!(*parsed_code.score(), *expected_code.score());
        }
    }
}