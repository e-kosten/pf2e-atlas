//! Versioned, data-only corpus embeddings shared by the exporter and setup.
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::time::{Duration, Instant};

use atlas_embedding::NativeInputDescriptor;
use flate2::{Compression, bufread::GzDecoder, write::GzEncoder};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::native_values::{NativeInput, NativeModel, NativeModelKey, expected_bytes};
use crate::{IndexError, SourceSearchProfileIdentity, SqliteIndexReader};

const MAGIC: &[u8; 8] = b"ATLSEED1";
const MAX_HEADER: usize = 32 * 1024;
const MAX_ROWS: usize = 512 * 1024;
const MAX_VECTOR_BYTES: usize = 4096;
pub const MAX_EMBEDDING_SEED_BYTES: u64 = 1024 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmbeddingSeedFile {
    pub sha256: String,
    pub bytes: u64,
}

impl EmbeddingSeedFile {
    pub fn validate(&self) -> Result<(), IndexError> {
        NativeInput::decode_hash(&self.sha256)?;
        if !(1..=MAX_EMBEDDING_SEED_BYTES).contains(&self.bytes) {
            return Err(invalid("compressed size is out of bounds"));
        }
        Ok(())
    }

    pub fn verify(&self, file: &File) -> Result<(), IndexError> {
        self.verify_inner(file, None)
    }

    fn verify_inner(
        &self,
        file: &File,
        mut progress: Option<&mut SeedProgress>,
    ) -> Result<(), IndexError> {
        self.validate()?;
        let measured = Self::measure_inner(file, progress.as_deref_mut())?;
        if measured.bytes != self.bytes || measured.sha256 != self.sha256 {
            return Err(invalid(
                "compressed size or SHA-256 does not match the release pin",
            ));
        }
        if let Some(progress) = progress {
            progress.report(self.bytes);
        }
        Ok(())
    }

    pub fn measure(file: &File) -> Result<Self, IndexError> {
        Self::measure_inner(file, None)
    }

    fn measure_inner(
        mut file: &File,
        mut progress: Option<&mut SeedProgress>,
    ) -> Result<Self, IndexError> {
        file.rewind()?;
        let mut digest = Sha256::new();
        let mut input = file.take(MAX_EMBEDDING_SEED_BYTES + 1);
        let mut buffer = [0; 64 * 1024];
        let mut bytes = 0;
        loop {
            let count = match input.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => count,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error.into()),
            };
            digest.update(&buffer[..count]);
            bytes += count as u64;
            if let Some(progress) = &mut progress {
                progress.advance(bytes);
            }
        }
        let measured = Self {
            sha256: format!("{:x}", digest.finalize()),
            bytes,
        };
        measured.validate()?;
        Ok(measured)
    }
}

struct SeedProgress {
    phase: &'static str,
    message: String,
    total: u64,
    unit: &'static str,
    last_report: Instant,
}

impl SeedProgress {
    fn start(phase: &'static str, message: String, total: u64, unit: &'static str) -> Self {
        let mut progress = Self {
            phase,
            message,
            total,
            unit,
            last_report: Instant::now(),
        };
        progress.report(0);
        progress
    }

    fn advance(&mut self, current: u64) {
        if current < self.total && self.last_report.elapsed() >= Duration::from_millis(250) {
            self.report(current);
        }
    }

