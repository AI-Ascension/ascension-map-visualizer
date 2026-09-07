// SPDX-License-Identifier: MIT

use super::{glyphs::label, layout::Layout};
use crate::presentation::{Map, Node, Settings};
use std::collections::BTreeMap;

pub(super) fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub(super) fn draw(
    map: &Map,
    settings: Settings,
    aliases: &BTreeMap<String, String>,
    layout: &Layout,
) -> String {
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {:.2} {:.2}\" role=\"img\" aria-labelledby=\"map-title map-desc\"><title id=\"map-title\">Ascension map</title><desc id=\"map-desc\">Complete presentation of {}. {}. {} nodes and {} directed edges. Arrows point toward destinations; crossings are not junctions.</desc><rect width=\"100%\" height=\"100%\" fill=\"#171a1e\"/><defs><marker id=\"arrow\" markerWidth=\"7\" markerHeight=\"7\" refX=\"6\" refY=\"3.5\" orient=\"auto\"><path d=\"M0 0L7 3.5L0 7Z\" fill=\"#a4adb6\"/></marker></defs>",
        settings.width,
        settings.height,
        layout.width,
        layout.height,
        escape(&map.identity),
        escape(&map.status),
        map.nodes.len(),
        map.edges.len()
    );
    label(&mut out, "ASCENSION MAP", 46.0, 32.0, 3.0, "#ece9e2");
    // The authoritative state is always present in the accessibility description.
    // Its short visual status is a presentation summary, not a host fact.
    let status: String = map.status.chars().take(74).collect();
    label(&mut out, &status, 46.0, 70.0, 1.35, "#d9b36a");
    let mut edges: Vec<_> = map.edges.iter().collect();
    edges.sort_by_key(|e| (&e.from, &e.to));
    for edge in edges {
        let (x1, y1) = layout.positions[&edge.from];
        let (x2, y2) = layout.positions[&edge.to];
        let distance = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt().max(1.0);
        let end_x = x2 - (x2 - x1) * 29.0 / distance;
        let end_y = y2 - (y2 - y1) * 29.0 / distance;
        let start_x = x1 + (x2 - x1) * 29.0 / distance;
        let start_y = y1 + (y2 - y1) * 29.0 / distance;
        let d = format!("M{start_x:.2} {start_y:.2}L{end_x:.2} {end_y:.2}");
        let color = if edge.selected { "#dfb66b" } else { "#78838f" };
        out.push_str(&format!("<g data-edge-from=\"{}\" data-edge-to=\"{}\"><title>{} to {}</title><path d=\"{d}\" fill=\"none\" stroke=\"#171a1e\" stroke-width=\"10\"/><path d=\"{d}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"3\" marker-end=\"url(#arrow)\"/></g>",escape(&edge.from),escape(&edge.to),escape(&aliases[&edge.from]),escape(&aliases[&edge.to])));
    }
    let mut nodes: Vec<_> = map.nodes.iter().collect();
    nodes.sort_by_key(|n| &n.id);
    for node in nodes {
        draw_node(
            &mut out,
            node,
            &aliases[&node.id],
            layout.positions[&node.id],
        );
    }
    let y = layout.height - 60.0;
    label(
        &mut out,
        "DOUBLE: CURRENT  SQUARE: LEGAL  DOT: VISITED",
        46.0,
        y,
        1.25,
        "#aeb7c0",
    );
    label(
        &mut out,
        "FADED: NOT DOWNSTREAM  ?: UNKNOWN  ARROWS: DIRECTION",
        46.0,
        y + 20.0,
        1.1,
        "#aeb7c0",
    );
    out.push_str("</svg>");
    out
}

fn draw_node(out: &mut String, node: &Node, alias: &str, (x, y): (f64, f64)) {
    let dim = !node.reachable && !node.current && !node.legal;
    let color = if node.current || node.legal {
        "#dfb66b"
    } else if dim {
        "#8a929b"
    } else {
        "#d3d9de"
    };
    let category = category_label(&node.category);
    out.push_str(&format!("<g data-node-id=\"{}\" data-alias=\"{alias}\" opacity=\"{}\"><title>{alias}: {}; category {}; current {}; legal {}; visited {}</title>",escape(&node.id),if dim {"0.65"} else {"1"},escape(&node.id),escape(&node.category),node.current,node.legal,node.visited));
    if node.legal {
        out.push_str(&format!("<rect x=\"{:.2}\" y=\"{:.2}\" width=\"54\" height=\"54\" rx=\"8\" fill=\"#22262d\" stroke=\"{color}\" stroke-width=\"3\"/>",x-27.0,y-27.0));
    } else {
        out.push_str(&format!("<circle cx=\"{x:.2}\" cy=\"{y:.2}\" r=\"25\" fill=\"#22262d\" stroke=\"{color}\" stroke-width=\"2\"/>"));
    }
    if node.current {
        out.push_str(&format!("<circle cx=\"{x:.2}\" cy=\"{y:.2}\" r=\"31\" fill=\"none\" stroke=\"{color}\" stroke-width=\"2\"/>"));
    }
    let symbol = &category[..1];
    label(out, symbol, x - 7.5, y - 10.5, 3.0, color);
    if node.visited {
        out.push_str(&format!(
            "<circle cx=\"{:.2}\" cy=\"{:.2}\" r=\"5\" fill=\"{color}\"/>",
            x + 22.0,
            y - 22.0
        ));
    }
    label(out, alias, x - 29.0, y + 39.0, 2.0, color);
    label(
        out,
        category,
        x - category.len() as f64 * 3.3,
        y + 60.0,
        1.1,
        color,
    );
    out.push_str("</g>");
}

fn category_label(category: &str) -> &'static str {
    match category {
        "monster" | "combat" | "battle" => "BATTLE",
        "elite" => "ELITE",
        "boss" => "BOSS",
        "rest" | "rest_site" => "REST",
        "shop" => "SHOP",
        "treasure" => "TREASURE",
        "event" => "EVENT",
        "start" => "START",
        "terminal" => "TERMINAL",
        _ => "? UNKNOWN",
    }
}
