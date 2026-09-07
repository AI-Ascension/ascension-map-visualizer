// SPDX-License-Identifier: MIT
//! Harness-generated synthetic artifacts embedded for standalone offline operation.

use crate::{bundle::Bundle, storage::File};

pub(crate) fn load() -> Result<Bundle, &'static str> {
    Bundle::load_with(|file| {
        let bytes: &[u8] = match file {
            File::Manifest => include_bytes!("../../../fixtures/demo/manifest.json"),
            File::Snapshot => include_bytes!("../../../fixtures/demo/visible-map.json"),
            File::Analysis => include_bytes!("../../../fixtures/demo/analysis.json"),
            File::Decision => include_bytes!("../../../fixtures/demo/decision.json"),
            File::Viewer => include_bytes!("../../../fixtures/demo/viewer.json"),
            _ => return Err("embedded demo artifact unavailable"),
        };
        Ok(bytes.to_vec())
    })
}
