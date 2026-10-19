use atlas_runtime::AtlasPathMode;
use clap::ValueEnum;

use crate::ProgressMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CliPathMode {
    Repo,
    Global,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum CliProgressMode {
    Auto,
    Always,
    Never,
}

impl From<CliPathMode> for AtlasPathMode {
    fn from(mode: CliPathMode) -> Self {
        match mode {
            CliPathMode::Repo => Self::Repo,
            CliPathMode::Global => Self::Global,
        }
    }
}

impl From<CliProgressMode> for ProgressMode {
    fn from(mode: CliProgressMode) -> Self {
        match mode {
            CliProgressMode::Auto => Self::Auto,
            CliProgressMode::Always => Self::Always,
            CliProgressMode::Never => Self::Never,
        }
    }
}
