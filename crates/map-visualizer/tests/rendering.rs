// SPDX-License-Identifier: MIT

use map_visualizer::{
    drawing::render,
    presentation::{Edge, Error, Map, Node, Settings},
};

fn graph() -> Map {
    let nodes = [
        ("a", 0, 0),
        ("b", 1, 0),
        ("c", 1, 1),
        ("d", 2, 0),
        ("off-path", 1, 2),
    ]
    .into_iter()
    .map(|(id, floor, lane)| Node {
        id: id.to_owned(),
        floor,
        lane,
        category: "rest".into(),
        current: id == "a",
        legal: id == "b" || id == "c",
        reachable: id != "off-path",
        visited: id == "a",
    })
    .collect();
    let edges = [
        ("a", "b"),
        ("a", "c"),
        ("b", "d"),
        ("c", "d"),
        ("off-path", "d"),
    ]
    .into_iter()
    .map(|(from, to)| Edge {
        from: from.into(),
        to: to.into(),
        selected: false,
    })
    .collect();
    Map {
        identity: "map-one".into(),
        status: "AVAILABLE / COMPLETE / CURRENT".into(),
        nodes,
        edges,
    }
}

#[test]
fn invisible_or_bidi_identity_is_rejected_before_svg_output() {
    for suffix in ['\u{202e}', '\u{2066}', '\u{200b}', '\u{feff}'] {
        let mut map = graph();
        map.identity.push(suffix);
        assert!(matches!(
            render(&map, Settings::default()),
            Err(Error::InvalidIdentity)
        ));
    }
}

#[test]
fn every_node_and_directed_edge_is_retained_including_unreachable_branch() {
    let map = graph();
    let output = render(&map, Settings::default()).unwrap();
    let svg = String::from_utf8(output.svg).unwrap();
    assert_eq!(svg.matches("data-node-id=").count(), 5);
    assert_eq!(svg.matches("data-edge-from=").count(), 5);
    assert!(svg.contains("data-node-id=\"off-path\""));
    assert!(svg.contains("data-edge-from=\"off-path\" data-edge-to=\"d\""));
    assert!(svg.contains("DOUBLE: CURRENT"));
    assert_eq!(&output.png[..8], b"\x89PNG\r\n\x1a\n");
    assert_eq!(
        u32::from_be_bytes(output.png[16..20].try_into().unwrap()),
        1600
    );
    assert_eq!(
        u32::from_be_bytes(output.png[20..24].try_into().unwrap()),
        2400
    );
}

#[test]
fn render_bytes_and_aliases_are_independent_of_input_order_and_wall_clock() {
    let map = graph();
    let first = render(
        &map,
        Settings {
            width: 640,
            height: 960,
        },
    )
    .unwrap();
    let mut reordered = map.clone();
    reordered.nodes.reverse();
    reordered.edges.reverse();
    let second = render(
        &reordered,
        Settings {
            width: 640,
            height: 960,
        },
    )
    .unwrap();
    assert_eq!(first.svg, second.svg);
    assert_eq!(first.png, second.png);
    assert_eq!(first.aliases, second.aliases);
    reordered.nodes[0].category = "unknown".into();
    assert_eq!(map.aliases(), reordered.aliases());
}

#[test]
fn hostile_identity_is_escaped_and_unknown_category_never_becomes_markup() {
    let mut map = graph();
    map.identity = "<script>alert(1)</script> & \"x\"".into();
    map.nodes[0].category = "<image href='https://invalid.example/x'/>".into();
    let output = render(
        &map,
        Settings {
            width: 640,
            height: 960,
        },
    )
    .unwrap();
    let svg = String::from_utf8(output.svg).unwrap();
    assert!(!svg.contains("<script>"));
    assert!(!svg.contains("<image"));
    assert!(svg.contains("&lt;script&gt;"));
    assert!(svg.contains("? UNKNOWN"));
    assert!(!svg.contains("<text")); // All glyphs are deterministic original paths/rectangles.
}

#[test]
fn pixel_and_element_budgets_fail_before_raster_allocation() {
    assert_eq!(
        render(
            &graph(),
            Settings {
                width: 8192,
                height: 16384
            }
        )
        .err(),
        Some(Error::PixelLimit)
    );
    let mut map = graph();
    map.nodes[0].floor = i32::MAX;
    assert_eq!(
        render(&map, Settings::default()).err(),
        Some(Error::InvalidCoordinate)
    );
    map = graph();
    let template = map.nodes[0].clone();
    map.nodes.extend((0..1024).map(|i| Node {
        id: format!("extra-{i}"),
        ..template.clone()
    }));
    assert_eq!(
        render(&map, Settings::default()).err(),
        Some(Error::ElementLimit)
    );
}

#[test]
fn malformed_edges_and_duplicate_identities_do_not_render() {
    let mut map = graph();
    map.edges[0].to = "absent".into();
    assert_eq!(
        render(&map, Settings::default()).err(),
        Some(Error::InvalidEdge)
    );
    map = graph();
    map.nodes[1].id = map.nodes[0].id.clone();
    assert_eq!(
        render(&map, Settings::default()).err(),
        Some(Error::DuplicateNode)
    );
}

#[test]
fn overlapping_coordinates_stay_distinct_and_tall_map_is_not_cropped() {
    let mut map = graph();
    map.nodes[2].lane = map.nodes[1].lane;
    map.nodes[3].floor = 100;
    let output = render(
        &map,
        Settings {
            width: 640,
            height: 960,
        },
    )
    .unwrap();
    assert!(output.readability.crowded);
    assert!(output.readability.suggested_detail_views);
    assert_eq!(output.aliases.len(), 5);
    let svg = String::from_utf8(output.svg).unwrap();
    assert_eq!(svg.matches("data-node-id=").count(), 5);
    assert!(svg.contains("viewBox=\"0 0 952.00 14794.00\""));
}
