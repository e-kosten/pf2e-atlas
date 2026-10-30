//! Bounded exact TextSearch neighbor retention, independent of DB lifetimes.
use atlas_index::{SourceVectorNeighbors, SqliteIndexReader};
use serde::{Serialize, ser::SerializeSeq};
use sha2::{Digest, Sha256};
use std::{
    io::Write,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

const MAX_BYTES: usize = 16 * 1024 * 1024;
const MAX_ENTRIES: usize = 64;
const MAX_ENTRY_BYTES: usize = 256 * 1024;
const MAX_KEY_BYTES: usize = 64 * 1024;
// Conservative allowance for allocator/Arc headers beyond typed payload sizes.
const ALLOCATION_OVERHEAD: usize = 64;
const RECIPE: &str = "exact-text-native-neighbors/v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Epoch {
    workspace: [u8; 32],
    revision: u64,
    context: [u8; 32],
}
impl Epoch {
    pub(crate) fn admitted(index: &SqliteIndexReader) -> Self {
        Self {
            workspace: Sha256::digest(index.artifact_instance().workspace_key().as_bytes()).into(),
            revision: index.artifact_instance().reference_revision(),
            context: Sha256::digest(index.context_sha256().as_bytes()).into(),
        }
    }
}

struct Budget {
    live: AtomicUsize,
    peak: AtomicUsize,
    limit: usize,
}
impl Budget {
    fn reserve(self: &Arc<Self>, bytes: usize) -> Option<Charge> {
        let next = self
            .live
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |live| {
                live.checked_add(bytes).filter(|next| *next <= self.limit)
            })
            .ok()?
            .checked_add(bytes)?;
        self.peak.fetch_max(next, Ordering::Relaxed);
        Some(Charge {
            budget: Arc::clone(self),
            bytes,
        })
    }
}
struct Charge {
    budget: Arc<Budget>,
    bytes: usize,
}
impl Drop for Charge {
    fn drop(&mut self) {
        self.budget.live.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}

pub(crate) struct ReuseKey {
    epoch: Epoch,
    hash: [u8; 32],
    bytes: Vec<u8>,
    _charge: Charge,
}
impl Write for ReuseKey {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > MAX_KEY_BYTES.saturating_sub(self.bytes.len()) {
            return Err(std::io::Error::other(
                "neighbor reuse key exceeds its budget",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(crate) struct RetainedNeighbors {
    key: ReuseKey,
    pub(crate) neighbors: SourceVectorNeighbors,
}
#[derive(Default)]
struct State {
    epoch: Option<Epoch>,
    entries: Vec<Arc<RetainedNeighbors>>,
}

/// Runtime-owned transient store. Only explicit ordinary TextSearch reads use it.
/// Misses compute independently; no lock spans SQL, hydration or inference.
pub struct TextNeighborReuse {
    state: Mutex<State>,
    budget: Arc<Budget>,
    #[cfg(test)]
    pub(crate) miss_barrier: Mutex<Option<Arc<std::sync::Barrier>>>,
}

/// Allocation counters describe this store, not process RSS or document vectors.
#[derive(Debug, Clone, Copy)]
pub struct TextNeighborReuseStats {
    pub ready_entries: usize,
    pub live_bytes: usize,
    pub peak_bytes: usize,
}
impl Default for TextNeighborReuse {
    fn default() -> Self {
        Self::with_limits(MAX_BYTES)
    }
}
impl TextNeighborReuse {
    fn with_limits(limit: usize) -> Self {
        let entries = Vec::with_capacity(MAX_ENTRIES);
        let base = std::mem::size_of::<Self>()
            + std::mem::size_of::<Budget>()
            + entries.capacity() * std::mem::size_of::<Arc<RetainedNeighbors>>()
            + 3 * ALLOCATION_OVERHEAD;
        Self {
            state: Mutex::new(State {
                epoch: None,
                entries,
            }),
            budget: Arc::new(Budget {
                live: AtomicUsize::new(base),
                peak: AtomicUsize::new(base),
                limit,
            }),
            #[cfg(test)]
            miss_barrier: Mutex::new(None),
        }
    }
    #[cfg(test)]
    pub(crate) fn miss_checkpoint(&self) {
        let barrier = self.miss_barrier.lock().unwrap().clone();
        if let Some(barrier) = barrier {
            barrier.wait();
        }
    }
    pub fn stats(&self) -> TextNeighborReuseStats {
        TextNeighborReuseStats {
            ready_entries: self.state.lock().map_or(0, |s| s.entries.len()),
            live_bytes: self.budget.live.load(Ordering::Acquire),
            peak_bytes: self.budget.peak.load(Ordering::Relaxed),
        }
    }
    /// Each newly admitted service observes publication before looking up work.
    /// A runtime serves one durable workspace; replacement requires a new runtime.
    pub(crate) fn observe(&self, epoch: Epoch) {
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        match state.epoch {
            None => state.epoch = Some(epoch),
            Some(previous)
                if previous.workspace == epoch.workspace && epoch.revision > previous.revision =>
            {
                state.entries.clear();
                state.epoch = Some(epoch);
            }
            _ => {}
        }
    }
    pub(crate) fn key(
        &self,
        epoch: Epoch,
        index: &SqliteIndexReader,
        text: &str,
        request: &atlas_index::SourceVectorCandidateRequest<'_>,
    ) -> Option<ReuseKey> {
        // Excluded consumers cannot enter through a future caller accidentally.
        if request.keys.is_some() || request.exclude.is_some() {
            return None;
        }
        self.encode(
            epoch,
            &KeyInput {
                recipe: RECIPE,
                workspace: index.artifact_instance().workspace_key(),
                revision: epoch.revision,
                context: epoch.context,
                profile: index.context().search_profile(request.profile).ok()?,
                text,
                vector: VectorBits(request.vector),
                predicate: request.query.predicate(),
                scope: "all",
                exclude: request.exclude,
                content: request.content_scope,
                k: request.limit,
            },
        )
    }
    fn encode(&self, epoch: Epoch, input: &impl Serialize) -> Option<ReuseKey> {
        // Reserve the largest possible native payload before a miss allocates
        // it. The charge remains conservative for smaller/empty windows and
        // also covers the temporary key of a hit. No uncharged staging buffer.
        let charge = self.budget.reserve(
            MAX_KEY_BYTES
                + 4096 * 16
                + std::mem::size_of::<ReuseKey>()
                + std::mem::size_of::<RetainedNeighbors>()
                + 3 * ALLOCATION_OVERHEAD,
        )?;
        let mut key = ReuseKey {
            epoch,
            hash: [0; 32],
            bytes: Vec::with_capacity(MAX_KEY_BYTES),
            _charge: charge,
        };
        serde_json::to_writer(&mut key, input).ok()?;
        key.hash = Sha256::digest(&key.bytes).into();
        Some(key)
    }
    pub(crate) fn get(&self, key: &ReuseKey) -> Option<Arc<RetainedNeighbors>> {
        let mut state = self.state.lock().ok()?;
        if state.epoch != Some(key.epoch) {
            return None;
        }
        let position = state
            .entries
            .iter()
            .position(|entry| equal_key(&entry.key, key))?;
        let found = state.entries.remove(position);
        state.entries.push(Arc::clone(&found));
        Some(found)
    }
    pub(crate) fn remove(&self, key: &ReuseKey) {
        if let Ok(mut state) = self.state.lock() {
            state.entries.retain(|entry| !equal_key(&entry.key, key));
        }
    }
    pub(crate) fn insert(&self, key: ReuseKey, neighbors: SourceVectorNeighbors) {
        let actual = neighbors.allocated_bytes()
            + key.bytes.capacity()
            + std::mem::size_of::<RetainedNeighbors>()
            + 3 * ALLOCATION_OVERHEAD;
        if actual > MAX_ENTRY_BYTES || actual > key._charge.bytes {
            return;
        }
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        if state.epoch != Some(key.epoch)
            || state
                .entries
                .iter()
                .any(|entry| equal_key(&entry.key, &key))
        {
            return;
        }
        // Borrowers keep their preallocated charges after ready-entry eviction.
        if state.entries.len() == MAX_ENTRIES {
            state.entries.remove(0);
        }
        state
            .entries
            .push(Arc::new(RetainedNeighbors { key, neighbors }));
    }
}
fn equal_key(a: &ReuseKey, b: &ReuseKey) -> bool {
    a.hash == b.hash && a.bytes == b.bytes
}

#[derive(Serialize)]
struct KeyInput<'a> {
    recipe: &'static str,
    workspace: &'a str,
    revision: u64,
    context: [u8; 32],
    profile: &'a atlas_index::SourceSearchProfileIdentity,
    text: &'a str,
    vector: VectorBits<'a>,
    predicate: &'a atlas_domain::QueryPredicate,
    scope: &'static str,
    exclude: Option<&'a atlas_domain::RecordKey>,
    content: atlas_domain::SearchContentScope,
    k: usize,
}
struct VectorBits<'a>(&'a [f32]);
impl Serialize for VectorBits<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for value in self.0 {
            seq.serialize_element(&value.to_bits())?;
        }
        seq.end()
    }
}

#[cfg(test)]
mod tests;
