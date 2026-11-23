use polars::prelude::*;
use triple_s::{
    AnalysisOptions,
    Survey,
    SurveyData,
    options::{LabelOptions, ResponseDisplay},
    read_triple_s,
};

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
    assert_eq!(survey.num_response().unwrap(), 3);
    assert_eq!(survey.question_names().len(), 12);
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
    assert_eq!(question.config().response_display, ResponseDisplay::Value);
    assert_eq!(question.to_series().unwrap().dtype(), &DataType::UInt32);
    let question = survey.get_question_data("Q2").unwrap();
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
fn test_crosstab_column_groups_and_row_total() {
    let survey = get_survey_struct();
    *survey.data_mut() = SurveyData::from_polars(
        df![
            "Q2" => [Some(0u32), Some(0), Some(1), Some(2), None],
            "Q4" => [1u32, 2, 1, 2, 1],
            "Q5" => [
                Series::new("".into(), [1u32, 2]),
                Series::new("".into(), [2u32]),
                Series::new("".into(), [1u32]),
                Series::new("".into(), [2u32]),
                Series::new("".into(), [1u32]),
            ],
        ]
        .unwrap(),
    )
    .unwrap();

    let expected_body = df![
        "label" => [
            "No, this is the first visit",
            "I have visited before within the year",
            "I visited before that",
        ],
        "total" => [2u32, 1, 1],
        "Q4 - Very Good" => [1u32, 1, 0],
        "Q4 - Good" => [1u32, 0, 1],
        "Q5 - Sherwood Forest" => [1u32, 1, 0],
        "Q5 - Nottingham Castle" => [2u32, 0, 1],
    ]
    .unwrap();
    let group_labels = [
        "Q4 - Very Good",
        "Q4 - Good",
        "Q5 - Sherwood Forest",
        "Q5 - Nottingham Castle",
    ];

    for num_groups in 1..=2 {
        for show_row_total in [false, true] {
            let result = survey
                .crosstab(
                    AnalysisOptions {
                        question_name: "Q2",
                        columns: &["Q4", "Q5"][..num_groups],
                        show_row_total,
                        show_column_total: false,
                        normalise: false,
                    },
                    LabelOptions::default(),
                )
                .unwrap();

            let mut names = vec!["label"];
            if show_row_total {
                names.push("total");
            }
            names.extend_from_slice(&group_labels[..num_groups * 2]);
            let expected_case = expected_body.select(names).unwrap();
            let result = result.sort(["label"], Default::default()).unwrap();
            let expected_case = expected_case.sort(["label"], Default::default()).unwrap();

            assert!(
                result.equals_missing(&expected_case),
                "groups={num_groups}, total={show_row_total}:\n{result}\nexpected:\n{expected_case}"
            );
        }
    }
}

#[test]
fn test_crosstab_same_question_as_row_and_column() {
    let survey = get_survey_struct();
    *survey.data_mut() = SurveyData::from_polars(df!["Q2" => [0u32, 1, 1]].unwrap()).unwrap();

    let result = survey
        .crosstab(
            AnalysisOptions {
                question_name: "Q2",
                columns: &["Q2"],
                show_row_total: true,
                show_column_total: true,
                normalise: false,
            },
            LabelOptions::default(),
        )
        .unwrap();
    let expected = df![
        "label" => [
            "Total",
            "No, this is the first visit",
            "I have visited before within the year",
        ],
        "total" => [3u32, 1, 2],
        "Q2 - No, this is the first visit" => [1u32, 1, 0],
        "Q2 - I have visited before within the year" => [2u32, 0, 2],
    ]
    .unwrap();

    assert!(
        result.equals_missing(&expected),
        "{result}\nexpected:\n{expected}"
    );
}

