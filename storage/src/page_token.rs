use crate::Error;
use serde::{de::DeserializeOwned, Serialize};

pub trait PageToken<T> {
    fn serialize_as_page_token(&self) -> Result<String, Error>;

    fn deserialize_from_page_token(input: &str) -> Result<Option<T>, Error>;
}

impl<T> PageToken<T> for T
where
    T: Serialize + DeserializeOwned,
{
    fn serialize_as_page_token(&self) -> Result<String, Error> {
        let json = serde_json::to_string(self)?;
        Ok(base64::encode(json))
    }

    fn deserialize_from_page_token(input: &str) -> Result<Option<T>, Error> {
        if input.is_empty() {
            Ok(None)
        } else {
            let octets = base64::decode(input)?;
            let utf8 = std::str::from_utf8(&octets)?;
            let deserialized = serde_json::from_str(utf8)?;
            Ok(Some(deserialized))
        }
    }
}
