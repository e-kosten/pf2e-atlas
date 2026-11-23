//! Workspace recovery composition. The CLI owns paths for archive files and confirmation.
use crate::workspace::{RELEASE_HISTORY, WORKSPACE_SCHEMA_VERSION, register_extensions};
use crate::{AtlasRuntime, StorageError, Workspace};
use atlas_domain::SearchProfileId;
use atlas_index::{
    RequestedReferenceConfig, clear_requested_reference_config, read_requested_reference_config,
    read_requested_reference_config_for_write, read_requested_reference_config_snapshot,
    validate_requested_reference_config, write_requested_reference_config,
};
use atlas_local_state::workspace_recovery::{SavedWorkspace, ValidatedWorkspace, check_empty};
use atlas_storage::{Connection, ReadSession, StorageResult};
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
#[cfg(unix)]
use std::fs::File;
use std::{
    io::{BufReader, BufWriter, Read, Seek, Write},
    path::{Path, PathBuf},
};

const FORMAT: &str = "pf2e-atlas-workspace";
const ACTIVE_ONLY_VERSION: &str = "1.1";
const ARCHIVED_LIST_VERSION: &str = "2.0";
const ADJUSTMENT_ARCHIVE_VERSION: &str = "3.0";
const CHARACTER_VERSION: &str = "4.0";
const FULL_PATHBUILDER_VERSION: &str = "5.0";
const ENCOUNTER_ARCHIVE_VERSION: &str = "7.0";
const ENCOUNTER_CREATION_VERSION: &str = "6.0";

#[derive(Serialize)]
pub struct WorkspaceArchive {
    format: String,
    version: String,
    saved: SavedWorkspace,
    reference_config: Option<ReferenceConfig>,
}

#[derive(Deserialize)]
struct ArchiveEnvelope {
    format: String,
    version: String,
    saved: Option<Box<RawValue>>,
    reference_config: Option<Box<RawValue>>,
}

#[derive(Serialize, Deserialize)]
struct ReferenceConfig {
    source_root: PathBuf,
    manifest_path: Option<PathBuf>,
    indexing_locale: String,
    search_profiles: Vec<String>,
}

impl From<RequestedReferenceConfig> for ReferenceConfig {
    fn from(config: RequestedReferenceConfig) -> Self {
        Self {
            source_root: config.source_root,
            manifest_path: config.manifest_path,
            indexing_locale: config.indexing_locale,
            search_profiles: config
                .search_profiles
                .into_iter()
                .map(|id| id.to_string())
                .collect(),
        }
    }
}

impl From<ReferenceConfig> for RequestedReferenceConfig {
    fn from(config: ReferenceConfig) -> Self {
        Self {
            source_root: config.source_root,
            manifest_path: config.manifest_path,
            indexing_locale: config.indexing_locale,
            search_profiles: config
                .search_profiles
                .into_iter()
                .map(SearchProfileId::new)
                .collect(),
        }
    }
}

pub struct ValidatedArchive {
    saved: ValidatedWorkspace,
    config: Option<RequestedReferenceConfig>,
}

pub struct RestorePreflight {
    workspace_key: String,
    config: Option<RequestedReferenceConfig>,
}

impl RestorePreflight {
    pub fn conflicts_with(&self, archive: &ValidatedArchive) -> bool {
        self.config.is_some() && self.config != archive.config
    }

    pub fn current_configuration(&self) -> Option<&RequestedReferenceConfig> {
        self.config.as_ref()
    }
}

impl ValidatedArchive {
    pub fn requested_configuration(&self) -> Option<&RequestedReferenceConfig> {
        self.config.as_ref()
    }
}

impl WorkspaceArchive {
    /// Capture through an authenticated SQLite snapshot. File publication is separate.
    pub fn capture(read: &ReadSession) -> Result<Self, String> {
        read.authenticate_history(RELEASE_HISTORY)
            .map_err(admission_error)?;
        Self::capture_snapshot(read.connection())
    }

