use crate::Error;
use anyhow::Context;
use serde::{de::DeserializeOwned, Serialize};

pub trait PageTokenable<P>
where
    P: PageToken<P>,
{
    fn page_token(&self) -> P;
}

pub trait PageToken<T> {
    // Serializes self into a string.
    fn serialize_page_token(&self) -> Result<String, Error>;

    // Deserializes string into some type T.
    fn deserialize_page_token(input: &str) -> Result<Option<T>, Error>;
}

// Implement page token for all serializable/deserializable types.
impl<T> PageToken<T> for T
where
    T: Serialize + DeserializeOwned,
{
    fn serialize_page_token(&self) -> Result<String, Error> {
        let json = serde_json::to_string(self)
            .context("serialization page token to json failed")
            .map_err(|e| Error::PageToken(e))?;
        Ok(base64::encode(json))
    }

    fn deserialize_page_token(input: &str) -> Result<Option<T>, Error> {
        if input.is_empty() {
            Ok(None)
        } else {
            let octets = base64::decode(input)
                .context("decoding input as base64 failed")
                .map_err(|e| Error::PageToken(e))?;
            let utf8 = std::str::from_utf8(&octets)
                .context("converting bytes into utf8 string failed")
                .map_err(|e| Error::PageToken(e))?;
            let deserialized = serde_json::from_str(utf8)
                .context("deserializing json string to page token struct failed")
                .map_err(|e| Error::PageToken(e))?;
            Ok(Some(deserialized))
        }
    }
}
