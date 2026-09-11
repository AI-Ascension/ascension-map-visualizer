// SPDX-License-Identifier: MIT

use crate::{
    RENDERER_VERSION,
    bundle::Bundle,
    contracts::{self, Kind},
    digest, drawing, json,
    presentation::Settings,
    publication,
};
use serde_json::{Value, json};
use std::path::Path;

impl Bundle {
    pub fn render_to(&self, destination: &Path, settings: Settings) -> Result<Value, &'static str> {
        let rendered =
            drawing::render(&self.presentation()?, settings).map_err(|_| "rendering failed")?;
        let viewer = json::canonical(&self.viewer(false, true)?)?;
        let mut manifest = self.manifest.clone();
        manifest["renderer_version"] = RENDERER_VERSION.into();
        manifest["presentation"] = json!({"width":settings.width,"height":settings.height,"layout_version":crate::LAYOUT_VERSION});
        manifest["contents"]["svg_ref"] = "overview.svg".into();
        manifest["contents"]["png_ref"] = "overview.png".into();
        manifest["contents"]["svg_digest"] = digest::sha256(&rendered.svg).into();
        manifest["contents"]["png_digest"] = digest::sha256(&rendered.png).into();
        manifest["contents"]["viewer_digest"] = digest::sha256(&viewer).into();
        manifest["bundle_digest"] = json::content_digest(&manifest, "bundle_digest")?.into();
        contracts::validate(Kind::Bundle, &manifest)?;
        let manifest_bytes = json::canonical(&manifest)?;
        publication::publish(
            destination,
            &[
                ("manifest.json", &manifest_bytes),
                ("visible-map.json", &self.snapshot_bytes),
                ("analysis.json", &self.analysis_bytes),
                ("decision.json", &self.decision_bytes),
                ("viewer.json", &viewer),
                ("overview.svg", &rendered.svg),
                ("overview.png", &rendered.png),
                ("index.html", include_bytes!("../../../web/index.html")),
                ("app.js", include_bytes!("../../../web/app.js")),
                ("style.css", include_bytes!("../../../web/style.css")),
            ],
        )
        .map_err(|_| "atomic output publication failed; destination must be new")?;
        Ok(
            json!({"bundle_digest":manifest["bundle_digest"], "svg_digest":manifest["contents"]["svg_digest"],
            "png_digest":manifest["contents"]["png_digest"], "png_bytes":rendered.png.len(), "width":settings.width, "height":settings.height,
            "readability":rendered.readability, "dispatchable":false}),
        )
    }
}
