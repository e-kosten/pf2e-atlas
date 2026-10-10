//! Named query accelerators derived exclusively from the checked source DTO.
use crate::{
    IndexError, input::SourceArtifactRecordInput, numeric::numeric_value,
    persistence::ProjectionRow,
};
use atlas_record::source_content::OwnedContentLocator;
use atlas_record::source_record::{
    ActorIwrKind, ActorIwrType, ActorQueryView, ActorSave, ItemSourceView, SourceFieldView,
    SourceNodeView, SourceQueryView,
};
use rusqlite::types::Value;
use serde::Serialize;
use serde_json::Number;

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
fn state<T>(view: SourceFieldView<'_, T>) -> SqlFact {
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
    r.id("record_id", record_id);
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
    sets(
        q.traits(),
        &mut rows,
        "record_traits",
        "record_id",
        record_id,
        |v| Ok(v.clone()),
    )?;
    match q.source {
        SourceNodeView::Item(i) => item_rows(i, Some(record_id), None, ids, &mut rows)?,
        SourceNodeView::Actor(_) => {
            actor_rows(q.actor(), record_id, ids, &mut rows)?;
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
    state(q.traits()).collection(row, "traits_state");
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
    r.id("id", id);
    r.id("record_id", root);
    r.id(
        "original_index",
        i64::try_from(index).map_err(|e| IndexError::Invalid(e.to_string()))?,
    );
    r.text("owner_selector_json", serde_json::to_string(owner)?);
    text(q.source.id().map(String::as_str))?.field(&mut r, "authored_id");
    common_fields(q, &mut r)?;
    rows.push(r);
    sets(q.traits(), rows, "actor_item_traits", "item_id", id, |v| {
        Ok(v.clone())
    })?;
    item_rows(item, None, Some(id), ids, rows)
}
fn sets<T>(
    v: SourceFieldView<'_, &[T]>,
    rows: &mut Vec<ProjectionRow>,
    table: &'static str,
    owner: &'static str,
    id: i64,
    to_string: impl Fn(&T) -> Result<String, IndexError>,
) -> Result<(), IndexError> {
    if let SourceFieldView::Value(values) = v {
        let values = values
            .iter()
            .map(to_string)
            .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
        for value in values {
            let mut r = ProjectionRow::new(table);
            r.id(owner, id);
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
    root: i64,
    ids: &mut ProjectionIds,
    rows: &mut Vec<ProjectionRow>,
) -> Result<(), IndexError> {
    let mut r = ProjectionRow::new("actor_projection");
    r.id("record_id", root);
    number(a.armor_class())?.field(&mut r, "armor_class");
    number(a.hp_maximum())?.field(&mut r, "hp_maximum");
    number(a.hazard_hardness())?.field(&mut r, "hardness");
    number(a.save(ActorSave::Fortitude))?.field(&mut r, "fortitude");
    number(a.save(ActorSave::Reflex))?.field(&mut r, "reflex");
    number(a.save(ActorSave::Will))?.field(&mut r, "will");
    number(a.perception())?.field(&mut r, "perception");
    number(a.land_speed())?.field(&mut r, "land_speed");
    boolean(a.hazard_complexity())?.field(&mut r, "complexity");
    state(a.items()).collection(&mut r, "items_state");
    state(a.languages()).collection(&mut r, "languages_state");
    state(a.speeds()).collection(&mut r, "speeds_state");
    state(a.senses()).collection(&mut r, "senses_state");
    for (kind, name) in [
        (ActorIwrKind::Immunity, "immunities_state"),
        (ActorIwrKind::Weakness, "weaknesses_state"),
        (ActorIwrKind::Resistance, "resistances_state"),
    ] {
        state(a.iwr(kind)).collection(&mut r, name);
    }
    rows.push(r);
    sets(
        a.languages(),
        rows,
        "actor_languages",
        "record_id",
        root,
        enum_name,
    )?;
    if let SourceFieldView::Value(speeds) = a.speeds() {
        for (index, speed) in speeds.iter().enumerate() {
            let mut r = ProjectionRow::new("actor_speeds");
            r.id("id", ids.take()?);
            r.id("record_id", root);
            r.id("original_index", index as i64);
            choice(speed.speed_type())?.field(&mut r, "type");
            number(speed.value())?.field(&mut r, "value");
            rows.push(r);
        }
    }
    if let SourceFieldView::Value(senses) = a.senses() {
        for (index, sense) in senses.iter().enumerate() {
            let mut r = ProjectionRow::new("actor_senses");
            r.id("id", ids.take()?);
            r.id("record_id", root);
            r.id("original_index", index as i64);
            choice(SourceFieldView::from(&sense.r#type))?.field(&mut r, "type");
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
                r.id("entry_id", ids.take()?);
                r.id("record_id", root);
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
    r.id("id", id);
    r.push(
        "root_record_id",
        root.map(Value::Integer).unwrap_or(Value::Null),
    );
    r.push(
        "actor_item_id",
        child.map(Value::Integer).unwrap_or(Value::Null),
    );
    Ok((r, id))
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
        choice(i.spell_defense_save())?.field(&mut r, "save");
        boolean(i.spell_defense_basic())?.field(&mut r, "basic_save");
        choice(i.spell_defense_passive())?.field(&mut r, "passive_defense");
        choice(i.spell_area_type())?.field(&mut r, "area_type");
        number(i.spell_area_size())?.field(&mut r, "area_size");
        text(i.spell_duration_text())?.field(&mut r, "duration_text");
        boolean(i.spell_sustained())?.field(&mut r, "sustained");
        state(i.spell_traditions()).collection(&mut r, "traditions_state");
        state(i.spell_damage()).collection(&mut r, "damage_state");
        rows.push(r);
        sets(
            i.spell_traditions(),
            rows,
            "spell_traditions",
            "spell_id",
            id,
            enum_name,
        )?;
        if root.is_some()
            && let SourceFieldView::Value(damage) = i.spell_damage()
        {
            for (index, (key, entry)) in damage.entries.iter().enumerate() {
                let entry_id = ids.take()?;
                let mut r = ProjectionRow::new("spell_damage_entries");
                r.id("id", entry_id);
                r.id("spell_id", id);
                r.id("original_index", index as i64);
                r.text("original_key", key);
                choice(SourceFieldView::from(&entry.r#type))?.field(&mut r, "type");
                let kinds = SourceFieldView::from(&entry.kinds).map(Vec::as_slice);
                state(kinds).collection(&mut r, "kinds_state");
                rows.push(r);
                sets(
                    kinds,
                    rows,
                    "spell_damage_kinds",
                    "entry_id",
                    entry_id,
                    enum_name,
                )?;
            }
        }
    }
    if matches!(
        i,
        ItemSourceView::ArmorSource(_)
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
        choice(i.consumable_category())?.field(&mut r, "consumable_category");
        rows.push(r);
        if matches!(i, ItemSourceView::WeaponSource(_)) {
            let mut r = ProjectionRow::new("weapon_projection");
            r.id("physical_id", id);
            choice(i.weapon_category())?.field(&mut r, "category");
            choice(i.weapon_group())?.field(&mut r, "weapon_group");
            choice(i.weapon_damage_type())?.field(&mut r, "damage_type");
            number(i.weapon_range_increment())?.field(&mut r, "range");
            choice(i.weapon_reload())?.field(&mut r, "reload");
            rows.push(r);
        }
        if matches!(i, ItemSourceView::ArmorSource(_)) {
            let mut r = ProjectionRow::new("armor_projection");
            r.id("physical_id", id);
            choice(i.armor_category())?.field(&mut r, "category");
            number(i.armor_ac_bonus())?.field(&mut r, "ac_bonus");
            number(i.armor_dex_cap())?.field(&mut r, "dex_cap");
            rows.push(r);
        }
        if matches!(i, ItemSourceView::ShieldSource(_)) {
            let mut r = ProjectionRow::new("shield_projection");
            r.id("physical_id", id);
            number(i.shield_hardness())?.field(&mut r, "hardness");
            number(i.shield_hp_maximum())?.field(&mut r, "hp_maximum");
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
        state(i.deity_domains_primary()).collection(&mut r, "primary_domains_state");
        state(i.deity_domains_alternate()).collection(&mut r, "alternate_domains_state");
        state(i.deity_fonts()).collection(&mut r, "fonts_state");
        rows.push(r);
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
                    r.id("deity_id", id);
                    r.text("kind", kind);
                    r.text("value", value);
                    rows.push(r);
                }
            }
        }
        if let SourceFieldView::Value(values) = i.deity_fonts() {
            for value in values {
                let mut r = ProjectionRow::new("deity_fonts");
                r.id("deity_id", id);
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
