// SPDX-License-Identifier: MIT
//! Inert upstream schemas. No schema or reference is supplied by a bundle.

use serde_json::Value;
use std::sync::OnceLock;

pub enum Kind {
    Analysis,
    Bundle,
    Feed,
}

pub fn validate(kind: Kind, value: &Value) -> Result<(), &'static str> {
    static ANALYSIS: OnceLock<Result<jsonschema::Validator, &'static str>> = OnceLock::new();
    static BUNDLE: OnceLock<Result<jsonschema::Validator, &'static str>> = OnceLock::new();
    static FEED: OnceLock<Result<jsonschema::Validator, &'static str>> = OnceLock::new();
    let (cell, bytes): (_, &[u8]) = match kind {
        Kind::Analysis => (
            &ANALYSIS,
            include_bytes!("../../../contract-artifacts/harness/analysis.schema.json"),
        ),
        Kind::Bundle => (
            &BUNDLE,
            include_bytes!("../../../contract-artifacts/harness/bundle.schema.json"),
        ),
        Kind::Feed => (
            &FEED,
            include_bytes!("../../../contract-artifacts/harness/feed.schema.json"),
        ),
    };
    let validator = cell
        .get_or_init(|| {
            let schema = crate::json::decode(bytes, 256 * 1024)?;
            reject_remote_references(&schema)?;
            jsonschema::validator_for(&schema).map_err(|_| "packaged contract is invalid")
        })
        .as_ref()
        .map_err(|error| *error)?;
    if validator.is_valid(value) {
        Ok(())
    } else {
        Err("artifact schema rejected")
    }
}

fn reject_remote_references(value: &Value) -> Result<(), &'static str> {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                if matches!(key.as_str(), "$ref" | "$dynamicRef")
                    && !value
                        .as_str()
                        .is_some_and(|reference| reference.starts_with('#'))
                {
                    return Err("external contract reference rejected");
                }
                reject_remote_references(value)?;
            }
        }
        Value::Array(values) => {
            for value in values {
                reject_remote_references(value)?;
            }
        }
        _ => {}
    }
    Ok(())
}
