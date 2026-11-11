use polars::prelude::AnyValue;
use triple_s::{
    Datamap,
    QuestionOptions,
    QuestionType,
    Survey,
    read_data,
    read_datamap,
    read_triple_s,
};

const PATH_DATAMAP_V2_0: &str = "tests/sss-examples/sss-2.0/example1.sss";
const PATH_DATAFILE_V2_0: &str = "tests/sss-examples/sss-2.0/example1.asc";

fn get_example_1() -> Survey {
    read_triple_s(PATH_DATAMAP_V2_0, Some(PATH_DATAFILE_V2_0)).unwrap()
}

#[test]
fn test_read_survey() {
    let survey = get_example_1();
    assert!(survey.has_data());
    assert_eq!(survey.num_response().unwrap(), 3);
}

#[test]
fn test_create_parquet() {
    let datamap = read_datamap(PATH_DATAMAP_V2_0).unwrap();
    let data = read_data(&datamap, PATH_DATAFILE_V2_0).unwrap();
    let df = data.to_polars().unwrap();
    println!("{:?}", df)
}

#[test]
fn test_csv_and_fixed_examples_load_equivalent_responses() {
    let csv = read_datamap("tests/sss-examples/sss-2.0/example2.sss").unwrap();
    let csv_data = read_data(&csv, "tests/sss-examples/sss-2.0/example2.csv").unwrap();

    let fixed = read_datamap("tests/sss-examples/sss-2.0/example1.sss").unwrap();
    let fixed_data = read_data(&fixed, "tests/sss-examples/sss-2.0/example1.asc").unwrap();

    let csv = csv_data.to_polars().unwrap();
    let fixed = fixed_data.to_polars().unwrap();
    assert_eq!(csv.shape(), (3, 12));
    assert_eq!(fixed.shape(), csv.shape());
    for (csv_column, fixed_column) in csv.columns().iter().zip(fixed.columns()) {
        assert_eq!(csv_column.name(), fixed_column.name());
        for index in 0..csv.height() {
            match (
                csv_column.get(index).unwrap(),
                fixed_column.get(index).unwrap(),
            ) {
                (AnyValue::String(csv), AnyValue::String(fixed)) => {
                    assert_eq!(csv, fixed.trim_end())
                }
                (csv, fixed) => assert_eq!(csv, fixed),
            }
        }
    }
}

#[test]
fn test_invalid_utf8_is_replaced_in_fixed_and_csv_data() {
    use std::io::Write;

    for (example, extension) in [(1, "asc"), (2, "csv")] {
        let base = format!("tests/sss-examples/sss-2.0/example{example}");
        let mut datamap = read_datamap(format!("{base}.sss")).unwrap();
        let mut bytes = std::fs::read(format!("{base}.{extension}")).unwrap();
        let answer = b"Nottingham Goose Fair";
        let offset = bytes.windows(answer.len()).position(|part| part == answer).unwrap();
        bytes[offset] = 0x96;
        let mut file =
            tempfile::Builder::new().suffix(&format!(".{extension}")).tempfile().unwrap();
        file.write_all(&bytes).unwrap();

        let record = datamap.record_mut();
        record.validate_data(file.path()).unwrap();
        let data = read_data(&datamap, file.path()).unwrap();
        let frame = data.to_polars().unwrap();
        assert_eq!(frame.height(), 3);
        assert_eq!(
            frame.column("Q3.a").unwrap().str().unwrap().get(0).unwrap().trim_end(),
            "�ottingham Goose Fair"
        );
        assert_eq!(
            frame.column("Q4").unwrap().get(0).unwrap(),
            AnyValue::UInt32(2)
        );
    }
}

#[test]
fn test_malformed_utf8_at_field_boundaries_and_eof_is_replaced() {
    use std::{io::Write, sync::Arc};

    for (bytes, expected) in [
        (&b"\x96a\xff"[..], "�a�"),
        (&b"\xff\x96\x80"[..], "���"),
        (&b"a\xe2\x82"[..], "a�"),
    ] {
        for extension in ["asc", "csv"] {
            let question = QuestionOptions::new("TEXT")
                .with_question_type(QuestionType::Character)
                .with_location(1, Some(expected.chars().count() as u32))
                .finish()
                .unwrap();
            let mut datamap = Datamap::empty();
            let record = datamap.record_mut();
            record.content_mut().push(Arc::new(parking_lot::RwLock::new(question)));
            let mut file =
                tempfile::Builder::new().suffix(&format!(".{extension}")).tempfile().unwrap();
            file.write_all(bytes).unwrap();

            record.validate_data(file.path()).unwrap();
            let data = read_data(&datamap, file.path()).unwrap();
            let frame = data.to_polars().unwrap();
            assert_eq!(frame.shape(), (1, 1));
            assert_eq!(
                frame.column("TEXT").unwrap().get(0).unwrap(),
                AnyValue::String(expected),
                "{extension}: {bytes:?}"
            );

            let record = datamap.record_mut();
            record.content()[0].write().set_question_type(QuestionType::Quantity).unwrap();
            assert!(record.validate_data(file.path()).is_err());
            assert!(read_data(&datamap, file.path()).is_err());
        }
    }
}
