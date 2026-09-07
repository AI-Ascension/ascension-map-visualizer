// SPDX-License-Identifier: MIT
//! Presentation-owned input, adapted only from validated upstream bundles.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Node {
    pub id: String,
    pub floor: i32,
    pub lane: i32,
    pub category: String,
    pub current: bool,
    pub legal: bool,
    pub reachable: bool,
    pub visited: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub selected: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Map {
    pub identity: String,
    pub status: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Settings {
    pub width: u32,
    pub height: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            width: 1600,
            height: 2400,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    PixelLimit,
    ElementLimit,
    InvalidIdentity,
    InvalidCoordinate,
    DuplicateNode,
    InvalidEdge,
    Rasterization,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::PixelLimit => "image dimensions exceed validated limits",
            Self::ElementLimit => "map exceeds presentation element limits",
            Self::InvalidIdentity => "invalid presentation identity or text",
            Self::InvalidCoordinate => "logical coordinate exceeds presentation limits",
            Self::DuplicateNode => "duplicate presentation node identity",
            Self::InvalidEdge => "invalid or duplicate presentation edge",
            Self::Rasterization => "deterministic rasterization failed",
        })
    }
}

impl std::error::Error for Error {}

pub const MAX_NODES: usize = 1024;
pub const MAX_EDGES: usize = 8192;
pub const MAX_PIXELS: u64 = 32 * 1024 * 1024;

impl Settings {
    pub fn validate(self) -> Result<(), Error> {
        if self.width < 320
            || self.height < 320
            || self.width > 8192
            || self.height > 16384
            || u64::from(self.width) * u64::from(self.height) > MAX_PIXELS
        {
            return Err(Error::PixelLimit);
        }
        Ok(())
    }
}

impl Map {
    pub fn validate(&self) -> Result<(), Error> {
        if self.nodes.len() > MAX_NODES || self.edges.len() > MAX_EDGES {
            return Err(Error::ElementLimit);
        }
        if !text(&self.identity) || !text(&self.status) {
            return Err(Error::InvalidIdentity);
        }
        let mut ids = BTreeSet::new();
        for node in &self.nodes {
            if !text(&node.id) || !text(&node.category) {
                return Err(Error::InvalidIdentity);
            }
            if !(-4096..=4096).contains(&node.floor) || !(-4096..=4096).contains(&node.lane) {
                return Err(Error::InvalidCoordinate);
            }
            if !ids.insert(node.id.as_str()) {
                return Err(Error::DuplicateNode);
            }
        }
        let mut edges = BTreeSet::new();
        for edge in &self.edges {
            if edge.from == edge.to
                || !ids.contains(edge.from.as_str())
                || !ids.contains(edge.to.as_str())
                || !edges.insert((&edge.from, &edge.to))
            {
                return Err(Error::InvalidEdge);
            }
        }
        Ok(())
    }

    /// Stable reversible display aliases independent of input array ordering and room labels.
    pub fn aliases(&self) -> BTreeMap<String, String> {
        let mut nodes: Vec<_> = self.nodes.iter().collect();
        nodes.sort_by_key(|n| (n.floor, n.lane, &n.id));
        nodes
            .into_iter()
            .enumerate()
            .map(|(i, n)| (n.id.clone(), format!("N{:04}", i + 1)))
            .collect()
    }
}

fn text(value: &str) -> bool {
    !value.is_empty() && value.len() <= 512 && !value.chars().any(char::is_control)
}
