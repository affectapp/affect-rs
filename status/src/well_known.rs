use crate::{invalid_argument, Status};
use std::fmt::Debug;

pub fn missing_required_field(field_name: &str) -> Status {
    invalid_argument!("missing required field: '{0}'", field_name)
}

pub fn invalid_field<T: Debug>(field_name: &str, error: T) -> Status {
    invalid_argument!("'{0}' is invalid: {1:?}", field_name, error)
}
