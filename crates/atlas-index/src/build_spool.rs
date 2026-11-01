//! Disposable typed build streams. No stream supplies SQL or persistent identity.
use crate::{
    ArtifactInstance, CompleteSourceInventory, IndexBuildPack, IndexError, PreparedRecordRef,
    PublicationSource, SemanticOccurrence, SourceAliasInput, SourceArtifactBuildContext,
    SourceArtifactDiagnostic, SourceArtifactRecordInput, SourceLexicalUnitInput,
    SourceRemasterPairInput, TraitDefinition, TraitSeedOccurrence,
};
use atlas_record::source_record::{
    SourceBackedRecord, SourceContentOutcome, SourceRelationshipOccurrence, SourceSearchSelection,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    marker::PhantomData,
};

fn io(error: impl std::fmt::Display) -> IndexError {
    IndexError::Operation(format!("build spool: {error}"))
}

/// File metadata observed at one boundary. Allocated bytes use filesystem
/// block accounting where available; they do not establish exclusive physical
/// allocation on copy-on-write filesystems or include SQLite's separate files.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildFileUsage {
    pub file_bytes: u64,
    pub allocated_bytes: Option<u64>,
}
impl BuildFileUsage {
    pub(crate) fn of(file: &File) -> Result<Self, IndexError> {
        let metadata = file.metadata().map_err(io)?;
        #[cfg(unix)]
        let allocated_bytes = {
            use std::os::unix::fs::MetadataExt;
            Some(
                metadata
                    .blocks()
                    .checked_mul(512)
                    .ok_or_else(|| io("allocated byte count overflow"))?,
            )
        };
        #[cfg(not(unix))]
        let allocated_bytes = None;
        Ok(Self {
            file_bytes: metadata.len(),
            allocated_bytes,
        })
    }
    pub fn combine(self, other: Self) -> Result<Self, IndexError> {
        Ok(Self {
            file_bytes: self
                .file_bytes
                .checked_add(other.file_bytes)
                .ok_or_else(|| io("file byte count overflow"))?,
            allocated_bytes: match (self.allocated_bytes, other.allocated_bytes) {
                (Some(a), Some(b)) => Some(
                    a.checked_add(b)
                        .ok_or_else(|| io("allocated byte count overflow"))?,
                ),
                _ => None,
            },
        })
    }
}

/// A record-sized, checksummed stream owned by this build and removed on drop.
pub struct BuildFrameStream<T> {
    file: tempfile::NamedTempFile,
    count: usize,
    max_encoded: u64,
    max_decoded: u64,
    written: u64,
    kind: PhantomData<T>,
}

/// An address produced by this build's writer, never a stored or supplied identity.
#[derive(Clone, Copy, Debug)]
pub struct BuildFrameOffset(u64);
impl<T: Serialize + DeserializeOwned> BuildFrameStream<T> {
    #[cfg(any(test, feature = "test-support"))]
    pub fn new() -> Result<Self, IndexError> {
        Ok(Self {
            file: tempfile::NamedTempFile::new().map_err(io)?,
            count: 0,
            max_encoded: 0,
            max_decoded: 0,
            written: 0,
            kind: PhantomData,
        })
    }
    pub fn new_in(path: &std::path::Path) -> Result<Self, IndexError> {
        Ok(Self {
            file: tempfile::NamedTempFile::new_in(path).map_err(io)?,
            count: 0,
            max_encoded: 0,
            max_decoded: 0,
            written: 0,
            kind: PhantomData,
        })
    }
    pub fn len(&self) -> usize {
        self.count
    }
    pub fn file_usage(&self) -> Result<BuildFileUsage, IndexError> {
        BuildFileUsage::of(self.file.as_file())
    }
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
    /// Largest serialized frame actually observed while writing this stream.
    pub fn max_decoded_frame_bytes(&self) -> u64 {
        self.max_decoded
    }
    pub fn max_encoded_frame_bytes(&self) -> u64 {
        self.max_encoded
    }

