// SPDX-License-Identifier: MIT
//! Read-only deterministic map presentation. No host or provider authority.

pub mod arguments;
pub mod bundle;
mod checkpoint;
pub mod cli;
mod cli_telemetry;
pub mod contracts;
pub mod digest;
pub mod drawing;
pub mod feed;
pub mod http;
pub mod json;
pub mod presentation;
pub mod publication;
pub mod render_bundle;
pub mod server;
pub mod source;
pub mod storage;
pub mod telemetry;
pub mod view;

/// Independent layout and rasterization identity.
pub const RENDERER_VERSION: &str = "ascension-map-renderer-v1";
pub const LAYOUT_VERSION: &str = "ascension-map-logical-v1";

mod demo;
