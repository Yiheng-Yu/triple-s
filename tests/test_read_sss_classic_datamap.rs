use triple_s::{Survey, error::BadLine, read_sss_file};

const PATH_SSS_CLASSIC: &str = "tests/sss-examples/sss_classic.sss";

fn example_datamap_classic() -> Survey {
    read_sss_file(PATH_SSS_CLASSIC).unwrap()
}

fn put_into_record(content: &str) -> impl Iterator<Item = Result<String, BadLine>> {
    let content = format!(
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
    );
    content
        .lines()
        .map(|line| Ok::<String, BadLine>(line.to_string()))
        .collect::<Vec<Result<String, BadLine>>>()
        .into_iter()
}

#[test]
fn test_classic_read_n_write() {
    let survey = example_datamap_classic();
    for question in survey.record.content.iter() {
        let correct_name = question.name.clone();
        let sss_string = question.to_sss_classic().unwrap();
        let sss_lines = put_into_record(&sss_string);
        let read = triple_s::read_sss_file_classic(sss_lines).unwrap();

        let question_read = read.get_question(&correct_name).unwrap();
        assert_eq!(question_read.name, correct_name);
        assert_eq!(question_read.response, question.response);
        assert_eq!(question_read.location, question.location);
        assert_eq!(question_read.text, question.text);
        assert_eq!(question_read.index, question.index);
    }
}