    pub fn push(&mut self, value: &T) -> Result<(), IndexError> {
        self.push_indexed(value).map(|_| ())
    }
    pub fn push_indexed(&mut self, value: &T) -> Result<BuildFrameOffset, IndexError> {
        let payload = serde_json::to_vec(value)?;
        let size = u64::try_from(payload.len()).map_err(io)?;
        let end = self
            .written
            .checked_add(40)
            .and_then(|n| n.checked_add(size))
            .ok_or_else(|| io("frame offset overflow"))?;
        let file = self.file.as_file_mut();
        if file.seek(SeekFrom::End(0)).map_err(io)? != self.written {
            return Err(io("frame file length differs"));
        }
        let offset = BuildFrameOffset(self.written);
        file.write_all(&size.to_le_bytes()).map_err(io)?;
        file.write_all(&Sha256::digest(&payload)).map_err(io)?;
        file.write_all(&payload).map_err(io)?;
        self.max_encoded = self.max_encoded.max(size);
        self.max_decoded = self.max_decoded.max(size);
        self.written = end;
        self.count = self
            .count
            .checked_add(1)
            .ok_or_else(|| io("frame count overflow"))?;
        Ok(offset)
    }
    pub fn visit(
        &self,
        visitor: &mut dyn FnMut(T) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        self.visit_if(&mut |_| true, visitor)
    }
    pub fn visit_selected(
        &self,
        ordinals: &std::collections::BTreeSet<usize>,
        visitor: &mut dyn FnMut(T) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        self.visit_if(&mut |ordinal| ordinals.contains(&ordinal), visitor)
    }
    fn visit_if(
        &self,
        selected: &mut dyn FnMut(usize) -> bool,
        visitor: &mut dyn FnMut(T) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        let mut cursor = self.cursor()?;
        while let Some(value) = cursor.next_selected(selected)? {
            visitor(value)?;
        }
        Ok(())
    }
    pub fn cursor(&self) -> Result<BuildFrameCursor<T>, IndexError> {
        let file = File::open(self.file.path()).map_err(io)?;
        let length = file.metadata().map_err(io)?.len();
        if length != self.written {
            return Err(io("frame file length differs"));
        }
        Ok(BuildFrameCursor {
            file,
            length,
            count: self.count,
            ordinal: 0,
            max_encoded: self.max_encoded,
            max_decoded: self.max_decoded,
            kind: PhantomData,
        })
    }
}

/// Independent forward reader for an owned, bounded frame stream.
/// Each call owns at most one decoded frame; repeatable readers have independent offsets.
pub struct BuildFrameCursor<T> {
    file: File,
    length: u64,
    count: usize,
    ordinal: usize,
    max_encoded: u64,
    max_decoded: u64,
    kind: PhantomData<T>,
}
impl<T: DeserializeOwned> BuildFrameCursor<T> {
    /// Direct reads leave the independent sequential position/count unchanged.
    pub fn read_at(&mut self, offset: BuildFrameOffset) -> Result<T, IndexError> {
        if offset.0 >= self.length || self.file.metadata().map_err(io)?.len() != self.length {
            return Err(io("frame offset or length differs"));
        }
        let position = self.file.stream_position().map_err(io)?;
        self.file.seek(SeekFrom::Start(offset.0)).map_err(io)?;
        let result = self.frame_size().and_then(|size| self.payload(size));
        self.file.seek(SeekFrom::Start(position)).map_err(io)?;
        result
    }

    fn frame_size(&mut self) -> Result<u64, IndexError> {
        let mut size = [0; 8];
        self.file.read_exact(&mut size).map_err(io)?;
        let size = u64::from_le_bytes(size);
        if size > self.max_encoded || size > self.max_decoded {
            return Err(io("frame exceeds observed record bounds"));
        }
        let remaining = self
            .length
            .checked_sub(self.file.stream_position().map_err(io)?)
            .ok_or_else(|| io("frame offset overflow"))?;
        if remaining < 32 || size > remaining - 32 {
            return Err(io("truncated frame"));
        }
        Ok(size)
    }

    fn payload(&mut self, size: u64) -> Result<T, IndexError> {
        let mut expected = [0; 32];
        self.file.read_exact(&mut expected).map_err(io)?;
        let mut payload = vec![0; usize::try_from(size).map_err(io)?];
        self.file.read_exact(&mut payload).map_err(io)?;
        if Sha256::digest(&payload).as_slice() != expected {
            return Err(io("frame checksum differs"));
        }
        Ok(serde_json::from_slice(&payload)?)
    }