#[test]
fn test_crosstab_label_options() {
    let survey = get_survey_struct();
    *survey.data_mut() =
        SurveyData::from_polars(df!["Q2" => [0u32, 1, 1], "Q4" => [1u32, 1, 2]].unwrap())
            .unwrap();

    for (question, code, label, total) in [
        ("question", "code", "label", "total"),
        ("item", "value", "answer", "n"),
    ] {
        for fill_question_name in [false, true] {
            let expected = df![
                question => if fill_question_name {
                    [Some("Q2"), Some("Q2"), Some("Q2")]
                } else {
                    [Some("Q2"), None, None]
                },
                code => [None, Some(0u32), Some(1)],
                label => [
                    "Total",
                    "No, this is the first visit",
                    "I have visited before within the year",
                ],
                total => [3u32, 1, 2],
                "Q4 - Very Good" => [2u32, 1, 1],
                "Q4 - Good" => [1u32, 0, 1],
            ]
            .unwrap();
            let result = survey
                .crosstab(
                    AnalysisOptions {
                        question_name: "Q2",
                        columns: &["Q4"],
                        show_row_total: true,
                        show_column_total: true,
                        normalise: false,
                    },
                    LabelOptions::new()
                        .with_include_question_name(true)
                        .with_fill_question_name_column(fill_question_name)
                        .with_include_response_codes(true)
                        .with_col_question_name(Some(question))
                        .with_col_code(Some(code))
                        .with_col_label(Some(label))
                        .with_col_count(Some(total)),
                )
                .unwrap();
            assert!(
                result.equals_missing(&expected),
                "{result}\nexpected:\n{expected}"
            );
        }
    }
}

#[test]
fn test_crosstab_normalise() -> PolarsResult<()> {
    let survey = get_survey_struct();
    *survey.data_mut() = SurveyData::from_polars(df![
        "Q2" => [0u32, 1, 1],
        "Q4" => [1u32, 1, 2],
        "Q5" => [
            Series::new("".into(), [1u32, 2]),
            Series::new("".into(), [1u32]),
            Series::new("".into(), [2u32]),
        ],
    ]?)?;

    for num_groups in 1..=2 {
        for show_row_total in [false, true] {
            for show_column_total in [false, true] {
                let mut expected = df![
                    "response" => [None, Some(0u32), Some(1)],
                    "answer" => [
                        "Total", "No, this is the first visit", "I have visited before within the year",
                    ],
                    "base" => [3.0, 1.0 / 3.0, 2.0 / 3.0],
                    "Q4 - Very Good" => [2.0, 0.5, 0.5],
                    "Q4 - Good" => [1.0, 0.0, 1.0],
                    "Q5 - Sherwood Forest" => [2.0, 0.5, 0.5],
                    "Q5 - Nottingham Castle" => [2.0, 0.5, 0.5],
                ]?;
                if !show_row_total {
                    expected.drop_in_place("base")?;
                }
                if num_groups == 1 {
                    expected.drop_in_place("Q5 - Sherwood Forest")?;
                    expected.drop_in_place("Q5 - Nottingham Castle")?;
                }
                if !show_column_total {
                    expected = expected.slice(1, 2);
                }
                let result = survey.crosstab(
                    AnalysisOptions {
                        question_name: "Q2",
                        columns: &["Q4", "Q5"][..num_groups],
                        show_row_total,
                        show_column_total,
                        normalise: true,
                    },
                    LabelOptions::new()
                        .with_include_response_codes(true)
                        .with_col_code(Some("response"))
                        .with_col_label(Some("answer"))
                        .with_col_count(Some("base")),
                )?;
                assert!(
                    result.equals_missing(&expected),
                    "{result}\nexpected:\n{expected}"
                );
            }
        }
    }
    Ok(())
}

