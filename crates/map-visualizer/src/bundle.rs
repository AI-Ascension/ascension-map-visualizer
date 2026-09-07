// SPDX-License-Identifier: MIT
//! Read-only validation adapter. Analysis policy remains in the harness.

use crate::{
    contracts::{self, Kind},
    digest, json,
    source::AcquisitionError,
    storage::{ArtifactRoot, File},
};
use serde_json::Value;
use sts2_protocol::{RUNTIME_MAP_V1_SCHEMA_DIGEST, RuntimeMapV1Snapshot};

pub struct Bundle {
    pub(crate) manifest: Value,
    pub(crate) snapshot: RuntimeMapV1Snapshot,
    pub(crate) snapshot_bytes: Vec<u8>,
    pub(crate) analysis: Value,
    pub(crate) analysis_bytes: Vec<u8>,
    pub(crate) decision_bytes: Vec<u8>,
    pub(crate) svg: Option<Vec<u8>>,
    pub(crate) png: Option<Vec<u8>>,
}

impl Bundle {
    pub fn load(root: &ArtifactRoot) -> Result<Self, AcquisitionError> {
        Self::load_with(|file| read(root, file))
    }

    pub(crate) fn load_with(
        read_file: impl Fn(File) -> Result<Vec<u8>, AcquisitionError>,
    ) -> Result<Self, AcquisitionError> {
        let manifest = json::decode(&read_file(File::Manifest)?, File::Manifest.limit() as usize)?;
        contracts::validate(Kind::Bundle, &manifest)?;
        let expected = string(&manifest, "bundle_digest")?;
        if json::content_digest(&manifest, "bundle_digest")? != expected {
            return Err(AcquisitionError::Invalid("bundle manifest digest mismatch"));
        }
        if string(&manifest, "schema_profile")? != "runtime-map-v1"
            || string(&manifest, "schema_digest")? != RUNTIME_MAP_V1_SCHEMA_DIGEST
        {
            return Err(AcquisitionError::Invalid("unsupported snapshot contract"));
        }
        let snapshot_bytes = read_file(File::Snapshot)?;
        verify_bytes(&snapshot_bytes, string(&manifest, "snapshot_digest")?)?;
        let snapshot = sts2_protocol::decode_map_snapshot(&snapshot_bytes)
            .map_err(|_| "snapshot rejected by protocol")?;
        let analysis_bytes = read_file(File::Analysis)?;
        let analysis = json::decode(&analysis_bytes, File::Analysis.limit() as usize)?;
        contracts::validate(Kind::Analysis, &analysis)?;
        let analysis_digest = string(&analysis, "content_digest")?;
        if json::content_digest(&analysis, "content_digest")? != analysis_digest
            || string(&manifest, "analysis_digest")? != analysis_digest
            || string(&manifest, "snapshot_digest")? != string(&analysis, "snapshot_digest")?
            || string(&manifest, "analysis_version")? != string(&analysis, "analysis_version")?
        {
            return Err(AcquisitionError::Invalid(
                "analysis digest or version mismatch",
            ));
        }
        validate_identity(&manifest, &snapshot, &analysis)?;
        let contents = manifest.get("contents").ok_or("bundle contents missing")?;
        let decision_bytes = read_file(File::Decision)?;
        verify_bytes(&decision_bytes, string(contents, "decision_digest")?)?;
        json::decode(&decision_bytes, File::Decision.limit() as usize)?;
        let viewer_bytes = read_file(File::Viewer)?;
        verify_bytes(&viewer_bytes, string(contents, "viewer_digest")?)?;
        let stored_viewer = json::decode(&viewer_bytes, File::Viewer.limit() as usize)?;
        let svg = optional(&read_file, contents, File::Svg, "svg_ref", "svg_digest")?;
        let png = optional(&read_file, contents, File::Png, "png_ref", "png_digest")?;
        if let Some(bytes) = &png {
            validate_png_header(bytes)?;
        }
        let bundle = Self {
            manifest,
            snapshot,
            snapshot_bytes,
            analysis,
            analysis_bytes,
            decision_bytes,
            svg,
            png,
        };
        let expected_viewer = bundle.viewer(false, true)?;
        if string(&bundle.manifest, "renderer_version")? == "unrendered" {
            if stored_viewer != serde_json::json!({})
                || !bundle.manifest["presentation"].is_null()
                || bundle.svg.is_some()
                || bundle.png.is_some()
            {
                return Err(AcquisitionError::Invalid(
                    "unrendered viewer must be an explicit empty object",
                ));
            }
        } else if string(&bundle.manifest, "renderer_version")? != crate::RENDERER_VERSION
            || stored_viewer != expected_viewer
        {
            return Err(AcquisitionError::Invalid(
                "stored viewer does not match validated bundle",
            ));
        } else {
            bundle.verify_rendered_images()?;
        }
        Ok(bundle)
    }

    pub fn id(&self) -> &str {
        // The schema and explicit digest check establish this invariant at construction.
        self.manifest["bundle_digest"].as_str().unwrap_or("")
    }