    fn capture_snapshot(snapshot: &Connection) -> Result<Self, String> {
        let saved = SavedWorkspace::capture(snapshot).map_err(message)?;
        let reference_config = read_requested_reference_config_snapshot(snapshot)
            .map_err(message)?
            .map(ReferenceConfig::from);
        let version = if saved.has_current_encounter_archive_flags() {
            ENCOUNTER_ARCHIVE_VERSION
        } else if saved.has_encounter_creation_receipts() {
            ENCOUNTER_CREATION_VERSION
        } else if saved.has_full_pathbuilder_imports() {
            FULL_PATHBUILDER_VERSION
        } else if saved.has_characters() {
            CHARACTER_VERSION
        } else if saved.has_encounter_adjustments() {
            ADJUSTMENT_ARCHIVE_VERSION
        } else if saved.has_archived_lists() {
            ARCHIVED_LIST_VERSION
        } else {
            ACTIVE_ONLY_VERSION
        };
        Ok(Self {
            format: FORMAT.into(),
            version: version.into(),
            saved,
            reference_config,
        })
    }

    pub fn write(&self, mut output: impl Write) -> Result<(), String> {
        serde_json::to_writer(&mut output, self).map_err(message)?;
        output.write_all(b"\n").map_err(message)
    }

    pub fn export(&self, path: &Path) -> Result<(), String> {
        self.publish(path, false)
    }

    fn publish(&self, path: &Path, upgrade: bool) -> Result<(), String> {
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut staged = tempfile::NamedTempFile::new_in(parent)
            .map_err(|error| format!("Cannot stage {}: {error}", path.display()))?;
        {
            let mut output = BufWriter::new(staged.as_file_mut());
            self.write(&mut output)?;
            output.flush().map_err(message)?;
        }
        if upgrade {
            staged.as_file_mut().rewind().map_err(message)?;
            Self::read(BufReader::new(staged.as_file()))?;
        }
        staged.as_file().sync_all().map_err(message)?;
        if upgrade {
            staged.persist(path)
        } else {
            staged.persist_noclobber(path)
        }
        .map_err(|error| format!("Cannot publish {}: {}", path.display(), error.error))?;
        #[cfg(unix)]
        File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|error| {
                format!(
                    "Archive {} was published but directory sync failed: {error}",
                    path.display()
                )
            })?;
        Ok(())
    }

    pub fn read(input: impl Read) -> Result<ValidatedArchive, String> {
        let archive: ArchiveEnvelope = serde_json::from_reader(input).map_err(archive_error)?;
        if archive.format != FORMAT {
            return Err(format!(
                "Unsupported workspace archive format {}",
                archive.format
            ));
        }
        let major = validate_version(&archive.version)?;
        let saved = archive
            .saved
            .ok_or("Workspace archive is missing saved state")?;
        let saved: SavedWorkspace = serde_json::from_str(saved.get()).map_err(archive_error)?;
        if major == 1 && saved.has_archived_lists() {
            return Err("Archived lists require workspace archive version 2.0 or later".into());
        }
        if major < 6 && saved.has_encounter_creation_receipts() {
            return Err(
                "Encounter creation receipts require workspace archive version 6.0 or later".into(),
            );
        }
        if major < 5 && saved.has_full_pathbuilder_imports() {
            return Err(
                "Complete Pathbuilder imports require workspace archive version 5.0 or later"
                    .into(),
            );
        }
        if major < 4 && saved.has_characters() {
            return Err("Shared characters require workspace archive version 4.0 or later".into());
        }
        if major < 3 && saved.has_pc_references() {
            return Err("PC rule references require workspace archive version 3.0 or later".into());
        }
        if major < 3 && saved.has_encounter_adjustments() {
            return Err("GM adjustments require workspace archive version 3.0 or later".into());
        }
        saved
            .validate_encounter_archive_format(major >= 7)
            .map_err(message)?;
        let config = archive
            .reference_config
            .map(|value| {
                serde_json::from_str::<ReferenceConfig>(value.get())
                    .map(RequestedReferenceConfig::from)
            })
            .transpose()
            .map_err(archive_error)?;
        if let Some(config) = &config {
            validate_requested_reference_config(config).map_err(message)?;
        }
        Ok(ValidatedArchive {
            saved: saved.validate().map_err(message)?,
            config,
        })
    }
}

pub(crate) fn capture_upgrade_archive(
    workspace: &Workspace,
    snapshot: &Connection,
) -> StorageResult<()> {
    let mut path = workspace.path().as_os_str().to_owned();
    path.push(".upgrade-archive.json");
    let path = PathBuf::from(path);
    WorkspaceArchive::capture_snapshot(snapshot)
        .and_then(|archive| archive.publish(&path, true))
        .map_err(|error| {
            StorageError::Migration(format!(
                "could not capture upgrade archive at {}: {error}",
                path.display()
            ))
        })?;
    tracing::info!(target: "atlas_progress", phase = "upgrade_archive", "Saved workspace recovery archive to {}", path.display());
    Ok(())
}

