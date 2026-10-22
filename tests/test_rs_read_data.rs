use triple_s::{Entry, read_datamap};

#[test]
fn csv_and_fixed_examples_load_equivalent_responses() {
    let mut csv = read_datamap("tests/sss-examples/sss-2.0/example2.sss").unwrap();
    csv.record_mut().load_data("tests/sss-examples/sss-2.0/example2.csv").unwrap();
    let mut fixed = read_datamap("tests/sss-examples/sss-2.0/example1.sss").unwrap();
    fixed.record_mut().load_data("tests/sss-examples/sss-2.0/example1.asc").unwrap();

    for (csv_question, fixed_question) in
        csv.record().content().iter().zip(fixed.record().content().iter())
    {
        let csv_question = csv_question.read();
        let fixed_question = fixed_question.read();
        assert_eq!(csv_question.name(), fixed_question.name());
        assert_eq!(csv_question.data().len(), 3);
        for (csv_entry, fixed_entry) in
            csv_question.data().iter().zip(fixed_question.data().iter())
        {
            match (&*csv_entry.read(), &*fixed_entry.read()) {
                (Entry::String(csv_value), Entry::String(fixed_value)) => {
                    assert_eq!(csv_value, fixed_value.trim_end());
                }
                (csv_value, fixed_value) => assert_eq!(csv_value, fixed_value),
            }
        }
    }
}