    pub fn next_frame(&mut self) -> Result<Option<T>, IndexError> {
        self.next_selected(&mut |_| true)
    }
    fn next_selected(
        &mut self,
        selected: &mut dyn FnMut(usize) -> bool,
    ) -> Result<Option<T>, IndexError> {
        while self.file.stream_position().map_err(io)? < self.length {
            if self.ordinal >= self.count {
                return Err(io("frame count differs"));
            }
            let size = self.frame_size()?;
            let include = selected(self.ordinal);
            self.ordinal += 1;
            if !include {
                self.file
                    .seek(SeekFrom::Current(
                        i64::try_from(
                            size.checked_add(32)
                                .ok_or_else(|| io("frame offset overflow"))?,
                        )
                        .map_err(io)?,
                    ))
                    .map_err(io)?;
                continue;
            }
            return self.payload(size).map(Some);
        }
        if self.ordinal != self.count || self.file.metadata().map_err(io)?.len() != self.length {
            return Err(io("frame count or length differs"));
        }
        Ok(None)
    }
}

#[derive(Serialize, Deserialize)]
pub struct PreparedRecordFrame {
    pub pack: String,
    /// Exact checked JSON bytes transported as text rather than a JSON byte array.
    pub snapshot: String,
    pub source_path: String,
    pub content_hash: String,
    pub preparation_context_hash: String,
    pub content: Vec<SourceContentOutcome>,
    pub relationships: Vec<SourceRelationshipOccurrence>,
    pub diagnostics: Vec<SourceArtifactDiagnostic>,
    pub selection: SourceSearchSelection,
    pub lexical: Vec<SourceLexicalUnitInput>,
}
impl PreparedRecordFrame {
    pub fn decode(
        self,
    ) -> Result<
        (
            SourceArtifactRecordInput,
            Vec<u8>,
            SourceSearchSelection,
            Vec<SourceLexicalUnitInput>,
        ),
        IndexError,
    > {
        let snapshot = self.snapshot.into_bytes();
        let source = atlas_foundry_model::decode_snapshot(&snapshot)?;
        let record = SourceBackedRecord::new(&self.pack, source)
            .map_err(|_| io("checked snapshot has no record identity"))?;
        if self.selection.root != *record.key() {
            return Err(io("selection belongs to another root"));
        }
        Ok((
            SourceArtifactRecordInput {
                record,
                source_path: self.source_path,
                content_hash: self.content_hash,
                preparation_context_hash: self.preparation_context_hash,
                content: self.content,
                relationships: self.relationships,
                diagnostics: self.diagnostics,
            },
            snapshot,
            self.selection,
            self.lexical,
        ))
    }
}

