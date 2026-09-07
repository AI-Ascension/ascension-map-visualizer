// SPDX-License-Identifier: MIT
//! New-directory publication with an atomic no-replace commit on supported OSes.

use cap_std::fs::{Dir, OpenOptions};
use std::{
    io::{self, Write},
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

pub fn publish(path: &Path, files: &[(&str, &[u8])]) -> io::Result<()> {
    let name = path.file_name().ok_or_else(invalid)?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent = Dir::open_ambient_dir(parent, cap_std::ambient_authority())?;
    if parent.symlink_metadata(name).is_ok() {
        return Err(io::ErrorKind::AlreadyExists.into());
    }
    if files.len() > 12
        || files
            .iter()
            .any(|(name, bytes)| !allowed(name) || bytes.len() > 16 * 1024 * 1024)
    {
        return Err(invalid());
    }
    let (temporary, directory) = staging(&parent)?;
    let result = (|| {
        for (name, bytes) in files {
            let mut file =
                directory.open_with(name, OpenOptions::new().write(true).create_new(true))?;
            file.write_all(bytes)?;
            file.sync_all()?;
        }
        drop(directory);
        rename_new(&parent, &temporary, name)
    })();
    if result.is_err() {
        let _ = parent.remove_dir_all(&temporary);
    }
    result
}

fn staging(parent: &Dir) -> io::Result<(String, Dir)> {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    for _ in 0..64 {
        let name = format!(
            ".map-render-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        match parent.create_dir(&name) {
            Ok(()) => return parent.open_dir(&name).map(|dir| (name, dir)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::other("publication staging capacity exhausted"))
}

#[cfg(target_os = "linux")]
fn rename_new(parent: &Dir, from: &str, to: &std::ffi::OsStr) -> io::Result<()> {
    rustix::fs::renameat_with(parent, from, parent, to, rustix::fs::RenameFlags::NOREPLACE)
        .map_err(Into::into)
}

#[cfg(target_os = "windows")]
fn rename_new(parent: &Dir, from: &str, to: &std::ffi::OsStr) -> io::Result<()> {
    // Windows directory rename fails when the destination already exists.
    parent.rename(from, parent, to)
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn rename_new(_: &Dir, _: &str, _: &std::ffi::OsStr) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "atomic publication unsupported on this OS",
    ))
}

fn allowed(name: &str) -> bool {
    matches!(
        name,
        "manifest.json"
            | "visible-map.json"
            | "analysis.json"
            | "decision.json"
            | "viewer.json"
            | "overview.svg"
            | "overview.png"
            | "index.html"
            | "app.js"
            | "style.css"
    )
}

fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, "invalid publication")
}
