use polars::prelude::AnyValue;
use triple_s::read_datamap;

const PATH_DATAMAP_V2_0: &str = "tests/sss-examples/sss-2.0/example1.sss";
const PATH_DATAFILE_V2_0: &str = "tests/sss-examples/sss-2.0/example1.asc";

#[test]
fn test_create_parquet() {
    let survey = read_datamap(PATH_DATAMAP_V2_0).unwrap();
    let mut record = survey.record().clone();
    record.load_data(PATH_DATAFILE_V2_0).unwrap();
    let df = record.to_polars().unwrap();
    println!("{:?}", df)
}

#[test]
fn csv_and_fixed_examples_load_equivalent_responses() {
    let mut csv = read_datamap("tests/sss-examples/sss-2.0/example2.sss").unwrap();
    csv.record_mut().load_data("tests/sss-examples/sss-2.0/example2.csv").unwrap();
    let mut fixed = read_datamap("tests/sss-examples/sss-2.0/example1.sss").unwrap();
    fixed.record_mut().load_data("tests/sss-examples/sss-2.0/example1.asc").unwrap();

    let csv = csv.record().to_polars().unwrap();
    let fixed = fixed.record().to_polars().unwrap();
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
