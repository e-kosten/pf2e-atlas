use std::fs;
use std::path::Path;

use serde::Serialize;
use sha2::{Digest, Sha256};

pub(crate) const ATLAS_SKILL_MANIFEST: &str = ".atlas-skill.json";

const PF2E_ATLAS_CLI_SKILL: &str = include_str!("../../../../skills/pf2e-atlas-cli/SKILL.md");

#[derive(Debug, Clone)]
pub(crate) struct BundledSkillPackage {
    pub(crate) name: &'static str,
    pub(crate) files: &'static [BundledSkillFile],
}

#[derive(Debug, Clone)]
pub(crate) struct BundledSkillFile {
    pub(crate) relative_path: &'static str,
    pub(crate) contents: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SkillPackageInfo {
    pub(crate) name: String,
    pub(crate) content_hash_algorithm: &'static str,
    pub(crate) content_hash: String,
    pub(crate) files: Vec<String>,
}

const PF2E_ATLAS_CLI_FILES: &[BundledSkillFile] = &[BundledSkillFile {
    relative_path: "SKILL.md",
    contents: PF2E_ATLAS_CLI_SKILL,
}];

const BUNDLED_SKILLS: &[BundledSkillPackage] = &[BundledSkillPackage {
    name: "pf2e-atlas-cli",
    files: PF2E_ATLAS_CLI_FILES,
}];

impl BundledSkillPackage {
    pub(crate) fn info(&self) -> Result<SkillPackageInfo, String> {
        self.validate()?;
        Ok(SkillPackageInfo {
            name: self.name.to_string(),
            content_hash_algorithm: "sha256",
            content_hash: self.content_hash(),
            files: self
                .files
                .iter()
                .map(|file| file.relative_path.to_string())
                .collect(),
        })
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        let skill_file = self
            .files
            .iter()
            .find(|file| file.relative_path == "SKILL.md")
            .ok_or_else(|| format!("bundled skill `{}` is missing SKILL.md", self.name))?;
        let frontmatter = parse_frontmatter(skill_file.contents)
            .ok_or_else(|| format!("bundled skill `{}` is missing YAML frontmatter", self.name))?;
        let name = frontmatter_value(frontmatter, "name")
            .ok_or_else(|| format!("bundled skill `{}` frontmatter is missing name", self.name))?;
        if name != self.name {
            return Err(format!(
                "bundled skill `{}` frontmatter name is `{}`",
                self.name, name
            ));
        }
        if frontmatter_value(frontmatter, "description").is_none() {
            return Err(format!(
                "bundled skill `{}` frontmatter is missing description",
                self.name
            ));
        }
        for file in self.files {
            validate_relative_path(file.relative_path)?;
        }
        Ok(())
    }

    pub(crate) fn content_hash(&self) -> String {
        hash_portable_files(self.files.iter().map(|file| {
            (
                file.relative_path.to_string(),
                file.contents.as_bytes().to_vec(),
            )
        }))
    }
}

pub(crate) fn bundled_skill(name: &str) -> Option<BundledSkillPackage> {
    BUNDLED_SKILLS
        .iter()
        .find(|package| package.name == name)
        .cloned()
}

pub(crate) fn bundled_skills() -> &'static [BundledSkillPackage] {
    BUNDLED_SKILLS
}

fn validate_relative_path(path: &str) -> Result<(), String> {
    let candidate = std::path::Path::new(path);
    if candidate.is_absolute()
        || path.is_empty()
        || candidate
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(format!("invalid bundled skill relative path `{path}`"));
    }
    let allowed = path == "SKILL.md"
        || path.starts_with("references/")
        || path.starts_with("scripts/")
        || path.starts_with("assets/")
        || path == "LICENSE"
        || path == "LICENSE.txt";
    if !allowed {
        return Err(format!(
            "bundled skill path `{path}` is outside allowed portable skill files"
        ));
    }
    Ok(())
}

fn parse_frontmatter(contents: &str) -> Option<&str> {
    let mut lines = contents.lines();
    if lines.next()? != "---" {
        return None;
    }
    let start = 4;
    let end = contents[start..].find("\n---")?;
    Some(&contents[start..start + end])
}

fn frontmatter_value<'a>(frontmatter: &'a str, key: &str) -> Option<&'a str> {
    frontmatter.lines().find_map(|line| {
        let (left, right) = line.split_once(':')?;
        (left.trim() == key).then(|| right.trim().trim_matches('"'))
    })
}

pub(crate) fn installed_content_hash(skill_path: &Path) -> Result<String, String> {
    let mut files = Vec::new();
    collect_installed_files(skill_path, skill_path, &mut files)?;
    Ok(hash_portable_files(files))
}

fn collect_installed_files(
    root: &Path,
    current: &Path,
    files: &mut Vec<(String, Vec<u8>)>,
) -> Result<(), String> {
    for entry in fs::read_dir(current).map_err(|error| {
        format!(
            "failed to read skill directory {}: {error}",
            current.display()
        )
    })? {
        let entry =
            entry.map_err(|error| format!("failed to read skill directory entry: {error}"))?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "installed skill contains unsupported symlink {}",
                path.display()
            ));
        }
        if metadata.is_dir() {
            collect_installed_files(root, &path, files)?;
            continue;
        }
        if !metadata.is_file() {
            return Err(format!(
                "installed skill contains unsupported filesystem entry {}",
                path.display()
            ));
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|error| {
                format!(
                    "failed to compute relative skill path for {}: {error}",
                    path.display()
                )
            })?
            .to_path_buf();
        let relative = portable_relative_path(&relative)?;
        if relative == ATLAS_SKILL_MANIFEST {
            continue;
        }
        validate_relative_path(&relative)?;
        let contents = fs::read(&path).map_err(|error| {
            format!(
                "failed to read installed skill file {}: {error}",
                path.display()
            )
        })?;
        files.push((relative, contents));
    }
    Ok(())
}

fn portable_relative_path(path: &Path) -> Result<String, String> {
    let text = path
        .to_str()
        .ok_or_else(|| format!("skill path is not valid UTF-8: {}", path.display()))?;
    Ok(text.replace(std::path::MAIN_SEPARATOR, "/"))
}

fn hash_portable_files(files: impl IntoIterator<Item = (String, Vec<u8>)>) -> String {
    let mut files = files.into_iter().collect::<Vec<_>>();
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let mut hasher = Sha256::new();
    for (relative_path, contents) in files {
        hasher.update(relative_path.as_bytes());
        hasher.update([0]);
        hasher.update(contents);
        hasher.update([0]);
    }
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_pf2e_skill_is_valid() {
        let package = bundled_skill("pf2e-atlas-cli").expect("skill exists");
        package.validate().expect("skill validates");
        assert_eq!(package.info().expect("info").files, vec!["SKILL.md"]);
    }
}
