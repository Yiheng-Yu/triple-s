mod survey;

use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
mod triple_s {
    use pyo3::prelude::*;

    /// Formats the sum of two numbers as string.
    #[pyfunction]
    fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
        Ok((a + b).to_string())
    }
}

#[cfg(test)]
mod test {
    /// just testing if external crate gets properlly imported or not
    #[test]
    fn test_init() {}
}