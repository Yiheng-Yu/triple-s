use triple_s::read_sss;

const PATH_SSS_CLASSIC: &str = "tests/sss_classic.sss";

#[test]
fn read_sss_classic() {
    let survey = read_sss(PATH_SSS_CLASSIC).unwrap();
    assert_eq!(&survey.title, "Customer Experience Survey - HTML Test");
}
