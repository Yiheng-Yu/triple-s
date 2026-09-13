use triple_s::read_sss;

const PATH_SSS_CLASSIC: &str = "tests/sss_classic.sss";

#[test]
fn read_sss_classic() {
    read_sss(PATH_SSS_CLASSIC).unwrap();
}
