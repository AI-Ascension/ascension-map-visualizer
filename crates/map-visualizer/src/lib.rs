// SPDX-License-Identifier: MIT
//! Read-only deterministic map presentation. No host or provider authority.

pub mod digest;
pub mod drawing;
pub mod http;
pub mod json;
pub mod presentation;
pub mod server;
pub mod storage;
pub mod telemetry;

/// Independent layout and rasterization identity.
pub const RENDERER_VERSION: &str = "ascension-map-renderer-v1";
