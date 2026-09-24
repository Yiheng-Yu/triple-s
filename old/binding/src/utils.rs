use std::sync::Arc;
use parking_lot::RwLock;

pyo3::create_exception!(
    triple_s,
    RustBackendError,
    pyo3::exceptions::PyException,
    "RustBackendError"
);

pub fn to_backend_err(value: anyhow::Error) -> pyo3::PyErr {
    let error_message = value
        .chain()
        .rev()
        .map(|val| val.to_string())
        .collect::<Vec<String>>()
        .join("\nFrom:\n  ");
    RustBackendError::new_err(error_message)
}

#[inline]
pub fn lock_it_up<T>(item: T) -> Arc<RwLock<T>> {
    Arc::new(RwLock::new(item))
}

pub fn normalize_py_index(index: isize, total: usize) -> Option<usize> {
    let len = total as isize;

    let index = if index < 0 {
        len.checked_add(index)?
    } else {
        index
    };

    if index < 0 || index >= len {
        None
    } else {
        Some(index as usize)
    }
}


// pub fn pprint_collection<'py, I>(
//     py: Python<'py>,
//     items: I,
// ) -> PyResult<String>
// where
//     I: IntoIterator,
//     I::Item: IntoPyObject<'py>,
// {
//     let list = PyList::new(py, items)?;

//     py.import("pprint")?
//         .getattr("pformat")?
//         .call1((list,))?
//         .extract()
// }