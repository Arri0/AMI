use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug, PartialEq)]
pub struct Error;

pub type SerializationResult = Result<serde_json::Value, Error>;
pub type DeserializationResult = Result<(), Error>;

pub type JsonFieldUpdate = (String, serde_json::Value);

pub fn serialize<T: Serialize>(value: T) -> SerializationResult {
    serde_json::to_value(value).map_err(|_| Error)
}

pub fn expect_serialize<T: Serialize>(value: T) -> serde_json::Value {
    serde_json::to_value(value).expect("Failed to serialize: {value:?}")
}

pub fn serialize_or_null<T: Serialize>(value: T) -> serde_json::Value {
    if let Ok(val) = serde_json::to_value(value) {
        val
    } else {
        serde_json::Value::Null
    }
}

//TODO: make this return value instead of calling callback
pub fn deser_field<T: DeserializeOwned>(
    source: &serde_json::Value,
    field_name: &str,
    callback: impl FnOnce(T),
) -> Result<(), Error> {
    if let Some(val) = source.get(field_name) {
        let val: T = serde_json::from_value::<T>(val.clone()).map_err(|_| Error)?;
        callback(val);
        Ok(())
    } else {
        Err(Error)
    }
}

pub fn deser_field_opt<T: DeserializeOwned>(
    source: &serde_json::Value,
    field_name: &str,
    callback: impl FnOnce(T),
) -> Result<(), Error> {
    if let Some(val) = source.get(field_name) {
        let val: T = serde_json::from_value(val.clone()).map_err(|_| Error)?;
        callback(val);
    }
    Ok(())
}

#[macro_export]
macro_rules! json_try {
    ($($stmt:stmt)*) => {
        (|| -> Result<(), $crate::json::Error> {
            $(
                $stmt
            )*
            Ok(())
        })().unwrap_or_else(|_| tracing::error!("Unexpected Serialization/Deserialization error!"));
    };
}
