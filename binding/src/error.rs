pyo3::create_exception!(
    triple_s, 
    RustBackendError, 
    pyo3::exceptions::PyException, 
    "RustBackendError"
);

pub fn to_backend_err(value: anyhow::Error) -> pyo3::PyErr {
    let error_message = value.chain()
            .rev()
            .map(|val| val.to_string())
            .collect::<Vec<String>>()
            .join("\nFrom:\n  ");
    RustBackendError::new_err(error_message)
}