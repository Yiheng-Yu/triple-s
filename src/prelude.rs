#[cfg(feature = "pyo3")]
use pyo3::{exceptions, prelude::PyAnyMethods};

pub trait Flag: strum::IntoEnumIterator {
    fn to_string(&self) -> String;
    fn available_options() -> Vec<String> {
        Self::iter().map(|version| version.to_string()).collect()
    }
}

#[cfg(feature = "pyo3")]
pub(crate) trait PyEnum: Flag + std::str::FromStr {
    fn from_py_any<'py>(value: pyo3::Bound<'py, pyo3::PyAny>) -> pyo3::PyResult<Self>
    where Self: for<'a> pyo3::FromPyObject<'a, 'py> {
        if let Ok(value) = value.extract::<Self>() {
            return Ok(value);
        }

        if let Ok(value) = value.extract::<String>() {
            return Self::from_str(&value).map_err(|_| {
                let mut available = Self::available_options()
                    .into_iter()
                    .map(|item| format!("'{}'", item))
                    .collect::<Vec<String>>()
                    .join(", ");
                available.push_str("None");
                exceptions::PyValueError::new_err(format!(
                    "Expected to be `{}` instance or one of: {}",
                    std::any::type_name::<Self>(),
                    available
                ))
            });
        }

        Err(exceptions::PyValueError::new_err({
            let mut available = Self::available_options()
                .into_iter()
                .map(|item| format!("'{}'", item))
                .collect::<Vec<String>>()
                .join(", ");
            available.push_str(", None");
            exceptions::PyValueError::new_err(format!(
                "Expected to be `{}` instance or one of: [{}]",
                std::any::type_name::<Self>(),
                available
            ))
        }))
    }
}

#[doc = include_str!("./doc/rs_deep_copy_trait.md")]
pub trait DeepCopy {
    /// Makes a deep copy of current instance
    fn deepcopy(&self) -> Self;
}

impl DeepCopy for String {
    fn deepcopy(&self) -> Self {
        self.clone()
    }
}

impl<T: DeepCopy> DeepCopy for Option<T> {
    fn deepcopy(&self) -> Self {
        self.as_ref().map(DeepCopy::deepcopy)
    }
}

impl<T: DeepCopy> DeepCopy for Vec<T> {
    fn deepcopy(&self) -> Self {
        self.iter().map(DeepCopy::deepcopy).collect()
    }
}
