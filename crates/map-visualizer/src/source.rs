// SPDX-License-Identifier: MIT
//! HTTP workers read an already validated immutable memory snapshot only.

use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard, RwLock},
    time::{Duration, Instant},
};

pub(crate) const MAX_MEMORY_BYTES: usize = 128 * 1024 * 1024;

pub(crate) struct State {
    pub current: Option<Arc<[u8]>>,
    pub frames: BTreeMap<String, Arc<Frame>>,
    pub replay: Arc<[u8]>,
    pub live: bool,
    pub refreshed: Instant,
    pub root_key: Option<PathBuf>,
    pub sequence: u64,
    pub head: Option<String>,
}

pub(crate) struct Frame {
    pub historical: Arc<[u8]>,
    pub observed: Arc<[u8]>,
    pub manifest: serde_json::Value,
    pub weight: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcquisitionError {
    Io(&'static str),
    Invalid(&'static str),
}

impl From<&'static str> for AcquisitionError {
    fn from(message: &'static str) -> Self {
        Self::Invalid(message)
    }
}

pub struct Source(RwLock<Arc<State>>, Mutex<()>);

impl Default for Source {
    fn default() -> Self {
        Self::new()
    }
}

impl Source {
    pub fn new() -> Self {
        Self(
            RwLock::new(Arc::new(State {
                current: None,
                frames: BTreeMap::new(),
                replay: Arc::from(b"[]".as_slice()),
                live: false,
                refreshed: Instant::now(),
                root_key: None,
                sequence: 0,
                head: None,
            })),
            Mutex::new(()),
        )
    }

    pub(crate) fn loading(&self) -> Result<MutexGuard<'_, ()>, &'static str> {
        self.1
            .try_lock()
            .map_err(|_| "artifact acquisition already in flight")
    }

    pub(crate) fn replace(&self, state: State) -> Result<(), &'static str> {
        let total: usize = state.frames.values().map(|value| value.weight).sum();
        if state.frames.len() > 4096
            || total > MAX_MEMORY_BYTES
            || state.replay.len() > 2 * 1024 * 1024
            || state.frames.values().any(|value| {
                value.historical.len() > 2 * 1024 * 1024 || value.observed.len() > 2 * 1024 * 1024
            })
            || state
                .current
                .as_ref()
                .is_some_and(|value| value.len() > 2 * 1024 * 1024)
        {
            return Err("viewer memory limit exceeded");
        }
        let replacement = Arc::new(state);
        let mut lock = self.0.try_write().map_err(|_| "viewer update busy")?;
        let old = std::mem::replace(&mut *lock, replacement);
        drop(lock);
        drop(old);
        Ok(())
    }

    pub(crate) fn snapshot(&self) -> Result<Arc<State>, &'static str> {
        // Never block a request worker on publication or filesystem activity.
        self.0
            .try_read()
            .map(|state| Arc::clone(&state))
            .map_err(|_| "viewer update busy")
    }

    pub(crate) fn current(&self) -> Result<Arc<[u8]>, &'static str> {
        let state = self.snapshot()?;
        if state.live && state.refreshed.elapsed() > Duration::from_secs(3) {
            return Err("artifact feed disconnected");
        }
        state.current.clone().ok_or("current bundle unavailable")
    }

    pub(crate) fn replay(&self) -> Result<Arc<[u8]>, &'static str> {
        Ok(self.snapshot()?.replay.clone())
    }

    pub(crate) fn frame(&self, id: &str) -> Result<Arc<[u8]>, &'static str> {
        self.snapshot()?
            .frames
            .get(id)
            .map(|frame| frame.historical.clone())
            .ok_or("historical frame unavailable")
    }
}
