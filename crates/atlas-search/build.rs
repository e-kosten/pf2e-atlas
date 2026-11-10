//! Compile the maintainer-edited equipment families into one embedded payload.

mod build_axes;

#[path = "src/variants/catalog.rs"]
mod catalog;

use atlas_domain::RecordKey;
use catalog::{Catalog, Exclusion, Family};
use serde::{Deserialize, de::DeserializeOwned};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Exclusions {
    exclusions: Vec<Exclusion>,
}

fn main() {
    if let Err(error) = bundle() {
        eprintln!("equipment family catalog: {error}");
        std::process::exit(1);
    }
}

fn bundle() -> Result<(), Box<dyn Error>> {
    let root = Path::new("families/equipment");
    let exclusions_path = root.join("exclusions.toml");
    println!("cargo:rerun-if-changed=src/variants/catalog.rs");
    println!("cargo:rerun-if-changed=build_axes.rs");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut paths = Vec::new();
    family_files(root, &mut paths)?;
    paths.sort();

    let mut families = BTreeMap::new();
    let mut ids = BTreeMap::new();
    let mut members = BTreeMap::new();
    for path in paths.iter().filter(|path| **path != exclusions_path) {
        let id = path
            .file_stem()
            .and_then(|name| name.to_str())
            .filter(|id| catalog::valid_family_id(id))
            .ok_or_else(|| invalid(path, "invalid family ID"))?;
        if let Some(previous) = ids.insert(id, path) {
            return Err(invalid(
                path,
                format!("duplicate family ID also in {}", previous.display()),
            )
            .into());
        }
        let family = read::<Family>(path)?;
        validate_family(path, &family)?;
        for key in family
            .members
            .iter()
            .map(|member| &member.key)
            .chain(family.general_records.iter().map(|record| &record.key))
        {
            if let Some(previous) = members.insert(key.clone(), id) {
                return Err(invalid(path, format!("{key} already belongs to {previous}")).into());
            }
        }
        families.insert(id.to_owned(), family);
    }
    if families.is_empty() {
        return Err(invalid(root, "no family files found").into());
    }
    let Exclusions { exclusions } = read(&exclusions_path)?;
    validate_exclusions(&exclusions_path, &exclusions, &members)?;
    let destination = PathBuf::from(std::env::var_os("OUT_DIR").ok_or("missing OUT_DIR")?)
        .join("equipment_families.json");
    fs::write(
        destination,
        serde_json::to_vec(&Catalog {
            families,
            exclusions,
        })?,
    )?;
    Ok(())
}

fn family_files(dir: &Path, paths: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let path = entry.path();
        if kind.is_dir() {
            family_files(&path, paths)?;
        } else if kind.is_file() && path.extension().is_some_and(|ext| ext == "toml") {
            paths.push(path);
        }
    }
    Ok(())
}

fn read<T: DeserializeOwned>(path: &Path) -> io::Result<T> {
    let input = fs::read_to_string(path).map_err(|error| invalid(path, error))?;
    toml_edit::de::from_str(&input).map_err(|error| invalid(path, error))
}

fn invalid(path: &Path, message: impl std::fmt::Display) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        format!("{}: {message}", path.display()),
    )
}

fn equipment_key(key: &RecordKey) -> bool {
    key.pack().as_str() == "equipment-srd"
}

fn validate_family(path: &Path, family: &Family) -> io::Result<()> {
    if family.name.trim().is_empty() || family.source_note.trim().is_empty() {
        return Err(invalid(path, "family name and source note are required"));
    }
    if !(2..=256).contains(&family.members.len()) {
        return Err(invalid(path, "expected 2 to 256 members"));
    }
    let item_type = family.item_type.as_str();
    let requires_category = matches!(item_type, "armor" | "consumable" | "weapon");
    let blank_compatibility = family
        .category
        .iter()
        .chain(&family.group)
        .any(|value| value.trim().is_empty());
    if !matches!(
        item_type,
        "ammo"
            | "armor"
            | "backpack"
            | "consumable"
            | "equipment"
            | "kit"
            | "shield"
            | "treasure"
            | "weapon"
    ) || requires_category != family.category.is_some()
        || (item_type == "weapon") != family.group.is_some()
        || blank_compatibility
    {
        return Err(invalid(
            path,
            "invalid item type or required compatibility fields",
        ));
    }
    build_axes::validate(family).map_err(|error| invalid(path, error))?;
    for member in &family.members {
        if !equipment_key(&member.key)
            || member.name.trim().is_empty()
            || member
                .attributes
                .iter()
                .any(|attribute| attribute.trim().is_empty())
            || (family.axes.is_empty() && member.attributes.is_empty() && member.fit.is_none())
        {
            return Err(invalid(
                path,
                format!("missing or invalid member facts for {}", member.key),
            ));
        }
        if let Some(category) = &member.category
            && (item_type != "consumable"
                || category.trim().is_empty()
                || family.category.as_ref() == Some(category))
        {
            return Err(invalid(
                path,
                format!("invalid category override for {}", member.key),
            ));
        }
        if let Some(fit) = &member.fit {
            let mut weapons = BTreeSet::new();
            if item_type != "ammo"
                || fit.ammo_type.trim().is_empty()
                || fit.weapons.is_empty()
                || fit.weapons.iter().any(|weapon| {
                    !equipment_key(&weapon.key)
                        || weapon.name.trim().is_empty()
                        || !weapons.insert(&weapon.key)
                })
            {
                return Err(invalid(
                    path,
                    format!("invalid weapon compatibility for {}", member.key),
                ));
            }
        }
    }
    Ok(())
}

fn validate_exclusions(
    path: &Path,
    exclusions: &[Exclusion],
    members: &BTreeMap<RecordKey, &str>,
) -> io::Result<()> {
    let mut pairs = BTreeSet::new();
    for exclusion in exclusions {
        if exclusion.members.is_empty() || exclusion.others.is_empty() {
            return Err(invalid(
                path,
                "each exclusion must name records on both sides",
            ));
        }
        for a in &exclusion.members {
            for b in &exclusion.others {
                let same_family = members.get(a).is_some_and(|id| members.get(b) == Some(id));
                if a == b
                    || !equipment_key(a)
                    || !equipment_key(b)
                    || same_family
                    || !pairs.insert((a.min(b), a.max(b)))
                {
                    return Err(invalid(
                        path,
                        format!("invalid or duplicate exclusion between {a} and {b}"),
                    ));
                }
            }
        }
    }
    Ok(())
}