    pub fn manifest(&self) -> &Value {
        &self.manifest
    }

    fn verify_rendered_images(&self) -> Result<(), &'static str> {
        let presentation = &self.manifest["presentation"];
        if string(presentation, "layout_version")? != crate::LAYOUT_VERSION {
            return Err("unsupported layout version");
        }
        let width = presentation["width"]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or("invalid render width")?;
        let height = presentation["height"]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or("invalid render height")?;
        let expected = crate::drawing::render(
            &self.presentation()?,
            crate::presentation::Settings { width, height },
        )
        .map_err(|_| "recorded rendering rejected")?;
        if self.svg.as_deref() != Some(expected.svg.as_slice())
            || self.png.as_deref() != Some(expected.png.as_slice())
        {
            return Err("image bytes do not match the exact graph and renderer");
        }
        Ok(())
    }

    /// Only a validation report; does not restore host/session authorization.
    pub fn report(&self) -> Value {
        serde_json::json!({
            "bundle_digest": self.id(),
            "snapshot_digest": self.manifest["snapshot_digest"],
            "analysis_digest": self.manifest["analysis_digest"],
            "nodes": self.snapshot.nodes.len(), "edges": self.snapshot.edges.len(),
            "generation": self.snapshot.generation,
            "availability": self.snapshot.availability,
            "completeness": self.snapshot.completeness,
            "freshness": self.snapshot.freshness,
            "svg_bytes": self.svg.as_ref().map_or(0, Vec::len),
            "png_bytes": self.png.as_ref().map_or(0, Vec::len),
            "dispatchable": false
        })
    }
}

fn validate_identity(
    manifest: &Value,
    snapshot: &RuntimeMapV1Snapshot,
    analysis: &Value,
) -> Result<(), &'static str> {
    for field in ["map_instance", "act"] {
        if string(manifest, field)? != string(analysis, field)? {
            return Err("analysis scope mismatch");
        }
    }
    let history = manifest.get("history").ok_or("bundle history missing")?;
    if string(history, "source_state_id")? != snapshot.state_id
        || string(analysis, "source_state_id")? != snapshot.state_id
        || history["generation"].as_u64() != Some(snapshot.generation)
        || analysis["generation"].as_u64() != Some(snapshot.generation)
    {
        return Err("snapshot generation mismatch");
    }
    if let Some(identity) = &snapshot.map_instance_id
        && string(manifest, "map_instance")? != identity
    {
        return Err("snapshot map mismatch");
    }
    if let Some(act) = snapshot.act_id
        && string(manifest, "act")? != act.to_string()
    {
        return Err("snapshot act mismatch");
    }
    let topology = analysis
        .get("topology")
        .ok_or("analysis topology missing")?;
    if topology["node_count"].as_u64() != Some(snapshot.nodes.len() as u64)
        || topology["edge_count"].as_u64() != Some(snapshot.edges.len() as u64)
    {
        return Err("analysis topology counts mismatch");
    }
    Ok(())
}

fn optional(
    read_file: &impl Fn(File) -> Result<Vec<u8>, AcquisitionError>,
    contents: &Value,
    file: File,
    reference: &str,
    hash: &str,
) -> Result<Option<Vec<u8>>, AcquisitionError> {
    match (contents.get(reference), contents.get(hash)) {
        (Some(Value::Null), Some(Value::Null)) => Ok(None),
        (Some(Value::String(name)), Some(Value::String(expected))) if name == file.name() => {
            let bytes = read_file(file)?;
            verify_bytes(&bytes, expected)?;
            Ok(Some(bytes))
        }
        _ => Err(AcquisitionError::Invalid("inconsistent optional artifact")),
    }
}

pub(crate) fn read(root: &ArtifactRoot, file: File) -> Result<Vec<u8>, AcquisitionError> {
    root.read(file).map_err(|error| {
        if error.kind() == std::io::ErrorKind::InvalidData {
            AcquisitionError::Invalid("artifact file rejected")
        } else {
            AcquisitionError::Io("artifact file unavailable")
        }
    })
}

pub(crate) fn string<'a>(value: &'a Value, field: &str) -> Result<&'a str, &'static str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or("required artifact string missing")
}

fn verify_bytes(bytes: &[u8], expected: &str) -> Result<(), &'static str> {
    if digest::valid(expected) && digest::sha256(bytes) == expected {
        Ok(())
    } else {
        Err("artifact file digest mismatch")
    }
}

fn validate_png_header(bytes: &[u8]) -> Result<(), &'static str> {
    if bytes.len() < 33
        || &bytes[..8] != b"\x89PNG\r\n\x1a\n"
        || bytes[8..12] != [0, 0, 0, 13]
        || &bytes[12..16] != b"IHDR"
    {
        return Err("invalid PNG header");
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().map_err(|_| "invalid PNG width")?);
    let height = u32::from_be_bytes(bytes[20..24].try_into().map_err(|_| "invalid PNG height")?);
    crate::presentation::Settings { width, height }
        .validate()
        .map_err(|_| "PNG pixel limit exceeded")
}