    fn report(&mut self, current: u64) {
        tracing::info!(
            target: "atlas_progress",
            phase = self.phase,
            message = self.message.as_str(),
            current,
            total = self.total,
            unit = self.unit
        );
        self.last_report = Instant::now();
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmbeddingSeedHeader {
    pub producer_commit: String,
    pub source_revision: String,
    pub source_fingerprint: String,
    pub reference_inputs_sha256: String,
    pub locale: String,
    pub model: SourceSearchProfileIdentity,
    pub rows: usize,
}

impl EmbeddingSeedHeader {
    fn validate(&self) -> Result<usize, IndexError> {
        if !(1..=MAX_ROWS).contains(&self.rows)
            || self.locale != "en"
            || !git_commit(&self.producer_commit)
            || !git_commit(&self.source_revision)
        {
            return Err(invalid("invalid producer, source or row count"));
        }
        NativeInput::decode_hash(&self.source_fingerprint)?;
        NativeInput::decode_hash(&self.reference_inputs_sha256)?;
        let vector_bytes = expected_bytes(&self.model)?;
        if vector_bytes > MAX_VECTOR_BYTES {
            return Err(invalid("vector size is out of bounds"));
        }
        Ok(vector_bytes)
    }
}

fn git_commit(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub struct EmbeddingSeed {
    model: SourceSearchProfileIdentity,
    vectors: File,
    keys: Vec<([u8; 32], usize)>,
    vector_bytes: usize,
}

/// Authenticated vectors selected for exact inputs from a particular seed.
pub struct SeedSelection {
    pub(crate) model: SourceSearchProfileIdentity,
    pub(crate) values: Vec<(NativeInput, Vec<u8>)>,
}

impl EmbeddingSeed {
    /// Authenticate the compressed bytes and validate the full payload against
    /// its declared execution contract before allowing lookups.
    pub fn read(
        file: File,
        expected: &EmbeddingSeedFile,
        scratch: &Path,
    ) -> Result<Self, IndexError> {
        Self::read_inner(file, expected, scratch, None)
    }

    pub fn read_with_progress(
        file: File,
        expected: &EmbeddingSeedFile,
        scratch: &Path,
        profile_label: &str,
    ) -> Result<Self, IndexError> {
        Self::read_inner(file, expected, scratch, Some(profile_label))
    }

    fn read_inner(
        mut file: File,
        expected: &EmbeddingSeedFile,
        scratch: &Path,
        profile_label: Option<&str>,
    ) -> Result<Self, IndexError> {
        let mut checksum_progress = profile_label.map(|label| {
            SeedProgress::start(
                "verify_embedding_seed",
                format!("Checking the {label} download"),
                expected.bytes,
                "bytes",
            )
        });
        expected.verify_inner(&file, checksum_progress.as_mut())?;
        file.rewind()?;
        let mut input = GzDecoder::new(BufReader::new(file));
        if &read_array::<8>(&mut input)? != MAGIC {
            return Err(invalid("unknown format version"));
        }
        let header_len = u32::from_le_bytes(read_array(&mut input)?) as usize;
        if !(1..=MAX_HEADER).contains(&header_len) {
            return Err(invalid("header size is out of bounds"));
        }
        let mut header_bytes = vec![0; header_len];
        input.read_exact(&mut header_bytes)?;
        let header: EmbeddingSeedHeader = serde_json::from_slice(&header_bytes)?;
        let vector_bytes = header.validate()?;
        let model = header.model;
        let mut keys = Vec::with_capacity(header.rows);
        let mut output = BufWriter::new(tempfile::tempfile_in(scratch)?);
        let mut bytes = vec![0; vector_bytes];
        let mut unpack_progress = profile_label.map(|label| {
            SeedProgress::start(
                "unpack_embedding_seed",
                format!("Unpacking and checking {label} embeddings"),
                header.rows as u64,
                "embeddings",
            )
        });
        for _ in 0..header.rows {
            let hash = read_array(&mut input)?;
            let token_count = u32::from_le_bytes(read_array(&mut input)?) as usize;
            NativeInput::from_hash(&model, hash, token_count)?;
            if keys.last().is_some_and(|(prior, _)| prior >= &hash) {
                return Err(invalid("input hashes are duplicated or out of order"));
            }
            input.read_exact(&mut bytes)?;
            let vector = crate::read_units::decode_vector(&bytes)?;
            crate::validation::check_vector(&vector, model.dimensions)?;
            output.write_all(&bytes)?;
            keys.push((hash, token_count));
            if let Some(progress) = &mut unpack_progress {
                progress.advance(keys.len() as u64);
            }
        }
        if input.read(&mut [0])? != 0 || input.into_inner().read(&mut [0])? != 0 {
            return Err(invalid("unexpected data after the final row"));
        }
        output.flush()?;
        if let Some(progress) = &mut unpack_progress {
            progress.report(keys.len() as u64);
        }
        let vectors = output.into_parts().0;
        Ok(Self {
            model,
            vectors,
            keys,
            vector_bytes,
        })
    }

    pub fn model(&self) -> &SourceSearchProfileIdentity {
        &self.model
    }

    pub fn try_bind(&mut self, model: &SourceSearchProfileIdentity) -> bool {
        if !model.vector_compatible_with(&self.model) {
            return false;
        }
        self.model = model.clone();
        true
    }

    pub fn validate_delta(&self, delta: &Self) -> Result<(), IndexError> {
        if !self.model.vector_compatible_with(&delta.model) {
            return Err(invalid("delta model identity differs from its base"));
        }
        for (hash, _) in &delta.keys {
            if self
                .keys
                .binary_search_by_key(hash, |(key, _)| *key)
                .is_ok()
            {
                return Err(invalid("base and delta contain the same input hash"));
            }
        }
        Ok(())
    }

    pub fn count_uncovered(&self, index: &SqliteIndexReader) -> Result<usize, IndexError> {
        let model = index.context().search_profile(&self.model.profile_id)?;
        if !model.vector_compatible_with(&self.model) {
            return Err(invalid("export model identity differs from the base"));
        }
        let db = index.workspace_session().connection();
        let mut inputs = db.prepare(SEED_INPUTS)?;
        let mut rows = inputs.query([model.profile_id.as_str()])?;
        let mut missing = 0;
        while let Some(row) = rows.next()? {
            let input = NativeInput::from_hash(model, row.get(0)?, row.get(1)?)?;
            missing += usize::from(self.ordinal(&input)?.is_none());
        }
        Ok(missing)
    }

    /// Collect missing positions and validate every matching input in this window.
    pub fn partition(
        &mut self,
        inputs: &[NativeInputDescriptor],
    ) -> Result<(SeedSelection, Vec<usize>), IndexError> {
        if inputs.len() > crate::NATIVE_VALUE_BATCH_LIMIT {
            return Err(invalid("selection batch exceeds the native value limit"));
        }
        let mut values = Vec::with_capacity(inputs.len());
        let mut missing = Vec::new();
        for (position, descriptor) in inputs.iter().enumerate() {
            let input = NativeInput::new(&self.model, descriptor)?;
            if let Some(bytes) = self.lookup(&input)? {
                values.push((input, bytes));
            } else {
                missing.push(position);
            }
        }
        Ok((
            SeedSelection {
                model: self.model.clone(),
                values,
            },
            missing,
        ))
    }

    fn ordinal(&self, input: &NativeInput) -> Result<Option<usize>, IndexError> {
        let Ok(ordinal) = self
            .keys
            .binary_search_by_key(input.hash(), |(key, _)| *key)
        else {
            return Ok(None);
        };
        if self.keys[ordinal].1 != input.token_count() {
            return Err(invalid("native token count differs for a matching input"));
        }
        Ok(Some(ordinal))
    }

    fn lookup(&mut self, input: &NativeInput) -> Result<Option<Vec<u8>>, IndexError> {
        let Some(ordinal) = self.ordinal(input)? else {
            return Ok(None);
        };
        self.vectors
            .seek(SeekFrom::Start(ordinal as u64 * self.vector_bytes as u64))?;
        let mut bytes = vec![0; self.vector_bytes];
        self.vectors.read_exact(&mut bytes)?;
        Ok(Some(bytes))
    }
}

impl SeedSelection {
    pub fn extend(&mut self, selection: Self) -> Result<(), IndexError> {
        if self.model != selection.model {
            return Err(invalid(
                "cannot combine selections from different model identities",
            ));
        }
        self.values.extend(selection.values);
        Ok(())
    }
}

const SEED_INPUTS: &str = "SELECT input_hash, token_count, input_id FROM embedding_inputs
     WHERE model_id=(SELECT model_id FROM search_profiles WHERE profile_id=?)
       AND published_hash IS NOT NULL ORDER BY input_hash";

/// Export published native values only, under the index's admitted read snapshot.
pub fn export_embedding_seed(
    index: &SqliteIndexReader,
    model: &SourceSearchProfileIdentity,
    producer_commit: String,
    output: impl Write,
) -> Result<EmbeddingSeedHeader, IndexError> {
    export_seed(index, model, producer_commit, None, output)
}

pub fn export_embedding_seed_delta(
    index: &SqliteIndexReader,
    model: &SourceSearchProfileIdentity,
    producer_commit: String,
    base: &EmbeddingSeed,
    output: impl Write,
) -> Result<EmbeddingSeedHeader, IndexError> {
    export_seed(index, model, producer_commit, Some(base), output)
}

fn export_seed(
    index: &SqliteIndexReader,
    model: &SourceSearchProfileIdentity,
    producer_commit: String,
    base: Option<&EmbeddingSeed>,
    output: impl Write,
) -> Result<EmbeddingSeedHeader, IndexError> {
    let context = index.context();
    let source_revision = context
        .source_revision
        .as_ref()
        .filter(|revision| {
            revision.as_str() == atlas_foundry_model::generated::REFERENCE_SOURCE_COMMIT
        })
        .ok_or_else(|| invalid("export requires the canonical pinned source revision"))?;
    if context.search_profile(&model.profile_id)? != model {
        return Err(invalid("export requires the exact published profile"));
    }
    let db = index.workspace_session().connection();
    let model_key = NativeModelKey::new(0, model)?;
    let binding = NativeModel::bind(db, &model_key, false)?;
    let rows = if let Some(base) = base {
        if base.model().profile_id != model.profile_id {
            return Err(invalid("export model identity differs from the base"));
        }
        base.count_uncovered(index)?
    } else {
        db.query_row(
            "SELECT count(*) FROM embedding_inputs
             WHERE model_id=(SELECT model_id FROM search_profiles WHERE profile_id=?)
               AND published_hash IS NOT NULL",
            [model.profile_id.as_str()],
            |row| row.get(0),
        )?
    };
    let header = EmbeddingSeedHeader {
        producer_commit,
        source_revision: source_revision.clone(),
        source_fingerprint: context.source_fingerprint.clone(),
        reference_inputs_sha256: context
            .reference_preparation
            .as_ref()
            .filter(|preparation| preparation.coverage == crate::ReferenceCoverage::Complete)
            .ok_or_else(|| invalid("export requires complete reference preparation"))?
            .input_hash
            .clone(),
        locale: context.indexing_locale.clone(),
        model: model.clone(),
        rows,
    };
    header.validate()?;
    let metadata = serde_json::to_vec(&header)?;
    if metadata.len() > MAX_HEADER {
        return Err(invalid("export header is too large"));
    }
    let mut output = GzEncoder::new(output, Compression::fast());
    output.write_all(MAGIC)?;
    output.write_all(&(metadata.len() as u32).to_le_bytes())?;
    output.write_all(&metadata)?;
    let mut statement = db.prepare(SEED_INPUTS)?;
    let mut rows = statement.query([model.profile_id.as_str()])?;
    while let Some(row) = rows.next()? {
        let hash = row.get::<_, [u8; 32]>(0)?;
        let input = NativeInput::from_hash(model, hash, row.get(1)?)?;
        if let Some(base) = base
            && base.ordinal(&input)?.is_some()
        {
            continue;
        }
        let vector = binding.read(row.get(2)?)?;
        output.write_all(input.hash())?;
        output.write_all(&(input.token_count() as u32).to_le_bytes())?;
        output.write_all(&crate::read_units::vector_bytes(&vector))?;
    }
    output.finish()?.flush()?;
    Ok(header)
}

fn read_array<const N: usize>(input: &mut impl Read) -> io::Result<[u8; N]> {
    let mut bytes = [0; N];
    input.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn invalid(message: &str) -> IndexError {
    IndexError::Operation(format!("embedding seed {message}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model() -> SourceSearchProfileIdentity {
        SourceSearchProfileIdentity::from_spec(atlas_embedding::profile_for_model(
            atlas_embedding::EmbeddingModelId::BgeBaseEnV15,
        ))
    }

    fn asset(
        model: SourceSearchProfileIdentity,
        rows: usize,
        keys: &[[u8; 32]],
        vector: &[f32],
        tail: &[u8],
    ) -> (File, EmbeddingSeedFile) {
        let header = EmbeddingSeedHeader {
            producer_commit: "a".repeat(40),
            source_revision: "b".repeat(40),
            source_fingerprint: "c".repeat(64),
            reference_inputs_sha256: "d".repeat(64),
            locale: "en".into(),
            model,
            rows,
        };
        let metadata = serde_json::to_vec(&header).unwrap();
        let mut file = tempfile::tempfile().unwrap();
        let mut output = GzEncoder::new(&mut file, Compression::fast());
        output.write_all(MAGIC).unwrap();
        output
            .write_all(&(metadata.len() as u32).to_le_bytes())
            .unwrap();
        output.write_all(&metadata).unwrap();
        for key in keys {
            output.write_all(key).unwrap();
            output.write_all(&12_u32.to_le_bytes()).unwrap();
            output
                .write_all(&crate::read_units::vector_bytes(vector))
                .unwrap();
        }
        output.finish().unwrap().write_all(tail).unwrap();
        let pin = EmbeddingSeedFile::measure(&file).unwrap();
        (file, pin)
    }

    fn vector(model: &SourceSearchProfileIdentity) -> Vec<f32> {
        let mut vector = vec![0.; model.dimensions];
        vector[0] = 1.;
        vector
    }

    #[test]
    fn reports_pinned_bytes_then_validated_embeddings() {
        #[derive(Clone, Default)]
        struct Log(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);
        impl Write for Log {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                self.0.lock().unwrap().write(bytes)
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let log = Log::default();
        let writer = log.clone();
        let subscriber = tracing_subscriber::fmt()
            .without_time()
            .with_ansi(false)
            .with_writer(move || writer.clone())
            .finish();
        let model = model();
        let vector = vector(&model);
        let (file, pin) = asset(model, 2, &[[1; 32], [2; 32]], &vector, &[]);
        let scratch = tempfile::tempdir().unwrap();
        tracing::subscriber::with_default(subscriber, || {
            EmbeddingSeed::read_with_progress(file, &pin, scratch.path(), "BGE Base").unwrap();
        });

        let output = log.0.lock().unwrap();
        let output = std::str::from_utf8(&output).unwrap();
        let lines = output.lines().collect::<Vec<_>>();
        let unpack_start = lines
            .iter()
            .position(|line| line.contains("Unpacking and checking BGE Base embeddings"))
            .unwrap();
        assert!(unpack_start >= 2 && lines.len() - unpack_start >= 2);
        for (line, label, current, total, unit) in [
            (
                lines[0],
                "Checking the BGE Base download",
                0,
                pin.bytes,
                "bytes",
            ),
            (
                lines[unpack_start - 1],
                "Checking the BGE Base download",
                pin.bytes,
                pin.bytes,
                "bytes",
            ),
            (
                lines[unpack_start],
                "Unpacking and checking BGE Base embeddings",
                0,
                2,
                "embeddings",
            ),
            (
                lines[lines.len() - 1],
                "Unpacking and checking BGE Base embeddings",
                2,
                2,
                "embeddings",
            ),
        ] {
            assert!(line.contains(label), "{line}");
            assert!(line.contains(&format!("current={current} ")), "{line}");
            assert!(line.contains(&format!("total={total} ")), "{line}");
            assert!(line.contains(&format!("unit=\"{unit}\"")), "{line}");
        }
    }

    #[test]
    fn admitted_seed_matches_the_exact_input_and_token_count() {
        let model = model();
        let vector = vector(&model);
        let (file, pin) = asset(model.clone(), 2, &[[1; 32], [2; 32]], &vector, &[]);
        let directory = tempfile::tempdir().unwrap();
        let mut seed = EmbeddingSeed::read(file, &pin, directory.path()).unwrap();
        assert!(seed.try_bind(&model));
        let mut input = NativeInputDescriptor {
            input_sha256: "02".repeat(32),
            token_count: 12,
        };
        let (selected, missing) = seed.partition(&[input.clone()]).unwrap();
        assert!(missing.is_empty());
        assert_eq!(
            selected.values[0].1,
            crate::read_units::vector_bytes(&vector)
        );
        input.token_count -= 1;
        assert!(seed.partition(&[input]).is_err());
    }

    #[test]
    fn partition_reports_absent_rows_and_checks_present_rows_after_a_miss() {
        let model = model();
        let vector = vector(&model);
        let (file, pin) = asset(model.clone(), 2, &[[1; 32], [2; 32]], &vector, &[]);
        let directory = tempfile::tempdir().unwrap();
        let mut seed = EmbeddingSeed::read(file, &pin, directory.path()).unwrap();
        assert!(seed.try_bind(&model));
        let mut descriptors = [
            NativeInputDescriptor {
                input_sha256: "03".repeat(32),
                token_count: 12,
            },
            NativeInputDescriptor {
                input_sha256: "02".repeat(32),
                token_count: 12,
            },
        ];
        let (matched, missing) = seed.partition(&descriptors).unwrap();
        assert_eq!(missing, vec![0]);
        assert_eq!(matched.values.len(), 1);
        assert_eq!(
            matched.values[0].1,
            crate::read_units::vector_bytes(&vector)
        );
        descriptors[1].token_count = 13;
        assert!(seed.partition(&descriptors).is_err());
    }

    #[test]
    fn cumulative_delta_requires_disjoint_inputs_and_compatible_model() {
        let model = model();
        let vector = vector(&model);
        let scratch = tempfile::tempdir().unwrap();
        let read = |model, hash| {
            let (file, pin) = asset(model, 1, &[hash], &vector, &[]);
            EmbeddingSeed::read(file, &pin, scratch.path()).unwrap()
        };
        let base = read(model.clone(), [1; 32]);
        assert!(base.validate_delta(&read(model.clone(), [2; 32])).is_ok());
        assert!(base.validate_delta(&read(model.clone(), [1; 32])).is_err());
        let mut incompatible = model;
        incompatible.execution_policy.push_str("-other");
        assert!(base.validate_delta(&read(incompatible, [2; 32])).is_err());
    }

    #[test]
    fn malformed_or_incompatible_seed_fails_before_lookup() {
        let model = model();
        let vector = vector(&model);
        let directory = tempfile::tempdir().unwrap();
        let mut incompatible = model.clone();
        incompatible.execution_policy.push_str("-different");
        let mut invalid_vector = vector.clone();
        invalid_vector[0] = f32::NAN;
        for (file, pin) in [
            asset(model.clone(), 2, &[[1; 32]], &vector, &[]),
            asset(model.clone(), 2, &[[1; 32], [1; 32]], &vector, &[]),
            asset(model.clone(), MAX_ROWS + 1, &[], &vector, &[]),
            asset(model.clone(), 1, &[[1; 32]], &invalid_vector, &[]),
            asset(model.clone(), 1, &[[1; 32]], &vector, &[0]),
        ] {
            assert!(EmbeddingSeed::read(file, &pin, directory.path()).is_err());
        }
        let (file, pin) = asset(incompatible, 1, &[[1; 32]], &vector, &[]);
        let mut seed = EmbeddingSeed::read(file, &pin, directory.path()).unwrap();
        assert!(!seed.try_bind(&model));
        let (file, mut pin) = asset(model.clone(), 1, &[[1; 32]], &vector, &[]);
        pin.sha256 = "0".repeat(64);
        assert!(EmbeddingSeed::read(file, &pin, directory.path()).is_err());
    }
}
