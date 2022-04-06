use crate::{invalid_argument, not_found, Status};
use std::fmt::Debug;

pub fn field_must_be_specified(field_name: &str) -> Status {
    invalid_argument!("'{0}' must be specified", field_name)
}

pub fn invalid_field<T: Debug>(field_name: &str, error: T) -> Status {
    invalid_argument!("'{0}' is invalid: {1:?}", field_name, error)
}

pub fn entity_not_found<T: Debug>(entity_name: &str) -> Status {
    not_found!("{0} not found", entity_name)
}

pub trait UnwrapField<T> {
    fn unwrap_field(self, field_name: &str) -> Result<T, Status>;
}

impl<T> UnwrapField<T> for Option<T> {
    fn unwrap_field(self, field_name: &str) -> Result<T, Status> {
        self.ok_or(field_must_be_specified(field_name))
    }
}

impl UnwrapField<String> for String {
    fn unwrap_field(self, field_name: &str) -> Result<String, Status> {
        if self.is_empty() {
            return Err(field_must_be_specified(field_name));
        }
        Ok(self)
    }
}
