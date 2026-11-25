#![cfg(not(feature = "datamap-only"))]

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
    let _df = data.to_polars().unwrap();
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
        let offset = bytes
            .windows(answer.len())
            .position(|part| part == answer)
            .unwrap();
        bytes[offset] = 0x96;
        let mut file = tempfile::Builder::new()
            .suffix(&format!(".{extension}"))
            .tempfile()
            .unwrap();
        file.write_all(&bytes).unwrap();

        let record = datamap.record_mut();
        record.validate_data(file.path()).unwrap();
        let data = read_data(&datamap, file.path()).unwrap();
        let frame = data.to_polars().unwrap();
        assert_eq!(frame.height(), 3);
        assert_eq!(
            frame
                .column("Q3.a")
                .unwrap()
                .str()
                .unwrap()
                .get(0)
                .unwrap()
                .trim_end(),
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
            record
                .content_mut()
                .push(Arc::new(parking_lot::RwLock::new(question)));
            let mut file = tempfile::Builder::new()
                .suffix(&format!(".{extension}"))
                .tempfile()
                .unwrap();
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
            record.content()[0]
                .write()
                .set_question_type(QuestionType::Quantity)
                .unwrap();
            assert!(record.validate_data(file.path()).is_err());
            assert!(read_data(&datamap, file.path()).is_err());
        }
    }
}

#[test]
fn test_read_chinese() {
    const SENTENCE: &str = "🦔 this 中文含有 mixed 🦔 with 🦔 chinese 中文中文";

    for (count, (idx, cha)) in SENTENCE.char_indices().enumerate() {
        println!("count: {} | idx: {} | char: {}", count, idx, cha);
    }

    // let range = 0..5;
    // let start = SENTENCE
    //     .char_indices()
    //     .nth(range.start)
    //     .map(|(i, _)| i)
    //     .unwrap_or(SENTENCE.len());
    // let end = SENTENCE[start..]
    //     .char_indices()
    //     .nth(range.len())
    //     .map(|(i, _)| start + i)
    //     .unwrap_or(SENTENCE.len());
    // let slice: &str = &SENTENCE[start..end];
    // println!("Range: {:?} | Text: {:?}", range, slice);

    // let range = 1..6;
    // let start = SENTENCE
    //     .char_indices()
    //     .nth(range.start)
    //     .map(|(i, _)| i)
    //     .unwrap_or(SENTENCE.len());
    // let end = SENTENCE[start..]
    //     .char_indices()
    //     .nth(range.len())
    //     .map(|(i, _)| start + i)
    //     .unwrap_or(SENTENCE.len());
    // let slice: &str = &SENTENCE[start..end];
    // println!("Range: {:?} | Text: {:?}", range, slice);
}

#[test]
fn test_parallel_scanners_preserve_respondents() {
    use std::io::Write;
    for extension in ["asc", "csv"] {
        let mut datamap = Datamap::empty();
        datamap.record_mut().add_question(
            QuestionOptions::new("TEXT")
                .with_question_type(QuestionType::Character)
                .with_location(1, Some(8))
                .finish()
                .unwrap(),
        );
        let mut file = tempfile::Builder::new()
            .suffix(&format!(".{extension}"))
            .tempfile()
            .unwrap();
        for i in 0..2200 {
            writeln!(file, "中{i:07}").unwrap();
        }
        for threads in [1, 4] {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap();
            let data = pool.install(|| read_data(&datamap, file.path())).unwrap();
            let frame = data.to_polars().unwrap();
            assert_eq!(frame.height(), 2200);
            for i in 0..2200 {
                assert_eq!(
                    frame.column("TEXT").unwrap().str().unwrap().get(i),
                    Some(format!("中{i:07}").as_str())
                );
            }
        }
    }
}

#[test]
fn test_parallel_csv_quoted_newlines() {
    use std::io::Write;
    let mut datamap = Datamap::empty();
    datamap.record_mut().add_question(
        QuestionOptions::new("TEXT")
            .with_question_type(QuestionType::Character)
            .with_location(1, Some(20))
            .finish()
            .unwrap(),
    );
    let mut file = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
    file.write_all(b"\"hello\nworld\"\r\n\"a,b\"\r\n").unwrap();
    let data = read_data(&datamap, file.path()).unwrap();
    let frame = data.to_polars().unwrap();
    let values = frame.column("TEXT").unwrap().str().unwrap();
    assert_eq!(values.get(0), Some("hello\nworld"));
    assert_eq!(values.get(1), Some("a,b"));
}

#[test]
fn test_load_batch_boundaries_with_nulls_and_unicode() {
    use std::io::Write;

    for extension in ["asc", "csv"] {
        let mut datamap = Datamap::empty();
        for (name, kind, start, end) in [
            ("NUMBER", QuestionType::Quantity, 1, 8),
            (
                "TEXT",
                QuestionType::Character,
                if extension == "asc" { 9 } else { 2 },
                if extension == "asc" { 16 } else { 2 },
            ),
        ] {
            datamap.record_mut().add_question(
                QuestionOptions::new(name)
                    .with_question_type(kind)
                    .with_location(start, Some(end))
                    .finish()
                    .unwrap(),
            );
        }
        for count in [0, 4096, 4097, 8192] {
            let mut file = tempfile::Builder::new()
                .suffix(&format!(".{extension}"))
                .tempfile()
                .unwrap();
            for i in 0..count {
                let number = if i % 3 == 0 {
                    "        ".into()
                } else {
                    format!("{i:8}")
                };
                let separator = if extension == "asc" { "" } else { "," };
                writeln!(file, "{number}{separator}中{i:07}").unwrap();
            }
            let data = read_data(&datamap, file.path()).unwrap();
            let frame = data.to_polars().unwrap();
            assert_eq!(frame.shape(), (count, 2), "{extension}: {count} rows");
            let numbers = frame.column("NUMBER").unwrap().f64().unwrap();
            let text = frame.column("TEXT").unwrap().str().unwrap();
            for i in 0..count {
                assert_eq!(numbers.get(i), (i % 3 != 0).then_some(i as f64));
                assert_eq!(text.get(i), Some(format!("中{i:07}").as_str()));
            }
        }
    }
}
