// SPDX-License-Identifier: MIT

use crate::presentation::Map;
use std::collections::BTreeMap;

pub(super) struct Layout {
    pub positions: BTreeMap<String, (f64, f64)>,
    pub width: f64,
    pub height: f64,
}

impl Layout {
    pub fn new(map: &Map) -> Self {
        let mut groups: BTreeMap<(i32, i32), Vec<&str>> = BTreeMap::new();
        for node in &map.nodes {
            groups
                .entry((node.floor, node.lane))
                .or_default()
                .push(&node.id);
        }
        for ids in groups.values_mut() {
            ids.sort_unstable();
        }
        let min_lane = map.nodes.iter().map(|n| n.lane).min().unwrap_or(0);
        let max_lane = map.nodes.iter().map(|n| n.lane).max().unwrap_or(0);
        let min_floor = map.nodes.iter().map(|n| n.floor).min().unwrap_or(0);
        let max_floor = map.nodes.iter().map(|n| n.floor).max().unwrap_or(0);
        let stack = groups.values().map(Vec::len).max().unwrap_or(1) as f64;
        let lane_gap = 132.0 * stack;
        let width = (f64::from(max_lane - min_lane) + 1.0) * lane_gap + 160.0;
        let height = (f64::from(max_floor - min_floor) + 1.0) * 144.0 + 250.0;
        let positions = groups
            .into_iter()
            .flat_map(|((floor, lane), ids)| {
                ids.into_iter().enumerate().map(move |(i, id)| {
                    (
                        id.to_owned(),
                        (
                            110.0 + f64::from(lane - min_lane) * lane_gap + i as f64 * 132.0,
                            180.0 + f64::from(max_floor - floor) * 144.0,
                        ),
                    )
                })
            })
            .collect();
        Self {
            positions,
            width: width.max(740.0),
            height: height.max(460.0),
        }
    }
}
