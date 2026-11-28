//! Named query accelerators derived exclusively from the checked source DTO.
use crate::{
    IndexError,
    input::SourceArtifactRecordInput,
    numeric::numeric_value,
    persistence::{
        ProjectionId::{Local, Root},
        ProjectionRow,
    },
};
use atlas_domain::{OwnedContentLocator, RecordKey};
use atlas_foundry_model::generated::SkillSlug;
use atlas_record::source_record::{
    ActorIwrKind, ActorIwrType, ActorQueryView, ActorSave, ClassProficiency, ItemSourceView,
    SourceFieldView, SourceNodeView, SourceQueryView, hazard_disable_skills,
};
use rusqlite::types::Value;
use serde::Serialize;
use serde_json::Number;

pub(crate) const ABILITY_FIELDS: &[(&str, &str)] = &[
    ("str", "ability_str"),
    ("dex", "ability_dex"),
    ("con", "ability_con"),
    ("int", "ability_int"),
    ("wis", "ability_wis"),
    ("cha", "ability_cha"),
];
pub(crate) const CLASS_PROFICIENCIES: &[(&str, &str, ClassProficiency)] = &[
    (
        "class.starting_proficiency.simple_weapons",
        "simple_weapons",
        ClassProficiency::Simple,
    ),
    (
        "class.starting_proficiency.martial_weapons",
        "martial_weapons",
        ClassProficiency::Martial,
    ),
    (
        "class.starting_proficiency.advanced_weapons",
        "advanced_weapons",
        ClassProficiency::Advanced,
    ),
    (
        "class.starting_proficiency.unarmed_attacks",
        "unarmed_attacks",
        ClassProficiency::Unarmed,
    ),
    (
        "class.starting_proficiency.unarmored",
        "unarmored",
        ClassProficiency::Unarmored,
    ),
    (
        "class.starting_proficiency.light_armor",
        "light_armor",
        ClassProficiency::Light,
    ),
    (
        "class.starting_proficiency.medium_armor",
        "medium_armor",
        ClassProficiency::Medium,
    ),
    (
        "class.starting_proficiency.heavy_armor",
        "heavy_armor",
        ClassProficiency::Heavy,
    ),
];

