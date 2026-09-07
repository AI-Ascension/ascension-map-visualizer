// SPDX-License-Identifier: MIT
//! Load complete bundles outside the HTTP worker pool, then publish one state.

use crate::{
    bundle::{Bundle, string},
    contracts::{self, Kind},
    json,
    source::{AcquisitionError, Frame, MAX_MEMORY_BYTES, Source, State},
    storage::{ArtifactRoot, File},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::Arc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

impl Source {
    pub fn load_bundle(&self, path: &Path) -> Result<(), AcquisitionError> {
        let _loading = self.loading()?;
        let root = ArtifactRoot::open(path)
            .map_err(|_| AcquisitionError::Io("bundle root unavailable"))?;
        let bundle = Bundle::load(&root)?;
        let frame = frame(&bundle)?;
        let payload = frame.historical.clone();
        let mut frames = BTreeMap::new();
        frames.insert(bundle.id().to_owned(), frame);
        let replay = json::canonical(&json!([descriptor(bundle.id())]))?.into();
        self.replace(State {
            current: Some(payload),
            frames,
            replay,
            live: false,
            refreshed: Instant::now(),
            root_key: None,
            sequence: 0,
            head: Some(bundle.id().to_owned()),
        })
        .map_err(Into::into)
    }

    pub fn load_root(&self, path: &Path, historical: bool) -> Result<(), AcquisitionError> {
        let _loading = self.loading()?;
        let root_key = std::fs::canonicalize(path)
            .map_err(|_| AcquisitionError::Io("artifact root unavailable"))?;
        let previous_state = self.snapshot()?;
        let same_root = previous_state.root_key.as_ref() == Some(&root_key);
        let root = ArtifactRoot::open(path)
            .map_err(|_| AcquisitionError::Io("artifact root unavailable"))?;
        let feed_bytes = read_feed(&root)?;
        let feed = json::decode(&feed_bytes, File::Feed.limit() as usize)?;
        contracts::validate(Kind::Feed, &feed)?;
        let entries = feed["entries"].as_array().ok_or("feed entries missing")?;
        let head = feed["head"].as_str();
        let sequence = feed["sequence"].as_u64().ok_or("feed sequence missing")?;
        if entries.is_empty() {
            if !historical && same_root && previous_state.head.is_some() {
                return Err(AcquisitionError::Invalid(
                    "live feed cleared; await a fresh epoch snapshot",
                ));
            }
            if head.is_some() || sequence != 0 {
                return Err(AcquisitionError::Invalid("empty feed has a head"));
            }
            return self
                .replace(State {
                    current: None,
                    frames: BTreeMap::new(),
                    replay: Arc::from(b"[]".as_slice()),
                    live: !historical,
                    refreshed: Instant::now(),
                    root_key: Some(root_key),
                    sequence: 0,
                    head: None,
                })
                .map_err(Into::into);
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "clock unavailable")?
            .as_millis();
        let mut previous = 0;
        let mut frames = BTreeMap::new();
        let mut descriptors = Vec::new();
        let mut current = None;
        let mut total = 0;
        for entry in entries {
            let entry_sequence = entry["sequence"].as_u64().ok_or("entry sequence missing")?;
            if entry_sequence <= previous {
                return Err(AcquisitionError::Invalid(
                    "feed sequence is not strictly increasing",
                ));
            }
            previous = entry_sequence;
            let id = string(entry, "bundle_digest")?;
            if frames.contains_key(id) {
                return Err(AcquisitionError::Invalid("duplicate feed bundle"));
            }
            let cached = same_root
                .then(|| previous_state.frames.get(id).cloned())
                .flatten();
            let frame = if let Some(frame) = cached {
                frame
            } else {
                let bundle =
                    Bundle::load(&root.bundle(id).map_err(|_| "feed bundle unavailable")?)?;
                if bundle.id() != id {
                    return Err(AcquisitionError::Invalid("feed bundle digest mismatch"));
                }
                frame(&bundle)?
            };
            validate_entry(entry, &frame.manifest)?;
            total += frame.weight;
            if total > MAX_MEMORY_BYTES {
                return Err(AcquisitionError::Invalid(
                    "replay memory limit exceeded; use a bounded retained feed",
                ));
            }
            if Some(id) == head {
                let observed = entry["observed_at_unix_ms"]
                    .as_u64()
                    .ok_or("feed capture time missing")? as u128;
                if !historical && observed > now.saturating_add(5000) {
                    return Err(AcquisitionError::Invalid("feed clock is in the future"));
                }
                if historical || now.saturating_sub(observed) <= 30_000 {
                    current = Some(if historical {
                        frame.historical.clone()
                    } else {
                        frame.observed.clone()
                    });
                }
            }
            frames.insert(id.to_owned(), frame);
            descriptors.push(descriptor(id));
        }
        let last = entries.last().ok_or("feed head missing")?;
        if previous != sequence || head != last["bundle_digest"].as_str() {
            return Err(AcquisitionError::Invalid(
                "feed head does not match final sequence",
            ));
        }
        if !historical && same_root {
            validate_advance(&previous_state, sequence, head, &frames)?;
        }
        // A concurrently replaced index cannot splice a new head onto old frames.
        if read_feed(&root)? != feed_bytes {
            return Err(AcquisitionError::Invalid(
                "feed changed during acquisition; retry read",
            ));
        }
        let replay = json::canonical(&Value::Array(descriptors))?.into();
        self.replace(State {
            current,
            frames,
            replay,
            live: !historical,
            refreshed: Instant::now(),
            root_key: Some(root_key),
            sequence,
            head: head.map(str::to_owned),
        })
        .map_err(Into::into)
    }
}

fn descriptor(id: &str) -> Value {
    json!({"bundle_id":id, "viewer":format!("{id}/viewer.json")})
}

fn validate_entry(entry: &Value, manifest: &Value) -> Result<(), &'static str> {
    for field in ["run_id", "episode_id", "trajectory_id"] {
        if entry[field] != manifest[field] {
            return Err("feed lineage mismatch");
        }
    }
    for field in ["source_state_id", "generation"] {
        if entry[field] != manifest["history"][field] {
            return Err("feed source identity mismatch");
        }
    }
    Ok(())
}

