// SPDX-License-Identifier: MIT
//! Root-confined, bounded reads. Capability handles prevent symlink race escapes.

use cap_std::fs::Dir;
use std::io::{self, Read};
use std::path::Path;

pub struct ArtifactRoot(Dir);

#[derive(Clone, Copy)]
pub enum File {
    Manifest,
    Snapshot,
    Analysis,
    Decision,
    Viewer,
    Svg,
    Png,
    Feed,
}

impl File {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Manifest => "manifest.json",
            Self::Snapshot => "visible-map.json",
            Self::Analysis => "analysis.json",
            Self::Decision => "decision.json",
            Self::Viewer => "viewer.json",
            Self::Svg => "overview.svg",
            Self::Png => "overview.png",
            Self::Feed => "feed.json",
        }
    }

    pub const fn limit(self) -> u64 {
        match self {
            Self::Manifest | Self::Decision => 64 * 1024,
            Self::Snapshot => 256 * 1024,
            Self::Analysis | Self::Viewer | Self::Feed => 2 * 1024 * 1024,
            Self::Svg => 16 * 1024 * 1024,
            Self::Png => 16 * 1024 * 1024,
        }
    }
}

impl ArtifactRoot {
    pub fn open(path: &Path) -> io::Result<Self> {
        Dir::open_ambient_dir(path, cap_std::ambient_authority()).map(Self)
    }

    pub fn bundle(&self, id: &str) -> io::Result<Self> {
        if !valid_bundle_id(id) {
            return Err(invalid());
        }
        let metadata = self.0.symlink_metadata(id)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(invalid());
        }
        self.0.open_dir(id).map(Self)
    }

    pub fn read(&self, file: File) -> io::Result<Vec<u8>> {
        let metadata = self.0.symlink_metadata(file.name())?;
        if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > file.limit()
        {
            return Err(invalid());
        }
        let handle = self.0.open(file.name())?;
        let metadata = handle.metadata()?;
        if !metadata.is_file() || metadata.len() > file.limit() {
            return Err(invalid());
        }
        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        handle.take(file.limit() + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > file.limit() {
            return Err(invalid());
        }
        Ok(bytes)
    }

    pub fn bundles(&self) -> io::Result<Vec<String>> {
        let mut ids = Vec::new();
        // All directory entries are bounded, including ignored non-bundle files.
        for (index, entry) in self.0.entries()?.enumerate() {
            if index >= 8192 {
                return Err(invalid());
            }
            let entry = entry?;
            let kind = entry.file_type()?;
            let Some(id) = entry.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if kind.is_dir() && !kind.is_symlink() && valid_bundle_id(&id) {
                if ids.len() >= 4096 {
                    return Err(invalid());
                }
                ids.push(id);
            }
        }
        ids.sort();
        Ok(ids)
    }
}

pub fn valid_bundle_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
}

fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "artifact rejected")
}