pub struct PublicationSpool {
    pub context: SourceArtifactBuildContext,
    pub packs: Vec<IndexBuildPack>,
    pub inventory: CompleteSourceInventory,
    pub expected: ArtifactInstance,
    pub records: BuildFrameStream<PreparedRecordFrame>,
    pub lexical: BuildFrameStream<SourceLexicalUnitInput>,
    pub semantic: BuildFrameStream<SemanticOccurrence>,
    pub aliases: BuildFrameStream<SourceAliasInput>,
    pub remaster_pairs: BuildFrameStream<SourceRemasterPairInput>,
    pub traits: BuildFrameStream<TraitDefinition>,
    pub trait_seeds: BuildFrameStream<TraitSeedOccurrence>,
}
impl PublicationSpool {
    pub fn new_in(
        scratch: &std::path::Path,
        context: SourceArtifactBuildContext,
        packs: Vec<IndexBuildPack>,
        inventory: CompleteSourceInventory,
        expected: ArtifactInstance,
    ) -> Result<Self, IndexError> {
        Ok(Self {
            context,
            packs,
            inventory,
            expected,
            records: BuildFrameStream::new_in(scratch)?,
            lexical: BuildFrameStream::new_in(scratch)?,
            semantic: BuildFrameStream::new_in(scratch)?,
            aliases: BuildFrameStream::new_in(scratch)?,
            remaster_pairs: BuildFrameStream::new_in(scratch)?,
            traits: BuildFrameStream::new_in(scratch)?,
            trait_seeds: BuildFrameStream::new_in(scratch)?,
        })
    }
    #[cfg(any(test, feature = "test-support"))]
    pub fn new(
        context: SourceArtifactBuildContext,
        packs: Vec<IndexBuildPack>,
        inventory: CompleteSourceInventory,
        expected: ArtifactInstance,
    ) -> Result<Self, IndexError> {
        Ok(Self {
            context,
            packs,
            inventory,
            expected,
            records: BuildFrameStream::new()?,
            lexical: BuildFrameStream::new()?,
            semantic: BuildFrameStream::new()?,
            aliases: BuildFrameStream::new()?,
            remaster_pairs: BuildFrameStream::new()?,
            traits: BuildFrameStream::new()?,
            trait_seeds: BuildFrameStream::new()?,
        })
    }
    pub fn file_usage(&self) -> Result<BuildFileUsage, IndexError> {
        let mut usage = self.records.file_usage()?;
        for next in [
            self.lexical.file_usage()?,
            self.semantic.file_usage()?,
            self.aliases.file_usage()?,
            self.remaster_pairs.file_usage()?,
            self.traits.file_usage()?,
            self.trait_seeds.file_usage()?,
        ] {
            usage = usage.combine(next)?;
        }
        Ok(usage)
    }
}
impl PublicationSource for PublicationSpool {
    fn context(&self) -> &SourceArtifactBuildContext {
        &self.context
    }
    fn packs(&self) -> &[IndexBuildPack] {
        &self.packs
    }
    fn inventory(&self) -> &CompleteSourceInventory {
        &self.inventory
    }
    fn record_count(&self) -> usize {
        self.records.len()
    }
    fn semantic_count(&self) -> usize {
        self.semantic.len()
    }
    fn expected_instance(&self) -> Option<&ArtifactInstance> {
        Some(&self.expected)
    }
    fn records(
        &self,
        visitor: &mut dyn FnMut(PreparedRecordRef<'_>) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        self.records.visit(&mut |frame| {
            let (source, snapshot, selection, lexical) = frame.decode()?;
            visitor(PreparedRecordRef {
                source: &source,
                snapshot: &snapshot,
                selection: &selection,
                lexical: &lexical,
            })
        })
    }
    fn lexical(
        &self,
        v: &mut dyn FnMut(&SourceLexicalUnitInput) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        self.lexical.visit(&mut |row| v(&row))
    }
    fn semantic(
        &self,
        v: &mut dyn FnMut(&SemanticOccurrence) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        self.semantic.visit(&mut |row| v(&row))
    }
    fn aliases(
        &self,
        v: &mut dyn FnMut(&SourceAliasInput) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        self.aliases.visit(&mut |row| v(&row))
    }
    fn remaster_pairs(
        &self,
        v: &mut dyn FnMut(&SourceRemasterPairInput) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        self.remaster_pairs.visit(&mut |row| v(&row))
    }
    fn traits(
        &self,
        v: &mut dyn FnMut(&TraitDefinition) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        self.traits.visit(&mut |row| v(&row))
    }
    fn trait_seeds(
        &self,
        v: &mut dyn FnMut(&TraitSeedOccurrence) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        self.trait_seeds.visit(&mut |row| v(&row))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn indexed_reads_are_direct_checked_and_do_not_disturb_forward_order() {
        let mut stream = BuildFrameStream::<String>::new().unwrap();
        let mut packs = [Vec::new(), Vec::new(), Vec::new()];
        for ordinal in 0..513 {
            let value = format!("{ordinal} 🐉");
            packs[ordinal % 3].push((stream.push_indexed(&value).unwrap(), value));
        }
        let mut cursor = stream.cursor().unwrap();
        assert_eq!(cursor.next_frame().unwrap().unwrap(), "0 🐉");
        for pack in &packs {
            for (offset, expected) in pack {
                assert_eq!(&cursor.read_at(*offset).unwrap(), expected);
            }
        }
        assert_eq!(cursor.next_frame().unwrap().unwrap(), "1 🐉");
        // Damaging an unselected frame cannot turn indexed access into a full scan.
        stream
            .file
            .as_file_mut()
            .seek(SeekFrom::Start(packs[0][0].0.0))
            .unwrap();
        stream
            .file
            .as_file_mut()
            .write_all(&u64::MAX.to_le_bytes())
            .unwrap();
        assert_eq!(cursor.read_at(packs[2][170].0).unwrap(), packs[2][170].1);
        assert!(cursor.read_at(packs[0][0].0).is_err());
        assert!(cursor.read_at(BuildFrameOffset(u64::MAX)).is_err());
    }

    #[test]
    fn indexed_cursor_rejects_changed_length_and_checked_invalid_json() {
        let mut stream = BuildFrameStream::<String>::new().unwrap();
        let address = stream.push_indexed(&"bounded".into()).unwrap();
        let mut cursor = stream.cursor().unwrap();
        stream.file.as_file_mut().seek(SeekFrom::Start(8)).unwrap();
        let invalid = b"!bounded!";
        stream
            .file
            .as_file_mut()
            .write_all(&Sha256::digest(invalid))
            .unwrap();
        stream.file.as_file_mut().write_all(invalid).unwrap();
        assert!(cursor.read_at(address).is_err());
        stream
            .file
            .as_file_mut()
            .set_len(stream.written + 1)
            .unwrap();
        assert!(cursor.read_at(address).is_err());
        assert!(stream.cursor().is_err());
        stream
            .file
            .as_file_mut()
            .set_len(stream.written - 1)
            .unwrap();
        assert!(stream.cursor().is_err());
    }

    #[test]
    fn forward_cursors_are_independent_and_reject_changed_frames_on_consumption() {
        let mut stream = BuildFrameStream::<usize>::new().unwrap();
        for value in 0..513 {
            stream.push(&value).unwrap();
        }
        let mut sealed = stream;
        let mut left = sealed.cursor().unwrap();
        let mut right = sealed.cursor().unwrap();
        for value in 0..513 {
            assert_eq!(left.next_frame().unwrap(), Some(value));
            assert_eq!(right.next_frame().unwrap(), Some(value));
        }
        assert_eq!(left.next_frame().unwrap(), None);
        assert_eq!(left.next_frame().unwrap(), None);
        let mut cursor = sealed.cursor().unwrap();
        sealed.file.as_file_mut().seek(SeekFrom::Start(40)).unwrap();
        sealed.file.as_file_mut().write_all(&[0]).unwrap();
        assert!(cursor.next_frame().is_err());
        assert!(sealed.cursor().unwrap().next_frame().is_err());
    }
    #[test]
    fn framed_stream_is_repeatable_and_rejects_truncated_or_changed_payloads() {
        let mut stream = BuildFrameStream::<String>::new().unwrap();
        stream.push(&"Unicode 🐉".into()).unwrap();
        stream.push(&"second".into()).unwrap();
        for _ in 0..2 {
            let mut rows = Vec::new();
            stream
                .visit(&mut |row| {
                    rows.push(row);
                    Ok(())
                })
                .unwrap();
            assert_eq!(rows, ["Unicode 🐉", "second"]);
        }
        let length = stream.file.as_file().metadata().unwrap().len();
        stream.file.as_file_mut().set_len(length - 1).unwrap();
        assert!(stream.visit(&mut |_| Ok(())).is_err());
        let mut changed = BuildFrameStream::<String>::new().unwrap();
        changed.push(&"first".into()).unwrap();
        changed
            .file
            .as_file_mut()
            .seek(SeekFrom::Start(40))
            .unwrap();
        changed.file.as_file_mut().write_all(&[0]).unwrap();
        assert!(changed.visit(&mut |_| Ok(())).is_err());
    }
}

