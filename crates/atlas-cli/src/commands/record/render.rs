use atlas_domain::DetailLevel;
use atlas_record::{
    CreatureActionCostJson, CreatureAvailabilityFieldJson, CreatureAvailabilityJson,
    CreatureContentJson, CreatureDamageJson, CreatureRollJson, CreatureSpellJson,
    CreatureUnmodeledSkillAvailabilityJson, PresentationContent, PresentationContentBlock,
    PresentationInline, RecordBlockJson, RecordEditionCounterpartLookupJson,
    RecordEditionCounterpartRoleJson, RecordEditionStatusJson, RecordJson, RecordPresentationJson,
    RecordRelationshipDirectionJson, RecordRelationshipLookupJson,
};

use crate::terminal::TerminalStyle;

const DEFAULT_WIDTH: usize = 80;
const MIN_WIDTH: usize = 20;

pub(super) fn effective_width() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|width| *width >= MIN_WIDTH)
        .unwrap_or(DEFAULT_WIDTH)
}

pub(super) fn render_record(
    record: &RecordJson,
    detail: DetailLevel,
    width: usize,
    style: TerminalStyle,
) -> String {
    if detail == DetailLevel::Summary {
        return format!("{}\t{}\t{}\n", record.key, record.name, record.kind);
    }

    let mut out = Writer::new(width.max(MIN_WIDTH), style);
    out.header(record);
    match &record.presentation {
        RecordPresentationJson::Creature {
            teaser,
            size,
            adjustment,
            initiative,
            abilities,
            defenses,
            perception,
            languages,
            skills,
            movement,
            resources,
            strikes,
            actions,
            spellcasting,
            rituals,
            equipment,
            lore,
            content,
            relationships: _,
            provenance: _,
            edition,
            record_relationships,
            availability,
            unmodeled_skill_availability,
            availability_evidence: _,
        } => {
            if detail != DetailLevel::Description {
                out.classification(record, size.as_deref(), adjustment.as_deref());
                if matches!(
                    detail,
                    DetailLevel::Preview | DetailLevel::Standard | DetailLevel::Full
                ) {
                    out.scan(
                        initiative.as_ref().map(|value| value.statistic.as_str()),
                        abilities.as_ref(),
                        defenses.as_ref(),
                        perception.as_ref(),
                        languages.as_deref(),
                        skills.as_deref(),
                        movement.as_ref(),
                        resources.as_deref(),
                    );
                    out.occurrences(
                        detail,
                        strikes.as_deref(),
                        actions.as_deref(),
                        spellcasting.as_ref(),
                        rituals.as_ref(),
                        equipment.as_deref(),
                        lore.as_deref(),
                    );
                }
                if let Some(teaser) = teaser.as_deref() {
                    out.section("Description");
                    out.paragraph(teaser, 2);
                }
            }

            if matches!(detail, DetailLevel::Description | DetailLevel::Full) {
                out.owned_content(
                    detail,
                    content.as_deref(),
                    strikes.as_deref(),
                    actions.as_deref(),
                    spellcasting.as_ref(),
                    equipment.as_deref(),
                    lore.as_deref(),
                );
            }

            if matches!(detail, DetailLevel::Standard | DetailLevel::Full) {
                out.relationships(record_relationships.as_ref());
            }
            if detail == DetailLevel::Full {
                out.source_and_edition(record, edition.as_ref());
            }
            out.availability(detail, availability, unmodeled_skill_availability);
        }
        RecordPresentationJson::Unmigrated { sections, .. } => out.generic_sections(sections),
    }
    out.finish()
}

struct Writer {
    width: usize,
    style: TerminalStyle,
    lines: Vec<String>,
}

impl Writer {
    fn new(width: usize, style: TerminalStyle) -> Self {
        Self {
            width,
            style,
            lines: Vec::new(),
        }
    }

    fn finish(mut self) -> String {
        while self.lines.last().is_some_and(String::is_empty) {
            self.lines.pop();
        }
        self.lines.push(String::new());
        self.lines.join("\n")
    }

    fn header(&mut self, record: &RecordJson) {
        self.lines.push(self.style.label(&record.name));
        let pack = record
            .source
            .as_ref()
            .and_then(|source| source.pack.as_ref())
            .map(|pack| pack.label.as_str());
        let subordinate = match pack {
            Some(pack) => format!("Type: {} · Pack: {pack}", record.kind),
            None => format!("Type: {}", record.kind),
        };
        self.wrapped("  ", &subordinate, 2, 2);
        self.field("Key", self.style.metadata(&record.key), 2);
    }

    fn section(&mut self, title: &str) {
        if !self.lines.is_empty() && !self.lines.last().is_some_and(String::is_empty) {
            self.lines.push(String::new());
        }
        self.heading(title, 0);
    }

    fn heading(&mut self, title: &str, indent: usize) {
        let start = self.lines.len();
        self.wrapped(&" ".repeat(indent), title, indent, indent);
        for line in &mut self.lines[start..] {
            let (prefix, text) = line.split_at(indent.min(line.len()));
            *line = format!("{prefix}{}", self.style.label(text));
        }
    }

    fn field(&mut self, label: &str, value: impl AsRef<str>, indent: usize) {
        let value = value.as_ref().trim();
        if value.is_empty() {
            return;
        }
        let plain_prefix = format!("{}{}: ", " ".repeat(indent), label);
        let styled_prefix = format!("{}{}: ", " ".repeat(indent), self.style.label(label));
        self.wrapped(&styled_prefix, value, indent, plain_prefix.len());
    }

    fn bullet(&mut self, value: impl AsRef<str>, indent: usize) {
        let prefix = format!("{}- ", " ".repeat(indent));
        self.wrapped(&prefix, value.as_ref(), indent, prefix.len());
    }

    fn paragraph(&mut self, value: &str, indent: usize) {
        for (index, paragraph) in value.split('\n').enumerate() {
            if index > 0 {
                self.lines.push(String::new());
            }
            self.wrapped(&" ".repeat(indent), paragraph, indent, indent);
        }
    }