fn archive_error(error: serde_json::Error) -> String {
    format!("Invalid workspace archive or unsupported saved value: {error}")
}

fn validate_version(version: &str) -> Result<u32, String> {
    fn component(part: &str) -> Option<u32> {
        let parsed: u32 = part.parse().ok()?;
        (parsed.to_string() == part).then_some(parsed)
    }
    let (major, _) = version
        .split_once('.')
        .and_then(|(major, minor)| Some((component(major)?, component(minor)?)))
        .ok_or_else(|| format!("Invalid workspace archive version {version:?}"))?;
    if !matches!(major, 1..=7) {
        return Err(format!("Unsupported workspace archive version {version}"));
    }
    Ok(major)
}

impl AtlasRuntime {
    pub fn capture_workspace_archive(&self) -> Result<WorkspaceArchive, String> {
        let workspace = self.recovery_workspace()?;
        let read = workspace
            .read(WORKSPACE_SCHEMA_VERSION)
            .map_err(admission_error)?;
        WorkspaceArchive::capture(&read)
    }

    pub fn preflight_workspace_restore(&self) -> Result<RestorePreflight, String> {
        let workspace = self.recovery_workspace()?;
        let read = workspace
            .read(WORKSPACE_SCHEMA_VERSION)
            .map_err(admission_error)?;
        read.authenticate_history(RELEASE_HISTORY)
            .map_err(admission_error)?;
        check_empty(read.connection()).map_err(message)?;
        Ok(RestorePreflight {
            workspace_key: read.workspace_key().map_err(message)?,
            config: read_requested_reference_config(&read).map_err(message)?,
        })
    }

    pub fn restore_workspace_archive(
        &self,
        archive: &ValidatedArchive,
        expected: &RestorePreflight,
        confirmed_replacement: bool,
    ) -> Result<(), String> {
        let workspace = self.recovery_workspace()?;
        let _setup = workspace.setup_guard().map_err(message)?;
        let _saved = workspace.saved_write_guard().map_err(message)?;
        let _update = workspace.update_owner().map_err(message)?;
        let write = workspace
            .write(WORKSPACE_SCHEMA_VERSION)
            .map_err(admission_error)?;
        write
            .authenticate_history(RELEASE_HISTORY)
            .map_err(admission_error)?;
        let actual_key: String = write
            .connection()
            .query_row(
                "SELECT workspace_key FROM workspace WHERE id=1",
                [],
                |row| row.get(0),
            )
            .map_err(message)?;
        let actual_config = read_requested_reference_config_for_write(&write).map_err(message)?;
        if actual_key != expected.workspace_key || actual_config != expected.config {
            return Err(
                "The destination workspace or reference configuration changed. Retry restore."
                    .into(),
            );
        }
        if expected.conflicts_with(archive) && !confirmed_replacement {
            return Err("The archive's requested reference configuration differs. Use --replace-reference-config to confirm replacement.".into());
        }
        archive.saved.restore(&write).map_err(message)?;
        if actual_config != archive.config {
            match &archive.config {
                Some(config) => write_requested_reference_config(&write, config),
                None => clear_requested_reference_config(&write),
            }
            .map_err(message)?;
        }
        let mut check = write
            .connection()
            .prepare("PRAGMA foreign_key_check")
            .map_err(message)?;
        if check.exists([]).map_err(message)? {
            return Err("The restored workspace failed foreign key validation.".into());
        }
        drop(check);
        write.commit().map_err(message)
    }

    fn recovery_workspace(&self) -> Result<Workspace, String> {
        register_extensions().map_err(message)?;
        Workspace::existing(self.index_path()).map_err(admission_error)
    }
}

fn admission_error(error: StorageError) -> String {
    match error {
        StorageError::Missing(_) => {
            format!("{error}. Run `atlas setup` for the destination workspace first.")
        }
        StorageError::Schema { actual, expected } if actual < expected => {
            format!("{error}. Run `atlas setup` to upgrade the workspace.")
        }
        _ => error.to_string(),
    }
}

