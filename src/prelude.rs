
pub trait Flag: strum::IntoEnumIterator {
    fn to_string(&self) -> String;
    fn available_options() -> Vec<String> {
        Self::iter().map(|version| version.to_string()).collect()
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
