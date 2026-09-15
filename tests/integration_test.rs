use std::{fs, path::PathBuf};

use triple_s::{Survey, read_sss};

const PATH_SSS_CLASSIC: &str = "tests/sss_classic.sss";

#[test]
fn read_sss_classic() {
    let survey = read_sss(PATH_SSS_CLASSIC).unwrap();
    assert_eq!(
        survey.title.unwrap(),
        "Customer Experience Survey - HTML Test"
    );
}

#[test]
fn read_and_write_sss_classic() {
    let original = read_sss(PATH_SSS_CLASSIC).unwrap();
    let path = PathBuf::from(format!(
        "{}/triple_s_classic_roundtrip_{}.sss",
        std::env::temp_dir().display(),
        std::process::id()
    ));

    fs::write(&path, original.to_sss_classic()).unwrap();
    let roundtrip = read_sss(path.to_str().unwrap()).unwrap();
    fs::remove_file(&path).unwrap();

    assert_eq!(roundtrip, original);
}

#[test]
fn survey_writer_is_available_on_the_public_model() {
    let survey = Survey::empty();
    assert!(survey.to_sss_classic().starts_with("SSS\n"));
}