fn message(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AtlasPathMode, AtlasPathOverrides, AtlasRuntimeOptions, initialize_workspace};
    use atlas_local_state::{
        EncounterTurnState, EncounterTurnUpdate, LocalStateRead, LocalStateWrite, NewSavedList,
        PreparedConditionEvent,
    };
    use serde_json::{Value, json};
    use std::path::Path;

    fn runtime(path: &Path) -> AtlasRuntime {
        AtlasRuntime::resolve(AtlasRuntimeOptions {
            path_mode: AtlasPathMode::Global,
            overrides: AtlasPathOverrides {
                index_path: Some(path.to_owned()),
                ..Default::default()
            },
            default_search_profile: None,
        })
        .unwrap()
    }

    fn capture_json(archive: WorkspaceArchive) -> Value {
        let mut bytes = Vec::new();
        archive.write(&mut bytes).unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[test]
    fn historical_encounter_creation_receipt_requires_archive_six_and_blocks_a_second_restore() {
        let dir = tempfile::tempdir().unwrap();
        let source = initialize_workspace(dir.path().join("source.sqlite")).unwrap();
        let target = initialize_workspace(dir.path().join("target.sqlite")).unwrap();
        let write = source.write(WORKSPACE_SCHEMA_VERSION).unwrap();
        let encounters = LocalStateWrite::for_local(&write).unwrap().encounters();
        let key = encounters
            .create(atlas_local_state::NewEncounter {
                slug: "patrol".into(),
                name: "Patrol".into(),
                description: None,
                note: None,
            })
            .unwrap()
            .encounter_key;
        let digest = "a".repeat(64);
        encounters
            .save_creation_receipt("first", &digest, &key)
            .unwrap();
        assert!(encounters.delete(&key).unwrap());
        write.commit().unwrap();

        let raw = capture_json(runtime(source.path()).capture_workspace_archive().unwrap());
        assert_eq!(raw["version"], "6.0");
        let read =
            |value: &Value| WorkspaceArchive::read(serde_json::to_vec(value).unwrap().as_slice());
        let mut downgraded = raw.clone();
        downgraded["version"] = "5.0".into();
        assert!(read(&downgraded).err().unwrap().contains("version 6.0"));
        let mut corrupt = raw.clone();
        corrupt["saved"]["owners"][0]["encounter_creation_receipts"][0]["intent_digest"] =
            "invalid".into();
        assert!(
            read(&corrupt)
                .err()
                .unwrap()
                .contains("invalid encounter creation receipt")
        );
        let restored = runtime(target.path());
        let preflight = restored.preflight_workspace_restore().unwrap();
        restored
            .restore_workspace_archive(&read(&raw).unwrap(), &preflight, false)
            .unwrap();
        assert!(
            restored
                .preflight_workspace_restore()
                .err()
                .unwrap()
                .contains("already contains saved state")
        );
        let snapshot = target.read(WORKSPACE_SCHEMA_VERSION).unwrap();
        let encounters = LocalStateRead::for_local(&snapshot).unwrap().encounters();
        assert!(encounters.get(&key).unwrap().is_none());
        assert_eq!(
            encounters.creation_receipt("first", &digest).unwrap(),
            Some(key)
        );
    }

    #[test]
    fn capture_uses_one_snapshot_for_concurrent_saved_state_and_configuration() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("workspace.sqlite");
        let workspace = initialize_workspace(&path).unwrap();
        let read = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
        let write = workspace.write(WORKSPACE_SCHEMA_VERSION).unwrap();
        LocalStateWrite::for_local(&write)
            .unwrap()
            .saved_lists()
            .create(NewSavedList {
                slug: "concurrent".into(),
                name: "Concurrent".into(),
                description: None,
                tags: vec![],
            })
            .unwrap();
        write_requested_reference_config(&write, &config("/new/source")).unwrap();
        write.commit().unwrap();

        let earlier = capture_json(WorkspaceArchive::capture(&read).unwrap());
        assert_eq!(earlier["saved"]["owners"][0]["lists"], json!([]));
        assert!(earlier["reference_config"].is_null());
        let latest = capture_json(runtime(&path).capture_workspace_archive().unwrap());
        assert_eq!(
            latest["saved"]["owners"][0]["lists"][0]["slug"],
            "concurrent"
        );
        assert_eq!(latest["reference_config"]["source_root"], "/new/source");
    }

    #[test]
    fn restored_receipt_retries_and_reference_corpus_is_preserved() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.sqlite");
        let dest = dir.path().join("dest.sqlite");
        let from = initialize_workspace(&source).unwrap();
        let to = initialize_workspace(&dest).unwrap();
        let input = atlas_search::test_support::input(
            vec![atlas_search::test_support::record(
                "bestiary",
                "Actor",
                json!({"_id":"aaaaaaaaaaaaaaaa","name":"Current title","type":"npc","system":{}}),
            )],
            false,
        );
        atlas_index::test_support::publish_input_in(&to, WORKSPACE_SCHEMA_VERSION, &input).unwrap();
        let before = to
            .read(WORKSPACE_SCHEMA_VERSION)
            .unwrap()
            .reference_revision()
            .unwrap();

        let write = from.write(WORKSPACE_SCHEMA_VERSION).unwrap();
        write.connection().execute_batch(
            "INSERT INTO record_identity(key,last_known_name,availability)
                VALUES('bestiary:aaaaaaaaaaaaaaaa','Old title','unavailable');
             INSERT INTO encounters(owner_id,encounter_key,slug,name,archived,created_at,updated_at,revision)
                VALUES(1,'stable-key','encounter','Encounter',0,'created','updated',5);
             INSERT INTO encounter_participants(encounter_id,participant_key,record_key,participant_kind,
                position,display_name,record_title_snapshot,initiative,initiative_order,created_at,updated_at)
                SELECT id,'stable-participant','bestiary:aaaaaaaaaaaaaaaa','creature',1,'Old title','Old title',10,1,'created','updated'
                FROM encounters;",
        ).unwrap();
        let owner = LocalStateWrite::for_local(&write).unwrap();
        let intent = json!({"action":"fixture"});
        let receipt = owner
            .encounters()
            .apply_condition_event(
                "stable-key",
                "original",
                &intent,
                5,
                PreparedConditionEvent::default(),
            )
            .unwrap();
        let encounter = owner.encounters().get("stable-key").unwrap().unwrap();
        owner
            .encounters()
            .advance_turn_once(
                "stable-key",
                "advance",
                &json!({"turn": 2}),
                encounter.revision,
                |_| {
                    Ok((
                        EncounterTurnUpdate {
                            adjustments: vec![],
                            manual_trackers: vec![],
                            start_event: None,
                            start_claims: vec![],
                            turn: EncounterTurnState {
                                participant_key: Some("stable-participant".into()),
                                round_number: 2,
                            },
                            durations: vec![],
                        },
                        true,
                    ))
                },
            )
            .unwrap();
        write.commit().unwrap();

        let mut archive = capture_json(runtime(&source).capture_workspace_archive().unwrap());
        assert_eq!(archive["version"], "7.0");
        archive["saved"]["owners"][0]["encounters"][0]["receipts"][0]["outcome"]["display_hint"] =
            "optional".into();
        let archive =
            WorkspaceArchive::read(serde_json::to_vec(&archive).unwrap().as_slice()).unwrap();
        let target = runtime(&dest);
        let preflight = target.preflight_workspace_restore().unwrap();
        target
            .restore_workspace_archive(&archive, &preflight, false)
            .unwrap();
        let source_read = from.read(WORKSPACE_SCHEMA_VERSION).unwrap();
        let dest_read = to.read(WORKSPACE_SCHEMA_VERSION).unwrap();
        let original_review = LocalStateRead::for_local(&source_read)
            .unwrap()
            .encounters()
            .latest_outgoing_turn("stable-key")
            .unwrap()
            .unwrap();
        let recovered_review = LocalStateRead::for_local(&dest_read)
            .unwrap()
            .encounters()
            .latest_outgoing_turn("stable-key")
            .unwrap()
            .unwrap();
        assert_eq!(original_review, recovered_review);
        drop(dest_read);
        let write = to.write(WORKSPACE_SCHEMA_VERSION).unwrap();
        assert_eq!(write.reference_revision().unwrap(), before);
        let identity: (String, String) = write.connection().query_row(
            "SELECT last_known_name,availability FROM record_identity WHERE key='bestiary:aaaaaaaaaaaaaaaa'",
            [], |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap();
        assert_eq!(identity, ("Current title".into(), "available".into()));
        let encounters = LocalStateWrite::for_local(&write).unwrap().encounters();
        let restored = encounters
            .get_with_participants("stable-key")
            .unwrap()
            .unwrap();
        assert_eq!(
            restored.participants[0].record_title_snapshot.as_deref(),
            Some("Old title")
        );
        let retried = encounters
            .apply_condition_event_with("stable-key", "original", &intent, 0, || {
                panic!("retry must return its stored receipt")
            })
            .unwrap();
        assert_eq!(retried, receipt);
    }

    #[test]
    fn gm_adjustments_round_trip_with_archive_membership() {
        use atlas_local_state::{AdjustmentChange, EncounterAdjustment, PreparedConditionEvent};

        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.sqlite");
        let dest = dir.path().join("dest.sqlite");
        let from = initialize_workspace(&source).unwrap();
        initialize_workspace(&dest).unwrap();
        let write = from.write(WORKSPACE_SCHEMA_VERSION).unwrap();
        write.connection().execute_batch(
            "INSERT INTO encounters(owner_id,encounter_key,slug,name,archived,created_at,updated_at)
                VALUES(1,'enc','example','Example',0,'created','updated');
             INSERT INTO encounter_participants(encounter_id,participant_key,participant_kind,
                position,display_name,initiative,initiative_order,created_at,updated_at)
                SELECT id,'actor','pc',1,'Actor',10,1,'created','updated' FROM encounters;",
        ).unwrap();

        let saved = LocalStateWrite::for_local(&write).unwrap().encounters();
        let adjustment = EncounterAdjustment {
            adjustment_key: "gm-armor".into(),
            target: atlas_domain::AdjustmentTarget::ArmorClass,
            name: "Armor loss".into(),
            amount: -2,
            modifier_type: atlas_domain::AdjustmentModifierType::Stacks,
            timing: None,
        };
        let revision = saved.get("enc").unwrap().unwrap().revision;
        saved
            .apply_condition_event_with(
                "enc",
                "apply",
                &json!({"action":"gm_adjustment"}),
                revision,
                || {
                    Ok(PreparedConditionEvent {
                        adjustments: vec![AdjustmentChange {
                            participant_key: "actor".into(),
                            adjustment_key: adjustment.adjustment_key.clone(),
                            value: Some(adjustment.clone()),
                        }],
                        ..Default::default()
                    })
                },
            )
            .unwrap();
        write.commit().unwrap();

        let mut archive = capture_json(runtime(&source).capture_workspace_archive().unwrap());
        assert_eq!(archive["version"], "7.0");
        let validated =
            WorkspaceArchive::read(serde_json::to_vec(&archive).unwrap().as_slice()).unwrap();
        archive["version"] = "2.0".into();
        assert!(WorkspaceArchive::read(serde_json::to_vec(&archive).unwrap().as_slice()).is_err());

        let runtime = runtime(&dest);
        let preflight = runtime.preflight_workspace_restore().unwrap();
        runtime
            .restore_workspace_archive(&validated, &preflight, false)
            .unwrap();
        let restored = initialize_workspace(&dest).unwrap();
        let read = restored.read(WORKSPACE_SCHEMA_VERSION).unwrap();
        let encounters = atlas_local_state::LocalStateRead::for_local(&read)
            .unwrap()
            .encounters();
        let detail = encounters.get_with_participants("enc").unwrap().unwrap();
        assert_eq!(detail.participants[0].adjustments, [adjustment]);
    }

    #[test]
    fn restore_checks_the_configuration_confirmed_by_the_caller_again() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("workspace.sqlite");
        let workspace = initialize_workspace(&path).unwrap();
        let write = workspace.write(WORKSPACE_SCHEMA_VERSION).unwrap();
        write_requested_reference_config(&write, &config("/old")).unwrap();
        write.commit().unwrap();
        let runtime = runtime(&path);
        let preflight = runtime.preflight_workspace_restore().unwrap();
        let archive = WorkspaceArchive::read(
            include_bytes!("../../atlas-cli/tests/fixtures/workspace-archive-1.0.json").as_slice(),
        )
        .unwrap();
        assert!(preflight.conflicts_with(&archive));
        let write = workspace.write(WORKSPACE_SCHEMA_VERSION).unwrap();
        write_requested_reference_config(&write, &config("/intervening")).unwrap();
        write.commit().unwrap();
        assert!(
            runtime
                .restore_workspace_archive(&archive, &preflight, true)
                .unwrap_err()
                .contains("changed")
        );
        let fresh = runtime.preflight_workspace_restore().unwrap();
        runtime
            .restore_workspace_archive(&archive, &fresh, true)
            .unwrap();
        let read = workspace.read(WORKSPACE_SCHEMA_VERSION).unwrap();
        assert!(read_requested_reference_config(&read).unwrap().is_none());
    }

    fn config(source: &str) -> RequestedReferenceConfig {
        RequestedReferenceConfig {
            source_root: source.into(),
            manifest_path: None,
            indexing_locale: "en".into(),
            search_profiles: vec![],
        }
    }
}
