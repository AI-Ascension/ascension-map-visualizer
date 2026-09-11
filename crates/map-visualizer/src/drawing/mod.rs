// SPDX-License-Identifier: MIT
//! Pure deterministic renderer. All SVG is authored here; caller SVG is never parsed.

mod glyphs;
mod layout;
mod svg;

use crate::presentation::{Error, Map, Settings};

pub struct Rendered {
    pub svg: Vec<u8>,
    pub png: Vec<u8>,
    pub aliases: std::collections::BTreeMap<String, String>,
    pub readability: Readability,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub struct Readability {
    pub crowded: bool,
    pub suggested_detail_views: bool,
}

pub fn render(map: &Map, settings: Settings) -> Result<Rendered, Error> {
    map.validate()?;
    settings.validate()?;
    let aliases = map.aliases();
    let layout = layout::Layout::new(map);
    let svg = svg::draw(map, settings, &aliases, &layout).into_bytes();
    let options = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data(&svg, &options).map_err(|_| Error::Rasterization)?;
    let mut pixels =
        resvg::tiny_skia::Pixmap::new(settings.width, settings.height).ok_or(Error::PixelLimit)?;
    pixels.fill(resvg::tiny_skia::Color::from_rgba8(23, 26, 30, 255));
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::identity(),
        &mut pixels.as_mut(),
    );
    let png = pixels.encode_png().map_err(|_| Error::Rasterization)?;
    let scale = (settings.width as f64 / layout.width).min(settings.height as f64 / layout.height);
    let crowded = scale < 0.75;
    Ok(Rendered {
        svg,
        png,
        aliases,
        readability: Readability {
            crowded,
            suggested_detail_views: crowded,
        },
    })
}
