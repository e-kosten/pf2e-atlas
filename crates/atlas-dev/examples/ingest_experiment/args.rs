use clap::{Parser, ValueEnum};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub(super) enum Stage {
    Upgrade,
    Build,
    Interrupt,
    Recover,
    Roster,
    Observe,
}
#[derive(Debug, Parser)]
#[command(about = "Explicit developer-only ingest experiment; paths must name private task data")]
pub(super) struct Args {
    #[arg(long)]
    pub task_root: PathBuf,
    #[arg(long)]
    pub database: PathBuf,
    #[arg(long)]
    pub source: PathBuf,
    #[arg(long)]
    pub models: PathBuf,
    #[arg(long)]
    pub output: PathBuf,
    #[arg(long)]
    pub candidate: String,
    #[arg(long, value_enum)]
    pub stage: Stage,
    /// Complete requested set: bge-base,embeddinggemma; either single ID; or none.
    #[arg(long)]
    pub profiles: Option<String>,
    #[arg(long, default_value = "en")]
    pub locale: String,
    #[arg(long)]
    pub interruption_marker: Option<PathBuf>,
    #[arg(long)]
    pub encounter: Option<String>,
    #[arg(long, default_value_t=5, value_parser=clap::value_parser!(u32).range(1..=20))]
    pub repetitions: u32,
    #[arg(long)]
    pub stop_file: Option<PathBuf>,
    #[arg(long, default_value_t=100, value_parser=clap::value_parser!(u64).range(25..=1000))]
    pub interval_ms: u64,
    #[arg(long, default_value_t=10000, value_parser=clap::value_parser!(u32).range(1..=10000))]
    pub max_observations: u32,
}
fn existing(path: &Path, directory: bool) -> Result<(), String> {
    if !path.is_absolute() || path.canonicalize().map_err(|e| e.to_string())? != path {
        return Err(format!(
            "path must be absolute and canonical: {}",
            path.display()
        ));
    }
    if if directory {
        !path.is_dir()
    } else {
        !path.is_file()
    } {
        return Err(format!("unexpected path type: {}", path.display()));
    }
    Ok(())
}
fn fresh(path: &Path, root: &Path) -> Result<(), String> {
    let parent = path.parent().ok_or("fresh path needs a parent")?;
    let name = path.file_name().ok_or("fresh path needs a filename")?;
    if !path.is_absolute()
        || path.exists()
        || parent.canonicalize().map_err(|e| e.to_string())?.join(name) != path
        || !path.starts_with(root)
    {
        return Err(format!(
            "fresh canonical task-owned path required: {}",
            path.display()
        ));
    }
    Ok(())
}
impl Args {
    pub fn check(&self) -> Result<(), String> {
        existing(&self.task_root, true)?;
        if !self.task_root.join("Cargo.toml").is_file() || !self.task_root.join(".git").exists() {
            return Err("task root must be an existing repository checkout".into());
        }
        existing(&self.database, false)?;
        existing(&self.source, true)?;
        existing(&self.models, true)?;
        if !self.database.starts_with(&self.task_root)
            || self.database.starts_with(&self.source)
            || self.output.starts_with(&self.source)
            || self.output.starts_with(&self.models)
        {
            return Err(
                "database/output must be private task data outside source/model inputs".into(),
            );
        }
        fresh(&self.output, &self.task_root)?;
        if self.candidate.trim().is_empty() {
            return Err("nonempty declared candidate required".into());
        }
        if matches!(self.stage, Stage::Build | Stage::Interrupt) && self.profiles.is_none() {
            return Err("build/interrupt requires the complete --profiles set".into());
        }
        if self.stage == Stage::Recover && self.interruption_marker.is_none() {
            return Err("recover requires --interruption-marker".into());
        }
        if self.stage == Stage::Roster && self.encounter.as_ref().is_none_or(|v| v.is_empty()) {
            return Err("roster requires --encounter".into());
        }
        if self.stage == Stage::Observe && self.stop_file.is_none() {
            return Err("observe requires an initially absent --stop-file".into());
        }
        if let Some(path) = &self.interruption_marker {
            existing(path, false)?;
            if !path.starts_with(&self.task_root) {
                return Err("marker must be task owned".into());
            }
        }
        if let Some(path) = &self.stop_file {
            fresh(path, &self.task_root)?;
            if path == &self.output
                || path.starts_with(&self.source)
                || path.starts_with(&self.models)
            {
                return Err(
                    "stop signal must differ from output and remain outside source/model inputs"
                        .into(),
                );
            }
        }
        self.profile_ids()?;
        Ok(())
    }
    pub fn profile_ids(&self) -> Result<Vec<atlas_domain::SearchProfileId>, String> {
        let Some(value) = &self.profiles else {
            return Ok(Vec::new());
        };
        if value == "none" {
            return Ok(Vec::new());
        }
        let mut seen = std::collections::BTreeSet::new();
        value
            .split(',')
            .map(|id| {
                let id = atlas_domain::SearchProfileId::new(id);
                atlas_embedding::search_profile(&id).map_err(|e| e.to_string())?;
                if !seen.insert(id.clone()) {
                    return Err("duplicate requested profile".into());
                }
                Ok(id)
            })
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn profile_set_is_explicit_complete_and_deduplicated() {
        let mut args = Args::try_parse_from([
            "probe",
            "--task-root",
            "/task",
            "--database",
            "/task/db",
            "--source",
            "/source",
            "--models",
            "/models",
            "--output",
            "/task/out",
            "--candidate",
            "candidate",
            "--stage",
            "build",
            "--profiles",
            "bge-base,embeddinggemma",
        ])
        .unwrap();
        assert_eq!(args.profile_ids().unwrap().len(), 2);
        args.profiles = Some("bge-base,bge-base".into());
        assert!(args.profile_ids().is_err());
        args.profiles = Some("none".into());
        assert!(args.profile_ids().unwrap().is_empty());
        args.profiles = Some("".into());
        assert!(args.profile_ids().is_err());
    }
    #[test]
    fn observation_cannot_signal_itself_by_creating_its_output_directory() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        std::fs::write(root.join("Cargo.toml"), "").unwrap();
        std::fs::write(root.join(".git"), "").unwrap();
        std::fs::write(root.join("db"), "private fixture").unwrap();
        std::fs::create_dir(root.join("source")).unwrap();
        std::fs::create_dir(root.join("models")).unwrap();
        let mut args = Args {
            task_root: root.clone(),
            database: root.join("db"),
            source: root.join("source"),
            models: root.join("models"),
            output: root.join("output"),
            candidate: "candidate".into(),
            stage: Stage::Observe,
            profiles: None,
            locale: "en".into(),
            interruption_marker: None,
            encounter: None,
            repetitions: 1,
            stop_file: Some(root.join("output")),
            interval_ms: 25,
            max_observations: 1,
        };
        assert!(args.check().is_err());
        args.stop_file = Some(root.join("stop"));
        assert!(args.check().is_ok());
        args.stop_file = Some(root.join("source/stop"));
        assert!(args.check().is_err());
    }
    #[test]
    fn fresh_paths_reject_existing_outputs_and_symlink_ancestors() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        let output = root.join("output");
        assert!(fresh(&output, &root).is_ok());
        std::fs::create_dir(&output).unwrap();
        assert!(fresh(&output, &root).is_err());
        #[cfg(unix)]
        {
            let link = root.join("link");
            std::os::unix::fs::symlink(&output, &link).unwrap();
            assert!(fresh(&link.join("fresh"), &root).is_err());
        }
    }
}
