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
    kind: PhantomData<T>,
}
impl<T: Serialize + DeserializeOwned> BuildFrameStream<T> {
    pub fn new() -> Result<Self, IndexError> {
        Ok(Self {
            file: tempfile::NamedTempFile::new().map_err(io)?,
            count: 0,
            max_encoded: 0,
            max_decoded: 0,
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
        let decoded = serde_json::to_vec(value)?;
        let payload = crate::codec::gzip(&decoded)?;
        let file = self.file.as_file_mut();
        file.seek(SeekFrom::End(0)).map_err(io)?;
        file.write_all(&(payload.len() as u64).to_le_bytes())
            .map_err(io)?;
        file.write_all(&(decoded.len() as u64).to_le_bytes())
            .map_err(io)?;
        file.write_all(&Sha256::digest(&payload)).map_err(io)?;
        file.write_all(&payload).map_err(io)?;
        self.max_encoded = self.max_encoded.max(payload.len() as u64);
        self.max_decoded = self.max_decoded.max(decoded.len() as u64);
        self.count = self
            .count
            .checked_add(1)
            .ok_or_else(|| io("frame count overflow"))?;
        Ok(())
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
    fn cursor(&self) -> Result<BuildFrameCursor<T>, IndexError> {
        let file = File::open(self.file.path()).map_err(io)?;
        let length = file.metadata().map_err(io)?.len();
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
    pub fn digest(&self) -> Result<String, IndexError> {
        let mut file = File::open(self.file.path()).map_err(io)?;
        let mut hash = Sha256::new();
        let mut buffer = [0; 65536];
        loop {
            let n = file.read(&mut buffer).map_err(io)?;
            if n == 0 {
                break;
            }
            hash.update(&buffer[..n]);
        }
        Ok(format!(
            "{}:{}:{}:{:x}",
            self.count,
            self.max_encoded,
            self.max_decoded,
            hash.finalize()
        ))
    }
}

/// Independent forward reader for a sealed, bounded frame stream.
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
            let mut size = [0; 8];
            self.file.read_exact(&mut size).map_err(io)?;
            let size = u64::from_le_bytes(size);
            let mut decoded = [0; 8];
            self.file.read_exact(&mut decoded).map_err(io)?;
            let decoded = u64::from_le_bytes(decoded);
            if size > self.max_encoded || decoded > self.max_decoded {
                return Err(io("frame exceeds observed record bounds"));
            }
            let remaining = self
                .length
                .checked_sub(self.file.stream_position().map_err(io)?)
                .ok_or_else(|| io("frame offset overflow"))?;
            if remaining < 32 || size > remaining - 32 {
                return Err(io("truncated frame"));
            }
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
            let mut expected = [0; 32];
            self.file.read_exact(&mut expected).map_err(io)?;
            let mut payload = vec![0; usize::try_from(size).map_err(io)?];
            self.file.read_exact(&mut payload).map_err(io)?;
            if Sha256::digest(&payload).as_slice() != expected {
                return Err(io("frame checksum differs"));
            }
            let limit = decoded
                .checked_add(1)
                .ok_or_else(|| io("decoded frame size overflow"))?;
            let mut plain = Vec::new();
            flate2::read::GzDecoder::new(payload.as_slice())
                .take(limit)
                .read_to_end(&mut plain)
                .map_err(io)?;
            if plain.len() as u64 != decoded {
                return Err(io("decoded frame length differs"));
            }
            return Ok(Some(serde_json::from_slice(&plain)?));
        }
        if self.ordinal != self.count || self.file.metadata().map_err(io)?.len() != self.length {
            return Err(io("frame count or length differs"));
        }
        Ok(None)
    }
}

/// Immutable, context-bound metadata used across inference phases. The seal is
/// held outside the file, so truncated/replaced bytes cannot supply authority.
pub struct SealedBuildFrameStream<T> {
    stream: BuildFrameStream<T>,
    binding: String,
    digest: String,
}
impl<T: Serialize + DeserializeOwned> BuildFrameStream<T> {
    pub fn seal(self, binding: &impl Serialize) -> Result<SealedBuildFrameStream<T>, IndexError> {
        let digest = self.digest()?;
        Ok(SealedBuildFrameStream {
            stream: self,
            binding: crate::codec::sha256(&serde_json::to_vec(binding)?),
            digest,
        })
    }
}
impl<T: Serialize + DeserializeOwned> SealedBuildFrameStream<T> {
    pub fn len(&self) -> usize {
        self.stream.len()
    }
    pub fn file_usage(&self) -> Result<BuildFileUsage, IndexError> {
        self.stream.file_usage()
    }
    pub fn is_empty(&self) -> bool {
        self.stream.is_empty()
    }
    pub fn check_binding(&self, binding: &impl Serialize) -> Result<(), IndexError> {
        if self.binding != crate::codec::sha256(&serde_json::to_vec(binding)?)
            || self.digest != self.stream.digest()?
        {
            return Err(io("sealed stream binding or bytes differ"));
        }
        Ok(())
    }
    pub fn cursor(&self) -> Result<BuildFrameCursor<T>, IndexError> {
        if self.digest != self.stream.digest()? {
            return Err(io("sealed stream bytes differ"));
        }
        self.stream.cursor()
    }
    pub fn visit(
        &self,
        visitor: &mut dyn FnMut(T) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        if self.digest != self.stream.digest()? {
            return Err(io("sealed stream bytes differ"));
        }
        self.stream.visit(visitor)
    }
}

#[derive(Serialize, Deserialize)]
pub struct PreparedRecordFrame {
    pub pack: String,
    pub snapshot: Vec<u8>,
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
        let source = atlas_foundry_model::decode_snapshot(&self.snapshot)?;
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
            self.snapshot,
            self.selection,
            self.lexical,
        ))
    }
}

