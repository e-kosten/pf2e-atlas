use crate::creature_catalog::{CreatureCatalog, CreatureFamily};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::{Path, PathBuf},
};

pub(super) fn bundle() -> Result<(), Box<dyn Error>> {
    let root = Path::new("families/creatures");
    println!("cargo:rerun-if-changed={}", root.display());
    println!("cargo:rerun-if-changed=build_creature_families.rs");
    println!("cargo:rerun-if-changed=src/creature_catalog.rs");
    let mut paths = Vec::new();
    super::family_files(root, &mut paths)?;
    paths.sort();
    let mut families = CreatureCatalog::new();
    let mut member_keys = BTreeSet::new();
    for path in paths {
        let id = path
            .file_stem()
            .and_then(|name| name.to_str())
            .filter(|id| super::catalog::valid_family_id(id))
            .ok_or_else(|| super::invalid(&path, "invalid family ID"))?;
        let family: CreatureFamily = super::read(&path)?;
        validate(&family).map_err(|error| super::invalid(&path, error))?;
        for member in &family.members {
            if !member_keys.insert(member.key.clone()) {
                return Err(
                    super::invalid(&path, format!("duplicate member {}", member.key)).into(),
                );
            }
        }
        if families.insert(id.to_owned(), family).is_some() {
            return Err(super::invalid(&path, "duplicate family ID").into());
        }
    }
    let destination = PathBuf::from(std::env::var_os("OUT_DIR").ok_or("missing OUT_DIR")?)
        .join("creature_families.json");
    fs::write(destination, serde_json::to_vec(&families)?)?;
    Ok(())
}

fn validate(family: &CreatureFamily) -> Result<(), String> {
    if family.name.trim().is_empty()
        || family.source_note.trim().is_empty()
        || family.members.len() < 2
        || family
            .members
            .iter()
            .all(|member| member.named_base.is_some())
    {
        return Err("a family needs a name, a source note and at least two members".into());
    }
    crate::family_axis_validation::ValidatedAxes::new(&family.axes)?.validate_members(
        family
            .members
            .iter()
            .filter(|member| member.named_base.is_none())
            .map(|member| (&member.key, &member.coordinates)),
    )?;
    let members: BTreeMap<_, _> = family
        .members
        .iter()
        .map(|member| (&member.key, member))
        .collect();
    for member in &family.members {
        if (member.named_base.is_some() && !member.coordinates.is_empty())
            || (!member.coordinates.is_empty() && member.qualifier.is_some())
        {
            return Err(format!(
                "invalid axis coordinates or qualifier for {}",
                member.key
            ));
        }
        if member.name.trim().is_empty()
            || member
                .label
                .as_ref()
                .is_some_and(|label| label.trim().is_empty())
            || (!member.uses.is_empty()
                && member
                    .publication
                    .as_ref()
                    .is_none_or(|title| title.trim().is_empty()))
        {
            return Err(format!("invalid title or publication for {}", member.key));
        }
        if let Some(base) = &member.named_base
            && (base == &member.key
                || members
                    .get(base)
                    .is_none_or(|target| target.named_base.is_some()))
        {
            return Err(format!(
                "unknown or invalid named-creature base for {}",
                member.key
            ));
        }
        let mut uses = BTreeSet::new();
        for use_ in &member.uses {
            if !super::catalog::valid_family_id(&use_.id)
                || !uses.insert(&use_.id)
                || use_
                    .party_levels
                    .is_some_and(|[low, high]| low == 0 || low > high || high > 20)
                || use_.minimum_challenge_points == Some(0)
            {
                return Err(format!(
                    "invalid scenario use {} for {}",
                    use_.id, member.key
                ));
            }
        }
    }
    for (index, entry) in family.overrides.iter().enumerate() {
        let (Some(base), Some(target)) = (members.get(&entry.base), members.get(&entry.target))
        else {
            return Err("an override must reference two family members".into());
        };
        let Some(scope) = base.uses.iter().find(|use_| use_.id == entry.scenario_use) else {
            return Err(format!(
                "unknown override scenario use {}",
                entry.scenario_use
            ));
        };
        if base.named_base.is_some()
            || target.named_base.is_some()
            || entry.base == entry.target
            || base.publication != target.publication
            || entry.source_note.trim().is_empty()
            || !target.uses.iter().any(|use_| {
                use_.party_levels == scope.party_levels
                    && use_.minimum_challenge_points == scope.minimum_challenge_points
            })
            || family.overrides[..index].iter().any(|other| {
                other.base == entry.base
                    && other.adjustment == entry.adjustment
                    && other.scenario_use == entry.scenario_use
            })
        {
            return Err(format!(
                "invalid or conflicting override for {}",
                entry.base
            ));
        }
    }
    Ok(())
}
