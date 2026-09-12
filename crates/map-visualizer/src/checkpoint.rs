// SPDX-License-Identifier: MIT
//! Consumer validation of the frozen public reference; no checkpoint state codec.

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reference {
    schema: String,
    reference_version: String,
    handle: String,
    occurrence: String,
    boundary_kind: String,
    boundary_phase: String,
    assurance: String,
    restore_verified: bool,
}

pub(crate) fn validate(value: &Value) -> Result<(), &'static str> {
    let reference: Reference =
        serde_json::from_value(value.clone()).map_err(|_| "invalid public checkpoint reference")?;
    let handle = reference.handle.strip_prefix("ckpt-h1:");
    if reference.schema != "ascension.exact_checkpoint_reference.v1"
        || reference.reference_version != "exact-checkpoint-reference-v1"
        || !handle.is_some_and(|hex| {
            hex.len() == 64
                && hex
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        })
        || reference.occurrence.is_empty()
        || reference.occurrence.len() > 256
        || !reference
            .occurrence
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_.:-".contains(&byte))
        || [&reference.boundary_kind, &reference.boundary_phase]
            .iter()
            .any(|text| text.is_empty() || text.chars().count() > 256)
        || !matches!(
            reference.assurance.as_str(),
            "public_observation_only"
                | "capture_only"
                | "restore_supported"
                | "restore_verified"
                | "continuation_certified"
        )
        || reference.restore_verified
            != matches!(
                reference.assurance.as_str(),
                "restore_verified" | "continuation_certified"
            )
    {
        return Err("unsupported public checkpoint reference");
    }
    Ok(())
}
