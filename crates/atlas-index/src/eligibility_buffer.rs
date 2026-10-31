//! Request-local exact eligibility JSON with aggregate allocation accounting.
//! Capacity refusal selects the caller's unchanged SQL path; other errors escape.
use crate::IndexError;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    io::Write,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

const SLOTS: usize = 16;
const ALLOCATION_ALLOWANCE: usize = 64;
// One admitted setup command (raw prefix+unit SQL <= complete encoded key)
// and its fixed stack/allocator bookkeeping are reserved before construction.
const SETUP_SCRATCH_ALLOWANCE: usize = 64 * 1024 + 512;

#[derive(Clone, Copy)]
pub(crate) struct EligibilityLimits {
    pub(crate) owned_bytes: usize,
    pub(crate) key_bytes: usize,
    pub(crate) json_bytes: usize,
    pub(crate) rows: usize,
    pub(crate) ready_entries: usize,
    pub(crate) bypass_entries: usize,
}
impl Default for EligibilityLimits {
    fn default() -> Self {
        Self {
            owned_bytes: 4 * 1024 * 1024,
            key_bytes: 64 * 1024,
            json_bytes: 2 * 1024 * 1024,
            rows: 262_144,
            ready_entries: SLOTS,
            bypass_entries: SLOTS,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct EligibilityStats {
    pub materializations: usize,
    pub hits: usize,
    pub bypasses: usize,
    pub oversized: usize,
    pub disabled: bool,
    pub ready: usize,
    pub live: usize,
    pub peak: usize,
}
struct Budget {
    live: AtomicUsize,
    peak: AtomicUsize,
    limit: usize,
}
impl Budget {
    fn reserve(self: &Arc<Self>, bytes: usize) -> Option<Charge> {
        let previous = self
            .live
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |live| {
                live.checked_add(bytes).filter(|next| *next <= self.limit)
            })
            .ok()?;
        self.peak.fetch_max(previous + bytes, Ordering::Relaxed);
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
        self.budget.live.fetch_sub(self.bytes, Ordering::Relaxed);
    }
}

/// Initialized boxed bytes make retained capacity explicit. Growth charges and
/// allocates a fresh box before copying and dropping the old charged box.
struct Buffer {
    bytes: Box<[u8]>,
    used: usize,
    _charge: Charge,
}
impl Buffer {
    fn new(budget: &Arc<Budget>, capacity: usize) -> Option<Self> {
        let charge = budget.reserve(capacity.checked_add(ALLOCATION_ALLOWANCE)?)?;
        Some(Self {
            bytes: vec![0; capacity].into_boxed_slice(),
            used: 0,
            _charge: charge,
        })
    }
    fn append(&mut self, bytes: &[u8], limit: usize) -> Result<(), Refusal> {
        let needed = self.used.checked_add(bytes.len()).ok_or(Refusal::Bytes)?;
        self.ensure(needed, limit)?;
        self.bytes[self.used..needed].copy_from_slice(bytes);
        self.used = needed;
        Ok(())
    }
    fn ensure(&mut self, needed: usize, limit: usize) -> Result<(), Refusal> {
        if needed > limit {
            return Err(Refusal::Bytes);
        }
        if needed <= self.bytes.len() {
            return Ok(());
        }
        let next = self.bytes.len().saturating_mul(2).max(needed).min(limit);
        let mut replacement = Self::new(&self._charge.budget, next).ok_or(Refusal::Budget)?;
        replacement.bytes[..self.used].copy_from_slice(&self.bytes[..self.used]);
        replacement.used = self.used;
        *self = replacement;
        Ok(())
    }
    fn content(&self) -> &[u8] {
        &self.bytes[..self.used]
    }
    fn json(&self) -> Result<&str, IndexError> {
        // IdWriter writes only ASCII integer tokens, commas and brackets.
        std::str::from_utf8(self.content()).map_err(|error| {
            IndexError::Invalid(format!("eligibility ID JSON encoding invariant: {error}"))
        })
    }
}
#[derive(Clone, Copy)]
enum Refusal {
    Bytes,
    Rows,
    Budget,
}
impl Refusal {
    fn oversized(self) -> bool {
        matches!(self, Self::Bytes | Self::Rows)
    }
}

pub(crate) struct IdWriter {
    buffer: Buffer,
    rows: usize,
    byte_limit: usize,
    row_limit: usize,
    refusal: Option<Refusal>,
}
impl IdWriter {
    fn new(budget: &Arc<Budget>, limits: EligibilityLimits) -> Result<Self, Refusal> {
        if limits.json_bytes < 2 {
            return Err(Refusal::Bytes);
        }
        let mut buffer = Buffer::new(budget, limits.json_bytes.min(64)).ok_or(Refusal::Budget)?;
        buffer.append(b"[", limits.json_bytes)?;
        Ok(Self {
            buffer,
            rows: 0,
            byte_limit: limits.json_bytes,
            row_limit: limits.rows,
            refusal: None,
        })
    }
    /// False is a capacity refusal. The caller must stop populating; the partial
    /// buffer will be discarded and the original exact SQL path remains usable.
    pub(crate) fn push_id(&mut self, id: i64) -> bool {
        if self.refusal.is_some() {
            return false;
        }
        if self.rows == self.row_limit {
            self.refusal = Some(Refusal::Rows);
            return false;
        }
        let mut digits = [0u8; 20];
        let mut position = digits.len();
        let mut magnitude = id.unsigned_abs();
        loop {
            position -= 1;
            digits[position] = b'0' + (magnitude % 10) as u8;
            magnitude /= 10;
            if magnitude == 0 {
                break;
            }
        }
        if id < 0 {
            position -= 1;
            digits[position] = b'-';
        }
        let comma = usize::from(self.rows != 0);
        let needed = self
            .buffer
            .used
            .checked_add(comma)
            .and_then(|n| n.checked_add(digits.len() - position))
            .and_then(|n| n.checked_add(1)); // reserve the closing bracket
        let result = needed
            .ok_or(Refusal::Bytes)
            .and_then(|needed| self.buffer.ensure(needed, self.byte_limit));
        if let Err(reason) = result {
            self.refusal = Some(reason);
            return false;
        }
        // ensure() already reserved the complete token and closing bracket.
        if comma != 0 {
            self.buffer.bytes[self.buffer.used] = b',';
            self.buffer.used += 1;
        }
        let end = self.buffer.used + digits.len() - position;
        self.buffer.bytes[self.buffer.used..end].copy_from_slice(&digits[position..]);
        self.buffer.used = end;
        self.rows += 1;
        true
    }
    fn finish(mut self) -> Result<Buffer, Refusal> {
        if let Some(reason) = self.refusal {
            return Err(reason);
        }
        self.buffer.append(b"]", self.byte_limit)?;
        Ok(self.buffer)
    }
}

struct Entry {
    hash: [u8; 32],
    key: Buffer,
    json: Buffer,
}

pub(crate) struct EligibilityStore {
    limits: EligibilityLimits,
    entries: [Option<Entry>; SLOTS],
    bypass: [Option<[u8; 32]>; SLOTS],
    stats: EligibilityStats,
    budget: Arc<Budget>,
    // Declared after entries so their buffers release before the fixed charge.
    _owner: Charge,
}
impl EligibilityStore {
    pub(crate) fn new() -> Self {
        Self::build(EligibilityLimits::default())
    }
    fn build(limits: EligibilityLimits) -> Self {
        let base = Self::base_bytes();
        let budget = Arc::new(Budget {
            live: AtomicUsize::new(base),
            peak: AtomicUsize::new(base),
            limit: limits.owned_bytes,
        });
        let owner = Charge {
            budget: Arc::clone(&budget),
            bytes: base,
        };
        Self {
            limits,
            entries: std::array::from_fn(|_| None),
            bypass: [None; SLOTS],
            stats: EligibilityStats {
                disabled: limits.ready_entries == 0 || limits.bypass_entries == 0,
                ..EligibilityStats::default()
            },
            budget,
            _owner: owner,
        }
    }
    fn base_bytes() -> usize {
        std::mem::size_of::<Self>()
            + std::mem::size_of::<Budget>()
            + ALLOCATION_ALLOWANCE
            + SETUP_SCRATCH_ALLOWANCE
    }
    #[cfg(test)]
    pub(crate) fn with_limits(limits: EligibilityLimits) -> Self {
        assert!(limits.ready_entries <= SLOTS && limits.bypass_entries <= SLOTS);
        assert!(
            limits.owned_bytes >= Self::base_bytes(),
            "eligibility budget must cover fixed owner"
        );
        Self::build(limits)
    }
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn stats(&self) -> EligibilityStats {
        EligibilityStats {
            live: self.budget.live.load(Ordering::Relaxed),
            peak: self.budget.peak.load(Ordering::Relaxed),
            ..self.stats
        }
    }
    #[cfg(test)]
    pub(crate) fn accounting_probe(&self) -> EligibilityAccountingProbe {
        EligibilityAccountingProbe(Arc::clone(&self.budget))
    }
    fn refuse(&mut self, hash: [u8; 32], reason: Refusal) {
        self.stats.bypasses += 1;
        self.stats.oversized += usize::from(reason.oversized());
        if let Some(slot) = self.bypass[..self.limits.bypass_entries]
            .iter_mut()
            .find(|slot| slot.is_none())
        {
            *slot = Some(hash);
        }
        if self.bypass[..self.limits.bypass_entries]
            .iter()
            .all(Option::is_some)
        {
            self.stats.disabled = true;
        }
    }
    pub(crate) fn get_or_try_insert<K: Serialize, E: From<IndexError>>(
        &mut self,
        key: &K,
        populate: impl FnOnce(&mut IdWriter) -> Result<(), E>,
    ) -> Result<Option<&str>, E> {
        // Hash the complete serialization, even if its encoded length is too
        // large to retain. Fingerprints can only bypass, never admit a hit.
        let mut hash = HashWriter::default();
        serde_json::to_writer(&mut hash, key).map_err(IndexError::from)?;
        let key_len = hash.len;
        let digest: [u8; 32] = hash.hash.finalize().into();
        let mut found = None;
        for (position, entry) in self.entries.iter().enumerate() {
            let Some(entry) = entry else {
                continue;
            };
            if entry.hash != digest {
                continue;
            }
            let mut compare = CompareWriter {
                bytes: entry.key.content(),
                offset: 0,
                equal: true,
            };
            serde_json::to_writer(&mut compare, key).map_err(IndexError::from)?;
            if compare.equal && compare.offset == compare.bytes.len() {
                found = Some(position);
                break;
            }
        }
        if let Some(position) = found {
            let entry = self
                .entries
                .get(position)
                .and_then(Option::as_ref)
                .ok_or_else(|| {
                    IndexError::Invalid("eligibility lookup entry disappeared".into())
                })?;
            let json = entry.json.json()?;
            self.stats.hits += 1;
            return Ok(Some(json));
        }
        if self.stats.disabled || self.bypass.contains(&Some(digest)) {
            self.stats.bypasses += 1;
            return Ok(None);
        }
        if key_len > self.limits.key_bytes {
            self.refuse(digest, Refusal::Bytes);
            return Ok(None);
        }
        let Some(mut encoded) = Buffer::new(&self.budget, key_len) else {
            self.refuse(digest, Refusal::Budget);
            return Ok(None);
        };
        let mut writer = KeyWriter {
            buffer: &mut encoded,
            refusal: None,
            limit: self.limits.key_bytes,
        };
        let result = serde_json::to_writer(&mut writer, key);
        if let Some(reason) = writer.refusal {
            self.refuse(digest, reason);
            return Ok(None);
        }
        result.map_err(IndexError::from)?;
        if <[u8; 32]>::from(Sha256::digest(encoded.content())) != digest {
            return Err(IndexError::Invalid(
                "eligibility key serialization changed during admission".into(),
            )
            .into());
        }
        let mut ids = match IdWriter::new(&self.budget, self.limits) {
            Ok(ids) => ids,
            Err(reason) => {
                self.refuse(digest, reason);
                return Ok(None);
            }
        };
        self.stats.materializations += 1;
        populate(&mut ids)?;
        let json = match ids.finish() {
            Ok(json) => json,
            Err(reason) => {
                self.refuse(digest, reason);
                return Ok(None);
            }
        };
        let position = self
            .entries
            .iter()
            .position(Option::is_none)
            .ok_or_else(|| {
                IndexError::Invalid("eligibility entry admission exceeded fixed slots".into())
            })?;
        let entry = self.entries[position].insert(Entry {
            hash: digest,
            key: encoded,
            json,
        });
        self.stats.ready += 1;
        if self.stats.ready == self.limits.ready_entries {
            self.stats.disabled = true;
        }
        Ok(Some(entry.json.json()?))
    }
}

#[cfg(test)]
pub(crate) struct EligibilityAccountingProbe(Arc<Budget>);
#[cfg(test)]
impl EligibilityAccountingProbe {
    pub(crate) fn live(&self) -> usize {
        self.0.live.load(Ordering::Relaxed)
    }
    pub(crate) fn peak(&self) -> usize {
        self.0.peak.load(Ordering::Relaxed)
    }
}

#[derive(Default)]
struct HashWriter {
    hash: Sha256,
    len: usize,
}
impl Write for HashWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.len = self
            .len
            .checked_add(bytes.len())
            .ok_or_else(|| std::io::Error::other("eligibility key length overflow"))?;
        self.hash.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
struct CompareWriter<'a> {
    bytes: &'a [u8],
    offset: usize,
    equal: bool,
}
impl Write for CompareWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let end = self
            .offset
            .checked_add(bytes.len())
            .ok_or_else(|| std::io::Error::other("eligibility key length overflow"))?;
        self.equal &= self.bytes.get(self.offset..end) == Some(bytes);
        self.offset = end;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
struct KeyWriter<'a> {
    buffer: &'a mut Buffer,
    limit: usize,
    refusal: Option<Refusal>,
}
impl Write for KeyWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if let Err(reason) = self.buffer.append(bytes, self.limit) {
            self.refusal = Some(reason);
            return Err(std::io::Error::other("eligibility key capacity refusal"));
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
