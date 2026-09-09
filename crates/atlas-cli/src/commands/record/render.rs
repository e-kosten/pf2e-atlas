use atlas_domain::DetailLevel;
use atlas_record::{
    CreatureActionCostJson, CreatureAvailabilityFieldJson, CreatureAvailabilityJson,
    CreatureContentJson, CreatureDamageJson, CreatureRollJson, CreatureSpellJson,
    CreatureUnmodeledSkillAvailabilityJson, FactPresentationDisposition, FactPresentationRole,
    FactPresentationState, FactRequirement, H8FactJson, HazardAvailabilityJson,
    HazardAvailabilityStateJson, HazardDefenseTerminalFact, JournalJson, JournalPageEntryJson,
    PresentationContent, PresentationContentBlock, PresentationInline, RecordBlockJson,
    RecordEditionCounterpartLookupJson, RecordEditionCounterpartRoleJson, RecordEditionStatusJson,
    RecordFactTerminalPresentation, RecordJson, RecordPresentationJson,
    RecordRelationshipDirectionJson, RecordRelationshipLookupJson, RollTableJson, SpellDamageJson,
    SpellDamagePatchJson, SpellDamagePatchOperationJson, SpellFactJson, SpellFormJson,
    SpellFormLabelKind, SpellFormResultJson, SpellHeighteningJson, SpellPatchJson,
    SpellResolvedDefinitionJson, SpellResolvedFieldJson, SpellRuleDetailJson, SpellRuleJson,
    SpellRulePredicateJson, SpellTextPatchOperationJson, TableResultEntryJson,
    classify_fact_presentation, project_spell_json_presentation_issues,
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
        RecordPresentationJson::Hazard {
            sections,
            availability,
            edition,
            record_relationships,
            ..
        } => {
            out.hazard_sections(sections);
            if matches!(detail, DetailLevel::Standard | DetailLevel::Full) {
                out.relationships(record_relationships.as_ref());
            }
            if detail == DetailLevel::Full {
                out.source_and_edition(record, edition.as_ref());
            }
            out.hazard_availability(detail, availability);
        }
        RecordPresentationJson::Spell {
            spell,
            edition,
            record_relationships,
        } => {
            out.spell_record(record, spell, detail);
            if matches!(detail, DetailLevel::Standard | DetailLevel::Full) {
                out.relationships(record_relationships.as_ref());
            }
            if detail == DetailLevel::Full {
                out.source_and_edition(record, edition.as_ref());
            }
        }
        RecordPresentationJson::Journal {
            journal,
            edition,
            record_relationships,
        } => {
            out.journal_record(journal, detail);
            if matches!(detail, DetailLevel::Standard | DetailLevel::Full) {
                out.relationships(record_relationships.as_ref());
            }
            if detail == DetailLevel::Full {
                out.source_and_edition(record, edition.as_ref());
            }
        }
        RecordPresentationJson::RollTable {
            roll_table,
            edition,
            record_relationships,
        } => {
            out.roll_table_record(roll_table, detail);
            if matches!(detail, DetailLevel::Standard | DetailLevel::Full) {
                out.relationships(record_relationships.as_ref());
            }
            if detail == DetailLevel::Full {
                out.source_and_edition(record, edition.as_ref());
            }
        }
        RecordPresentationJson::Unmigrated { sections, .. } => {
            out.generic_sections(sections);
            out.generic_sections(&record.supplementary_sections);
        }
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

    fn optional_section(&mut self, title: &str, render: impl FnOnce(&mut Self)) {
        let start = self.lines.len();
        self.section(title);
        let content_start = self.lines.len();
        render(self);
        if self.lines.len() == content_start {
            self.lines.truncate(start);
        }
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

    fn spell_record(
        &mut self,
        record: &RecordJson,
        spell: &atlas_record::SpellJson,
        detail: DetailLevel,
    ) {
        if detail != DetailLevel::Description {
            let classification = spell_fact(&spell.classification);
            self.optional_section("Classification", |out| {
                if let Some(classification) = classification {
                    if let Some(rank) = spell_fact(&classification.rank) {
                        out.field("Rank", rank.to_string(), 2);
                    }
                    if let Some(traits) = nonempty_spell_list(&classification.traits) {
                        out.list_field("Traits", traits, 2);
                    }
                }
                if let Some(rarity) = &record.rarity {
                    out.field("Rarity", rarity, 2);
                }
            });

            self.optional_section("Casting", |out| {
                if let Some(classification) = classification
                    && let Some(traditions) = nonempty_spell_list(&classification.traditions)
                {
                    out.list_field("Traditions", traditions, 2);
                }
                if let Some(casting) = spell_fact(&spell.casting) {
                    for (label, value) in [
                        ("Time", nonempty_spell_text(&casting.time)),
                        ("Cost", nonempty_spell_text(&casting.cost)),
                        ("Requirements", nonempty_spell_text(&casting.requirements)),
                    ] {
                        if let Some(value) = value {
                            out.field(label, value, 2);
                        }
                    }
                    if spell_fact(&casting.counteraction).copied() == Some(true) {
                        out.field("Counteract", "yes", 2);
                    }
                }
            });

            self.optional_section("Range and targets", |out| {
                if let Some(targeting) = spell_fact(&spell.targeting) {
                    if let Some(target) = nonempty_spell_text(&targeting.target) {
                        out.field("Target", target, 2);
                    }
                    if let Some(range) = spell_fact(&targeting.range) {
                        out.field("Range", &range.authored_text, 2);
                    }
                    if let Some(area) = spell_fact(&targeting.area) {
                        out.field("Area", spell_area_text(area), 2);
                    }
                }
                if let Some(defense) = spell_fact(&spell.defense) {
                    if let Some(passive) = nonempty_spell_text(&defense.passive) {
                        out.field("Defense", passive, 2);
                    }
                    if let Some(save) = spell_fact(&defense.save)
                        && let Some(statistic) = nonempty_spell_text(&save.statistic)
                    {
                        let qualifier = if spell_fact(&save.basic).copied() == Some(true) {
                            " (basic)"
                        } else {
                            ""
                        };
                        out.field("Save", format!("{statistic}{qualifier}"), 2);
                    }
                }
                if let Some(duration) = spell_fact(&spell.duration) {
                    if let Some(value) = nonempty_spell_text(&duration.value) {
                        out.field("Duration", value, 2);
                    }
                    if spell_fact(&duration.sustained).copied() == Some(true) {
                        out.field("Sustained", "yes", 2);
                    }
                }
            });

            self.optional_section("Effect", |out| {
                if let Some(damage) = spell_fact(&spell.damage) {
                    out.spell_damage_rows(damage, 2);
                }
            });

            self.optional_section("Heightening", |out| {
                if let Some(heightening) = spell_fact(&spell.heightening) {
                    out.spell_heightening(
                        heightening,
                        spell_fact(&spell.damage).map(Vec::as_slice),
                    );
                }
            });

            self.optional_section("Ritual", |out| {
                if let Some(ritual) = spell_fact(&spell.ritual) {
                    if let Some(value) = nonempty_spell_text(&ritual.primary_check) {
                        out.field("Primary check", value, 2);
                    }
                    if let Some(value) = spell_fact(&ritual.secondary_casters) {
                        out.field("Secondary casters", value.to_string(), 2);
                    }
                    if let Some(value) = nonempty_spell_text(&ritual.secondary_checks) {
                        out.field("Secondary checks", value, 2);
                    }
                }
            });

            self.optional_section("Rules", |out| {
                if let Some(rules) = spell_fact(&spell.rules) {
                    for rule in rules {
                        out.spell_rule(rule, 2);
                    }
                }
            });

            self.optional_section("Forms", |out| {
                for form in &spell.forms {
                    out.bullet(spell_form_label(form), 2);
                }
            });
            self.spell_issues(spell);
        }

        if matches!(detail, DetailLevel::Description | DetailLevel::Full) {
            for document in &spell.content {
                self.section(document.label.as_deref().unwrap_or("Description"));
                self.render_content(&document.document, 2);
            }
        }
    }

    fn journal_record(&mut self, journal: &JournalJson, detail: DetailLevel) {
        let Some(pages) = h8_fact(&journal.pages) else {
            self.section("Pages");
            self.field("Availability", h8_fact_state(&journal.pages), 2);
            return;
        };
        self.section("Pages");
        for entry in pages {
            match entry {
                JournalPageEntryJson::Page { page } => {
                    let label = h8_fact(&page.name).map_or("Untitled page", String::as_str);
                    let kind = h8_fact(&page.page_kind).copied().unwrap_or("unknown");
                    let title = h8_fact(&page.title);
                    let title_visible = title.and_then(|title| h8_fact(&title.show)).copied();
                    if title_visible == Some(true) {
                        self.heading(label, 2);
                        if let Some(level) = title.and_then(|title| h8_fact(&title.level)) {
                            self.field("Title level", level.to_string(), 4);
                        }
                    } else {
                        self.field("Page", label, 2);
                    }
                    self.field("Type", kind, 4);
                    if let Some(sort) = h8_fact(&page.sort) {
                        self.field("Sort", sort.to_string(), 4);
                    }
                    if matches!(detail, DetailLevel::Description | DetailLevel::Full)
                        && let Some(text) = h8_fact(&page.text)
                        && let Some(document) = h8_fact(&text.content)
                    {
                        self.paragraph(&atlas_record::render_plain_text(document), 4);
                    }
                    if matches!(kind, "image" | "pdf" | "video") {
                        self.field("Media", "metadata only; viewing is not supported", 4);
                    }
                }
                JournalPageEntryJson::Unsupported { unsupported } => {
                    self.bullet(
                        format!(
                            "Page {} unavailable: {}",
                            unsupported.source_ordinal.saturating_add(1),
                            unsupported.reason
                        ),
                        2,
                    );
                }
            }
        }
    }

    fn roll_table_record(&mut self, table: &RollTableJson, detail: DetailLevel) {
        self.optional_section("Table", |out| {
            if let Some(formula) = h8_fact(&table.formula) {
                out.field("Formula", formula, 2);
            }
            if let Some(replacement) = h8_fact(&table.replacement) {
                out.field("Replacement", if *replacement { "yes" } else { "no" }, 2);
            }
            if let Some(display_roll) = h8_fact(&table.display_roll) {
                out.field("Display roll", if *display_roll { "yes" } else { "no" }, 2);
            }
        });
        if matches!(detail, DetailLevel::Description | DetailLevel::Full)
            && let Some(description) = h8_fact(&table.description)
        {
            self.section("Description");
            self.paragraph(&atlas_record::render_plain_text(description), 2);
        }
        let Some(results) = h8_fact(&table.results) else {
            self.section("Results");
            self.field("Availability", h8_fact_state(&table.results), 2);
            return;
        };
        self.section("Results");
        for entry in results {
            match entry {
                TableResultEntryJson::Result { result } => {
                    let range = h8_fact(&result.range)
                        .map(|value| format!("{}–{}", value.first, value.last))
                        .unwrap_or_else(|| "?".to_string());
                    let label = h8_fact(&result.text)
                        .map(atlas_record::render_plain_text)
                        .filter(|value| !value.trim().is_empty())
                        .unwrap_or_else(|| "Untitled result".to_string());
                    self.bullet(format!("{range}: {label}"), 2);
                }
                TableResultEntryJson::Unsupported { unsupported } => self.bullet(
                    format!(
                        "Result {} unavailable: {}",
                        unsupported.source_ordinal.saturating_add(1),
                        unsupported.reason
                    ),
                    2,
                ),
            }
        }
    }

    fn spell_issues(&mut self, spell: &atlas_record::SpellJson) {
        let issues = project_spell_json_presentation_issues(spell);
        if issues.is_empty() {
            return;
        }
        self.section("Data issues");
        for issue in issues {
            self.field(issue.field.label(), issue.message(), 2);
        }
    }

    fn spell_damage_rows(&mut self, damage: &[SpellDamageJson], indent: usize) {
        for row in damage {
            let value = [
                nonempty_spell_text(&row.formula).map(str::to_string),
                nonempty_spell_text(&row.damage_type).map(str::to_string),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ");
            let label = spell_damage_label(row);
            self.field(&label, value, indent);
            if let Some(materials) = nonempty_spell_list(&row.materials) {
                self.list_field("Materials", materials, indent + 2);
            }
            if spell_fact(&row.apply_modifier).copied() == Some(true) {
                self.field("Modifier", "applies", indent + 2);
            }
        }
    }

    fn spell_heightening(
        &mut self,
        heightening: &SpellHeighteningJson,
        base_damage: Option<&[SpellDamageJson]>,
    ) {
        match heightening {
            SpellHeighteningJson::Interval {
                interval,
                area,
                damage,
            } => {
                if let Some(value) = spell_fact(interval) {
                    self.heading(&format!("Heightened (+{value})"), 2);
                }
                if let Some(value) = spell_fact(area) {
                    self.field("Area", format!("+{value} feet"), 4);
                }
                if let Some(values) = spell_fact(damage) {
                    for value in values {
                        let label = unique_spell_damage_member(base_damage, &value.key)
                            .map(spell_damage_label)
                            .unwrap_or_else(|| "Effect".to_string());
                        self.field(&label, format!("+{}", value.value), 4);
                    }
                }
            }
            SpellHeighteningJson::Fixed { layers } => {
                for layer in layers {
                    if let Some(rank) = spell_fact(&layer.rank) {
                        self.heading(&format!("Heightened ({})", ordinal_rank(*rank)), 2);
                    }
                    self.spell_patch(&layer.patch, base_damage, 4);
                }
            }
        }
    }

    fn spell_rule(&mut self, rule: &SpellRuleJson, indent: usize) {
        let (summary, fields) = match &rule.rule {
            SpellRuleDetailJson::DamageDice(value) => {
                let adjustment = [
                    nonempty_spell_text(&value.dice_number),
                    nonempty_spell_text(&value.die_size),
                    nonempty_spell_text(&value.damage_type),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" ");
                (
                    if adjustment.is_empty() {
                        "Damage dice adjustment".to_string()
                    } else {
                        format!("Damage dice adjustment: {adjustment}")
                    },
                    vec![
                        ("Scope", spell_rule_scope(&value.selector)),
                        ("Condition", spell_predicate_text(&value.predicate)),
                        (
                            "Display",
                            (spell_fact(&value.hide_if_disabled).copied() == Some(true))
                                .then(|| "only while enabled".to_string()),
                        ),
                    ],
                )
            }
            SpellRuleDetailJson::EphemeralEffect(value) => (
                "Conditional effect".to_string(),
                vec![
                    ("Scope", spell_rule_scopes(&value.selectors)),
                    ("Condition", spell_predicate_text(&value.predicate)),
                ],
            ),
            SpellRuleDetailJson::DamageAlteration(value) => (
                "Damage alteration".to_string(),
                vec![
                    ("Mode", nonempty_spell_text(&value.mode).map(str::to_string)),
                    (
                        "Property",
                        nonempty_spell_text(&value.property).map(str::to_string),
                    ),
                    (
                        "Value",
                        nonempty_spell_text(&value.value).map(str::to_string),
                    ),
                    ("Scope", spell_rule_scopes(&value.selectors)),
                    ("Condition", spell_predicate_text(&value.predicate)),
                ],
            ),
            SpellRuleDetailJson::RollOption(value) => {
                let summary = nonempty_spell_text(&value.label)
                    .map(|label| format!("Roll option: {label}"))
                    .unwrap_or_else(|| "Roll option".to_string());
                let mut fields = vec![("Condition", spell_predicate_text(&value.predicate))];
                if let Some(suboptions) = spell_fact(&value.suboptions) {
                    fields.extend(suboptions.iter().filter_map(|suboption| {
                        nonempty_spell_text(&suboption.label)
                            .map(|label| ("Choice", Some(label.to_string())))
                    }));
                }
                if spell_fact(&value.toggleable).copied() == Some(true) {
                    fields.push(("Toggleable", Some("yes".to_string())));
                }
                (summary, fields)
            }
            SpellRuleDetailJson::ItemAlteration(value) => (
                "Item alteration".to_string(),
                vec![
                    ("Mode", nonempty_spell_text(&value.mode).map(str::to_string)),
                    (
                        "Property",
                        nonempty_spell_text(&value.property).map(str::to_string),
                    ),
                    (
                        "Value",
                        nonempty_spell_text(&value.value).map(str::to_string),
                    ),
                    ("Condition", spell_predicate_text(&value.predicate)),
                ],
            ),
            SpellRuleDetailJson::Unsupported(_) => return,
        };
        self.bullet(summary, indent);
        for (label, value) in fields {
            if let Some(value) = value {
                self.field(label, value, indent + 2);
            }
        }
    }

    fn spell_patch(
        &mut self,
        patch: &SpellPatchJson,
        base_damage: Option<&[SpellDamageJson]>,
        indent: usize,
    ) {
        if let Some(classification) = spell_fact(&patch.classification) {
            if let Some(rank) = spell_fact(&classification.rank) {
                self.field("Rank", rank.to_string(), indent);
            }
            if let Some(traits) = spell_fact(&classification.traits) {
                self.list_field("Traits", traits, indent);
            }
            if let Some(traditions) = spell_fact(&classification.traditions) {
                self.list_field("Traditions", traditions, indent);
            }
        }
        if let Some(casting) = spell_fact(&patch.casting) {
            if let Some(value) = spell_fact(&casting.time) {
                self.field("Time", value, indent);
            }
            if let Some(value) = spell_fact(&casting.cost) {
                self.field("Cost", value, indent);
            }
            if let Some(value) = spell_fact(&casting.requirements) {
                self.field("Requirements", value, indent);
            }
            if spell_fact(&casting.counteraction).copied() == Some(true) {
                self.field("Counteract", "yes", indent);
            }
        }
        if let Some(targeting) = spell_fact(&patch.targeting) {
            if let Some(target) = nonempty_spell_text(&targeting.target) {
                self.field("Target", target, indent);
            }
            if let Some(range) = spell_fact(&targeting.range) {
                self.field("Range", &range.authored_text, indent);
            }
            if let Some(area) = spell_fact(&targeting.area) {
                self.field("Area", spell_area_text(area), indent);
            }
        }
        if let Some(defense) = spell_fact(&patch.defense) {
            if let Some(value) = nonempty_spell_text(&defense.passive) {
                self.field("Defense", value, indent);
            }
            if let Some(save) = spell_fact(&defense.save)
                && let Some(value) = nonempty_spell_text(&save.statistic)
            {
                let qualifier = if spell_fact(&save.basic).copied() == Some(true) {
                    " (basic)"
                } else {
                    ""
                };
                self.field("Save", format!("{value}{qualifier}"), indent);
            }
        }
        if let Some(damage) = spell_fact(&patch.damage) {
            for member in &damage.members {
                let base_member = unique_spell_damage_member(base_damage, &member.key);
                match &member.operation {
                    SpellDamagePatchOperationJson::Merge(value) => {
                        let text = [
                            spell_fact(&value.formula).cloned(),
                            spell_fact(&value.damage_type).cloned(),
                        ]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>()
                        .join(" ");
                        let label = spell_damage_patch_label(value, base_member);
                        self.field(&label, text, indent);
                        if let Some(materials) = nonempty_spell_list(&value.materials) {
                            self.list_field("Materials", materials, indent + 2);
                        }
                        if spell_fact(&value.apply_modifier).copied() == Some(true) {
                            self.field("Modifier", "applies", indent + 2);
                        }
                    }
                    SpellDamagePatchOperationJson::Delete => {
                        let label = base_member
                            .map(spell_damage_label)
                            .unwrap_or_else(|| "Effect".to_string());
                        self.field(&label, "removed", indent);
                    }
                    SpellDamagePatchOperationJson::Unsupported(_) => {}
                }
            }
        }
        if let Some(duration) = spell_fact(&patch.duration) {
            if let Some(value) = nonempty_spell_text(&duration.value) {
                self.field("Duration", value, indent);
            }
            if spell_fact(&duration.sustained).copied() == Some(true) {
                self.field("Sustained", "yes", indent);
            }
        }
        if let Some(heightening) = spell_fact(&patch.heightening) {
            if let Some(value) = spell_fact(&heightening.kind) {
                self.field("Heightening", value, indent);
            }
            if let Some(value) = spell_fact(&heightening.interval) {
                self.field("Heightening interval", value.to_string(), indent);
            }
            if let Some(value) = spell_fact(&heightening.area) {
                self.field("Heightening area", value.to_string(), indent);
            }
            if let Some(damage) = spell_fact(&heightening.damage) {
                for member in &damage.members {
                    match &member.operation {
                        SpellTextPatchOperationJson::Merge(value) => {
                            if let Some(value) = spell_fact(value) {
                                self.field("Effect increase", value, indent);
                            }
                        }
                        SpellTextPatchOperationJson::Delete => {
                            self.field("Effect increase", "removed", indent);
                        }
                        SpellTextPatchOperationJson::Unsupported(_) => {}
                    }
                }
            }
        }
        if let Some(rules) = spell_fact(&patch.rules) {
            for rule in rules {
                self.spell_rule(rule, indent);
            }
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

    fn hazard_availability(
        &mut self,
        detail: DetailLevel,
        availability: &[HazardAvailabilityJson],
    ) {
        if !matches!(detail, DetailLevel::Standard | DetailLevel::Full) {
            return;
        }
        let actionable = availability
            .iter()
            .filter(|row| {
                let state = match row.state {
                    HazardAvailabilityStateJson::Missing => FactPresentationState::Missing,
                    HazardAvailabilityStateJson::Null => FactPresentationState::Null,
                    HazardAvailabilityStateJson::Unsupported => FactPresentationState::Unsupported,
                };
                matches!(
                    classify_fact_presentation(
                        FactPresentationRole::Gameplay,
                        state,
                        FactRequirement::Optional,
                    ),
                    FactPresentationDisposition::Issue(_)
                )
            })
            .collect::<Vec<_>>();
        if actionable.is_empty() {
            return;
        }
        self.section("Data issues");
        for row in actionable {
            self.field(row.human_label, &row.human_message, 2);
        }
    }

    fn hazard_sections(&mut self, sections: &[atlas_record::RecordSectionJson]) {
        for section in sections {
            if section.kind == "defense" {
                self.hazard_defenses(section);
                continue;
            }
            self.optional_section(&section.title, |out| {
                for block in &section.blocks {
                    match block {
                        RecordBlockJson::FactList { facts } => {
                            for fact in facts {
                                match fact.terminal {
                                    Some(RecordFactTerminalPresentation::HazardStrike {
                                        mode,
                                        action_cost,
                                    }) => {
                                        let kind = mode
                                            .map(|mode| format!("{mode} Strike"))
                                            .unwrap_or_else(|| "Strike".to_string());
                                        out.field(&fact.label, kind, 2);
                                        if let Some(action_cost) = action_cost {
                                            out.field("Actions", action_cost.to_string(), 4);
                                        }
                                    }
                                    Some(RecordFactTerminalPresentation::HazardDamage) => {
                                        out.field("Damage", &fact.value, 4);
                                    }
                                    Some(RecordFactTerminalPresentation::HazardDefense(_)) => {}
                                    None => out.field(&fact.label, &fact.value, 2),
                                }
                            }
                        }
                        RecordBlockJson::Prose { text } => out.paragraph(text, 2),
                        RecordBlockJson::Content { content } => out.render_content(content, 2),
                        RecordBlockJson::Relationships { .. } => {}
                    }
                }
            });
        }
    }

    fn hazard_defenses(&mut self, section: &atlas_record::RecordSectionJson) {
        let mut armor_class = None;
        let mut hardness = None;
        let mut hp_current = None;
        let mut hp_maximum = None;
        let mut hp_temporary = None;
        let mut broken_threshold = None;
        let mut fortitude = None;
        let mut reflex = None;
        let mut will = None;
        let mut remaining = Vec::new();
        for fact in section.blocks.iter().flat_map(|block| match block {
            RecordBlockJson::FactList { facts } => facts.as_slice(),
            RecordBlockJson::Prose { .. }
            | RecordBlockJson::Content { .. }
            | RecordBlockJson::Relationships { .. } => &[],
        }) {
            match fact.terminal {
                Some(RecordFactTerminalPresentation::HazardDefense(value)) => match value {
                    HazardDefenseTerminalFact::ArmorClass(value) => armor_class = Some(value),
                    HazardDefenseTerminalFact::Hardness(value) => hardness = Some(value),
                    HazardDefenseTerminalFact::HitPointsCurrent(value) => hp_current = Some(value),
                    HazardDefenseTerminalFact::HitPointsMaximum(value) => hp_maximum = Some(value),
                    HazardDefenseTerminalFact::HitPointsTemporary(value) => {
                        hp_temporary = Some(value)
                    }
                    HazardDefenseTerminalFact::BrokenThreshold(value) => {
                        broken_threshold = Some(value)
                    }
                    HazardDefenseTerminalFact::Fortitude(value) => fortitude = Some(value),
                    HazardDefenseTerminalFact::Reflex(value) => reflex = Some(value),
                    HazardDefenseTerminalFact::Will(value) => will = Some(value),
                },
                _ => remaining.push(fact),
            }
        }

        self.optional_section("Defenses", |out| {
            if let Some(value) = armor_class {
                out.field("AC", value.to_string(), 2);
            }
            match (hp_current, hp_maximum) {
                (Some(current), Some(maximum)) if current == maximum => {
                    out.field("HP", maximum.to_string(), 2)
                }
                (Some(current), Some(maximum)) => {
                    out.field("HP", format!("{current}/{maximum}"), 2)
                }
                (Some(current), None) => out.field("HP", format!("current {current}"), 2),
                (None, Some(maximum)) => out.field("HP", maximum.to_string(), 2),
                (None, None) => {}
            }
            if let Some(value) = hp_temporary.filter(|value| *value != 0) {
                out.field("Temporary HP", value.to_string(), 2);
            }
            if let Some(value) = hardness {
                out.field("Hardness", value.to_string(), 2);
            }
            if let Some(value) = broken_threshold {
                out.field("BT", value.to_string(), 2);
            }
            for (label, value) in [("Fort", fortitude), ("Ref", reflex), ("Will", will)] {
                if let Some(value) = value {
                    out.field(label, signed_number(value), 2);
                }
            }
            for fact in remaining {
                out.field(&fact.label, &fact.value, 2);
            }
        });
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

fn spell_fact<T>(value: &SpellFactJson<T>) -> Option<&T> {
    match value {
        SpellFactJson::Known(value) => Some(value),
        SpellFactJson::Missing | SpellFactJson::Null | SpellFactJson::Unsupported(_) => None,
    }
}

fn h8_fact<T>(value: &H8FactJson<T>) -> Option<&T> {
    match value {
        H8FactJson::Known(value) => Some(value),
        H8FactJson::Missing | H8FactJson::Null | H8FactJson::Unsupported(_) => None,
    }
}

fn h8_fact_state<T>(value: &H8FactJson<T>) -> &'static str {
    match value {
        H8FactJson::Missing => "missing",
        H8FactJson::Null => "null",
        H8FactJson::Known(_) => "available",
        H8FactJson::Unsupported(_) => "unsupported",
    }
}

fn resolved_spell_fact<T>(value: &SpellResolvedFieldJson<T>) -> Option<&T> {
    match value {
        SpellResolvedFieldJson::Available { value } => spell_fact(value),
        SpellResolvedFieldJson::Unavailable { .. } => None,
    }
}

fn nonempty_spell_text(value: &SpellFactJson<String>) -> Option<&str> {
    spell_fact(value)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
}

fn nonempty_spell_list(value: &SpellFactJson<Vec<String>>) -> Option<&[String]> {
    spell_fact(value)
        .filter(|values| !values.is_empty())
        .map(Vec::as_slice)
}

fn spell_area_text(area: &atlas_record::SpellAreaJson) -> String {
    let measurement = spell_fact(&area.value).map(|value| format!("{value}-foot"));
    [
        measurement,
        nonempty_spell_text(&area.area_type).map(str::to_string),
        nonempty_spell_text(&area.details).map(str::to_string),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ")
}

fn spell_damage_label(value: &SpellDamageJson) -> String {
    semantic_damage_label(&value.category, &value.kinds)
}

fn spell_damage_patch_label(
    value: &SpellDamagePatchJson,
    base_member: Option<&SpellDamageJson>,
) -> String {
    if nonempty_spell_text(&value.category).is_some() || nonempty_spell_list(&value.kinds).is_some()
    {
        return semantic_damage_label(&value.category, &value.kinds);
    }
    if matches!(value.category, SpellFactJson::Unsupported(_))
        || matches!(value.kinds, SpellFactJson::Unsupported(_))
    {
        return "Effect".to_string();
    }
    base_member
        .map(spell_damage_label)
        .unwrap_or_else(|| "Effect".to_string())
}

fn unique_spell_damage_member<'a>(
    damage: Option<&'a [SpellDamageJson]>,
    key: &str,
) -> Option<&'a SpellDamageJson> {
    let mut matches = damage?.iter().filter(|member| member.key == key);
    let member = matches.next()?;
    matches.next().is_none().then_some(member)
}

fn semantic_damage_label(
    category: &SpellFactJson<String>,
    kinds: &SpellFactJson<Vec<String>>,
) -> String {
    if let Some(category) = nonempty_spell_text(category) {
        let mut label = category.to_string();
        if let Some(first) = label.get_mut(0..1) {
            first.make_ascii_uppercase();
        }
        return format!("{label} damage");
    }
    let kinds = nonempty_spell_list(kinds).unwrap_or_default();
    let damage = kinds.iter().any(|kind| kind == "damage");
    let healing = kinds.iter().any(|kind| kind == "healing");
    match (damage, healing) {
        (true, true) => "Damage or healing".to_string(),
        (false, true) => "Healing".to_string(),
        (true, false) => "Damage".to_string(),
        (false, false) => "Effect".to_string(),
    }
}

fn spell_form_label(form: &SpellFormJson) -> String {
    match form.label_kind {
        SpellFormLabelKind::Base | SpellFormLabelKind::Authored => form.label.clone(),
        SpellFormLabelKind::Derived => match &form.result {
            SpellFormResultJson::Available { definition } => {
                derived_spell_form_label(definition).unwrap_or_else(|| "Alternate form".to_string())
            }
            SpellFormResultJson::Unavailable { .. } => "Alternate form".to_string(),
        },
    }
}

fn derived_spell_form_label(definition: &SpellResolvedDefinitionJson) -> Option<String> {
    let casting = resolved_spell_fact(&definition.casting)
        .and_then(|casting| nonempty_spell_text(&casting.time))
        .map(spell_action_label);
    let target = resolved_spell_fact(&definition.targeting).and_then(|targeting| {
        spell_fact(&targeting.area)
            .map(spell_area_text)
            .or_else(|| {
                spell_fact(&targeting.range)
                    .map(|range| range.authored_text.clone())
                    .filter(|value| !value.trim().is_empty())
            })
            .or_else(|| nonempty_spell_text(&targeting.target).map(str::to_string))
    });
    let parts = [casting, target].into_iter().flatten().collect::<Vec<_>>();
    (!parts.is_empty()).then(|| parts.join(" · "))
}

fn spell_action_label(time: &str) -> String {
    match time.trim() {
        "1" => "1 action".to_string(),
        "2" => "2 actions".to_string(),
        "3" => "3 actions".to_string(),
        value => value.to_string(),
    }
}

fn spell_rule_scope(value: &SpellFactJson<String>) -> Option<String> {
    nonempty_spell_text(value).map(readable_spell_rule_scope)
}

fn spell_rule_scopes(value: &SpellFactJson<Vec<String>>) -> Option<String> {
    let scopes = nonempty_spell_list(value)?;
    Some(
        scopes
            .iter()
            .map(|scope| readable_spell_rule_scope(scope))
            .collect::<Vec<_>>()
            .join(", "),
    )
}

fn readable_spell_rule_scope(value: &str) -> String {
    match value {
        "all" => "all checks".to_string(),
        "damage" => "damage".to_string(),
        "spell-attack-roll" => "spell attack rolls".to_string(),
        "spell-damage" => "spell damage".to_string(),
        _ => "not available for presentation".to_string(),
    }
}

fn ordinal_rank(value: u8) -> String {
    let suffix = match value % 100 {
        11..=13 => "th",
        _ => match value % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    };
    format!("{value}{suffix}")
}

fn signed_number(value: i64) -> String {
    format!("{value:+}")
}

fn spell_predicate_text(value: &SpellFactJson<Vec<SpellRulePredicateJson>>) -> Option<String> {
    let values = spell_fact(value)?;
    let rendered = values
        .iter()
        .filter_map(|value| match value {
            SpellRulePredicateJson::Term(value) => Some(value.clone()),
            SpellRulePredicateJson::Or(values) => Some(format!("one of ({})", values.join(", "))),
            SpellRulePredicateJson::Unsupported(_) => None,
        })
        .collect::<Vec<_>>();
    (!rendered.is_empty()).then(|| rendered.join("; "))
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
        FoundryRecordType, H8FactJson, H8PageSourceMetadataJson, H8ProvenanceJson,
        H8SourceMetadataJson, JournalJson, JournalPageEntryJson, JournalPageJson,
        JournalPageTextJson, JournalPageTitleJson, JournalPageVideoJson, RecordBody,
        RecordCanonicalRelationshipJson, RecordClassification, RecordEditionContextJson,
        RecordEditionCounterpartJson, RecordIdentity, RecordJsonBase, RecordJsonContext,
        RecordJsonOptions, RecordProvenance, RecordRelationshipProvenanceJson,
        ReferenceRelationKind, RetrievedRecord, RollTableJson, SpellClassification, SpellIdentity,
        SpellProvenance, SpellRangeValue, SpellRecord, SpellRitual, SpellSourceId,
        SpellSourceValue, SpellTargeting, SpellTradition, SpellTrait, TableResultEntryJson,
        TableResultJson, TableResultRangeJson, record_json, record_json_with_context,
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
    fn h8_terminal_renders_parent_owned_content_without_exposing_media_locators() {
        let page = JournalPageJson {
            locator: "opaque-page".to_string(),
            identity_stability: "stable_source_id",
            source_id: H8FactJson::Known("page-1".to_string()),
            source_ordinal: 0,
            name: H8FactJson::Known("Basic Actions".to_string()),
            page_kind: H8FactJson::Known("text"),
            sort: H8FactJson::Known(0),
            title: H8FactJson::Known(JournalPageTitleJson {
                show: H8FactJson::Known(true),
                level: H8FactJson::Known(2),
            }),
            text: H8FactJson::Known(JournalPageTextJson {
                content: H8FactJson::Known(atlas_record::RichDocument::new(vec![
                    atlas_record::RichNode::Text {
                        text: "Spend an action.".to_string(),
                    },
                ])),
                format: H8FactJson::Known(1),
                markdown: H8FactJson::Missing,
            }),
            source: H8FactJson::Known("secret-page.webp".to_string()),
            image_source: H8FactJson::Known("{}".to_string()),
            image_caption: H8FactJson::Missing,
            video: H8FactJson::Known(JournalPageVideoJson {
                controls: H8FactJson::Missing,
                loop_playback: H8FactJson::Missing,
                autoplay: H8FactJson::Missing,
                volume: H8FactJson::Missing,
                timestamp: H8FactJson::Missing,
                width: H8FactJson::Missing,
                height: H8FactJson::Missing,
            }),
            source_system: H8FactJson::Missing,
            source_metadata: H8PageSourceMetadataJson {
                ownership: H8FactJson::Missing,
                flags: H8FactJson::Missing,
                stats: H8FactJson::Missing,
            },
            unsupported_fields: Vec::new(),
        };
        let journal = JournalJson {
            source_id: "journal-1".to_string(),
            pages: H8FactJson::Known(vec![JournalPageEntryJson::Page {
                page: Box::new(page),
            }]),
            source_metadata: h8_source_metadata_json(),
            unsupported_fields: Vec::new(),
            provenance: h8_provenance_json(),
            content: Vec::new(),
        };
        let mut journal_writer = Writer::new(100, TerminalStyle::plain());
        journal_writer.journal_record(&journal, DetailLevel::Full);
        let journal_text = journal_writer.finish();
        assert!(journal_text.contains("Basic Actions"));
        assert!(journal_text.contains("Title level: 2"));
        assert!(journal_text.contains("Spend an action."));
        assert!(!journal_text.contains("secret-page.webp"));

        let table = RollTableJson {
            source_id: "table-1".to_string(),
            description: H8FactJson::Missing,
            results: H8FactJson::Known(vec![TableResultEntryJson::Result {
                result: Box::new(TableResultJson {
                    locator: "opaque-result".to_string(),
                    identity_stability: "stable_source_id",
                    source_id: H8FactJson::Known("result-1".to_string()),
                    source_ordinal: 0,
                    result_kind: H8FactJson::Known("text"),
                    text: H8FactJson::Known(atlas_record::RichDocument::new(vec![
                        atlas_record::RichNode::Text {
                            text: "Gain a hero point.".to_string(),
                        },
                    ])),
                    collection: H8FactJson::Null,
                    document_id: H8FactJson::Null,
                    weight: H8FactJson::Known("1".to_string()),
                    range: H8FactJson::Known(TableResultRangeJson { first: 1, last: 1 }),
                    drawn: H8FactJson::Known(false),
                    image: H8FactJson::Known("secret-result.webp".to_string()),
                    flags: H8FactJson::Missing,
                    unsupported_fields: Vec::new(),
                }),
            }]),
            formula: H8FactJson::Known("1d1".to_string()),
            replacement: H8FactJson::Known(true),
            display_roll: H8FactJson::Known(true),
            image: H8FactJson::Known("secret-table.webp".to_string()),
            source_metadata: h8_source_metadata_json(),
            unsupported_fields: Vec::new(),
            provenance: h8_provenance_json(),
            content: Vec::new(),
        };
        let mut table_writer = Writer::new(100, TerminalStyle::plain());
        table_writer.roll_table_record(&table, DetailLevel::Full);
        let table_text = table_writer.finish();
        assert!(table_text.contains("Formula: 1d1"));
        assert!(table_text.contains("1–1: Gain a hero point."));
        assert!(!table_text.contains("secret-table.webp"));
        assert!(!table_text.contains("secret-result.webp"));
    }

    fn h8_source_metadata_json() -> H8SourceMetadataJson {
        H8SourceMetadataJson {
            folder: H8FactJson::Missing,
            sort: H8FactJson::Known(0),
            ownership: H8FactJson::Missing,
            flags: H8FactJson::Missing,
            stats: H8FactJson::Missing,
        }
    }

    fn h8_provenance_json() -> H8ProvenanceJson {
        H8ProvenanceJson {
            source_path: "packs/h8/fixture.json".to_string(),
            source_contract_version: "pf2e-serialized-source/v1".to_string(),
            source_system_version: "7.7.0".to_string(),
            source_upstream_commit: "fixture".to_string(),
        }
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

    fn projected_spell_record(name: &str, source_id: &str, rank: u8) -> RecordJson {
        let key =
            atlas_domain::RecordKey::parse(&format!("spells-srd:{source_id}")).expect("spell key");
        let record = AtlasRecord::new(
            RecordIdentity::new(key.clone(), name),
            RecordClassification::new(atlas_domain::RecordKind::Spell),
            FoundryRecordInfo::new(
                "Spells",
                FoundryDocumentType::Item,
                FoundryRecordType::Spell,
            ),
            RecordProvenance::new(format!("packs/spells/{source_id}.json")),
        );
        let mut spell = SpellRecord::new(
            SpellIdentity {
                record_key: key,
                source_id: SpellSourceId::new(source_id).expect("source id"),
                name: name.to_string(),
            },
            SpellProvenance {
                source_path: format!("packs/spells/{source_id}.json"),
                source_contract_version: "fixture".to_string(),
                source_system_version: "6.12.4".to_string(),
                source_upstream_commit: "4cbdaa37d6c33e9519561bae2c59a23e0288cbce".to_string(),
                standalone_location: atlas_record::FactValue::Null,
            },
        );
        spell.definition.classification =
            atlas_record::FactValue::Value(SpellSourceValue::Known(SpellClassification {
                rank: atlas_record::FactValue::Value(SpellSourceValue::Known(rank)),
                traits: atlas_record::FactValue::Value(SpellSourceValue::Known(Vec::new())),
                traditions: atlas_record::FactValue::Value(SpellSourceValue::Known(Vec::new())),
            }));
        let retrieved = RetrievedRecord {
            record,
            body: Some(RecordBody::Spell(spell)),
            spell_children: Vec::new(),
        };
        record_json(
            &retrieved,
            RecordJsonOptions {
                detail: DetailLevel::Standard,
                include_source_json: false,
            },
        )
        .expect("spell projects")
    }

    #[test]
    fn canonical_spell_terminal_uses_authored_range_ritual_and_semantic_forms() {
        let key = atlas_domain::RecordKey::parse("spells-srd:testSpell").expect("key");
        let record = AtlasRecord::new(
            RecordIdentity::new(key.clone(), "Planar Test"),
            RecordClassification::new(atlas_domain::RecordKind::Spell),
            FoundryRecordInfo::new(
                "Spells",
                FoundryDocumentType::Item,
                FoundryRecordType::Spell,
            ),
            RecordProvenance::new("packs/spells/planar-test.json"),
        );
        let mut spell = SpellRecord::new(
            SpellIdentity {
                record_key: key,
                source_id: SpellSourceId::new("testSpell").expect("source id"),
                name: "Planar Test".to_string(),
            },
            SpellProvenance {
                source_path: "packs/spells/planar-test.json".to_string(),
                source_contract_version: "fixture".to_string(),
                source_system_version: "6.12.4".to_string(),
                source_upstream_commit: "fixture".to_string(),
                standalone_location: atlas_record::FactValue::Null,
            },
        );
        spell.definition.classification =
            atlas_record::FactValue::Value(SpellSourceValue::Known(SpellClassification {
                rank: atlas_record::FactValue::Value(SpellSourceValue::Known(7)),
                traits: atlas_record::FactValue::Value(SpellSourceValue::Known(vec![
                    SpellTrait::new("teleportation").expect("trait"),
                ])),
                traditions: atlas_record::FactValue::Value(SpellSourceValue::Known(vec![
                    SpellTradition::new("arcane").expect("tradition"),
                ])),
            }));
        spell.definition.targeting =
            atlas_record::FactValue::Value(SpellSourceValue::Known(SpellTargeting {
                target: atlas_record::FactValue::Null,
                range: atlas_record::FactValue::Value(SpellSourceValue::Known(
                    SpellRangeValue::from_authored_text("planetary"),
                )),
                area: atlas_record::FactValue::Null,
            }));
        spell.definition.ritual =
            atlas_record::FactValue::Value(SpellSourceValue::Known(SpellRitual {
                primary_check: atlas_record::FactValue::Value(SpellSourceValue::Known(
                    "Arcana (master)".to_string(),
                )),
                secondary_casters: atlas_record::FactValue::Value(SpellSourceValue::Known(2)),
                secondary_checks: atlas_record::FactValue::Value(SpellSourceValue::Known(
                    "Survival".to_string(),
                )),
            }));
        spell
            .definition
            .unsupported_notes
            .push(atlas_record::SpellUnsupportedSourceFact {
                field: atlas_record::SpellUnsupportedSourceField::RitualMember,
                source_path: "/system/ritual/private".to_string(),
                authored_key: "private".to_string(),
                authored_order: Some(0),
                value: atlas_record::UnsupportedSourceValue {
                    shape: atlas_record::UnsupportedSourceShape::Object,
                    value: "{\"source_note_private\":true}".to_string(),
                    reason: atlas_record::UnsupportedSourceReason::SourceFieldDrift,
                },
            });
        let retrieved = RetrievedRecord {
            record,
            body: Some(RecordBody::Spell(spell)),
            spell_children: Vec::new(),
        };
        let provenance = record_json_with_context(
            &retrieved,
            RecordJsonOptions {
                detail: DetailLevel::Standard,
                include_source_json: false,
            },
            RecordJsonContext::without_lookups(&retrieved.record).with_provenance_evidence(),
        )
        .expect("spell provenance projects");
        let provenance = serde_json::to_value(provenance).expect("provenance JSON serializes");
        assert_eq!(
            provenance.pointer("/spell/provenance/unsupported/0/value/value"),
            Some(&serde_json::json!("{\"source_note_private\":true}"))
        );
        assert!(provenance.get("presentation_issues").is_none());
        let json = record_json(
            &retrieved,
            RecordJsonOptions {
                detail: DetailLevel::Standard,
                include_source_json: false,
            },
        )
        .expect("spell projects");
        let rendered = render_record(&json, DetailLevel::Standard, 100, TerminalStyle::plain());

        assert!(rendered.contains("Rank: 7"));
        assert!(rendered.contains("Range: planetary"));
        assert!(rendered.contains("Primary check: Arcana (master)"));
        assert!(rendered.contains("Secondary casters: 2"));
        assert!(rendered.contains("Forms\n  - Base"));
        assert!(!rendered.contains("spell-form:"));
        assert_eq!(rendered.matches("Rank: 7").count(), 1);
        assert_eq!(rendered.matches("Traits: teleportation").count(), 1);
        assert!(!rendered.contains("range_value"));
        assert_eq!(
            rendered
                .matches("Ritual source fact: Ritual source fact is retained but not modeled.")
                .count(),
            1
        );
        assert!(!rendered.contains("source_note_private"));

        let mut derived_form_json = json.clone();
        let RecordPresentationJson::Spell { spell, .. } = &mut derived_form_json.presentation
        else {
            panic!("spell fixture presentation");
        };
        let mut derived = spell.forms.first().expect("base form").clone();
        derived.id = "spell-form:private-derived-identity".to_string();
        derived.label = "Overlay 1".to_string();
        derived.label_kind = SpellFormLabelKind::Derived;
        derived.order = 1;
        if let SpellFormResultJson::Available { definition } = &mut derived.result {
            definition.casting = SpellResolvedFieldJson::Available {
                value: SpellFactJson::Known(atlas_record::SpellCastingJson {
                    time: SpellFactJson::Known("1".to_string()),
                    cost: SpellFactJson::Null,
                    requirements: SpellFactJson::Known(String::new()),
                    counteraction: SpellFactJson::Known(false),
                }),
            };
            definition.targeting = SpellResolvedFieldJson::Available {
                value: SpellFactJson::Known(atlas_record::SpellTargetingJson {
                    target: SpellFactJson::Null,
                    range: SpellFactJson::Known(atlas_record::SpellRangeJson {
                        authored_text: "touch".to_string(),
                    }),
                    area: SpellFactJson::Null,
                }),
            };
        }
        spell.forms.push(derived);
        let mut three_action = spell.forms.first().expect("base form").clone();
        three_action.id = "spell-form:private-three-action".to_string();
        three_action.label = "Overlay 4".to_string();
        three_action.label_kind = SpellFormLabelKind::Derived;
        three_action.order = 2;
        if let SpellFormResultJson::Available { definition } = &mut three_action.result {
            definition.casting = SpellResolvedFieldJson::Available {
                value: SpellFactJson::Known(atlas_record::SpellCastingJson {
                    time: SpellFactJson::Known("3".to_string()),
                    cost: SpellFactJson::Null,
                    requirements: SpellFactJson::Known(String::new()),
                    counteraction: SpellFactJson::Known(false),
                }),
            };
            definition.targeting = SpellResolvedFieldJson::Available {
                value: SpellFactJson::Known(atlas_record::SpellTargetingJson {
                    target: SpellFactJson::Null,
                    range: SpellFactJson::Null,
                    area: SpellFactJson::Known(atlas_record::SpellAreaJson {
                        value: SpellFactJson::Known(30),
                        area_type: SpellFactJson::Known("emanation".to_string()),
                        details: SpellFactJson::Missing,
                    }),
                }),
            };
        }
        spell.forms.push(three_action);
        let rendered = render_record(
            &derived_form_json,
            DetailLevel::Standard,
            100,
            TerminalStyle::plain(),
        );
        assert!(rendered.contains("- 1 action · touch"));
        assert!(rendered.contains("- 3 actions · 30-foot emanation"));
        assert!(!rendered.contains("Overlay 1"));
        assert!(!rendered.contains("Overlay 4"));
        assert!(!rendered.contains("spell-form:private"));

        let mut field_unavailable_json = json.clone();
        let RecordPresentationJson::Spell { spell, .. } = &mut field_unavailable_json.presentation
        else {
            panic!("spell fixture presentation");
        };
        let SpellFormResultJson::Available { definition } =
            &mut spell.forms.first_mut().expect("base form").result
        else {
            panic!("available base form");
        };
        definition.casting = SpellResolvedFieldJson::Unavailable {
            field: "casting",
            source: "fixed_heightening",
            reason: "unsupported_patch".to_string(),
        };
        let machine =
            serde_json::to_value(&field_unavailable_json).expect("record JSON should serialize");
        assert_eq!(
            machine.pointer("/spell/forms/0/result/definition/casting/state"),
            Some(&serde_json::json!("unavailable"))
        );
        assert_eq!(
            machine.pointer("/spell/forms/0/result/definition/casting/reason"),
            Some(&serde_json::json!("unsupported_patch"))
        );
        let rendered = render_record(
            &field_unavailable_json,
            DetailLevel::Standard,
            100,
            TerminalStyle::plain(),
        );
        assert_eq!(
            rendered.matches("Casting: Casting is unavailable.").count(),
            1
        );
        assert!(!rendered.contains("fixed_heightening"));
        assert!(!rendered.contains("unsupported_patch"));

        let mut form_unavailable_json = json.clone();
        let RecordPresentationJson::Spell { spell, .. } = &mut form_unavailable_json.presentation
        else {
            panic!("spell fixture presentation");
        };
        spell.forms.first_mut().expect("base form").result = SpellFormResultJson::Unavailable {
            message: "private selection failure".to_string(),
        };
        let rendered = render_record(
            &form_unavailable_json,
            DetailLevel::Standard,
            100,
            TerminalStyle::plain(),
        );
        assert_eq!(
            rendered
                .matches("Spell form: Spell form is unavailable.")
                .count(),
            1
        );
        assert!(!rendered.contains("private selection failure"));

        let mut unsupported_json = json;
        let RecordPresentationJson::Spell { spell, .. } = &mut unsupported_json.presentation else {
            panic!("spell fixture presentation");
        };
        spell.classification =
            SpellFactJson::Unsupported(atlas_record::SpellUnsupportedValueJson {
                shape: "object",
                value: "{\"future\":true}".to_string(),
                reason: "source_field_drift",
            });
        let rendered = render_record(
            &unsupported_json,
            DetailLevel::Standard,
            100,
            TerminalStyle::plain(),
        );
        assert!(rendered.contains("Data issues"));
        assert!(
            rendered.contains("Classification: Classification is not supported for presentation.")
        );
        assert!(!rendered.contains("future"));
        assert!(!rendered.contains("packs/spells"));

        use atlas_record::{
            SpellCastingJson, SpellClassificationJson, SpellRollOptionRuleJson,
            SpellRuleSuboptionJson,
        };
        fn unsupported<T>() -> SpellFactJson<T> {
            SpellFactJson::Unsupported(atlas_record::SpellUnsupportedValueJson {
                shape: "object",
                value: "{\"private\":true}".to_string(),
                reason: "source_field_drift",
            })
        }
        let RecordPresentationJson::Spell { spell, .. } = &mut unsupported_json.presentation else {
            panic!("spell fixture presentation");
        };
        spell.classification = SpellFactJson::Known(SpellClassificationJson {
            rank: unsupported(),
            traits: SpellFactJson::Known(Vec::new()),
            traditions: SpellFactJson::Known(Vec::new()),
        });
        spell.casting = SpellFactJson::Known(SpellCastingJson {
            time: unsupported(),
            cost: SpellFactJson::Null,
            requirements: SpellFactJson::Known(String::new()),
            counteraction: SpellFactJson::Known(false),
        });
        spell.rules = SpellFactJson::Known(vec![
            SpellRuleJson {
                authored_key: "RollOption".to_string(),
                order: 0,
                rule: SpellRuleDetailJson::RollOption(Box::new(SpellRollOptionRuleJson {
                    domain: SpellFactJson::Known("all".to_string()),
                    label: SpellFactJson::Missing,
                    option: SpellFactJson::Known("fixture-option".to_string()),
                    placement: SpellFactJson::Known("spellcasting".to_string()),
                    predicate: SpellFactJson::Known(vec![SpellRulePredicateJson::Unsupported(
                        atlas_record::SpellUnsupportedValueJson {
                            shape: "object",
                            value: "{\"predicate\":true}".to_string(),
                            reason: "source_field_drift",
                        },
                    )]),
                    suboptions: SpellFactJson::Known(vec![SpellRuleSuboptionJson {
                        label: unsupported(),
                        value: SpellFactJson::Known("fire".to_string()),
                    }]),
                    toggleable: SpellFactJson::Known(false),
                })),
            },
            SpellRuleJson {
                authored_key: "FutureRule".to_string(),
                order: 1,
                rule: SpellRuleDetailJson::Unsupported(atlas_record::SpellUnsupportedValueJson {
                    shape: "object",
                    value: "{\"whole_rule\":true}".to_string(),
                    reason: "unmapped_discriminant",
                }),
            },
        ]);
        let machine =
            serde_json::to_value(&unsupported_json).expect("record JSON should serialize");
        assert_eq!(
            machine.pointer("/spell/classification/value/rank/state"),
            Some(&serde_json::json!("unsupported"))
        );
        assert_eq!(
            machine.pointer("/spell/rules/value/0/value/predicate/value/0/kind"),
            Some(&serde_json::json!("unsupported"))
        );

        let rendered = render_record(
            &unsupported_json,
            DetailLevel::Standard,
            100,
            TerminalStyle::plain(),
        );
        for expected in [
            "Rank: Rank is not supported for presentation.",
            "Casting time: Casting time is not supported for presentation.",
            "Roll option predicate: Roll option predicate is not supported for presentation.",
            "Roll option suboption label: Roll option suboption label is not supported for presentation.",
            "Unsupported rule: Unsupported rule is not supported for presentation.",
        ] {
            assert_eq!(
                rendered.matches(expected).count(),
                1,
                "nested issue should render exactly once: {expected}\n{rendered}"
            );
        }
        assert!(!rendered.contains("private"));
        assert!(!rendered.contains("predicate\":true"));
        assert!(!rendered.contains("whole_rule"));
    }

    #[test]
    fn spell_terminal_renders_source_shaped_damage_and_heightening_without_machine_labels() {
        use atlas_record::{
            SpellAreaJson, SpellDamagePatchJson, SpellDamagePatchMemberJson,
            SpellDamagePatchSetJson, SpellFixedHeighteningJson, SpellHeighteningDamageJson,
            SpellRangeJson, SpellTargetingJson,
        };

        fn missing<T>() -> SpellFactJson<T> {
            SpellFactJson::Missing
        }
        fn fixed_layer(rank: u8, area: u32, formula: &str) -> SpellFixedHeighteningJson {
            SpellFixedHeighteningJson {
                key: rank.to_string(),
                order: u32::from(rank),
                rank: SpellFactJson::Known(rank),
                changed_fields: vec!["targeting", "damage"],
                patch: SpellPatchJson {
                    classification: missing(),
                    casting: missing(),
                    targeting: SpellFactJson::Known(SpellTargetingJson {
                        target: missing(),
                        range: missing(),
                        area: SpellFactJson::Known(SpellAreaJson {
                            value: SpellFactJson::Known(area),
                            area_type: SpellFactJson::Known("burst".to_string()),
                            details: missing(),
                        }),
                    }),
                    defense: missing(),
                    damage: SpellFactJson::Known(SpellDamagePatchSetJson {
                        members: vec![SpellDamagePatchMemberJson {
                            key: "0".to_string(),
                            order: 0,
                            operation: SpellDamagePatchOperationJson::Merge(Box::new(
                                SpellDamagePatchJson {
                                    formula: SpellFactJson::Known(formula.to_string()),
                                    damage_type: SpellFactJson::Known("cold".to_string()),
                                    category: missing(),
                                    kinds: missing(),
                                    materials: SpellFactJson::Known(Vec::new()),
                                    apply_modifier: SpellFactJson::Known(false),
                                },
                            )),
                        }],
                    }),
                    duration: missing(),
                    heightening: missing(),
                    rules: missing(),
                    unsupported: Vec::new(),
                },
            }
        }

        let mut rime = projected_spell_record("Rime Slick", "rime-slick", 2);
        let RecordPresentationJson::Spell { spell, .. } = &mut rime.presentation else {
            panic!("spell presentation")
        };
        spell.targeting = SpellFactJson::Known(SpellTargetingJson {
            target: SpellFactJson::Null,
            range: SpellFactJson::Known(SpellRangeJson {
                authored_text: "120 feet".to_string(),
            }),
            area: SpellFactJson::Known(SpellAreaJson {
                value: SpellFactJson::Known(15),
                area_type: SpellFactJson::Known("burst".to_string()),
                details: SpellFactJson::Known(String::new()),
            }),
        });
        spell.damage = SpellFactJson::Known(vec![SpellDamageJson {
            key: "0".to_string(),
            order: 0,
            formula: SpellFactJson::Known("2d4".to_string()),
            damage_type: SpellFactJson::Known("cold".to_string()),
            category: SpellFactJson::Null,
            kinds: SpellFactJson::Known(vec!["damage".to_string()]),
            materials: SpellFactJson::Known(Vec::new()),
            apply_modifier: SpellFactJson::Known(false),
        }]);
        spell.duration = SpellFactJson::Known(atlas_record::SpellDurationJson {
            value: SpellFactJson::Known(String::new()),
            sustained: SpellFactJson::Known(false),
        });
        spell.heightening = SpellFactJson::Known(SpellHeighteningJson::Fixed {
            layers: vec![fixed_layer(5, 30, "8d4"), fixed_layer(8, 60, "14d4")],
        });
        let machine = serde_json::to_value(&rime).expect("Rime machine JSON");
        assert_eq!(
            machine.pointer("/spell/damage/value/0/key"),
            Some(&serde_json::json!("0"))
        );
        assert_eq!(
            machine.pointer("/spell/damage/value/0/materials/value"),
            Some(&serde_json::json!([]))
        );
        assert_eq!(
            machine.pointer("/spell/damage/value/0/apply_modifier/value"),
            Some(&serde_json::json!(false))
        );
        assert_eq!(
            machine.pointer("/spell/targeting/value/area/value/details/value"),
            Some(&serde_json::json!(""))
        );
        assert!(
            machine
                .pointer("/spell/forms/0/id")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| value.starts_with("spell-form:"))
        );
        assert!(machine.pointer("/spell/forms/0/label_kind").is_none());

        let rendered = render_record(&rime, DetailLevel::Standard, 100, TerminalStyle::plain());
        for expected in [
            "Rank: 2",
            "Area: 15-foot burst",
            "Damage: 2d4 cold",
            "Heightened (5th)",
            "Area: 30-foot burst",
            "Damage: 8d4 cold",
            "Heightened (8th)",
            "Area: 60-foot burst",
            "Damage: 14d4 cold",
            "Forms\n  - Base",
        ] {
            assert!(
                rendered.contains(expected),
                "missing {expected:?}:\n{rendered}"
            );
        }
        for absent in [
            "Damage 0",
            "0: 2d6",
            "spell-form:",
            "Overlay 1",
            "Not authored",
            "Authored null",
            "Duration\n",
            "Modifier: false",
        ] {
            assert!(!rendered.contains(absent), "leaked {absent:?}:\n{rendered}");
        }
        assert_eq!(rendered.matches("Rank: 2").count(), 1);

        let mut fireball = projected_spell_record("Fireball", "fireball", 3);
        let RecordPresentationJson::Spell { spell, .. } = &mut fireball.presentation else {
            panic!("spell presentation")
        };
        spell.targeting = SpellFactJson::Known(SpellTargetingJson {
            target: SpellFactJson::Null,
            range: SpellFactJson::Known(SpellRangeJson {
                authored_text: "500 feet".to_string(),
            }),
            area: SpellFactJson::Known(SpellAreaJson {
                value: SpellFactJson::Known(20),
                area_type: SpellFactJson::Known("burst".to_string()),
                details: SpellFactJson::Missing,
            }),
        });
        spell.damage = SpellFactJson::Known(vec![
            SpellDamageJson {
                key: "0".to_string(),
                order: 0,
                formula: SpellFactJson::Known("6d6".to_string()),
                damage_type: SpellFactJson::Known("fire".to_string()),
                category: SpellFactJson::Null,
                kinds: SpellFactJson::Known(vec!["damage".to_string()]),
                materials: SpellFactJson::Known(Vec::new()),
                apply_modifier: SpellFactJson::Known(false),
            },
            SpellDamageJson {
                key: "persistent".to_string(),
                order: 1,
                formula: SpellFactJson::Known("1d6".to_string()),
                damage_type: SpellFactJson::Known("fire".to_string()),
                category: SpellFactJson::Known("persistent".to_string()),
                kinds: SpellFactJson::Known(vec!["damage".to_string()]),
                materials: SpellFactJson::Known(Vec::new()),
                apply_modifier: SpellFactJson::Known(false),
            },
        ]);
        spell.heightening = SpellFactJson::Known(SpellHeighteningJson::Interval {
            interval: SpellFactJson::Known(1),
            area: SpellFactJson::Missing,
            damage: SpellFactJson::Known(vec![SpellHeighteningDamageJson {
                key: "0".to_string(),
                order: 0,
                value: "2d6".to_string(),
            }]),
        });
        let rendered = render_record(
            &fireball,
            DetailLevel::Standard,
            100,
            TerminalStyle::plain(),
        );
        for expected in [
            "Area: 20-foot burst",
            "Damage: 6d6 fire",
            "Persistent damage: 1d6 fire",
            "Heightened (+1)",
            "Damage: +2d6",
        ] {
            assert!(
                rendered.contains(expected),
                "missing {expected:?}:\n{rendered}"
            );
        }
        assert!(!rendered.contains("0: 2d6"));
        assert!(!rendered.contains("Damage 0"));
        assert!(!rendered.contains("Persistent damage: 1d6 fire persistent"));

        let RecordPresentationJson::Spell { spell, .. } = &mut fireball.presentation else {
            panic!("spell presentation")
        };
        let base_damage = spell_fact(&spell.damage)
            .expect("Fireball damage")
            .first()
            .expect("Fireball base damage")
            .clone();
        let SpellFactJson::Known(damage) = &mut spell.damage else {
            panic!("Fireball damage")
        };
        damage.push(base_damage);
        let rendered = render_record(
            &fireball,
            DetailLevel::Standard,
            100,
            TerminalStyle::plain(),
        );
        assert!(rendered.contains("Effect: +2d6"));
        assert!(!rendered.contains("Damage: +2d6"));

        for (name, source_id, rank) in [
            ("Heal", "heal", 1),
            ("Planar Displacement", "planar-displacement", 7),
        ] {
            let mut quiet = projected_spell_record(name, source_id, rank);
            let RecordPresentationJson::Spell { spell, .. } = &mut quiet.presentation else {
                panic!("spell presentation")
            };
            spell.damage = SpellFactJson::Known(Vec::new());
            spell.duration = SpellFactJson::Known(atlas_record::SpellDurationJson {
                value: SpellFactJson::Known(String::new()),
                sustained: SpellFactJson::Known(false),
            });
            let machine = serde_json::to_value(&quiet).expect("quiet spell machine JSON");
            assert_eq!(
                machine.pointer("/spell/damage/state"),
                Some(&serde_json::json!("known"))
            );
            assert_eq!(
                machine.pointer("/spell/damage/value"),
                Some(&serde_json::json!([]))
            );
            assert_eq!(
                machine.pointer("/spell/duration/value/sustained/value"),
                Some(&serde_json::json!(false))
            );
            let rendered =
                render_record(&quiet, DetailLevel::Standard, 100, TerminalStyle::plain());
            assert!(!rendered.contains("\nEffect\n"), "{name}:\n{rendered}");
            assert!(!rendered.contains("Duration:"), "{name}:\n{rendered}");
            assert!(!rendered.contains("Sustained:"), "{name}:\n{rendered}");
        }
    }

    #[test]
    fn fixed_heightening_uses_only_unique_typed_member_semantics() {
        use atlas_record::{SpellDamagePatchMemberJson, SpellDamagePatchSetJson};

        fn missing<T>() -> SpellFactJson<T> {
            SpellFactJson::Missing
        }
        fn base_damage(key: &str, order: u32, kinds: &[&str]) -> SpellDamageJson {
            SpellDamageJson {
                key: key.to_string(),
                order,
                formula: SpellFactJson::Known("1d8".to_string()),
                damage_type: SpellFactJson::Known("vitality".to_string()),
                category: missing(),
                kinds: SpellFactJson::Known(kinds.iter().map(|kind| (*kind).to_string()).collect()),
                materials: SpellFactJson::Known(Vec::new()),
                apply_modifier: SpellFactJson::Known(false),
            }
        }
        fn patch_damage(
            formula: &str,
            damage_type: &str,
            category: SpellFactJson<String>,
            kinds: SpellFactJson<Vec<String>>,
        ) -> SpellDamagePatchJson {
            SpellDamagePatchJson {
                formula: SpellFactJson::Known(formula.to_string()),
                damage_type: SpellFactJson::Known(damage_type.to_string()),
                category,
                kinds,
                materials: SpellFactJson::Known(Vec::new()),
                apply_modifier: SpellFactJson::Known(false),
            }
        }
        let base = vec![
            base_damage("member-healing", 0, &["healing"]),
            base_damage("member-duplicate", 1, &["damage"]),
            base_damage("member-duplicate", 2, &["healing"]),
        ];
        let patch =
            SpellPatchJson {
                classification: missing(),
                casting: missing(),
                targeting: missing(),
                defense: missing(),
                damage: SpellFactJson::Known(SpellDamagePatchSetJson {
                    members: vec![
                        SpellDamagePatchMemberJson {
                            key: "member-healing".to_string(),
                            order: 0,
                            operation: SpellDamagePatchOperationJson::Merge(Box::new(
                                patch_damage("2d8", "vitality", missing(), missing()),
                            )),
                        },
                        SpellDamagePatchMemberJson {
                            key: "member-healing".to_string(),
                            order: 1,
                            operation: SpellDamagePatchOperationJson::Delete,
                        },
                        SpellDamagePatchMemberJson {
                            key: "member-absent".to_string(),
                            order: 2,
                            operation: SpellDamagePatchOperationJson::Merge(Box::new(
                                patch_damage("1d6", "force", missing(), missing()),
                            )),
                        },
                        SpellDamagePatchMemberJson {
                            key: "member-duplicate".to_string(),
                            order: 3,
                            operation: SpellDamagePatchOperationJson::Merge(Box::new(
                                patch_damage("2d6", "spirit", missing(), missing()),
                            )),
                        },
                        SpellDamagePatchMemberJson {
                            key: "member-healing".to_string(),
                            order: 4,
                            operation: SpellDamagePatchOperationJson::Merge(Box::new(
                                patch_damage(
                                    "1d4",
                                    "fire",
                                    SpellFactJson::Known("persistent".to_string()),
                                    missing(),
                                ),
                            )),
                        },
                        SpellDamagePatchMemberJson {
                            key: "member-healing".to_string(),
                            order: 5,
                            operation: SpellDamagePatchOperationJson::Merge(Box::new(
                                patch_damage("1d10", "vitality", SpellFactJson::Null, missing()),
                            )),
                        },
                        SpellDamagePatchMemberJson {
                            key: "member-healing".to_string(),
                            order: 6,
                            operation: SpellDamagePatchOperationJson::Merge(Box::new(
                                patch_damage(
                                    "1d12",
                                    "vitality",
                                    SpellFactJson::Unsupported(
                                        atlas_record::SpellUnsupportedValueJson {
                                            shape: "object",
                                            value: "{\"future\":true}".to_string(),
                                            reason: "source_field_drift",
                                        },
                                    ),
                                    missing(),
                                ),
                            )),
                        },
                    ],
                }),
                duration: missing(),
                heightening: missing(),
                rules: missing(),
                unsupported: Vec::new(),
            };
        let machine = serde_json::to_value(&patch).expect("fixed patch serializes");
        assert_eq!(
            machine.pointer("/damage/value/members/0/value/category/state"),
            Some(&serde_json::json!("missing"))
        );
        assert_eq!(
            machine.pointer("/damage/value/members/5/value/category/state"),
            Some(&serde_json::json!("null"))
        );

        let mut writer = Writer::new(120, TerminalStyle::plain());
        writer.spell_patch(&patch, Some(&base), 0);
        let rendered = writer.finish();
        for expected in [
            "Healing: 2d8 vitality",
            "Healing: removed",
            "Effect: 1d6 force",
            "Effect: 2d6 spirit",
            "Persistent damage: 1d4 fire",
            "Healing: 1d10 vitality",
            "Effect: 1d12 vitality",
        ] {
            assert!(
                rendered.contains(expected),
                "missing {expected:?}:\n{rendered}"
            );
        }
        for hidden_key in ["member-healing", "member-absent", "member-duplicate"] {
            assert!(
                !rendered.contains(hidden_key),
                "authored key leaked: {hidden_key}\n{rendered}"
            );
        }
        assert!(!rendered.contains("Persistent damage: 1d4 fire persistent"));
    }

    #[test]
    fn hazard_terminal_uses_typed_scan_annotations_and_semantic_issues() {
        use atlas_record::{RecordFactJson, RecordSectionJson};

        let fact =
            |key: &str,
             label: &str,
             value: &str,
             terminal: Option<RecordFactTerminalPresentation>| RecordFactJson {
                key: key.to_string(),
                label: label.to_string(),
                value: value.to_string(),
                terminal,
            };
        let sections = vec![
            RecordSectionJson {
                kind: "defense",
                title: "Defenses".to_string(),
                blocks: vec![RecordBlockJson::FactList {
                    facts: vec![
                        fact(
                            "armor_class",
                            "Armor Class",
                            "28",
                            Some(RecordFactTerminalPresentation::HazardDefense(
                                HazardDefenseTerminalFact::ArmorClass(28),
                            )),
                        ),
                        fact(
                            "hit_points.current",
                            "Hit Points",
                            "40",
                            Some(RecordFactTerminalPresentation::HazardDefense(
                                HazardDefenseTerminalFact::HitPointsCurrent(40),
                            )),
                        ),
                        fact(
                            "hit_points.maximum",
                            "Maximum Hit Points",
                            "40",
                            Some(RecordFactTerminalPresentation::HazardDefense(
                                HazardDefenseTerminalFact::HitPointsMaximum(40),
                            )),
                        ),
                        fact(
                            "hit_points.temporary",
                            "Temporary Hit Points",
                            "0",
                            Some(RecordFactTerminalPresentation::HazardDefense(
                                HazardDefenseTerminalFact::HitPointsTemporary(0),
                            )),
                        ),
                        fact(
                            "hit_points.broken_threshold",
                            "Broken Threshold",
                            "20",
                            Some(RecordFactTerminalPresentation::HazardDefense(
                                HazardDefenseTerminalFact::BrokenThreshold(20),
                            )),
                        ),
                        fact(
                            "save.will",
                            "Will",
                            "+0",
                            Some(RecordFactTerminalPresentation::HazardDefense(
                                HazardDefenseTerminalFact::Will(0),
                            )),
                        ),
                        fact("immunities", "Immunities", "critical hits, precision", None),
                    ],
                }],
            },
            RecordSectionJson {
                kind: "offense",
                title: "Activities".to_string(),
                blocks: vec![RecordBlockJson::FactList {
                    facts: vec![
                        fact(
                            "entity.synthetic-strike-rule",
                            "Acid Spray",
                            "Strike",
                            Some(RecordFactTerminalPresentation::HazardStrike {
                                mode: Some("Melee"),
                                action_cost: Some(1),
                            }),
                        ),
                        fact(
                            "entity.synthetic-strike-rule.damage.0",
                            "Damage 0",
                            "2d8 acid",
                            Some(RecordFactTerminalPresentation::HazardDamage),
                        ),
                    ],
                }],
            },
        ];
        let availability = vec![
            HazardAvailabilityJson {
                state: HazardAvailabilityStateJson::Null,
                field: "occurrence.source_folder".to_string(),
                component_id: Some("hazard-occurrence-private".to_string()),
                message: "occurrence.source_folder data is null.".to_string(),
                human_label: "Activity folder",
                human_message: "Activity folder is null.".to_string(),
            },
            HazardAvailabilityJson {
                state: HazardAvailabilityStateJson::Unsupported,
                field: "entity.unsupported.action.unexpected./items/0/system/traits/selected"
                    .to_string(),
                component_id: Some("hazard-entity-private".to_string()),
                message: "raw source path is unsupported".to_string(),
                human_label: "Action source fact",
                human_message: "Action source fact is unsupported.".to_string(),
            },
        ];
        let mut record = review_fixture(DetailLevel::Standard);
        record.presentation = RecordPresentationJson::Hazard {
            sections,
            availability,
            provenance: None,
            edition: None,
            record_relationships: None,
        };
        let machine = serde_json::to_value(&record).expect("hazard machine JSON");
        assert!(machine.to_string().contains("\"hit_points.temporary\""));
        assert!(machine.to_string().contains("\"value\":\"0\""));
        assert!(!machine.to_string().contains("human_label"));
        assert!(!machine.to_string().contains("terminal"));

        let rendered = render_record(&record, DetailLevel::Standard, 100, TerminalStyle::plain());
        for expected in [
            "AC: 28",
            "HP: 40",
            "BT: 20",
            "Will: +0",
            "Immunities: critical hits, precision",
            "Acid Spray: Melee Strike",
            "Actions: 1",
            "Damage: 2d8 acid",
            "Action source fact: Action source fact is unsupported.",
        ] {
            assert!(
                rendered.contains(expected),
                "missing {expected:?}:\n{rendered}"
            );
        }
        for absent in [
            "Temporary HP",
            "Maximum Hit Points",
            "Damage 0",
            "hazard-occurrence-private",
            "hazard-entity-private",
            "/items/0/",
            "Activity folder is null",
        ] {
            assert!(!rendered.contains(absent), "leaked {absent:?}:\n{rendered}");
        }
        assert_eq!(
            rendered
                .matches("Action source fact: Action source fact is unsupported.")
                .count(),
            1
        );
    }

    #[test]
    fn spell_terminal_traverses_fixed_patch_and_rule_values() {
        use atlas_record::{
            SpellAreaJson, SpellCastingJson, SpellClassificationJson, SpellDamageDiceRuleJson,
            SpellDamagePatchJson, SpellDamagePatchMemberJson, SpellDamagePatchSetJson,
            SpellDefenseJson, SpellDurationJson, SpellHeighteningPatchJson,
            SpellRollOptionRuleJson, SpellRuleSuboptionJson, SpellSaveJson, SpellTargetingJson,
            SpellTextPatchMemberJson, SpellTextPatchSetJson,
        };

        fn missing<T>() -> SpellFactJson<T> {
            SpellFactJson::Missing
        }
        let patch = SpellPatchJson {
            classification: SpellFactJson::Known(SpellClassificationJson {
                rank: missing(),
                traits: SpellFactJson::Known(vec![
                    "concentrate".to_string(),
                    "manipulate".to_string(),
                ]),
                traditions: SpellFactJson::Known(vec!["divine".to_string()]),
            }),
            casting: SpellFactJson::Known(SpellCastingJson {
                time: SpellFactJson::Known("3".to_string()),
                cost: SpellFactJson::Known("1 offering".to_string()),
                requirements: SpellFactJson::Known("peaceful remains".to_string()),
                counteraction: SpellFactJson::Known(false),
            }),
            targeting: SpellFactJson::Known(SpellTargetingJson {
                target: SpellFactJson::Known("1 corpse".to_string()),
                range: missing(),
                area: SpellFactJson::Known(SpellAreaJson {
                    value: SpellFactJson::Known(10),
                    area_type: SpellFactJson::Known("burst".to_string()),
                    details: missing(),
                }),
            }),
            defense: SpellFactJson::Known(SpellDefenseJson {
                passive: SpellFactJson::Known("ac".to_string()),
                save: SpellFactJson::Known(SpellSaveJson {
                    statistic: SpellFactJson::Known("will".to_string()),
                    basic: SpellFactJson::Known(false),
                }),
            }),
            damage: SpellFactJson::Known(SpellDamagePatchSetJson {
                members: vec![
                    SpellDamagePatchMemberJson {
                        key: "0".to_string(),
                        order: 0,
                        operation: SpellDamagePatchOperationJson::Merge(Box::new(
                            SpellDamagePatchJson {
                                formula: SpellFactJson::Known("4d4".to_string()),
                                damage_type: SpellFactJson::Known("cold".to_string()),
                                category: missing(),
                                kinds: SpellFactJson::Known(vec!["damage".to_string()]),
                                materials: SpellFactJson::Known(vec!["silver".to_string()]),
                                apply_modifier: SpellFactJson::Known(false),
                            },
                        )),
                    },
                    SpellDamagePatchMemberJson {
                        key: "healing".to_string(),
                        order: 1,
                        operation: SpellDamagePatchOperationJson::Merge(Box::new(
                            SpellDamagePatchJson {
                                formula: SpellFactJson::Known("1d8".to_string()),
                                damage_type: SpellFactJson::Known("vitality".to_string()),
                                category: missing(),
                                kinds: SpellFactJson::Known(vec![
                                    "damage".to_string(),
                                    "healing".to_string(),
                                ]),
                                materials: SpellFactJson::Known(Vec::new()),
                                apply_modifier: SpellFactJson::Known(true),
                            },
                        )),
                    },
                ],
            }),
            duration: SpellFactJson::Known(SpellDurationJson {
                value: SpellFactJson::Known("1 minute".to_string()),
                sustained: SpellFactJson::Known(false),
            }),
            heightening: SpellFactJson::Known(SpellHeighteningPatchJson {
                kind: SpellFactJson::Known("interval".to_string()),
                interval: SpellFactJson::Known(2),
                area: SpellFactJson::Known(5),
                damage: SpellFactJson::Known(SpellTextPatchSetJson {
                    members: vec![SpellTextPatchMemberJson {
                        key: "0".to_string(),
                        order: 0,
                        operation: SpellTextPatchOperationJson::Merge(SpellFactJson::Known(
                            "1d4".to_string(),
                        )),
                    }],
                }),
            }),
            rules: missing(),
            unsupported: Vec::new(),
        };
        let qi_rule = SpellRuleJson {
            authored_key: "RollOption".to_string(),
            order: 0,
            rule: SpellRuleDetailJson::RollOption(Box::new(SpellRollOptionRuleJson {
                domain: SpellFactJson::Known("all".to_string()),
                label: SpellFactJson::Known("Heaven's Thunder".to_string()),
                option: SpellFactJson::Known("heavens-thunder".to_string()),
                placement: SpellFactJson::Known("spellcasting".to_string()),
                predicate: missing(),
                suboptions: SpellFactJson::Known(vec![SpellRuleSuboptionJson {
                    label: SpellFactJson::Known("Electricity".to_string()),
                    value: SpellFactJson::Known("electricity".to_string()),
                }]),
                toggleable: SpellFactJson::Known(true),
            })),
        };
        let damage_dice_rule = SpellRuleJson {
            authored_key: "DamageDice".to_string(),
            order: 1,
            rule: SpellRuleDetailJson::DamageDice(Box::new(SpellDamageDiceRuleJson {
                selector: missing(),
                predicate: missing(),
                dice_number: missing(),
                die_size: missing(),
                damage_type: missing(),
                hide_if_disabled: SpellFactJson::Known(false),
            })),
        };

        let mut writer = Writer::new(120, TerminalStyle::plain());
        writer.spell_patch(&patch, None, 0);
        writer.spell_rule(&qi_rule, 0);
        writer.spell_rule(&damage_dice_rule, 0);
        let rendered = writer.finish();

        for expected in [
            "Traits: concentrate, manipulate",
            "Traditions: divine",
            "Time: 3",
            "Cost: 1 offering",
            "Requirements: peaceful remains",
            "Target: 1 corpse",
            "Area: 10-foot burst",
            "Defense: ac",
            "Save: will",
            "Damage: 4d4 cold",
            "Materials: silver",
            "Damage or healing: 1d8 vitality",
            "Modifier: applies",
            "Duration: 1 minute",
            "Heightening: interval",
            "Heightening interval: 2",
            "Heightening area: 5",
            "Effect increase: 1d4",
            "Roll option: Heaven's Thunder",
            "Choice: Electricity",
            "Toggleable: yes",
            "Damage dice adjustment",
        ] {
            assert!(
                rendered.contains(expected),
                "missing `{expected}` from:\n{rendered}"
            );
        }
        for internal in [
            "Counteraction: false",
            "Basic save: false",
            "Damage 0",
            "Sustained: false",
            "= electricity",
            "Hide if disabled",
        ] {
            assert!(
                !rendered.contains(internal),
                "machine-only value leaked `{internal}` into:\n{rendered}"
            );
        }
        assert!(!rendered.contains("authored_object_json"));
    }

    #[test]
    fn spell_terminal_preserves_typed_rule_visibility_without_internal_scopes() {
        use atlas_record::{
            SpellDamageAlterationRuleJson, SpellDamageDiceRuleJson, SpellEphemeralEffectRuleJson,
        };

        fn missing<T>() -> SpellFactJson<T> {
            SpellFactJson::Missing
        }
        let mut record = projected_spell_record("Rule Test", "rule-test", 3);
        let RecordPresentationJson::Spell { spell, .. } = &mut record.presentation else {
            panic!("spell presentation")
        };
        spell.rules = SpellFactJson::Known(vec![
            SpellRuleJson {
                authored_key: "DamageDice".to_string(),
                order: 0,
                rule: SpellRuleDetailJson::DamageDice(Box::new(SpellDamageDiceRuleJson {
                    selector: SpellFactJson::Known("spell-damage".to_string()),
                    predicate: missing(),
                    dice_number: SpellFactJson::Known("1".to_string()),
                    die_size: SpellFactJson::Known("d6".to_string()),
                    damage_type: SpellFactJson::Known("fire".to_string()),
                    hide_if_disabled: SpellFactJson::Known(true),
                })),
            },
            SpellRuleJson {
                authored_key: "DamageDice".to_string(),
                order: 1,
                rule: SpellRuleDetailJson::DamageDice(Box::new(SpellDamageDiceRuleJson {
                    selector: SpellFactJson::Known("damage".to_string()),
                    predicate: missing(),
                    dice_number: missing(),
                    die_size: missing(),
                    damage_type: missing(),
                    hide_if_disabled: SpellFactJson::Known(false),
                })),
            },
            SpellRuleJson {
                authored_key: "EphemeralEffect".to_string(),
                order: 2,
                rule: SpellRuleDetailJson::EphemeralEffect(Box::new(
                    SpellEphemeralEffectRuleJson {
                        predicate: missing(),
                        selectors: SpellFactJson::Known(vec![
                            "spell-attack-roll".to_string(),
                            "{item|id}-private".to_string(),
                        ]),
                        uuid: SpellFactJson::Known(
                            "Compendium.pf2e.spell-effects.Item.private".to_string(),
                        ),
                    },
                )),
            },
            SpellRuleJson {
                authored_key: "DamageAlteration".to_string(),
                order: 3,
                rule: SpellRuleDetailJson::DamageAlteration(Box::new(
                    SpellDamageAlterationRuleJson {
                        mode: SpellFactJson::Known("override".to_string()),
                        predicate: missing(),
                        property: SpellFactJson::Known("damage-type".to_string()),
                        selectors: SpellFactJson::Known(vec!["spell-damage".to_string()]),
                        slug: SpellFactJson::Known("private-slug".to_string()),
                        value: SpellFactJson::Known("cold".to_string()),
                    },
                )),
            },
        ]);
        let machine = serde_json::to_value(&record).expect("rule machine JSON serializes");
        assert_eq!(
            machine.pointer("/spell/rules/value/0/value/hide_if_disabled/value"),
            Some(&serde_json::json!(true))
        );
        assert_eq!(
            machine.pointer("/spell/rules/value/1/value/hide_if_disabled/value"),
            Some(&serde_json::json!(false))
        );
        assert_eq!(
            machine.pointer("/spell/rules/value/2/value/selectors/value/1"),
            Some(&serde_json::json!("{item|id}-private"))
        );

        let rendered = render_record(&record, DetailLevel::Standard, 120, TerminalStyle::plain());
        for expected in [
            "Damage dice adjustment: 1 d6 fire",
            "Scope: spell damage",
            "Scope: damage",
            "Display: only while enabled",
            "Conditional effect",
            "Scope: spell attack rolls, not available for presentation",
            "Damage alteration",
        ] {
            assert!(
                rendered.contains(expected),
                "missing {expected:?}:\n{rendered}"
            );
        }
        assert_eq!(rendered.matches("Display: only while enabled").count(), 1);
        for internal in ["{item|id}-private", "Compendium.", "private-slug"] {
            assert!(
                !rendered.contains(internal),
                "leaked {internal:?}:\n{rendered}"
            );
        }
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
                        child_locator: None,
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
                spell_children: Vec::new(),
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
                traits: if detail != DetailLevel::Summary { vec!["fiend".into(), "hag".into()] } else { Vec::new() },
                source, supplementary_sections: Vec::new(), source_json: None,
            },
            presentation: RecordPresentationJson::Creature {
                teaser: matches!(detail, DetailLevel::Preview | DetailLevel::Standard).then(|| "Night hags prey upon sleepers, trading in stolen dreams and stalking victims through a world of nightmares while their repeated innate magic remains tied to exact spell occurrences.".into()),
                size: scan.then(|| "medium".into()), adjustment: scan.then(|| "elite-ready".into()),
                initiative: scan.then(|| atlas_record::CreatureInitiativeJson { statistic: "Perception".into() }),
                abilities: scan.then_some(CreatureAbilitiesJson { strength: Some(5), dexterity: Some(4), constitution: Some(4), intelligence: Some(4), wisdom: Some(5), charisma: Some(6) }),
                defenses: scan.then(|| CreatureDefensesJson { ac: Some(CreatureArmorClassJson { value: Some(28), details: Some("+1 status against dreams".into()) }), hp: Some(CreatureHitPointsJson { value: Some(170), maximum: Some(170), temporary: None, temporary_maximum: None, details: None }), ..Default::default() }),
                perception: scan.then_some(CreaturePerceptionJson { modifier: Some(19), details: None, has_vision: Some(true), senses: None }),
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
                rituals: scan.then_some(CreatureRitualsJson { difficulty_class: Some(28) }), equipment: None, lore: None,
                content: description.then(|| vec![content]),
                relationships: scan.then(|| vec![CreatureRelationshipJson { source_occurrence_id: "change-shape".into(), kind: "helper" , target: CreatureRelationshipTargetJson::Occurrence { occurrence_id: "claw-occurrence".into() }, source_path: "system.items[1]".into() }]),
                provenance: (detail == DetailLevel::Full).then_some(provenance),
                edition: (detail == DetailLevel::Full).then(|| RecordEditionContextJson { status: RecordEditionStatusJson::Legacy, counterpart_lookup: RecordEditionCounterpartLookupJson::Verified { counterparts: vec![RecordEditionCounterpartJson { role: RecordEditionCounterpartRoleJson::RemasteredCounterpart, record_key: "bestiary:Dream-Hag".into(), title: "Dream Hag".into() }] } }),
                record_relationships: scan.then(|| RecordRelationshipLookupJson::Verified { relationships: vec![RecordCanonicalRelationshipJson { direction: RecordRelationshipDirectionJson::Reference, kind: ReferenceRelationKind::Reference, label: "Dream Message".into(), target_record_key: "spells:Dream-Message".into(), source_child_locator: None, target_child_locator: None, provenance: RecordRelationshipProvenanceJson { from_record_key: "bestiary:Night-Hag".into(), to_record_key: "spells:Dream-Message".into(), source_kind: ContentSourceKind::Description, visibility: ContentVisibility::Public } }] }),
                availability: Vec::new(),
                unmodeled_skill_availability: if scan { vec![CreatureUnmodeledSkillAvailabilityJson {
                    skill_id: "synthetic-unmodeled-skill".into(),
                    authored_order: 0,
                    authored_key: "synthetic-review-skill".into(),
                    modifier: CreatureIntegerPresenceJson::Value(17),
                    message: "The source supplied an unrecognized skill key.",
                }] } else { Vec::new() },
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
