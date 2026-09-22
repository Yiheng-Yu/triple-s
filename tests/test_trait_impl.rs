///! Used for testing traits implementations
use triple_s::{Code};

#[test]
fn test_code_type_constrain() {
    assert!(Code::<f64>::new(f64::NAN, "test").is_err());
    assert!(Code::<f64>::new(f64::INFINITY, "test").is_err());
    assert!(Code::<f64>::new(f64::NEG_INFINITY, "test").is_err());
}