#[derive(Serialize, Deserialize)]
struct SpoolSeal {
    version: u32,
    workspace: String,
    revision: u64,
    context: String,
    source: String,
    records: usize,
    semantic: usize,
    streams: Vec<String>,
    packs: Vec<IndexBuildPack>,
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
    fn seal_data(&self) -> Result<SpoolSeal, IndexError> {
        Ok(SpoolSeal {
            version: 1,
            workspace: self.expected.workspace_key().into(),
            revision: self.expected.reference_revision(),
            context: self.context.sha256()?,
            source: self.context.source_fingerprint.clone(),
            records: self.records.len(),
            semantic: self.semantic.len(),
            packs: self.packs.clone(),
            streams: vec![
                self.records.digest()?,
                self.lexical.digest()?,
                self.semantic.digest()?,
                self.aliases.digest()?,
                self.remaster_pairs.digest()?,
                self.traits.digest()?,
                self.trait_seeds.digest()?,
            ],
        })
    }
    pub fn seal(self) -> Result<SealedPublicationSpool, IndexError> {
        let seal = serde_json::to_vec(&self.seal_data()?)?;
        Ok(SealedPublicationSpool { spool: self, seal })
    }
}

pub struct SealedPublicationSpool {
    spool: PublicationSpool,
    seal: Vec<u8>,
}
impl SealedPublicationSpool {
    pub fn file_usage(&self) -> Result<BuildFileUsage, IndexError> {
        self.spool.file_usage()
    }
}
impl PublicationSource for SealedPublicationSpool {
    fn context(&self) -> &SourceArtifactBuildContext {
        &self.spool.context
    }
    fn packs(&self) -> &[IndexBuildPack] {
        &self.spool.packs
    }
    fn inventory(&self) -> &CompleteSourceInventory {
        &self.spool.inventory
    }
    fn record_count(&self) -> usize {
        self.spool.records.len()
    }
    fn semantic_count(&self) -> usize {
        self.spool.semantic.len()
    }
    fn expected_instance(&self) -> Option<&ArtifactInstance> {
        Some(&self.spool.expected)
    }
    fn check_seal(&self) -> Result<(), IndexError> {
        if serde_json::to_vec(&self.spool.seal_data()?)? != self.seal {
            return Err(io("sealed stream or context changed"));
        }
        Ok(())
    }
    fn records(
        &self,
        visitor: &mut dyn FnMut(PreparedRecordRef<'_>) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        self.spool.records.visit(&mut |frame| {
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
        self.spool.lexical.visit(&mut |row| v(&row))
    }
    fn semantic(
        &self,
        v: &mut dyn FnMut(&SemanticOccurrence) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        self.spool.semantic.visit(&mut |row| v(&row))
    }
    fn aliases(
        &self,
        v: &mut dyn FnMut(&SourceAliasInput) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        self.spool.aliases.visit(&mut |row| v(&row))
    }
    fn remaster_pairs(
        &self,
        v: &mut dyn FnMut(&SourceRemasterPairInput) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        self.spool.remaster_pairs.visit(&mut |row| v(&row))
    }
    fn traits(
        &self,
        v: &mut dyn FnMut(&TraitDefinition) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        self.spool.traits.visit(&mut |row| v(&row))
    }
    fn trait_seeds(
        &self,
        v: &mut dyn FnMut(&TraitSeedOccurrence) -> Result<(), IndexError>,
    ) -> Result<(), IndexError> {
        self.spool.trait_seeds.visit(&mut |row| v(&row))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sealed_forward_cursors_are_independent_and_reject_changed_frames() {
        let mut stream = BuildFrameStream::<usize>::new().unwrap();
        for value in 0..513 {
            stream.push(&value).unwrap();
        }
        let mut sealed = stream.seal(&"binding").unwrap();
        let mut left = sealed.cursor().unwrap();
        let mut right = sealed.cursor().unwrap();
        for value in 0..513 {
            assert_eq!(left.next_frame().unwrap(), Some(value));
            assert_eq!(right.next_frame().unwrap(), Some(value));
        }
        assert_eq!(left.next_frame().unwrap(), None);
        assert_eq!(left.next_frame().unwrap(), None);
        let mut cursor = sealed.cursor().unwrap();
        sealed
            .stream
            .file
            .as_file_mut()
            .seek(SeekFrom::Start(48))
            .unwrap();
        sealed.stream.file.as_file_mut().write_all(&[0]).unwrap();
        assert!(cursor.next_frame().is_err());
        assert!(sealed.cursor().is_err());
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
            .seek(SeekFrom::Start(48))
            .unwrap();
        changed.file.as_file_mut().write_all(&[0]).unwrap();
        assert!(changed.visit(&mut |_| Ok(())).is_err());
    }
    #[test]
    fn publication_seal_binds_context_revision_inventory_and_every_stream() {
        let input = crate::foundation_tests::fixture(false);
        let expected = ArtifactInstance::new("workspace".into(), 7);
        let builder = PublicationSpool::new(
            input.context.clone(),
            input.packs,
            input.inventory,
            expected,
        )
        .unwrap();
        let mut sealed = builder.seal().unwrap();
        sealed.check_seal().unwrap();
        sealed.spool.context.indexing_locale = "fr".into();
        assert!(sealed.check_seal().is_err());
        sealed.spool.context.indexing_locale = "en".into();
        sealed.check_seal().unwrap();
        sealed.spool.expected = ArtifactInstance::new("workspace".into(), 8);
        assert!(sealed.check_seal().is_err());
        sealed.spool.expected = ArtifactInstance::new("workspace".into(), 7);
        sealed.check_seal().unwrap();
        sealed
            .spool
            .aliases
            .push(&SourceAliasInput {
                record: input.records[0].record.key().clone(),
                alias: "changed".into(),
                evidence: "changed".into(),
            })
            .unwrap();
        assert!(sealed.check_seal().is_err());
    }
}

#[cfg(test)]
mod bound_tests {
    use super::*;
    #[test]
    fn damaged_lengths_cannot_allocate_or_expand_beyond_observed_record_sizes() {
        for (offset, declared) in [(0, u64::MAX), (8, u64::MAX), (8, 1)] {
            let mut stream = BuildFrameStream::<String>::new().unwrap();
            for _ in 0..32 {
                stream.push(&"one bounded record".into()).unwrap();
            }
            stream
                .file
                .as_file_mut()
                .seek(SeekFrom::Start(offset))
                .unwrap();
            stream
                .file
                .as_file_mut()
                .write_all(&declared.to_le_bytes())
                .unwrap();
            let error = stream.visit(&mut |_| Ok(())).unwrap_err().to_string();
            assert!(
                error.contains("observed record bounds") || error.contains("decoded frame length"),
                "{error}"
            );
        }
    }
}

#[cfg(test)]
mod file_usage_tests {
    use super::*;
    #[test]
    fn file_usage_tracks_complete_stream_files_and_survives_sealing() {
        let mut stream = BuildFrameStream::<String>::new().unwrap();
        assert_eq!(stream.file_usage().unwrap().file_bytes, 0);
        stream.push(&"first frame".into()).unwrap();
        let first = stream.file_usage().unwrap();
        stream.push(&"second frame".into()).unwrap();
        let next = stream.file_usage().unwrap();
        assert!(next.file_bytes > first.file_bytes);
        let sealed = stream.seal(&"binding").unwrap();
        assert_eq!(sealed.file_usage().unwrap(), next);
        assert_eq!(
            first.combine(next).unwrap().file_bytes,
            first.file_bytes + next.file_bytes
        );
        #[cfg(unix)]
        assert!(next.allocated_bytes.is_some());
    }
}