fn frame(bundle: &Bundle) -> Result<Arc<Frame>, &'static str> {
    let historical: Arc<[u8]> = json::canonical(&bundle.viewer(true, false)?)?.into();
    let observed: Arc<[u8]> = json::canonical(&bundle.viewer(false, false)?)?.into();
    let weight = historical.len() + observed.len() + json::canonical(bundle.manifest())?.len();
    Ok(Arc::new(Frame {
        historical,
        observed,
        manifest: bundle.manifest().clone(),
        weight,
    }))
}

fn validate_advance(
    previous: &State,
    sequence: u64,
    head: Option<&str>,
    frames: &BTreeMap<String, Arc<Frame>>,
) -> Result<(), &'static str> {
    let old = previous
        .head
        .as_ref()
        .and_then(|id| previous.frames.get(id));
    let new = head.and_then(|id| frames.get(id));
    if let (Some(old), Some(new)) = (old, new) {
        let same_epoch = ["run_id", "episode_id", "trajectory_id", "map_instance"]
            .iter()
            .all(|field| old.manifest[field] == new.manifest[field]);
        if same_epoch
            && (sequence < previous.sequence
                || new.manifest["history"]["generation"].as_u64()
                    < old.manifest["history"]["generation"].as_u64())
        {
            return Err("live feed regressed within the same epoch");
        }
    }
    Ok(())
}

fn read_feed(root: &ArtifactRoot) -> Result<Vec<u8>, AcquisitionError> {
    root.read(File::Feed).map_err(|error| {
        if error.kind() == std::io::ErrorKind::InvalidData {
            AcquisitionError::Invalid("feed file rejected")
        } else {
            AcquisitionError::Io("feed file unavailable")
        }
    })
}