#[cfg(test)]
mod bound_tests {
    use super::*;
    #[test]
    fn damaged_lengths_cannot_allocate_or_expand_beyond_observed_record_sizes() {
        for declared in [u64::MAX, 1] {
            let mut stream = BuildFrameStream::<String>::new().unwrap();
            for _ in 0..32 {
                stream.push(&"one bounded record".into()).unwrap();
            }
            stream.file.as_file_mut().seek(SeekFrom::Start(0)).unwrap();
            stream
                .file
                .as_file_mut()
                .write_all(&declared.to_le_bytes())
                .unwrap();
            let error = stream.visit(&mut |_| Ok(())).unwrap_err().to_string();
            assert!(
                error.contains("observed record bounds")
                    || error.contains("frame checksum differs"),
                "{error}"
            );
        }
    }
}

#[cfg(test)]
mod file_usage_tests {
    use super::*;
    #[test]
    fn file_usage_tracks_complete_stream_files() {
        let mut stream = BuildFrameStream::<String>::new().unwrap();
        assert_eq!(stream.file_usage().unwrap().file_bytes, 0);
        stream.push(&"first frame".into()).unwrap();
        let first = stream.file_usage().unwrap();
        stream.push(&"second frame".into()).unwrap();
        let next = stream.file_usage().unwrap();
        assert!(next.file_bytes > first.file_bytes);
        assert_eq!(stream.file_usage().unwrap(), next);
        assert_eq!(
            first.combine(next).unwrap().file_bytes,
            first.file_bytes + next.file_bytes
        );
        #[cfg(unix)]
        assert!(next.allocated_bytes.is_some());
    }
}