#[test]
fn test_crosstab_column_totals_count_respondents() {
    let survey = get_survey_struct();
    let multiple = Series::new(
        "Q5".into(),
        [
            Some(Series::new("".into(), [1u32, 2])),
            Some(Series::new("".into(), [1u32])),
            Some(Series::new("".into(), Vec::<u32>::new())),
            Some(Series::new("".into(), [2u32])),
            None,
            Some(Series::new("".into(), [1u32])),
        ],
    );
    *survey.data_mut() = SurveyData::from_polars(
        df![
            "Q5" => multiple,
            "Q4" => [Some(1u32), Some(1), Some(1), Some(2), Some(2), None],
            "Q2" => [0u32, 0, 1, 1, 2, 2],
        ]
        .unwrap(),
    )
    .unwrap();
    let expected_totals = df![
        "label" => ["Total"],
        "total" => [5u32],
        "Q4 - Very Good" => [3u32],
        "Q4 - Good" => [2u32],
        "null" => [None::<u32>],
        "Q2 - No, this is the first visit" => [2u32],
        "Q2 - I have visited before within the year" => [2u32],
        "Q2 - I visited before that" => [2u32],
    ]
    .unwrap();

    let columns = ["Q4", "Q2"];
    let body = survey
        .crosstab(
            AnalysisOptions {
                question_name: "Q5",
                columns: &columns,
                show_row_total: true,
                show_column_total: false,
                normalise: false,
            },
            LabelOptions::default(),
        )
        .unwrap();
    let result = survey
        .crosstab(
            AnalysisOptions {
                question_name: "Q5",
                columns: &columns,
                show_row_total: true,
                show_column_total: true,
                normalise: false,
            },
            LabelOptions::default(),
        )
        .unwrap();

    assert!(
        result.head(Some(1)).equals_missing(&expected_totals),
        "column totals should count respondent rows, not exploded answers:\n{result}"
    );
    let body_columns = [
        "label",
        "total",
        "Q4 - Very Good",
        "Q4 - Good",
        "null",
        "Q2 - No, this is the first visit",
        "Q2 - I have visited before within the year",
        "Q2 - I visited before that",
    ];
    let result_body = result
        .slice(1, body.height())
        .select(body_columns)
        .unwrap()
        .sort(["label"], Default::default())
        .unwrap();
    let expected_body = body
        .select(body_columns)
        .unwrap()
        .sort(["label"], Default::default())
        .unwrap();
    assert!(
        result_body.equals_missing(&expected_body),
        "column totals should not change the crosstab body:\n{result_body}\nexpected:\n{expected_body}"
    );
}

#[test]
fn test_read_stack_overflow_survey() {
    println!("Reading survey data");
    let survey = read_stack_overflow();
    println!("Data loaded. Question names:");
    println!("{:#?}", survey.question_names());

    println!("running crosstab");
    let row = "AIFreq_1";
    let topbreak = "AIFreq_1";

    let res = survey
        .crosstab(
            AnalysisOptions {
                question_name: row,
                columns: &[topbreak], //"JobEaseAI_1"],
                show_row_total: true,
                show_column_total: true,
                normalise: false,
            },
            LabelOptions {
                include_response_codes: true,
                ..Default::default()
            },
        )
        .unwrap();
    println!("Crosstab:");
    unsafe {
        std::env::set_var("POLARS_FMT_MAX_ROWS", "100");
    }
    println!("{:?}", res);

    println!("Base size:");
    let options = LabelOptions::new().with_include_response_codes(true);
    let base = survey
        .get_question_data(topbreak)
        .unwrap()
        .value_counts(options)
        .unwrap();
    println!("{:?}", base);
}

#[test]
fn test_crosstab_question_against_itself() {
    let survey = read_stack_overflow();
    let result = survey
        .crosstab(
            AnalysisOptions {
                question_name: "AIFreq_1",
                columns: &["AIFreq_1"],
                show_row_total: true,
                show_column_total: true,
                normalise: false,
            },
            LabelOptions::default(),
        )
        .unwrap();
    assert!(result.height() > 0);
}

#[test]
fn test_value_counts() {
    let survey = get_survey_struct();
    let question = survey.get_question_data("Q3").unwrap();

    let options = LabelOptions::new()
        .with_include_response_codes(true)
        .with_col_code(Some("HAHA"))
        .with_include_response_codes(true)
        .with_include_question_name(true)
        .with_fill_question_name_column(true);
    let stats = question.value_counts(options).unwrap();
    println!("{:?}", stats)
}

#[test]
fn test_crosstab_case_insensitive() {
    let survey = read_stack_overflow();
    let result = survey
        .quick_summary(
            Some(&["aifreq_1"]),
            None,
            triple_s::options::SkipQType {
                text: true,
                numeric: true,
                datetime: false,
                single: false,
                multiple: false,
            },
            false,
            LabelOptions::default(),
        )
        .unwrap();

    for (qname, qdata) in result.into_iter() {
        println!("Question: {}, shape: {:?}", qname, qdata.shape())
    }
}
