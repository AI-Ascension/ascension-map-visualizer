// SPDX-License-Identifier: MIT
//! Bounded JSON with duplicate-key rejection before typed boundary validation.

use serde::de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};

const MAX_BYTES: usize = 2 * 1024 * 1024;
const MAX_DEPTH: u8 = 24;
const MAX_COLLECTION: usize = 8192;

/// Sorted-key UTF-8 JSON with finite integers only. This is the artifact
/// contract's canonical form, not a general RFC 8785 implementation.
pub fn canonical(value: &Value) -> Result<Vec<u8>, &'static str> {
    fn normalize(value: &Value) -> Result<Value, &'static str> {
        Ok(match value {
            Value::Number(number) if !number.is_i64() && !number.is_u64() => {
                return Err("non-integral artifact number");
            }
            Value::Array(values) => {
                Value::Array(values.iter().map(normalize).collect::<Result<_, _>>()?)
            }
            Value::Object(values) => {
                let sorted: std::collections::BTreeMap<_, _> = values.iter().collect();
                let mut object = Map::new();
                for (key, value) in sorted {
                    object.insert(key.clone(), normalize(value)?);
                }
                Value::Object(object)
            }
            other => other.clone(),
        })
    }
    serde_json::to_vec(&normalize(value)?).map_err(|_| "artifact serialization failed")
}

pub fn content_digest(value: &Value, field: &str) -> Result<String, &'static str> {
    let mut value = value.clone();
    let object = value.as_object_mut().ok_or("artifact object required")?;
    if !object.get(field).is_some_and(Value::is_string) {
        return Err("artifact digest field missing");
    }
    object.insert(field.to_owned(), Value::String(String::new()));
    Ok(crate::digest::sha256(&canonical(&value)?))
}

pub fn decode(bytes: &[u8], limit: usize) -> Result<Value, &'static str> {
    if limit > MAX_BYTES || bytes.len() > limit {
        return Err("JSON byte limit exceeded");
    }
    let mut reader = serde_json::Deserializer::from_slice(bytes);
    let value = Bounded { depth: 0 }
        .deserialize(&mut reader)
        .map_err(|_| "invalid bounded JSON")?;
    reader.end().map_err(|_| "trailing JSON data")?;
    Ok(value)
}

struct Bounded {
    depth: u8,
}

impl<'de> DeserializeSeed<'de> for Bounded {
    type Value = Value;
    fn deserialize<D: serde::Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
        if self.depth > MAX_DEPTH {
            return Err(D::Error::custom("JSON depth limit"));
        }
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Bounded {
    type Value = Value;
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("bounded JSON")
    }
    fn visit_bool<E: Error>(self, v: bool) -> Result<Value, E> {
        Ok(Value::Bool(v))
    }
    fn visit_i64<E: Error>(self, v: i64) -> Result<Value, E> {
        Ok(Value::Number(v.into()))
    }
    fn visit_u64<E: Error>(self, v: u64) -> Result<Value, E> {
        Ok(Value::Number(v.into()))
    }
    fn visit_f64<E: Error>(self, v: f64) -> Result<Value, E> {
        Number::from_f64(v)
            .map(Value::Number)
            .ok_or_else(|| E::custom("invalid number"))
    }
    fn visit_str<E: Error>(self, v: &str) -> Result<Value, E> {
        self.visit_string(v.to_owned())
    }
    fn visit_string<E: Error>(self, v: String) -> Result<Value, E> {
        if v.len() > 64 * 1024 {
            return Err(E::custom("JSON string limit"));
        }
        Ok(Value::String(v))
    }
    fn visit_none<E: Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_unit<E: Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element_seed(Bounded {
            depth: self.depth + 1,
        })? {
            if values.len() >= MAX_COLLECTION {
                return Err(A::Error::custom("JSON array limit"));
            }
            values.push(value);
        }
        Ok(Value::Array(values))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut values = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if key.len() > 512 || values.len() >= MAX_COLLECTION || values.contains_key(&key) {
                return Err(A::Error::custom("duplicate or excessive JSON key"));
            }
            let value = map.next_value_seed(Bounded {
                depth: self.depth + 1,
            })?;
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
}
