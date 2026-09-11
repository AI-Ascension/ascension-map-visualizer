// SPDX-License-Identifier: MIT
//! Presentation of validated owner facts and derived values; no route scoring.

use crate::{
    bundle::{Bundle, string},
    presentation::{Edge, Map, Node},
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use sts2_protocol::{
    RuntimeMapV1Availability as Availability, RuntimeMapV1Completeness as Completeness,
    RuntimeMapV1Freshness as Freshness, RuntimeMapV1Position as Position,
};

impl Bundle {
    pub fn presentation(&self) -> Result<Map, &'static str> {
        let snapshot = &self.snapshot;
        let metrics = array(&self.analysis, "node_metrics")?;
        let mut reachable = BTreeMap::new();
        for metric in metrics {
            if reachable
                .insert(string(metric, "node_id")?, boolean(metric, "reachable")?)
                .is_some()
            {
                return Err("duplicate analysis node metrics");
            }
        }
        if reachable.len() != snapshot.nodes.len() {
            return Err("analysis node coverage mismatch");
        }
        let current = match &snapshot.position {
            Position::Current { node_id } => Some(node_id.as_str()),
            _ => None,
        };
        if self.analysis["topology"]["current_node"].as_str() != current {
            return Err("analysis current position mismatch");
        }
        let legal: BTreeSet<_> = snapshot
            .bindings
            .iter()
            .map(|binding| binding.graph_node_id.as_str())
            .collect();
        let nodes = snapshot
            .nodes
            .iter()
            .map(|node| {
                Ok(Node {
                    id: node.id.clone(),
                    floor: node.row,
                    lane: node.column,
                    category: enum_name(node.category)?,
                    current: current == Some(node.id.as_str()),
                    legal: legal.contains(node.id.as_str()),
                    reachable: *reachable
                        .get(node.id.as_str())
                        .ok_or("analysis node coverage mismatch")?,
                    visited: node.visited,
                })
            })
            .collect::<Result<Vec<_>, &'static str>>()?;
        let edges = snapshot
            .edges
            .iter()
            .map(|edge| Edge {
                from: edge.from.clone(),
                to: edge.to.clone(),
                selected: false,
            })
            .collect();
        let status = format!(
            "{} / {} / {}",
            enum_name(snapshot.availability)?,
            enum_name(snapshot.completeness)?,
            enum_name(snapshot.freshness)?
        )
        .to_ascii_uppercase();
        let map = Map {
            identity: snapshot
                .map_instance_id
                .clone()
                .unwrap_or_else(|| "UNAVAILABLE".into()),
            status,
            nodes,
            edges,
        };
        map.validate().map_err(|_| "map presentation rejected")?;
        Ok(map)
    }

    /// Runtime payload carries the verified final bundle digest. Stored viewer
    /// content must use an empty ID to avoid a self-referential manifest hash.
    pub fn viewer(&self, historical: bool, stored: bool) -> Result<Value, &'static str> {
        let map = self.presentation()?;
        let candidates = self.candidates(&map)?;
        let warnings = array(&self.analysis, "warnings")?.clone();
        Ok(
            json!({"schema":"ascension-map-viewer-v1", "bundle_id": if stored {""} else {self.id()},
            "snapshot_digest":self.manifest["snapshot_digest"], "analysis_digest":self.manifest["analysis_digest"],
            "aliases":map.aliases(), "map":map, "candidates":candidates,
            "historical":historical || matches!(self.snapshot.freshness, Freshness::Historical),
            "available":matches!(self.snapshot.availability, Availability::Available),
            "complete":matches!(self.snapshot.completeness, Completeness::Complete), "warnings":warnings }),
        )
    }

    fn candidates(&self, map: &Map) -> Result<Vec<Value>, &'static str> {
        let ids: BTreeSet<_> = map.nodes.iter().map(|node| node.id.as_str()).collect();
        let edges: BTreeSet<_> = map
            .edges
            .iter()
            .map(|edge| (edge.from.as_str(), edge.to.as_str()))
            .collect();
        let current = map
            .nodes
            .iter()
            .find(|node| node.current)
            .map(|node| node.id.as_str());
        let mut output = Vec::new();
        for (index, route) in array(&self.analysis, "candidate_routes")?
            .iter()
            .enumerate()
        {
            let nodes = array(route, "nodes")?
                .iter()
                .map(|node| node.as_str().ok_or("invalid candidate node"))
                .collect::<Result<Vec<_>, _>>()?;
            let mut seen = BTreeSet::new();
            if nodes.is_empty()
                || nodes
                    .iter()
                    .any(|node| !ids.contains(node) || !seen.insert(*node))
                || nodes
                    .windows(2)
                    .any(|pair| !edges.contains(&(pair[0], pair[1])))
            {
                return Err("candidate is not a concrete graph route");
            }
            let action = string(route, "first_action_id")?;
            let binding = self
                .snapshot
                .bindings
                .iter()
                .find(|binding| binding.host_action_id == action)
                .ok_or("candidate first action is not host-bound")?;
            let destination = if nodes.first().copied() == current {
                nodes.get(1).copied()
            } else {
                nodes.first().copied()
            };
            if destination != Some(binding.graph_node_id.as_str()) {
                return Err("candidate first destination mismatch");
            }
            let selection = string(route, "selection")?;
            let score = &route["score"];
            let summary = format!(
                "{} nodes; rest {}; shop {}; elite {}; elite before rest {}; branching {}; {}",
                nodes.len(),
                count(score, "rest_count")?,
                count(score, "shop_count")?,
                count(score, "elite_count")?,
                count(score, "elite_exposure_before_rest")?,
                count(score, "retained_branching")?,
                selection.replace('_', " ")
            );
            output.push(json!({"id":format!("route-{}", index+1), "nodes":nodes, "summary":summary, "selection":selection}));
        }
        Ok(output)
    }
}

fn enum_name(value: impl serde::Serialize) -> Result<String, &'static str> {
    serde_json::to_value(value)
        .map_err(|_| "invalid category")?
        .as_str()
        .map(str::to_owned)
        .ok_or("invalid category")
}

fn array<'a>(value: &'a Value, field: &str) -> Result<&'a Vec<Value>, &'static str> {
    value
        .get(field)
        .and_then(Value::as_array)
        .ok_or("required artifact array missing")
}

fn boolean(value: &Value, field: &str) -> Result<bool, &'static str> {
    value
        .get(field)
        .and_then(Value::as_bool)
        .ok_or("required artifact boolean missing")
}

fn count(value: &Value, field: &str) -> Result<u64, &'static str> {
    value
        .get(field)
        .and_then(Value::as_u64)
        .ok_or("required route count missing")
}
