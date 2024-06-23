use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug, PartialEq)]
pub struct SerializationError;

#[derive(Debug, PartialEq)]
pub struct DeserializationError;

pub type SerializationResult = Result<serde_json::Value, SerializationError>;
pub type DeserializationResult = Result<(), DeserializationError>;

pub fn serialize<T: Serialize>(value: T) -> SerializationResult {
    serde_json::to_value(value).map_err(|_| SerializationError)
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
) -> Result<(), DeserializationError> {
    if let Some(val) = source.get(field_name) {
        let val: T = serde_json::from_value::<T>(val.clone()).map_err(|_| DeserializationError)?;
        callback(val);
        Ok(())
    } else {
        Err(DeserializationError)
    }
}

pub fn deser_field_opt<T: DeserializeOwned>(
    source: &serde_json::Value,
    field_name: &str,
    callback: impl FnOnce(T),
) -> Result<(), DeserializationError> {
    if let Some(val) = source.get(field_name) {
        let val: T = serde_json::from_value(val.clone()).map_err(|_| DeserializationError)?;
        callback(val);
    }
    Ok(())
}
