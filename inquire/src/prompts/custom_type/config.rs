use crate::CustomType;

pub struct CustomTypeConfig {}

impl<T> From<&CustomType<'_, T>> for CustomTypeConfig {
    fn from(_value: &CustomType<'_, T>) -> Self {
        Self {}
    }
}