pub(crate) const SKILL_FIELDS: &[(&str, &str, &str)] = &[
    (
        "acrobatics",
        "skill_acrobatics_modifier",
        "skill_acrobatics_rank",
    ),
    ("arcana", "skill_arcana_modifier", "skill_arcana_rank"),
    (
        "athletics",
        "skill_athletics_modifier",
        "skill_athletics_rank",
    ),
    (
        "computers",
        "skill_computers_modifier",
        "skill_computers_rank",
    ),
    ("crafting", "skill_crafting_modifier", "skill_crafting_rank"),
    (
        "deception",
        "skill_deception_modifier",
        "skill_deception_rank",
    ),
    (
        "diplomacy",
        "skill_diplomacy_modifier",
        "skill_diplomacy_rank",
    ),
    (
        "intimidation",
        "skill_intimidation_modifier",
        "skill_intimidation_rank",
    ),
    ("medicine", "skill_medicine_modifier", "skill_medicine_rank"),
    ("nature", "skill_nature_modifier", "skill_nature_rank"),
    (
        "occultism",
        "skill_occultism_modifier",
        "skill_occultism_rank",
    ),
    (
        "performance",
        "skill_performance_modifier",
        "skill_performance_rank",
    ),
    ("piloting", "skill_piloting_modifier", "skill_piloting_rank"),
    ("religion", "skill_religion_modifier", "skill_religion_rank"),
    ("society", "skill_society_modifier", "skill_society_rank"),
    ("stealth", "skill_stealth_modifier", "skill_stealth_rank"),
    ("survival", "skill_survival_modifier", "skill_survival_rank"),
    ("thievery", "skill_thievery_modifier", "skill_thievery_rank"),
];

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SqlFact {
    pub state: &'static str,
    pub value: Value,
}
impl SqlFact {
    pub fn field(self, row: &mut ProjectionRow, name: &'static str) {
        row.push(format!("{name}_state"), Value::Text(self.state.into()));
        row.push(name, self.value);
    }
    pub fn collection(self, row: &mut ProjectionRow, name: &'static str) {
        row.text(name, self.state);
    }
}
fn convert<T>(
    view: SourceFieldView<'_, T>,
    next: impl FnOnce(T) -> Result<Value, IndexError>,
) -> Result<SqlFact, IndexError> {
    let (state, value) = match view {
        SourceFieldView::Value(v) => ("value", next(v)?),
        SourceFieldView::Missing => ("missing", Value::Null),
        SourceFieldView::Null => ("null", Value::Null),
        SourceFieldView::Invalid(_) | SourceFieldView::ProjectionInvalid { .. } => {
            ("invalid", Value::Null)
        }
        SourceFieldView::NotApplicable => ("not_applicable", Value::Null),
    };
    Ok(SqlFact { state, value })
}
fn state<T>(view: &SourceFieldView<'_, T>) -> SqlFact {
    SqlFact {
        state: match view {
            SourceFieldView::Value(_) => "value",
            SourceFieldView::Missing => "missing",
            SourceFieldView::Null => "null",
            SourceFieldView::Invalid(_) | SourceFieldView::ProjectionInvalid { .. } => "invalid",
            SourceFieldView::NotApplicable => "not_applicable",
        },
        value: Value::Null,
    }
}
fn text(v: SourceFieldView<'_, &str>) -> Result<SqlFact, IndexError> {
    convert(v, |s| Ok(Value::Text(s.into())))
}
fn number(v: SourceFieldView<'_, &Number>) -> Result<SqlFact, IndexError> {
    convert(v, numeric_value)
}
fn boolean(v: SourceFieldView<'_, bool>) -> Result<SqlFact, IndexError> {
    convert(v, |b| Ok(Value::Integer(i64::from(b))))
}
fn choice<T: Serialize>(v: SourceFieldView<'_, T>) -> Result<SqlFact, IndexError> {
    convert(v, |v| {
        Ok(Value::Text(serde_json::from_str(&serde_json::to_string(
            &v,
        )?)?))
    })
}
fn float(v: SourceFieldView<'_, f64>) -> Result<SqlFact, IndexError> {
    convert(v, |n| {
        if n.is_finite() {
            Ok(Value::Real(n))
        } else {
            Err(IndexError::Invalid("nonfinite derived projection".into()))
        }
    })
}

