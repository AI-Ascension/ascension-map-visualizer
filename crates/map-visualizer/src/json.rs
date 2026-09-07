// SPDX-License-Identifier: MIT
//! Bounded JSON with duplicate-key rejection before typed boundary validation.

use serde::de::{DeserializeSeed, Error, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};

const MAX_BYTES: usize = 2 * 1024 * 1024;
const MAX_DEPTH: u8 = 24;
const MAX_COLLECTION: usize = 8192;

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