    fn wrapped(&mut self, first_prefix: &str, value: &str, indent: usize, prefix_width: usize) {
        let continuation = " ".repeat(indent + prefix_width.saturating_sub(indent));
        let first_available = self.width.saturating_sub(prefix_width).max(1);
        let continuation_available = self.width.saturating_sub(continuation.len()).max(1);
        let mut words = value.split_whitespace().peekable();
        if words.peek().is_none() {
            return;
        }
        let mut current = String::new();
        let mut available = first_available;
        let mut prefix = first_prefix.to_string();
        if words
            .peek()
            .is_some_and(|word| word.len() > first_available && prefix_width > indent)
        {
            self.lines.push(first_prefix.trim_end().to_string());
            prefix = continuation.clone();
            available = continuation_available;
        }
        for word in words {
            let needed = word.len() + usize::from(!current.is_empty());
            if !current.is_empty() && current.len() + needed > available {
                self.lines.push(format!("{prefix}{current}"));
                prefix = continuation.clone();
                available = continuation_available;
                current.clear();
            }
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
        self.lines.push(format!("{prefix}{current}"));
    }

    fn classification(
        &mut self,
        record: &RecordJson,
        size: Option<&str>,
        adjustment: Option<&str>,
    ) {
        self.section("Classification");
        if let Some(level) = record.level {
            self.field("Level", level.to_string(), 2);
        }
        if let Some(rarity) = &record.rarity {
            self.field("Rarity", rarity, 2);
        }
        if !record.traits.is_empty() {
            self.field("Traits", record.traits.join(", "), 2);
        }
        if let Some(size) = size {
            self.field("Size", size, 2);
        }
        if let Some(adjustment) = adjustment {
            self.field("Adjustment", adjustment, 2);
        }
        if let Some(source) = &record.source
            && let Some(title) = source.publication_title.as_deref()
        {
            self.field("Source", title, 2);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn scan(
        &mut self,
        initiative: Option<&str>,
        abilities: Option<&atlas_record::CreatureAbilitiesJson>,
        defenses: Option<&atlas_record::CreatureDefensesJson>,
        perception: Option<&atlas_record::CreaturePerceptionJson>,
        languages: Option<&[String]>,
        skills: Option<&[atlas_record::CreatureSkillJson]>,
        movement: Option<&atlas_record::CreatureMovementJson>,
        resources: Option<&[atlas_record::CreatureResourceJson]>,
    ) {
        if let Some(defenses) = defenses {
            self.section("Defenses");
            if let Some(ac) = &defenses.ac {
                self.optional_number_with_detail("AC", ac.value, ac.details.as_deref());
            }
            if let Some(hp) = &defenses.hp {
                let mut values = Vec::new();
                if let Some(value) = hp.value {
                    values.push(format!("current {value}"));
                }
                if let Some(value) = hp.maximum {
                    values.push(format!("maximum {value}"));
                }
                if let Some(value) = hp.temporary {
                    values.push(format!("temporary {value}"));
                }
                if let Some(value) = hp.temporary_maximum {
                    values.push(format!("temporary maximum {value}"));
                }
                if let Some(details) = hp.details.as_ref().filter(|value| !value.is_empty()) {
                    values.push(details.clone());
                }
                self.field("HP", values.join("; "), 2);
            }
            if let Some(value) = defenses.hardness {
                self.field("Hardness", value.to_string(), 2);
            }
            if let Some(shield) = &defenses.shield {
                let values = [
                    shield.hardness.map(|v| format!("Hardness {v}")),
                    shield.broken_threshold.map(|v| format!("BT {v}")),
                    shield.maximum_hit_points.map(|v| format!("HP {v}")),
                    shield.armor_class_bonus.map(|v| format!("AC bonus {v:+}")),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
                self.field("Shield", values.join("; "), 2);
            }
            if let Some(saves) = &defenses.saves {
                for (label, save) in [
                    ("Fortitude", saves.fortitude.as_ref()),
                    ("Reflex", saves.reflex.as_ref()),
                    ("Will", saves.will.as_ref()),
                ] {
                    if let Some(save) = save {
                        self.optional_number_with_detail(
                            label,
                            save.value,
                            save.details.as_deref(),
                        );
                    }
                }
            }
            if let Some(note) = defenses
                .all_saves_note
                .as_ref()
                .filter(|value| !value.is_empty())
            {
                self.field("All saves", note, 2);
            }
            for (label, values) in [
                ("Immunities", defenses.immunities.as_deref()),
                ("Resistances", defenses.resistances.as_deref()),
                ("Weaknesses", defenses.weaknesses.as_deref()),
            ] {
                if let Some(values) = values.filter(|values| !values.is_empty()) {
                    self.field(
                        label,
                        values
                            .iter()
                            .map(|value| {
                                let mut text = value.iwr_type.clone();
                                if let Some(amount) = value.value {
                                    text.push_str(&format!(" {amount}"));
                                }
                                if !value.exceptions.is_empty() {
                                    text.push_str(&format!(
                                        " (except {})",
                                        value.exceptions.join(", ")
                                    ));
                                }
                                text
                            })
                            .collect::<Vec<_>>()
                            .join(", "),
                        2,
                    );
                }
            }
        }

        if perception.is_some()
            || initiative.is_some()
            || languages.is_some_and(|values| !values.is_empty())
            || skills.is_some_and(|values| !values.is_empty())
            || abilities.is_some()
        {
            self.section("Awareness and abilities");
        }
        if let Some(perception) = perception {
            let mut values = perception
                .modifier
                .map(|value| vec![format!("{value:+}")])
                .unwrap_or_default();
            if let Some(details) = perception
                .details
                .as_ref()
                .filter(|value| !value.is_empty())
            {
                values.push(details.clone());
            }
            if let Some(senses) = &perception.senses {
                values.extend(senses.iter().map(|sense| {
                    let mut value = sense.kind.clone();
                    if let Some(acuity) = &sense.acuity {
                        value.push_str(&format!(" ({acuity})"));
                    }
                    if let Some(range) = sense.range_feet {
                        value.push_str(&format!(" {range} feet"));
                    }
                    value
                }));
            }
            self.field("Perception", values.join("; "), 2);
        }
        if let Some(initiative) = initiative {
            self.field("Initiative", initiative, 2);
        }
        if let Some(languages) = languages.filter(|values| !values.is_empty()) {
            self.field("Languages", languages.join(", "), 2);
        }
        if let Some(abilities) = abilities {
            let values = [
                ("Str", abilities.strength),
                ("Dex", abilities.dexterity),
                ("Con", abilities.constitution),
                ("Int", abilities.intelligence),
                ("Wis", abilities.wisdom),
                ("Cha", abilities.charisma),
            ]
            .into_iter()
            .filter_map(|(label, value)| value.map(|value| format!("{label} {value:+}")))
            .collect::<Vec<_>>();
            self.field("Abilities", values.join(", "), 2);
        }
        if let Some(skills) = skills.filter(|values| !values.is_empty()) {
            for skill in skills.iter().filter(|skill| skill.unmodeled.is_none()) {
                let mut value = skill
                    .modifier
                    .map(|modifier| format!("{modifier:+}"))
                    .unwrap_or_default();
                if let Some(note) = skill.note.as_ref().filter(|value| !value.is_empty()) {
                    value.push_str(&format!("; {note}"));
                }
                self.field(&skill.label, &value, 2);
                for variant in &skill.variants {
                    let mut variant_text =
                        variant.label.clone().unwrap_or_else(|| "Variant".into());
                    if let Some(modifier) = variant.modifier {
                        variant_text.push_str(&format!(" {modifier:+}"));
                    }
                    if !variant.predicates.is_empty() {
                        variant_text.push_str(&format!(" when {}", variant.predicates.join(", ")));
                    }
                    self.bullet(variant_text, 4);
                }
            }
        }
        if let Some(movement) = movement.filter(|value| !value.modes.is_empty()) {
            self.section("Movement");
            for mode in &movement.modes {
                let mut value = mode
                    .value_feet
                    .map(|feet| format!("{feet} feet"))
                    .unwrap_or_default();
                if let Some(details) = mode.details.as_ref().filter(|value| !value.is_empty()) {
                    value.push_str(&format!("; {details}"));
                }
                self.field(mode.label.as_deref().unwrap_or(&mode.mode), value, 2);
            }
        }
        if let Some(resources) =
            resources.filter(|values| values.iter().any(|resource| resource.maximum.is_some()))
        {
            self.section("Resources");
            for resource in resources {
                if let Some(maximum) = resource.maximum {
                    self.field(&resource.label, format!("maximum {maximum}"), 2);
                }
            }
        }
    }

    fn optional_number_with_detail(
        &mut self,
        label: &str,
        value: Option<i64>,
        detail: Option<&str>,
    ) {
        let mut values = value
            .map(|value| vec![value.to_string()])
            .unwrap_or_default();
        if let Some(detail) = detail.filter(|value| !value.is_empty()) {
            values.push(detail.to_string());
        }
        self.field(label, values.join("; "), 2);
    }

    #[allow(clippy::too_many_arguments)]
    fn occurrences(
        &mut self,
        detail: DetailLevel,
        strikes: Option<&[atlas_record::CreatureStrikeJson]>,
        actions: Option<&[atlas_record::CreatureActionJson]>,
        spellcasting: Option<&atlas_record::CreatureSpellcastingJson>,
        rituals: Option<&atlas_record::CreatureRitualsJson>,
        equipment: Option<&[atlas_record::CreatureEquipmentJson]>,
        lore: Option<&[atlas_record::CreatureLoreJson]>,
    ) {
        let complete = matches!(detail, DetailLevel::Standard | DetailLevel::Full);
        if let Some(strikes) = strikes.filter(|values| !values.is_empty()) {
            self.section("Strikes");
            for strike in strikes {
                self.occurrence_heading(
                    &strike.label,
                    &strike.action_cost,
                    strike.target_record_key.as_deref(),
                );
                if complete {
                    self.list_field("Traits", &strike.traits, 4);
                    self.list_field("Attack effects", &strike.attack_effects, 4);
                    self.rolls(strike.rolls.as_deref());
                    self.damage(strike.damage.as_deref(), 4);
                }
            }
        }
        if let Some(actions) = actions.filter(|values| !values.is_empty()) {
            self.section("Activities");
            for action in actions {
                self.occurrence_heading(
                    &action.label,
                    &action.action_cost,
                    action.target_record_key.as_deref(),
                );
                if complete {
                    self.list_field("Traits", &action.traits, 4);
                    for (label, value) in [
                        ("Category", action.category.as_deref()),
                        ("Requirements", action.requirements.as_deref()),
                        ("Cost", action.cost.as_deref()),
                        (
                            "Self effect",
                            action
                                .self_effect_label
                                .as_deref()
                                .or(action.self_effect.as_deref()),
                        ),
                    ] {
                        if let Some(value) = value {
                            self.field(label, value, 4);
                        }
                    }
                    if let Some(display) = action
                        .frequency
                        .as_ref()
                        .and_then(|frequency| frequency.display.as_deref())
                    {
                        self.field("Frequency", display, 4);
                    }
                    self.rolls(action.rolls.as_deref());
                    self.damage(action.damage.as_deref(), 4);
                }
            }
        }
        if let Some(spellcasting) = spellcasting
            .filter(|value| !value.entries.is_empty() || !value.standalone_spells.is_empty())
        {
            self.section("Spellcasting");
            for entry in &spellcasting.entries {
                let entry_context = [entry.preparation.as_deref(), entry.tradition.as_deref()]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(" · ");
                self.bullet(
                    reference_row(
                        &entry.label,
                        (!entry_context.is_empty()).then_some(entry_context.as_str()),
                        entry.target_record_key.as_deref(),
                    ),
                    2,
                );
                if complete {
                    if let Some(attack) = entry.attack {
                        self.field("Attack", format!("{attack:+}"), 4);
                    }
                    if let Some(dc) = entry.dc {
                        self.field("DC", dc.to_string(), 4);
                    }
                }
                for group in spell_rank_groups(&entry.spells, entry.slots.as_deref()) {
                    let heading = match (group.rank, group.maximum) {
                        (Some(rank), Some(capacity)) => {
                            format!("Rank {rank} · {capacity} slots")
                        }
                        (Some(rank), None) => format!("Rank {rank}"),
                        (None, _) => "Other spells".to_string(),
                    };
                    self.heading(&heading, 4);
                    for spell in group.spells {
                        self.spell(spell, complete, 6);
                    }
                }
            }
            for spell in &spellcasting.standalone_spells {
                self.spell(spell, complete, 2);
            }
        }
        if let Some(rituals) = rituals.and_then(|value| value.difficulty_class) {
            self.section("Rituals");
            self.field("Actor DC", rituals.to_string(), 2);
        }
        if let Some(equipment) = equipment.filter(|values| !values.is_empty()) {
            self.section("Equipment");
            for item in equipment {
                self.bullet(
                    reference_row(&item.label, None, item.target_record_key.as_deref()),
                    2,
                );
                if complete {
                    self.list_field("Traits", &item.traits, 4);
                    if let Some(level) = item.level {
                        self.field("Level", level.to_string(), 4);
                    }
                    if let Some(usage) = &item.usage {
                        self.field("Usage", usage, 4);
                    }
                    if let Some(quantity) = item.quantity {
                        self.field("Quantity", quantity.to_string(), 4);
                    }
                }
            }
        }
        if let Some(lore) = lore.filter(|values| !values.is_empty()) {
            self.section("Lore");
            for item in lore {
                let modifier = item.modifier.map(|value| format!(" {value:+}"));
                self.bullet(
                    reference_row(
                        &item.label,
                        modifier.as_deref(),
                        item.target_record_key.as_deref(),
                    ),
                    2,
                );
            }
        }
    }

    fn occurrence_heading(
        &mut self,
        label: &str,
        cost: &CreatureActionCostJson,
        target: Option<&str>,
    ) {
        let cost = action_cost(cost);
        self.bullet(reference_row(label, Some(&cost), target), 2);
    }

    fn spell(&mut self, spell: &CreatureSpellJson, complete: bool, indent: usize) {
        let context = action_cost(&spell.action_cost);
        self.bullet(
            reference_row(
                &spell.label,
                Some(&context),
                spell.target_record_key.as_deref(),
            ),
            indent,
        );
        if complete {
            self.list_field("Traits", &spell.traits, indent + 2);
            for (label, value) in [
                ("Requirements", spell.requirements.as_deref()),
                ("Cost", spell.cost.as_deref()),
                ("Target", spell.target.as_deref()),
                ("Range", spell.range.as_deref()),
                ("Time", spell.time.as_deref()),
            ] {
                if let Some(value) = value {
                    self.field(label, value, indent + 2);
                }
            }
            self.damage(spell.damage.as_deref(), indent + 2);
        }
    }

    fn list_field(&mut self, label: &str, values: &[String], indent: usize) {
        if !values.is_empty() {
            self.field(label, values.join(", "), indent);
        }
    }

    fn rolls(&mut self, rolls: Option<&[CreatureRollJson]>) {
        if let Some(rolls) = rolls.filter(|values| !values.is_empty()) {
            for roll in rolls {
                let value = roll
                    .value
                    .map(|value| format!(" {value:+}"))
                    .unwrap_or_default();
                self.field(&roll.label, format!("{}{}", roll.kind, value), 4);
            }
        }
    }

    fn damage(&mut self, damage: Option<&[CreatureDamageJson]>, indent: usize) {
        if let Some(damage) = damage.filter(|values| !values.is_empty()) {
            for row in damage {
                let mut values = Vec::new();
                if let Some(formula) = &row.formula {
                    values.push(formula.clone());
                }
                if let Some(kind) = &row.damage_type {
                    values.push(kind.clone());
                }
                if let Some(category) = &row.category {
                    values.push(category.clone());
                }
                values.extend(row.kinds.iter().map(|value| (*value).to_string()));
                self.field("Damage", values.join(" "), indent);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn owned_content<'a>(
        &mut self,
        detail: DetailLevel,
        record_content: Option<&'a [CreatureContentJson]>,
        strikes: Option<&'a [atlas_record::CreatureStrikeJson]>,
        actions: Option<&'a [atlas_record::CreatureActionJson]>,
        spellcasting: Option<&'a atlas_record::CreatureSpellcastingJson>,
        equipment: Option<&'a [atlas_record::CreatureEquipmentJson]>,
        lore: Option<&'a [atlas_record::CreatureLoreJson]>,
    ) {
        let mut documents = record_content
            .into_iter()
            .flatten()
            .map(|document| (natural_content_heading(document, "Description"), document))
            .collect::<Vec<_>>();
        if detail == DetailLevel::Full {
            for row in strikes.into_iter().flatten() {
                documents.extend(
                    row.content
                        .iter()
                        .flatten()
                        .map(|document| (row.label.as_str(), document)),
                );
            }
            for row in actions.into_iter().flatten() {
                documents.extend(
                    row.content
                        .iter()
                        .flatten()
                        .map(|document| (row.label.as_str(), document)),
                );
            }
            if let Some(spellcasting) = spellcasting {
                for entry in &spellcasting.entries {
                    documents.extend(
                        entry
                            .content
                            .iter()
                            .flatten()
                            .map(|document| (entry.label.as_str(), document)),
                    );
                    for spell in &entry.spells {
                        documents.extend(
                            spell
                                .content
                                .iter()
                                .flatten()
                                .map(|document| (spell.label.as_str(), document)),
                        );
                    }
                }
                for spell in &spellcasting.standalone_spells {
                    documents.extend(
                        spell
                            .content
                            .iter()
                            .flatten()
                            .map(|document| (spell.label.as_str(), document)),
                    );
                }
            }
            for row in equipment.into_iter().flatten() {
                documents.extend(
                    row.content
                        .iter()
                        .flatten()
                        .map(|document| (row.label.as_str(), document)),
                );
            }
            for row in lore.into_iter().flatten() {
                documents.extend(
                    row.content
                        .iter()
                        .flatten()
                        .map(|document| (row.label.as_str(), document)),
                );
            }
        }
        documents
            .sort_by_key(|(_, document)| (document.authored_order, document.content_key.as_str()));
        for (heading, document) in documents {
            self.section(heading);
            self.render_content(&document.document, 2);
        }
    }

    fn render_content(&mut self, content: &PresentationContent, indent: usize) {
        for block in &content.blocks {
            match block {
                PresentationContentBlock::Heading { text, .. } => {
                    self.field("Heading", text, indent)
                }
                PresentationContentBlock::Paragraph { spans } => {
                    self.paragraph(&inline_text(spans), indent)
                }
                PresentationContentBlock::List { ordered, items } => {
                    for (index, item) in items.iter().enumerate() {
                        let marker = if *ordered {
                            format!("{}.", index + 1)
                        } else {
                            "-".into()
                        };
                        let text = item
                            .blocks
                            .iter()
                            .filter_map(block_plain_text)
                            .collect::<Vec<_>>()
                            .join(" ");
                        let prefix = format!("{}{} ", " ".repeat(indent), marker);
                        self.wrapped(&prefix, &text, indent, prefix.len());
                    }
                }
                PresentationContentBlock::Table { caption, rows } => {
                    if let Some(caption) = caption {
                        self.field("Table", caption, indent);
                    }
                    for (row_index, row) in rows.iter().enumerate() {
                        self.field("Row", (row_index + 1).to_string(), indent);
                        for (cell_index, cell) in row.cells.iter().enumerate() {
                            let text = cell
                                .blocks
                                .iter()
                                .filter_map(block_plain_text)
                                .collect::<Vec<_>>()
                                .join(" ");
                            self.field(&format!("Column {}", cell_index + 1), text, indent + 2);
                        }
                    }
                }
                PresentationContentBlock::Rule => {
                    self.lines
                        .push(format!("{}{}", " ".repeat(indent), self.style.separator()))
                }
            }
        }
    }

    fn relationships(&mut self, canonical: Option<&RecordRelationshipLookupJson>) {
        let canonical_rows = match canonical {
            Some(RecordRelationshipLookupJson::Verified { relationships }) => {
                relationships.as_slice()
            }
            _ => &[],
        };
        if canonical_rows.is_empty() {
            return;
        }
        self.section("Relationships");
        for row in canonical_rows {
            let direction = match row.direction {
                RecordRelationshipDirectionJson::Reference => "reference",
                RecordRelationshipDirectionJson::Backlink => "backlink",
            };
            self.bullet(
                format!("{direction}: {} ({})", row.label, row.target_record_key),
                2,
            );
        }
    }

    fn source_and_edition(
        &mut self,
        record: &RecordJson,
        edition: Option<&atlas_record::RecordEditionContextJson>,
    ) {
        if edition.is_none() && record.source.is_none() {
            return;
        }
        self.section("Source and edition");
        if let Some(source) = &record.source {
            if let Some(title) = &source.publication_title {
                self.field("Publication", title, 2);
            }
            if let Some(pack) = &source.pack {
                self.field("Pack", format!("{} ({})", pack.label, pack.name), 2);
            }
        }
        if let Some(edition) = edition {
            self.field(
                "Edition",
                match edition.status {
                    RecordEditionStatusJson::Legacy => "legacy",
                    RecordEditionStatusJson::Remaster => "remaster",
                },
                2,
            );
            match &edition.counterpart_lookup {
                RecordEditionCounterpartLookupJson::NotPerformed => {
                    self.field("Counterparts", "lookup not performed", 2)
                }
                RecordEditionCounterpartLookupJson::Verified { counterparts }
                    if counterparts.is_empty() =>
                {
                    self.field("Counterparts", "verified: none", 2)
                }
                RecordEditionCounterpartLookupJson::Verified { counterparts } => {
                    for counterpart in counterparts {
                        let role = match counterpart.role {
                            RecordEditionCounterpartRoleJson::LegacyCounterpart => {
                                "legacy counterpart"
                            }
                            RecordEditionCounterpartRoleJson::RemasteredCounterpart => {
                                "remastered counterpart"
                            }
                        };
                        self.field(
                            "Counterpart",
                            format!("{role}: {} ({})", counterpart.title, counterpart.record_key),
                            2,
                        );
                    }
                }
            }
        }
    }

    fn availability(
        &mut self,
        detail: DetailLevel,
        availability: &[CreatureAvailabilityJson],
        unmodeled_skills: &[CreatureUnmodeledSkillAvailabilityJson],
    ) {
        if detail == DetailLevel::Preview {
            return;
        }
        let rows = availability
            .iter()
            .filter(|row| {
                detail != DetailLevel::Description
                    || row.field == CreatureAvailabilityFieldJson::ContentAssociation
            })
            .collect::<Vec<_>>();
        if rows.is_empty() && unmodeled_skills.is_empty() {
            return;
        }
        self.section("Data availability");
        for row in unmodeled_skills {
            self.lines
                .push(format!("  {}", self.style.label("Unmodeled skill entry")));
            self.field("Key", &row.authored_key, 4);
            if let atlas_record::CreatureIntegerPresenceJson::Value(value) = row.modifier {
                self.field("Modifier", format!("{value:+}"), 4);
            }
            self.paragraph(row.message, 4);
        }
        for row in rows {
            self.lines.push(format!(
                "  {}",
                self.style.label(availability_label(row.field))
            ));
            if let Some(key) = &row.authored_key {
                self.field("Key", key, 4);
            }
            if let Some(value) = &row.source_value {
                self.field("Source value", value, 4);
            }
            self.paragraph(&row.message, 4);
        }
    }

    fn generic_sections(&mut self, sections: &[atlas_record::RecordSectionJson]) {
        for section in sections {
            self.section(&section.title);
            for block in &section.blocks {
                match block {
                    RecordBlockJson::FactList { facts } => {
                        for fact in facts {
                            self.field(&fact.label, &fact.value, 2);
                        }
                    }
                    RecordBlockJson::Prose { text } => self.paragraph(text, 2),
                    RecordBlockJson::Content { content } => self.render_content(content, 2),
                    RecordBlockJson::Relationships { relationships } => {
                        for row in relationships {
                            self.bullet(
                                reference_row(
                                    &row.label,
                                    Some(row.kind),
                                    row.record_key.as_deref(),
                                ),
                                2,
                            );
                        }
                    }
                }
            }
        }
    }
}

struct SpellRankGroup<'a> {
    rank: Option<i64>,
    maximum: Option<i64>,
    spells: Vec<&'a CreatureSpellJson>,
}

fn spell_rank_groups<'a>(
    spells: &'a [CreatureSpellJson],
    slots: Option<&'a [atlas_record::CreatureSpellSlotJson]>,
) -> Vec<SpellRankGroup<'a>> {
    let mut groups: Vec<SpellRankGroup<'_>> = Vec::new();
    for spell in spells {
        let rank = spell.context.rank.or(spell.base_rank);
        if let Some(group) = groups.iter_mut().find(|group| group.rank == rank) {
            group.spells.push(spell);
        } else {
            groups.push(SpellRankGroup {
                rank,
                maximum: slots
                    .and_then(|slots| slots.iter().find(|slot| Some(slot.rank) == rank))
                    .and_then(|slot| slot.maximum),
                spells: vec![spell],
            });
        }
    }
    for slot in slots.into_iter().flatten() {
        if let Some(group) = groups
            .iter_mut()
            .find(|group| group.rank == Some(slot.rank))
        {
            group.maximum = slot.maximum;
        } else {
            groups.push(SpellRankGroup {
                rank: Some(slot.rank),
                maximum: slot.maximum,
                spells: Vec::new(),
            });
        }
    }
    groups
}

fn natural_content_heading<'a>(document: &'a CreatureContentJson, fallback: &'a str) -> &'a str {
    if fallback != "Description" {
        return fallback;
    }
    document.label.as_deref().unwrap_or(match document.role {
        "primary_description" => "Description",
        "summary" => "Summary",
        "supplemental_rules" => "Rules",
        "embedded_capability" => "Ability",
        "journal_page" | "generated_narrative" | "provenance" => "Notes",
        "table_result" => "Result",
        _ => "Description",
    })
}

fn action_cost(cost: &CreatureActionCostJson) -> String {
    match cost.kind {
        "actions" => match cost.actions {
            Some(1) => "[1 action]".into(),
            Some(2) => "[2 actions]".into(),
            Some(3) => "[3 actions]".into(),
            Some(value) => format!("[{value} actions]"),
            None => "[actions]".into(),
        },
        "free_action" => "[free action]".into(),
        "reaction" => "[reaction]".into(),
        "passive" => "passive".into(),
        "time" => cost.time.clone().unwrap_or_else(|| "authored time".into()),
        _ => cost
            .unsupported
            .clone()
            .unwrap_or_else(|| "unsupported action cost".into()),
    }
}

fn reference_row(label: &str, context: Option<&str>, target: Option<&str>) -> String {
    let mut value = label.to_string();
    if let Some(context) = context.filter(|value| !value.is_empty()) {
        value.push_str(&format!(" — {context}"));
    }
    if let Some(target) = target {
        value.push_str(&format!(" ({target})"));
    }
    value
}

fn inline_text(spans: &[PresentationInline]) -> String {
    let mut output = String::new();
    for span in spans {
        match span {
            PresentationInline::Text { text } | PresentationInline::Code { text } => {
                output.push_str(text)
            }
            PresentationInline::Strong { spans } | PresentationInline::Emphasis { spans } => {
                output.push_str(&inline_text(spans))
            }
            PresentationInline::Reference {
                label, record_key, ..
            } => {
                output.push_str(label);
                if let Some(key) = record_key {
                    output.push_str(&format!(" ({key})"));
                }
            }
            PresentationInline::Check { display, .. } => output.push_str(display),
            PresentationInline::LineBreak => output.push('\n'),
        }
    }
    output
}

fn block_plain_text(block: &PresentationContentBlock) -> Option<String> {
    match block {
        PresentationContentBlock::Heading { text, .. } => Some(text.clone()),
        PresentationContentBlock::Paragraph { spans } => Some(inline_text(spans)),
        PresentationContentBlock::List { items, .. } => Some(
            items
                .iter()
                .flat_map(|item| &item.blocks)
                .filter_map(block_plain_text)
                .collect::<Vec<_>>()
                .join(" "),
        ),
        PresentationContentBlock::Table { rows, .. } => Some(
            rows.iter()
                .flat_map(|row| &row.cells)
                .flat_map(|cell| &cell.blocks)
                .filter_map(block_plain_text)
                .collect::<Vec<_>>()
                .join(" "),
        ),
        PresentationContentBlock::Rule => None,
    }
}

fn availability_label(field: CreatureAvailabilityFieldJson) -> &'static str {
    match field {
        CreatureAvailabilityFieldJson::Size => "Size",
        CreatureAvailabilityFieldJson::Defenses => "Defenses",
        CreatureAvailabilityFieldJson::Perception => "Perception",
        CreatureAvailabilityFieldJson::EmbeddedEntities => "Embedded entities",
        CreatureAvailabilityFieldJson::Adjustment => "Adjustment",
        CreatureAvailabilityFieldJson::InitiativeStatistic => "Initiative statistic",
        CreatureAvailabilityFieldJson::SourceAlliance => "Source alliance",
        CreatureAvailabilityFieldJson::HitPointsValue => "Hit points",
        CreatureAvailabilityFieldJson::SenseAcuity => "Sense acuity",
        CreatureAvailabilityFieldJson::SkillPredicate => "Skill predicate",
        CreatureAvailabilityFieldJson::MovementMode => "Movement mode",
        CreatureAvailabilityFieldJson::ResourceMaximum => "Resource maximum",
        CreatureAvailabilityFieldJson::ResourceSerializedValue => "Resource serialized value",
        CreatureAvailabilityFieldJson::ResourceSourceDrift => "Resource source drift",
        CreatureAvailabilityFieldJson::RitualDifficultyClass => "Ritual difficulty class",
        CreatureAvailabilityFieldJson::ActionCost => "Action cost",
        CreatureAvailabilityFieldJson::SpellPreparation => "Spell preparation",
        CreatureAvailabilityFieldJson::SpellSlotMaximum => "Spell slot maximum",
        CreatureAvailabilityFieldJson::SpellSlotSerializedValue => "Spell slot serialized value",
        CreatureAvailabilityFieldJson::PreparedSpellSlot => "Prepared spell slot",
        CreatureAvailabilityFieldJson::SpellRitualSecondaryCasters => "Ritual secondary casters",
        CreatureAvailabilityFieldJson::SpellDefenseSave => "Spell defense save",
        CreatureAvailabilityFieldJson::DamageKind => "Damage kind",
        CreatureAvailabilityFieldJson::DamageApplyModifier => "Damage modifier",
        CreatureAvailabilityFieldJson::UnsupportedMechanic => "Unsupported mechanic",
        CreatureAvailabilityFieldJson::UnmodeledSkill => "Unmodeled skill entry",
        CreatureAvailabilityFieldJson::UnsupportedCapability => "Unsupported capability",
        CreatureAvailabilityFieldJson::ContentAssociation => "Content association",
    }
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use atlas_record::{
        AtlasRecord, ContentSourceKind, ContentVisibility, CreatureAbilitiesJson,
        CreatureActionJson, CreatureArmorClassJson, CreatureAvailabilityEvidenceJson,
        CreatureAvailabilityStateJson, CreatureContentOwnerJson, CreatureContentProvenanceJson,
        CreatureDefensesJson, CreatureFactProvenanceJson, CreatureFactProvenanceSetJson,
        CreatureFrequencyJson, CreatureHitPointsJson, CreatureIntegerPresenceJson,
        CreatureOccurrenceContextJson, CreaturePerceptionJson, CreatureProvenanceJson,
        CreatureRelationshipJson, CreatureRelationshipTargetJson, CreatureResourceJson,
        CreatureRitualsJson, CreatureSkillJson, CreatureSkillSourceEntryJson,
        CreatureSpellSlotJson, CreatureSpellcastingEntryJson, CreatureSpellcastingJson,
        CreatureStrikeJson, CreatureUnmodeledSkillJson, FoundryDocumentType, FoundryRecordInfo,
        FoundryRecordType, RecordCanonicalRelationshipJson, RecordClassification,
        RecordEditionContextJson, RecordEditionCounterpartJson, RecordIdentity, RecordJsonBase,
        RecordJsonOptions, RecordProvenance, RecordRelationshipProvenanceJson,
        ReferenceRelationKind, RetrievedRecord, record_json,
    };

    #[test]
    fn width_changes_wrapping_without_changing_words() {
        let mut narrow = Writer::new(40, TerminalStyle::plain());
        narrow.field(
            "Reference",
            "A deliberately long exact typed reference (spells:Very-Long-Key)",
            2,
        );
        let narrow = narrow.finish();
        let mut wide = Writer::new(120, TerminalStyle::plain());
        wide.field(
            "Reference",
            "A deliberately long exact typed reference (spells:Very-Long-Key)",
            2,
        );
        let wide = wide.finish();
        assert!(narrow.lines().count() > wide.lines().count());
        assert_eq!(
            narrow.split_whitespace().collect::<Vec<_>>(),
            wide.split_whitespace().collect::<Vec<_>>()
        );
    }

    #[test]
    fn long_exact_field_value_moves_below_its_label_before_overflowing() {
        let mut writer = Writer::new(40, TerminalStyle::plain());
        writer.field("Key", "pathfinder-bestiary:WQy7HBUcgDLsfVJd", 2);
        assert_eq!(
            writer.finish(),
            "  Key:\n       pathfinder-bestiary:WQy7HBUcgDLsfVJd\n"
        );
    }

    #[test]
    fn textual_action_costs_do_not_require_glyphs() {
        let cost = |kind, actions, time| CreatureActionCostJson {
            kind,
            actions,
            time,
            unsupported: None,
        };
        assert_eq!(action_cost(&cost("actions", Some(1), None)), "[1 action]");
        assert_eq!(action_cost(&cost("actions", Some(2), None)), "[2 actions]");
        assert_eq!(action_cost(&cost("actions", Some(3), None)), "[3 actions]");
        assert_eq!(
            action_cost(&cost("free_action", None, None)),
            "[free action]"
        );
        assert_eq!(action_cost(&cost("reaction", None, None)), "[reaction]");
        assert_eq!(action_cost(&cost("passive", None, None)), "passive");
        assert_eq!(
            action_cost(&cost("time", None, Some("10 minutes".into()))),
            "10 minutes"
        );
    }

    #[test]
    fn inline_checks_use_the_typed_display_without_reconstructing_fields() {
        let spans = [
            PresentationInline::Text {
                text: "Saving Throw ".to_string(),
            },
            PresentationInline::Check {
                display: "Fortitude DC 28".to_string(),
                statistic: Some("will".to_string()),
                difficulty_class: Some(99),
            },
        ];

        assert_eq!(inline_text(&spans), "Saving Throw Fortitude DC 28");
    }

    #[test]
    fn ordinary_mechanics_use_display_frequency_maximum_resources_and_clean_perception() {
        let mut record = review_fixture(DetailLevel::Standard);
        let RecordPresentationJson::Creature {
            actions,
            perception,
            resources,
            ..
        } = &mut record.presentation
        else {
            panic!("creature fixture");
        };
        actions
            .as_mut()
            .and_then(|actions| actions.first_mut())
            .expect("review action")
            .frequency = Some(CreatureFrequencyJson {
            maximum: Some(1),
            period: Some("PT1M".into()),
            display: Some("1 per minute".into()),
            serialized_value: Some(1),
        });
        perception.as_mut().expect("review perception").details = Some(String::new());
        *resources = Some(vec![
            CreatureResourceJson {
                id: "focus".into(),
                order: 0,
                kind: "focus".into(),
                label: "Focus".into(),
                maximum: Some(1),
                serialized_value: Some(1),
                current_policy: "serialized_value_is_provenance_only",
            },
            CreatureResourceJson {
                id: "opaque".into(),
                order: 1,
                kind: "opaque".into(),
                label: "Opaque".into(),
                maximum: None,
                serialized_value: Some(3),
                current_policy: "serialized_value_is_provenance_only",
            },
        ]);

        let rendered = render_record(&record, DetailLevel::Standard, 120, TerminalStyle::plain());
        assert!(rendered.contains("Frequency: 1 per minute"));
        assert!(rendered.contains("Focus: maximum 1"));
        assert!(rendered.contains("Perception: +19"));
        assert!(!rendered.contains("PT1M"));
        assert!(!rendered.contains("serialized 1"));
        assert!(!rendered.contains("serialized_value_is_provenance_only"));
        assert!(!rendered.contains("Opaque:"));
        assert!(!rendered.contains("Perception: +19;"));

        let RecordPresentationJson::Creature { perception, .. } = &mut record.presentation else {
            panic!("creature fixture");
        };
        perception.as_mut().expect("review perception").details =
            Some("keen hearing; scent-specific".into());
        let rendered = render_record(&record, DetailLevel::Standard, 120, TerminalStyle::plain());
        assert!(rendered.contains("Perception: +19; keen hearing; scent-specific"));
    }

    #[test]
    fn exact_empty_optional_compound_details_never_add_punctuation() {
        let mut record = review_fixture(DetailLevel::Standard);
        let RecordPresentationJson::Creature {
            defenses,
            perception,
            skills,
            movement,
            ..
        } = &mut record.presentation
        else {
            panic!("creature fixture");
        };
        let defenses = defenses.as_mut().expect("defenses");
        defenses.ac.as_mut().expect("AC").details = Some(String::new());
        defenses.hp.as_mut().expect("HP").details = Some(String::new());
        defenses.saves = Some(atlas_record::CreatureSavesJson {
            fortitude: Some(atlas_record::CreatureSaveJson {
                id: "fortitude".into(),
                value: Some(18),
                details: Some(String::new()),
            }),
            reflex: Some(atlas_record::CreatureSaveJson {
                id: "reflex".into(),
                value: Some(17),
                details: Some("against exact authored hazards ".into()),
            }),
            will: None,
        });
        defenses.all_saves_note = Some(String::new());
        perception.as_mut().expect("perception").details = Some(String::new());
        skills.as_mut().expect("skills").push(CreatureSkillJson {
            id: "athletics".into(),
            order: 1,
            source_entries: Vec::new(),
            slug: "athletics".into(),
            label: "Athletics".into(),
            modifier: Some(20),
            note: Some(String::new()),
            variants: Vec::new(),
            source_item_id: None,
            unmodeled: None,
        });
        *movement = Some(atlas_record::CreatureMovementJson {
            modes: vec![atlas_record::CreatureMovementModeJson {
                id: "land".into(),
                order: 0,
                mode: "land".into(),
                label: Some("Speed".into()),
                value_feet: Some(25),
                details: Some(String::new()),
            }],
        });

        let rendered = render_record(&record, DetailLevel::Standard, 120, TerminalStyle::plain());
        for exact in [
            "AC: 28\n",
            "HP: current 170; maximum 170\n",
            "Fortitude: 18\n",
            "Perception: +19\n",
            "Athletics: +20\n",
            "Speed: 25 feet\n",
        ] {
            assert!(
                rendered.contains(exact),
                "missing clean line {exact:?}: {rendered}"
            );
        }
        assert!(rendered.contains("Reflex: 17; against exact authored hazards"));
        assert!(!rendered.contains("All saves:"));
    }

    #[test]
    fn candidate_review_samples_are_rendered_from_the_typed_contract() {
        let cells = [
            ("summary", DetailLevel::Summary, 80),
            ("preview-40", DetailLevel::Preview, 40),
            ("preview-80", DetailLevel::Preview, 80),
            ("preview-120", DetailLevel::Preview, 120),
            ("description", DetailLevel::Description, 80),
            ("standard-40", DetailLevel::Standard, 40),
            ("standard-80", DetailLevel::Standard, 80),
            ("standard-120", DetailLevel::Standard, 120),
            ("full-40", DetailLevel::Full, 40),
            ("full-80", DetailLevel::Full, 80),
            ("full-120", DetailLevel::Full, 120),
        ];
        let sample_dir =
            std::env::var_os("ATLAS_CLI_REVIEW_SAMPLE_DIR").map(std::path::PathBuf::from);
        if let Some(dir) = &sample_dir {
            std::fs::create_dir_all(dir).expect("create review sample directory");
        }
        for (name, detail, width) in cells {
            let rendered = render_record(
                &review_fixture(detail),
                detail,
                width,
                TerminalStyle::plain(),
            );
            if detail == DetailLevel::Summary {
                assert_eq!(rendered, "bestiary:Night-Hag\tNight Hag\tcreature\n");
            } else if detail == DetailLevel::Description {
                assert!(rendered.contains("Change Shape"));
                assert!(!rendered.contains("Defenses"));
            } else {
                assert!(rendered.contains("spells:Dream-Message"));
                if detail == DetailLevel::Preview {
                    assert!(!rendered.contains("Relationships"));
                    assert!(!rendered.contains("Data availability"));
                } else {
                    assert!(rendered.contains("Unmodeled skill entry"));
                    assert!(rendered.contains("Key: synthetic-review-skill"));
                    assert!(rendered.contains("Modifier: +17"));
                }
            }
            if matches!(detail, DetailLevel::Description | DetailLevel::Full) {
                for internal in [
                    "Authored content",
                    "Owner:",
                    "Identity:",
                    "Record-owned content",
                    "Entity-owned content",
                    "Occurrence-owned content",
                ] {
                    assert!(
                        !rendered.contains(internal),
                        "normal output leaked {internal}: {rendered}"
                    );
                }
            }
            if detail != DetailLevel::Summary {
                assert_eq!(rendered.lines().next(), Some("Night Hag"));
                assert!(rendered.contains("Type: creature"), "{detail}: {rendered}");
                assert!(rendered.contains("Pack: Bestiary"), "{detail}: {rendered}");
                for internal in [
                    "dream-message-rank-5-first",
                    "dream-message-rank-5-second",
                    "slot5:0",
                    "slot5:1",
                    "claw-occurrence",
                ] {
                    assert!(
                        !rendered.contains(internal),
                        "normal output leaked {internal}: {rendered}"
                    );
                }
            }
            if let Some(dir) = &sample_dir {
                std::fs::write(dir.join(format!("night-hag-injected-{name}.txt")), rendered)
                    .expect("write review sample");
            }
        }
    }

    #[test]
    fn repeated_spell_occurrences_keep_order_without_exposing_internal_identity() {
        let rendered = render_record(
            &review_fixture(DetailLevel::Preview),
            DetailLevel::Preview,
            120,
            TerminalStyle::plain(),
        );
        assert_eq!(rendered.matches("Rank 5").count(), 1);
        assert_eq!(rendered.matches("Dream Message — 10 minutes").count(), 2);
        assert!(!rendered.contains("dream-message-rank-5-first"));
        assert!(!rendered.contains("dream-message-rank-5-second"));
        assert!(!rendered.contains("slot5:0"));
        assert!(!rendered.contains("slot5:1"));
        assert!(!rendered.contains("1 of 2"));
        assert!(!rendered.contains("serialized"));
        assert_eq!(rendered.matches("(spells:Dream-Message)").count(), 2);

        let standard = render_record(
            &review_fixture(DetailLevel::Standard),
            DetailLevel::Standard,
            120,
            TerminalStyle::plain(),
        );
        assert!(standard.contains("Rank 5 · 2 slots"));
        assert!(!standard.contains("serialized"));
    }

    #[test]
    fn spell_damage_is_nested_beneath_its_spell_at_every_width() {
        for width in [40, 80, 120] {
            let mut record = review_fixture(DetailLevel::Standard);
            let RecordPresentationJson::Creature { spellcasting, .. } = &mut record.presentation
            else {
                panic!("creature fixture");
            };
            spellcasting
                .as_mut()
                .and_then(|spellcasting| spellcasting.entries.first_mut())
                .and_then(|entry| entry.spells.first_mut())
                .expect("ranked spell")
                .damage = Some(vec![CreatureDamageJson {
                id: "mental".into(),
                formula: Some("2d6".into()),
                damage_type: Some("mental".into()),
                category: None,
                kinds: Vec::new(),
                apply_modifier: None,
            }]);
            let rendered = render_record(
                &record,
                DetailLevel::Standard,
                width,
                TerminalStyle::plain(),
            );
            let line = rendered
                .lines()
                .find(|line| line.contains("Damage: 2d6"))
                .expect("spell damage line");
            assert_eq!(line.len() - line.trim_start().len(), 8, "width {width}");
        }
    }

    #[test]
    fn unmodeled_skill_modifier_uses_typed_presence_without_parsing_the_key() {
        let mut record = review_fixture(DetailLevel::Standard);
        let RecordPresentationJson::Creature {
            unmodeled_skill_availability,
            ..
        } = &mut record.presentation
        else {
            panic!("creature fixture");
        };
        let unmodeled = unmodeled_skill_availability
            .first_mut()
            .expect("synthetic unmodeled skill");
        unmodeled.authored_key = "synthetic-review-skill+23".into();
        unmodeled.modifier = CreatureIntegerPresenceJson::Value(17);

        let rendered = render_record(&record, DetailLevel::Standard, 80, TerminalStyle::plain());
        let key = rendered
            .find("Key: synthetic-review-skill+23")
            .expect("exact authored key");
        let modifier = rendered.find("Modifier: +17").expect("typed modifier");
        let explanation = rendered
            .find("The source supplied an unrecognized skill key.")
            .expect("approved explanation");
        assert!(key < modifier && modifier < explanation);
        assert!(!rendered.contains("Modifier: +23"));
    }

    #[test]
    fn unmodeled_skill_missing_and_null_are_silent_but_keep_one_notice() {
        for base in [
            CreatureIntegerPresenceJson::Missing,
            CreatureIntegerPresenceJson::Null,
        ] {
            let mut record = review_fixture(DetailLevel::Standard);
            let RecordPresentationJson::Creature {
                unmodeled_skill_availability,
                ..
            } = &mut record.presentation
            else {
                panic!("creature fixture");
            };
            unmodeled_skill_availability
                .first_mut()
                .expect("unmodeled fact")
                .modifier = base;
            let rendered =
                render_record(&record, DetailLevel::Standard, 80, TerminalStyle::plain());
            assert!(rendered.contains("Key: synthetic-review-skill"));
            assert!(!rendered.contains("Modifier:"));
            assert_eq!(
                rendered
                    .matches("The source supplied an unrecognized skill key.")
                    .count(),
                1
            );
        }
    }

    #[test]
    fn duplicate_unmodeled_skill_keys_render_each_occurrence_in_authored_order() {
        let mut record = review_fixture(DetailLevel::Standard);
        let rows = vec![
            CreatureUnmodeledSkillAvailabilityJson {
                skill_id: "skill-value".into(),
                authored_order: 8,
                authored_key: "same-authored-key".into(),
                modifier: CreatureIntegerPresenceJson::Value(17),
                message: "The source supplied an unrecognized skill key.",
            },
            CreatureUnmodeledSkillAvailabilityJson {
                skill_id: "skill-missing".into(),
                authored_order: 3,
                authored_key: "same-authored-key".into(),
                modifier: CreatureIntegerPresenceJson::Missing,
                message: "The source supplied an unrecognized skill key.",
            },
            CreatureUnmodeledSkillAvailabilityJson {
                skill_id: "skill-null".into(),
                authored_order: 5,
                authored_key: "same-authored-key".into(),
                modifier: CreatureIntegerPresenceJson::Null,
                message: "The source supplied an unrecognized skill key.",
            },
        ];
        let RecordPresentationJson::Creature {
            unmodeled_skill_availability,
            ..
        } = &mut record.presentation
        else {
            panic!("creature fixture");
        };
        *unmodeled_skill_availability = rows;

        let rendered = render_record(&record, DetailLevel::Standard, 80, TerminalStyle::plain());
        assert_eq!(rendered.matches("Key: same-authored-key").count(), 3);
        assert_eq!(rendered.matches("Modifier: +17").count(), 1);
        assert_eq!(
            rendered
                .matches("The source supplied an unrecognized skill key.")
                .count(),
            3
        );

        let RecordPresentationJson::Creature {
            unmodeled_skill_availability,
            ..
        } = &mut record.presentation
        else {
            panic!("creature fixture");
        };
        unmodeled_skill_availability.reverse();
        let reversed = render_record(&record, DetailLevel::Standard, 80, TerminalStyle::plain());
        assert_ne!(
            rendered, reversed,
            "presentation row order must be observable"
        );
        assert_eq!(reversed.matches("Key: same-authored-key").count(), 3);
        assert_eq!(reversed.matches("Modifier: +17").count(), 1);
    }

    #[test]
    fn other_retained_unsupported_source_values_render_verbatim() {
        let mut record = review_fixture(DetailLevel::Standard);
        let RecordPresentationJson::Creature { availability, .. } = &mut record.presentation else {
            panic!("creature fixture");
        };
        availability.push(CreatureAvailabilityJson {
            state: CreatureAvailabilityStateJson::Unsupported,
            field: CreatureAvailabilityFieldJson::Adjustment,
            authored_key: None,
            source_value: Some("safe opaque +23".into()),
            message: "The source supplied an unsupported adjustment.".into(),
        });
        let rendered = render_record(&record, DetailLevel::Standard, 80, TerminalStyle::plain());
        assert!(rendered.contains("Source value: safe opaque +23"));
    }

    pub(crate) fn review_fixture(detail: DetailLevel) -> RecordJson {
        let scan = matches!(
            detail,
            DetailLevel::Preview | DetailLevel::Standard | DetailLevel::Full
        );
        let complete = matches!(detail, DetailLevel::Standard | DetailLevel::Full);
        let description = detail == DetailLevel::Description;
        let full = detail == DetailLevel::Full;
        let content = CreatureContentJson {
            content_key: "bestiary:Night-Hag#occurrence:change-shape#description".into(),
            owner: CreatureContentOwnerJson::Occurrence {
                occurrence_id: "change-shape".into(),
            },
            role: "description",
            authored_order: 3,
            label: Some("Change Shape".into()),
            content_hash: "review-fixture-content".into(),
            visibility: "public",
            document: PresentationContent::new(vec![PresentationContentBlock::Paragraph {
                spans: vec![
                    PresentationInline::Text {
                        text:
                            "The hag assumes a Small or Medium humanoid form and retains her exact "
                                .into(),
                    },
                    PresentationInline::Reference {
                        label: "change shape".into(),
                        record_key: Some(
                            atlas_domain::RecordKey::parse("actions:Change-Shape").expect("key"),
                        ),
                        embedded: false,
                    },
                    PresentationInline::Text {
                        text: " rules reference.".into(),
                    },
                ],
            }]),
            provenance: Some(CreatureContentProvenanceJson {
                source_record_key: "bestiary:Night-Hag".into(),
                relative_source_path: "packs/bestiary/night-hag.json".into(),
                field_family: "embedded_item_description".into(),
                nested_source_id: Some("change-shape-source".into()),
            }),
        };
        let cost = CreatureActionCostJson {
            kind: "actions",
            actions: Some(1),
            time: None,
            unsupported: None,
        };
        let strike = CreatureStrikeJson {
            id: "claw-occurrence".into(),
            order: 0,
            label: "Claw".into(),
            traits: vec!["agile".into(), "magical".into()],
            action_cost: cost.clone(),
            attack_effects: vec!["dream drain".into()],
            rolls: complete.then(|| {
                vec![CreatureRollJson {
                    id: "attack".into(),
                    label: "Melee".into(),
                    kind: "attack",
                    value: Some(24),
                    ability: None,
                }]
            }),
            damage: complete.then(|| {
                vec![CreatureDamageJson {
                    id: "slashing".into(),
                    formula: Some("2d8+7".into()),
                    damage_type: Some("slashing".into()),
                    category: None,
                    kinds: vec!["physical"],
                    apply_modifier: Some(true),
                }]
            }),
            content: None,
            target_record_key: Some("weapons:Claw".into()),
            target_entity_id: Some("claw-entity".into()),
            provenance: None,
        };
        let action = CreatureActionJson {
            id: "change-shape".into(),
            order: 1,
            label: "Change Shape".into(),
            traits: vec!["concentrate".into(), "polymorph".into()],
            action_cost: cost.clone(),
            category: Some("offensive".into()),
            frequency: None,
            requirements: Some("The hag is in her natural form.".into()),
            cost: None,
            rolls: complete.then(Vec::new),
            damage: complete.then(Vec::new),
            uses: None,
            self_effect: None,
            self_effect_label: None,
            content: full.then(|| vec![content.clone()]),
            target_record_key: Some("actions:Change-Shape".into()),
            target_entity_id: Some("change-shape-entity".into()),
            provenance: None,
        };
        let first_spell = CreatureSpellJson {
            id: "dream-message-rank-5-first".into(),
            order: 2,
            label: "Dream Message".into(),
            traits: vec!["mental".into()],
            action_cost: CreatureActionCostJson {
                kind: "time",
                actions: None,
                time: Some("10 minutes".into()),
                unsupported: None,
            },
            target_record_key: Some("spells:Dream-Message".into()),
            target_entity_id: Some("dream-message-entity".into()),
            parent_entry_id: Some("innate-spells".into()),
            context: CreatureOccurrenceContextJson {
                group: Some("5th rank".into()),
                rank: Some(5),
                location: None,
                slot: Some("slot5:0".into()),
                uses: None,
                contextual_label: None,
            },
            base_rank: Some(3),
            signature: Some(false),
            traditions: vec!["occult".into()],
            requirements: None,
            cost: None,
            target: Some("1 creature".into()),
            range: Some("planetary".into()),
            time: Some("10 minutes".into()),
            counteraction: None,
            ritual: None,
            area: None,
            duration: None,
            defense: None,
            damage: complete.then(Vec::new),
            content: None,
            provenance: None,
        };
        let fact = CreatureFactProvenanceJson::Source {
            field: "system.details.level.value",
        };
        let provenance = CreatureProvenanceJson {
            source_path: "packs/bestiary/night-hag.json".into(),
            source_contract_version: "pf2e-v13".into(),
            source_system_version: "7.5.0".into(),
            source_upstream_commit: "review-fixture-upstream".into(),
            facts: CreatureFactProvenanceSetJson {
                level: fact.clone(),
                rarity: fact.clone(),
                traits: fact.clone(),
                size: fact.clone(),
                publication: fact.clone(),
                adjustment: fact.clone(),
                source_alliance: fact.clone(),
                perception: fact.clone(),
                initiative: fact.clone(),
                languages: fact.clone(),
                skills: fact.clone(),
                abilities: fact.clone(),
                defenses: fact.clone(),
                movement: fact.clone(),
                resources: fact.clone(),
                embedded_entities: fact,
            },
        };
        let source = {
            let source_record = RetrievedRecord {
                record: AtlasRecord::new(
                    RecordIdentity::new(
                        atlas_domain::RecordKey::parse("bestiary:Night-Hag").expect("key"),
                        "Night Hag",
                    ),
                    RecordClassification::new(atlas_domain::RecordKind::Rule),
                    FoundryRecordInfo::new(
                        "Bestiary",
                        FoundryDocumentType::Actor,
                        FoundryRecordType::Npc,
                    ),
                    RecordProvenance::new("packs/bestiary/night-hag.json"),
                ),
                body: None,
            };
            record_json(
                &source_record,
                RecordJsonOptions {
                    detail: DetailLevel::Preview,
                    include_source_json: false,
                },
            )
            .expect("typed source fixture")
            .base
            .source
        };
        RecordJson {
            base: RecordJsonBase {
                key: "bestiary:Night-Hag".into(), name: "Night Hag".into(), kind: "creature",
                level: (detail != DetailLevel::Summary).then_some(9), rarity: (detail != DetailLevel::Summary).then(|| "uncommon".into()),
                traits: (detail != DetailLevel::Summary).then(|| vec!["fiend".into(), "hag".into()]).unwrap_or_default(),
                source, supplementary_sections: Vec::new(), source_json: None,
            },
            presentation: RecordPresentationJson::Creature {
                teaser: matches!(detail, DetailLevel::Preview | DetailLevel::Standard).then(|| "Night hags prey upon sleepers, trading in stolen dreams and stalking victims through a world of nightmares while their repeated innate magic remains tied to exact spell occurrences.".into()),
                size: scan.then(|| "medium".into()), adjustment: scan.then(|| "elite-ready".into()),
                initiative: scan.then(|| atlas_record::CreatureInitiativeJson { statistic: "Perception".into() }),
                abilities: scan.then(|| CreatureAbilitiesJson { strength: Some(5), dexterity: Some(4), constitution: Some(4), intelligence: Some(4), wisdom: Some(5), charisma: Some(6) }),
                defenses: scan.then(|| CreatureDefensesJson { ac: Some(CreatureArmorClassJson { value: Some(28), details: Some("+1 status against dreams".into()) }), hp: Some(CreatureHitPointsJson { value: Some(170), maximum: Some(170), temporary: None, temporary_maximum: None, details: None }), ..Default::default() }),
                perception: scan.then(|| CreaturePerceptionJson { modifier: Some(19), details: None, has_vision: Some(true), senses: None }),
                languages: scan.then(|| vec!["Aklo".into(), "Common".into(), "Infernal".into()]),
                skills: scan.then(|| vec![CreatureSkillJson {
                    id: "synthetic-unmodeled-skill".into(),
                    order: 0,
                    source_entries: vec![CreatureSkillSourceEntryJson {
                        authored_key: "synthetic-review-skill".into(),
                        modifier: CreatureIntegerPresenceJson::Value(17),
                    }],
                    slug: "unmodeled".into(),
                    label: "synthetic-review-skill".into(),
                    modifier: None,
                    note: None,
                    variants: Vec::new(),
                    source_item_id: None,
                    unmodeled: Some(CreatureUnmodeledSkillJson {
                        authored_key: "synthetic-review-skill".into(),
                        base: CreatureIntegerPresenceJson::Value(17),
                        reason: "unknown_authored_key",
                    }),
                }]),
                movement: None, resources: None,
                strikes: scan.then(|| vec![strike]), actions: scan.then(|| vec![action]),
                spellcasting: scan.then(|| CreatureSpellcastingJson { entries: vec![CreatureSpellcastingEntryJson {
                    id: "innate-spells".into(), order: 0, label: "Occult Innate Spells".into(), preparation: Some("innate".into()), tradition: Some("occult".into()),
                    attack: Some(19), dc: Some(28), slots: complete.then(|| vec![CreatureSpellSlotJson { rank: 5, maximum: Some(2), serialized_value: Some(2), prepared: Some(vec![atlas_record::CreaturePreparedSpellJson { order: 0, id: Some("dream-message-source-item".into()), name: Some("Dream Message".into()), expended: Some(false), prepared: Some(true) }]) }]), spells: vec![first_spell.clone(), CreatureSpellJson { id: "dream-message-rank-5-second".into(), order: 3, context: CreatureOccurrenceContextJson { slot: Some("slot5:1".into()), ..first_spell.context.clone() }, ..first_spell }],
                    target_record_key: None, target_entity_id: Some("innate-spells-entity".into()), provenance: None, content: None,
                }], standalone_spells: Vec::new() }),
                rituals: scan.then(|| CreatureRitualsJson { difficulty_class: Some(28) }), equipment: None, lore: None,
                content: description.then(|| vec![content]),
                relationships: scan.then(|| vec![CreatureRelationshipJson { source_occurrence_id: "change-shape".into(), kind: "helper" , target: CreatureRelationshipTargetJson::Occurrence { occurrence_id: "claw-occurrence".into() }, source_path: "system.items[1]".into() }]),
                provenance: (detail == DetailLevel::Full).then_some(provenance),
                edition: (detail == DetailLevel::Full).then(|| RecordEditionContextJson { status: RecordEditionStatusJson::Legacy, counterpart_lookup: RecordEditionCounterpartLookupJson::Verified { counterparts: vec![RecordEditionCounterpartJson { role: RecordEditionCounterpartRoleJson::RemasteredCounterpart, record_key: "bestiary:Dream-Hag".into(), title: "Dream Hag".into() }] } }),
                record_relationships: scan.then(|| RecordRelationshipLookupJson::Verified { relationships: vec![RecordCanonicalRelationshipJson { direction: RecordRelationshipDirectionJson::Reference, kind: ReferenceRelationKind::Reference, label: "Dream Message".into(), target_record_key: "spells:Dream-Message".into(), provenance: RecordRelationshipProvenanceJson { from_record_key: "bestiary:Night-Hag".into(), to_record_key: "spells:Dream-Message".into(), source_kind: ContentSourceKind::Description, visibility: ContentVisibility::Public } }] }),
                availability: Vec::new(),
                unmodeled_skill_availability: scan.then(|| vec![CreatureUnmodeledSkillAvailabilityJson {
                    skill_id: "synthetic-unmodeled-skill".into(),
                    authored_order: 0,
                    authored_key: "synthetic-review-skill".into(),
                    modifier: CreatureIntegerPresenceJson::Value(17),
                    message: "The source supplied an unrecognized skill key.",
                }]).unwrap_or_default(),
                availability_evidence: full.then(|| vec![CreatureAvailabilityEvidenceJson {
                    state: CreatureAvailabilityStateJson::Unsupported,
                    field: CreatureAvailabilityFieldJson::ResourceSerializedValue,
                    component_id: Some("focus-resource".into()),
                    authored_key: None,
                    source_value: Some("{\"value\":0}".into()),
                    source_shape: Some("object"),
                    source_reason: Some("unsupported_shape"),
                    source_path: Some("system.resources.focus.value".into()),
                    message: "The source supplied an unsupported serialized resource value.".into(),
                }]),
            },
        }
    }
}