#[derive(Debug, Default)]
pub(crate) struct ProjectionIds {
    next: i64,
}
impl ProjectionIds {
    pub(crate) fn seeded(seed: i64) -> Self {
        Self { next: seed }
    }
    pub(crate) fn end(&self) -> i64 {
        self.next
    }
    fn take(&mut self) -> Result<i64, IndexError> {
        self.next = self
            .next
            .checked_add(1)
            .ok_or_else(|| IndexError::Invalid("projection row ID overflow".into()))?;
        Ok(self.next)
    }
}
pub(crate) fn root_rows(
    input: &SourceArtifactRecordInput,
    pack_label: &str,
    record_id: i64,
    ids: &mut ProjectionIds,
) -> Result<Vec<ProjectionRow>, IndexError> {
    let q = SourceQueryView::new(
        input.record.source(),
        input.record.key().pack().as_str(),
        pack_label,
    );
    let mut rows = Vec::new();
    let mut r = ProjectionRow::new("records");
    r.projection_id("record_id", Root(record_id));
    r.text("key", input.record.key().to_string());
    r.text("pack_id", input.record.key().pack().as_str());
    r.text("document_kind", q.source.document_kind());
    r.text("source_path", &input.source_path);
    r.text("content_hash", &input.content_hash);
    text(q.source.name().map(String::as_str))?.field(&mut r, "name");
    let lookup = q
        .source
        .name()
        .value()
        .map(|s| Value::Text(atlas_domain::normalize_record_name(s)))
        .unwrap_or(Value::Null);
    r.push("name_lookup_key", lookup);
    common_fields(q, &mut r)?;
    rows.push(r);
    match q.source {
        SourceNodeView::Item(i) => {
            creation_rows(i, input.record.key(), record_id, ids, &mut rows)?;
            item_rows(i, Some(record_id), None, ids, &mut rows)?;
        }
        SourceNodeView::Actor(_) => {
            actor_rows(
                q.actor(),
                hazard_disable_skills(q.source, &input.content),
                record_id,
                ids,
                &mut rows,
            )?;
            let mut error = None;
            input
                .record
                .visit_immediate_actor_items(|index, owner, item| {
                    if error.is_some() {
                        return;
                    }
                    if let Err(e) = child_rows(
                        item,
                        index,
                        owner,
                        record_id,
                        (q.pack_id, q.pack_label),
                        ids,
                        &mut rows,
                    ) {
                        error = Some(e);
                    }
                });
            if let Some(e) = error {
                return Err(e);
            }
        }
        _ => {}
    };
    Ok(rows)
}
fn common_fields(q: SourceQueryView<'_>, row: &mut ProjectionRow) -> Result<(), IndexError> {
    text(q.source.source_type())?.field(row, "source_type");
    if row.table == "records" {
        choice(q.record_kind())?.field(row, "record_kind");
    }
    choice(q.rarity())?.field(row, "rarity");
    text(q.publication_title())?.field(row, "publication_title");
    boolean(q.publication_remaster())?.field(row, "publication_remaster");
    state(&q.traits()).collection(row, "traits_state");
    let (level, size) = match q.source {
        SourceNodeView::Actor(_) => (q.actor().level(), q.actor().size()),
        SourceNodeView::Item(i) => {
            let size = match i.physical_size() {
                SourceFieldView::NotApplicable => i.ancestry_size(),
                v => v,
            };
            (i.item_level(), size)
        }
        _ => (
            SourceFieldView::NotApplicable,
            SourceFieldView::NotApplicable,
        ),
    };
    number(level)?.field(row, "level");
    choice(size)?.field(row, "size");
    Ok(())
}
fn child_rows(
    item: ItemSourceView<'_>,
    index: usize,
    owner: &OwnedContentLocator,
    root: i64,
    pack: (&str, &str),
    ids: &mut ProjectionIds,
    rows: &mut Vec<ProjectionRow>,
) -> Result<(), IndexError> {
    let id = ids.take()?;
    let q = SourceQueryView {
        source: SourceNodeView::Item(item),
        pack_id: pack.0,
        pack_label: pack.1,
    };
    let mut r = ProjectionRow::new("actor_items");
    r.projection_id("id", Local(id));
    r.projection_id("record_id", Root(root));
    r.id(
        "original_index",
        i64::try_from(index).map_err(|e| IndexError::Invalid(e.to_string()))?,
    );
    r.text("owner_selector_json", serde_json::to_string(owner)?);
    text(q.source.id().map(String::as_str))?.field(&mut r, "authored_id");
    common_fields(q, &mut r)?;
    rows.push(r);
    item_rows(item, None, Some(id), ids, rows)
}
fn sets<T>(
    v: SourceFieldView<'_, &[T]>,
    rows: &mut Vec<ProjectionRow>,
    table: &'static str,
    owner: &'static str,
    id: crate::persistence::ProjectionId,
    to_string: impl Fn(&T) -> Result<String, IndexError>,
) -> Result<(), IndexError> {
    if let SourceFieldView::Value(values) = v {
        let values = values
            .iter()
            .map(to_string)
            .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
        for value in values {
            let mut r = ProjectionRow::new(table);
            r.projection_id(owner, id);
            r.text("value", value);
            rows.push(r);
        }
    };
    Ok(())
}
fn enum_name<T: Serialize>(v: &T) -> Result<String, IndexError> {
    Ok(serde_json::from_str(&serde_json::to_string(v)?)?)
}
fn actor_rows(
    a: ActorQueryView<'_>,
    disable_skills: SourceFieldView<'_, Vec<SkillSlug>>,
    root: i64,
    ids: &mut ProjectionIds,
    rows: &mut Vec<ProjectionRow>,
) -> Result<(), IndexError> {
    let mut r = ProjectionRow::new("actor_projection");
    r.projection_id("record_id", Root(root));

    if !matches!(disable_skills, SourceFieldView::NotApplicable) {
        state(&disable_skills).collection(&mut r, "disable_skills_state");
    }
    number(a.armor_class())?.field(&mut r, "armor_class");
    number(a.hp_maximum())?.field(&mut r, "hp_maximum");
    number(a.hazard_hardness())?.field(&mut r, "hardness");
    number(a.save(ActorSave::Fortitude))?.field(&mut r, "fortitude");
    number(a.save(ActorSave::Reflex))?.field(&mut r, "reflex");
    number(a.save(ActorSave::Will))?.field(&mut r, "will");
    number(a.perception())?.field(&mut r, "perception");
    number(a.land_speed())?.field(&mut r, "land_speed");
    number(a.hazard_stealth())?.field(&mut r, "stealth");
    for (key, column) in ABILITY_FIELDS {
        number(a.ability_modifier(key))?.field(&mut r, column);
    }
    for (key, modifier, rank) in SKILL_FIELDS {
        number(a.skill_modifier(key))?.field(&mut r, modifier);
        number(a.skill_rank(key))?.field(&mut r, rank);
    }
    boolean(a.hazard_complexity())?.field(&mut r, "complexity");
    state(&a.items()).collection(&mut r, "items_state");
    state(&a.languages()).collection(&mut r, "languages_state");
    state(&a.speeds()).collection(&mut r, "speeds_state");
    state(&a.senses()).collection(&mut r, "senses_state");
    for (kind, name) in [
        (ActorIwrKind::Immunity, "immunities_state"),
        (ActorIwrKind::Weakness, "weaknesses_state"),
        (ActorIwrKind::Resistance, "resistances_state"),
    ] {
        state(&a.iwr(kind)).collection(&mut r, name);
    }
    rows.push(r);
    if let SourceFieldView::Value(skills) = disable_skills {
        sets(
            SourceFieldView::Value(skills.as_slice()),
            rows,
            "hazard_disable_skills",
            "record_id",
            Root(root),
            enum_name,
        )?;
    }
    sets(
        a.languages(),
        rows,
        "actor_languages",
        "record_id",
        Root(root),
        enum_name,
    )?;
    if let SourceFieldView::Value(speeds) = a.speeds() {
        for (index, speed) in speeds.iter().enumerate() {
            let mut r = ProjectionRow::new("actor_speeds");
            r.projection_id("id", Local(ids.take()?));
            r.projection_id("record_id", Root(root));
            r.id("original_index", index as i64);
            choice(speed.speed_type())?.field(&mut r, "type");
            number(speed.value())?.field(&mut r, "value");
            rows.push(r);
        }
    }
    if let SourceFieldView::Value(senses) = a.senses() {
        for (index, sense) in senses.iter().enumerate() {
            let mut r = ProjectionRow::new("actor_senses");
            r.projection_id("id", Local(ids.take()?));
            r.projection_id("record_id", Root(root));
            r.id("original_index", index as i64);
            choice(SourceFieldView::from(&sense.r#type))?.field(&mut r, "type");
            choice(SourceFieldView::from(&sense.acuity))?.field(&mut r, "acuity");
            number(SourceFieldView::from(&sense.range))?.field(&mut r, "range");
            rows.push(r);
        }
    }
    for (kind, name) in [
        (ActorIwrKind::Immunity, "immunity"),
        (ActorIwrKind::Weakness, "weakness"),
        (ActorIwrKind::Resistance, "resistance"),
    ] {
        if let SourceFieldView::Value(entries) = a.iwr(kind) {
            for (index, entry) in entries.iter().enumerate() {
                let mut r = ProjectionRow::new("actor_iwr_entries");
                r.projection_id("entry_id", Local(ids.take()?));
                r.projection_id("record_id", Root(root));
                r.id("original_index", index as i64);
                r.text("kind", name);
                convert(entry.iwr_type(), |v| {
                    Ok(Value::Text(match v {
                        ActorIwrType::Immunity(v) => enum_name(v)?,
                        ActorIwrType::Weakness(v) => enum_name(v)?,
                        ActorIwrType::Resistance(v) => enum_name(v)?,
                    }))
                })?
                .field(&mut r, "type");
                number(entry.value())?.field(&mut r, "value");
                rows.push(r);
            }
        }
    }
    Ok(())
}
fn owner_row(
    table: &'static str,
    root: Option<i64>,
    child: Option<i64>,
    ids: &mut ProjectionIds,
) -> Result<(ProjectionRow, i64), IndexError> {
    let id = ids.take()?;
    let mut r = ProjectionRow::new(table);
    r.projection_id("id", Local(id));
    match root {
        Some(root) => r.projection_id("root_record_id", Root(root)),
        None => r.push("root_record_id", Value::Null),
    }
    match child {
        Some(child) => r.projection_id("actor_item_id", Local(child)),
        None => r.push("actor_item_id", Value::Null),
    }
    Ok((r, id))
}
fn root_family_row(
    table: &'static str,
    root: i64,
    ids: &mut ProjectionIds,
) -> Result<(ProjectionRow, crate::persistence::ProjectionId), IndexError> {
    let mut row = ProjectionRow::new(table);
    let id = Local(ids.take()?);
    row.projection_id("id", id);
    row.projection_id("root_record_id", Root(root));
    Ok((row, id))
}

fn creation_rows(
    item: ItemSourceView<'_>,
    key: &RecordKey,
    root: i64,
    ids: &mut ProjectionIds,
    rows: &mut Vec<ProjectionRow>,
) -> Result<(), IndexError> {
    match item {
        ItemSourceView::BackgroundSource(_) => {
            let (mut row, id) = root_family_row("background_projection", root, ids)?;
            let boosts = item.background_boosts();
            let skills = item.background_skill_options(key);
            state(&boosts.restricted).collection(&mut row, "restricted_boosts_state");
            boolean(boosts.has_free)?.field(&mut row, "has_free_boost");
            state(&skills).collection(&mut row, "skill_training_state");
            rows.push(row);
            if let SourceFieldView::Value(attributes) = boosts.restricted {
                sets(
                    SourceFieldView::Value(attributes.as_slice()),
                    rows,
                    "background_restricted_boosts",
                    "background_id",
                    id,
                    enum_name,
                )?;
            }
            if let SourceFieldView::Value(skills) = skills {
                sets(
                    SourceFieldView::Value(skills.as_slice()),
                    rows,
                    "background_skill_training",
                    "background_id",
                    id,
                    enum_name,
                )?;
            }
        }
        ItemSourceView::AncestrySource(_) => {
            let (mut row, id) = root_family_row("ancestry_projection", root, ids)?;
            choice(item.ancestry_vision())?.field(&mut row, "vision");
            number(item.ancestry_land_speed())?.field(&mut row, "land_speed");
            number(item.ancestry_starting_hp())?.field(&mut row, "starting_hp");
            let boosts = item.ancestry_default_attributes(false);
            let flaws = item.ancestry_default_attributes(true);
            state(&boosts).collection(&mut row, "default_boosts_state");
            state(&flaws).collection(&mut row, "default_flaws_state");
            rows.push(row);
            for (attributes, table) in [
                (boosts, "ancestry_default_boosts"),
                (flaws, "ancestry_default_flaws"),
            ] {
                if let SourceFieldView::Value(attributes) = attributes {
                    sets(
                        SourceFieldView::Value(attributes.as_slice()),
                        rows,
                        table,
                        "ancestry_id",
                        id,
                        enum_name,
                    )?;
                }
            }
        }
        ItemSourceView::ClassSource(_) => {
            let (mut row, id) = root_family_row("class_projection", root, ids)?;
            let attributes = item.class_key_attributes().map(Vec::as_slice);
            state(&attributes).collection(&mut row, "key_attributes_state");
            for (_, column, proficiency) in CLASS_PROFICIENCIES {
                text(item.class_starting_proficiency(*proficiency))?.field(&mut row, column);
            }
            rows.push(row);
            sets(
                attributes,
                rows,
                "class_key_attributes",
                "class_id",
                id,
                enum_name,
            )?;
        }
        _ => {}
    }
    Ok(())
}

fn item_rows(
    i: ItemSourceView<'_>,
    root: Option<i64>,
    child: Option<i64>,
    ids: &mut ProjectionIds,
    rows: &mut Vec<ProjectionRow>,
) -> Result<(), IndexError> {
    if matches!(i, ItemSourceView::SpellSource(_)) {
        let (mut r, id) = owner_row("spell_projection", root, child, ids)?;
        number(i.spell_rank())?.field(&mut r, "rank");
        boolean(i.spell_focus())?.field(&mut r, "focus");
        boolean(i.spell_ritual())?.field(&mut r, "ritual");
        text(i.spell_casting_time())?.field(&mut r, "casting_time");
        text(i.spell_casting_form())?.field(&mut r, "casting_form");
        let cast_range = i.spell_cast_range();
        let feet = cast_range.and_then(|range| match range.feet {
            Some(feet) => SourceFieldView::Value(feet),
            None => SourceFieldView::NotApplicable,
        });
        convert(feet, |feet| Ok(Value::Integer(feet)))?.field(&mut r, "cast_range");
        boolean(cast_range.map(|range| range.touch))?.field(&mut r, "touch_range");
        choice(i.spell_defense_save())?.field(&mut r, "save");
        boolean(i.spell_defense_basic())?.field(&mut r, "basic_save");
        choice(i.spell_defense_passive())?.field(&mut r, "passive_defense");
        choice(i.spell_area_type())?.field(&mut r, "area_type");
        number(i.spell_area_size())?.field(&mut r, "area_size");
        text(i.spell_duration_text())?.field(&mut r, "duration_text");
        boolean(i.spell_sustained())?.field(&mut r, "sustained");
        state(&i.spell_traditions()).collection(&mut r, "traditions_state");
        state(&i.spell_damage()).collection(&mut r, "damage_state");
        rows.push(r);
        sets(
            i.spell_traditions(),
            rows,
            "spell_traditions",
            "spell_id",
            Local(id),
            enum_name,
        )?;
        if root.is_some()
            && let SourceFieldView::Value(damage) = i.spell_damage()
        {
            for (index, (key, entry)) in damage.entries.iter().enumerate() {
                let entry_id = ids.take()?;
                let mut r = ProjectionRow::new("spell_damage_entries");
                r.projection_id("id", Local(entry_id));
                r.projection_id("spell_id", Local(id));
                r.id("original_index", index as i64);
                r.text("original_key", key);
                choice(SourceFieldView::from(&entry.r#type))?.field(&mut r, "type");
                let kinds = SourceFieldView::from(&entry.kinds).map(Vec::as_slice);
                state(&kinds).collection(&mut r, "kinds_state");
                rows.push(r);
                sets(
                    kinds,
                    rows,
                    "spell_damage_kinds",
                    "entry_id",
                    Local(entry_id),
                    enum_name,
                )?;
            }
        }
    }
    if matches!(
        i,
        ItemSourceView::AmmoSource(_)
            | ItemSourceView::ArmorSource(_)
            | ItemSourceView::ContainerSource(_)
            | ItemSourceView::BookSource(_)
            | ItemSourceView::ConsumableSource(_)
            | ItemSourceView::EquipmentSource(_)
            | ItemSourceView::ShieldSource(_)
            | ItemSourceView::TreasureSource(_)
            | ItemSourceView::WeaponSource(_)
    ) {
        let (mut r, id) = owner_row("physical_projection", root, child, ids)?;
        float(i.physical_price_per_item_cp())?.field(&mut r, "price_per_item_cp");
        number(i.physical_bulk())?.field(&mut r, "bulk");
        text(i.physical_usage())?.field(&mut r, "usage");
        choice(i.physical_base_type())?.field(&mut r, "base_type");
        choice(i.consumable_category())?.field(&mut r, "consumable_category");
        choice(i.consumable_damage_kind())?.field(&mut r, "consumable_damage_kind");
        choice(i.consumable_damage_type())?.field(&mut r, "consumable_damage_type");
        rows.push(r);
        if matches!(i, ItemSourceView::WeaponSource(_)) {
            let mut r = ProjectionRow::new("weapon_projection");
            r.projection_id("physical_id", Local(id));
            choice(i.weapon_category())?.field(&mut r, "category");
            choice(i.weapon_group())?.field(&mut r, "weapon_group");
            choice(i.weapon_damage_type())?.field(&mut r, "damage_type");
            number(i.weapon_range_increment())?.field(&mut r, "range");
            choice(i.weapon_reload())?.field(&mut r, "reload");
            number(i.weapon_damage_dice())?.field(&mut r, "damage_dice");
            convert(i.weapon_damage_die_faces(), |value| numeric_value(&value))?
                .field(&mut r, "damage_die_faces");
            rows.push(r);
        }
        if matches!(i, ItemSourceView::ArmorSource(_)) {
            let mut r = ProjectionRow::new("armor_projection");
            r.projection_id("physical_id", Local(id));
            choice(i.armor_category())?.field(&mut r, "category");
            choice(i.armor_group())?.field(&mut r, "armor_group");
            number(i.armor_ac_bonus())?.field(&mut r, "ac_bonus");
            number(i.armor_dex_cap())?.field(&mut r, "dex_cap");
            number(i.armor_strength())?.field(&mut r, "strength");
            number(i.armor_check_penalty())?.field(&mut r, "check_penalty");
            number(i.armor_speed_penalty())?.field(&mut r, "speed_penalty");
            rows.push(r);
        }
        if matches!(i, ItemSourceView::ShieldSource(_)) {
            let mut r = ProjectionRow::new("shield_projection");
            r.projection_id("physical_id", Local(id));
            number(i.shield_hardness())?.field(&mut r, "hardness");
            number(i.shield_hp_maximum())?.field(&mut r, "hp_maximum");
            number(i.shield_ac_bonus())?.field(&mut r, "ac_bonus");
            convert(i.shield_broken_threshold(), |value| numeric_value(&value))?
                .field(&mut r, "broken_threshold");
            rows.push(r);
        }
    }
    if matches!(
        i,
        ItemSourceView::AbilitySource(_)
            | ItemSourceView::FeatSource(_)
            | ItemSourceView::CampaignFeatureSource(_)
    ) {
        let (mut r, _) = owner_row("ability_projection", root, child, ids)?;
        choice(i.action_type())?.field(&mut r, "action_type");
        number(i.action_count())?.field(&mut r, "action_count");
        match i {
            ItemSourceView::AbilitySource(_) => choice(i.action_category())?,
            ItemSourceView::FeatSource(_) => choice(i.feat_category())?,
            _ => choice(i.campaign_feature_category())?,
        }
        .field(&mut r, "category");
        rows.push(r);
    }
    if matches!(i, ItemSourceView::HeritageSource(_)) {
        let (mut r, _) = owner_row("heritage_projection", root, child, ids)?;
        text(i.heritage_ancestry_uuid())?.field(&mut r, "ancestry_uuid");
        text(i.heritage_ancestry_slug())?.field(&mut r, "ancestry_slug");
        boolean(i.heritage_versatile())?.field(&mut r, "versatile");
        rows.push(r);
    }
    if matches!(i, ItemSourceView::EffectSource(_)) {
        let (mut r, _) = owner_row("effect_projection", root, child, ids)?;
        choice(i.effect_duration_unit())?.field(&mut r, "duration_unit");
        number(
            i.effect_duration_unit()
                .and_then(|unit| i.effect_duration(*unit)),
        )?
        .field(&mut r, "duration_value");
        rows.push(r);
    }
    if matches!(i, ItemSourceView::ConditionSource(_)) {
        let (mut r, _) = owner_row("condition_projection", root, child, ids)?;
        boolean(i.condition_is_valued())?.field(&mut r, "is_valued");
        rows.push(r);
    }
    if matches!(i, ItemSourceView::DeitySource(_)) {
        let (mut r, id) = owner_row("deity_projection", root, child, ids)?;
        state(&i.deity_domains_primary()).collection(&mut r, "primary_domains_state");
        state(&i.deity_domains_alternate()).collection(&mut r, "alternate_domains_state");
        state(&i.deity_fonts()).collection(&mut r, "fonts_state");
        let weapons = i.deity_weapons().map(Vec::as_slice);
        let skills = i.deity_skills().map(Vec::as_slice);
        let sanctification = i.deity_sanctification_options();
        if root.is_some() {
            choice(i.deity_category())?.field(&mut r, "category");
            text(i.deity_sanctification_requirement())?.field(&mut r, "sanctification_requirement");
            state(&weapons).collection(&mut r, "favored_weapons_state");
            state(&skills).collection(&mut r, "divine_skills_state");
            state(&sanctification).collection(&mut r, "sanctification_options_state");
        }
        rows.push(r);
        if root.is_some() {
            sets(
                weapons,
                rows,
                "deity_favored_weapons",
                "deity_id",
                Local(id),
                enum_name,
            )?;
            sets(
                skills,
                rows,
                "deity_divine_skills",
                "deity_id",
                Local(id),
                enum_name,
            )?;
            sets(
                sanctification,
                rows,
                "deity_sanctification_options",
                "deity_id",
                Local(id),
                enum_name,
            )?;
        }
        for (kind, values) in [
            ("primary", i.deity_domains_primary()),
            ("alternate", i.deity_domains_alternate()),
        ] {
            if let SourceFieldView::Value(values) = values {
                for value in values
                    .iter()
                    .map(enum_name)
                    .collect::<Result<std::collections::BTreeSet<_>, _>>()?
                {
                    let mut r = ProjectionRow::new("deity_domains");
                    r.projection_id("deity_id", Local(id));
                    r.text("kind", kind);
                    r.text("value", value);
                    rows.push(r);
                }
            }
        }
        if let SourceFieldView::Value(values) = i.deity_fonts() {
            for value in values {
                let mut r = ProjectionRow::new("deity_fonts");
                r.projection_id("deity_id", Local(id));
                r.text("value", *value);
                rows.push(r);
            }
        }
    }
    Ok(())
}

pub(crate) fn projection_diagnostics(
    record: &atlas_record::source_record::SourceBackedRecord,
) -> Vec<crate::SourceArtifactDiagnostic> {
    fn capture<T>(
        view: SourceFieldView<'_, T>,
        chain: &[OwnedContentLocator],
        out: &mut Vec<crate::SourceArtifactDiagnostic>,
    ) {
        if let SourceFieldView::ProjectionInvalid {
            source_path,
            reason,
        } = view
        {
            out.push(crate::SourceArtifactDiagnostic {
                source_diagnostic: None,
                owners: Some(chain.to_vec()),
                field: Some(source_path.into()),
                stage: "projection".into(),
                code: "unavailable_query_projection".into(),
                message: reason.into(),
            });
        }
    }
    fn item(
        view: ItemSourceView<'_>,
        chain: &[OwnedContentLocator],
        out: &mut Vec<crate::SourceArtifactDiagnostic>,
    ) {
        capture(view.physical_price_per_item_cp(), chain, out);
        capture(view.spell_area_size(), chain, out);
    }
    let mut result = vec![];
    if let SourceNodeView::Item(view) = SourceNodeView::from(record.source()) {
        item(view, &[], &mut result);
    }
    record.visit_immediate_actor_items(|_, owner, view| {
        item(view, std::slice::from_ref(owner), &mut result)
    });
    result
}
