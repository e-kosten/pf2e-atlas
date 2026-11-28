use super::{
    Attribute, PrerequisiteEvidence, PrerequisiteExpression as Expression, ProficiencyRank,
    ProficiencySubject, ProgressionIndex, Qualification, TextRange, is_prerequisite_family,
    normalize_title,
};
use crate::source_content::{
    ContentReferenceKind, ContentReferenceOccurrence, ContentReferenceResolution,
    ContentReferenceTarget,
};
use atlas_domain::RecordKey;

pub(super) struct ClauseParser<'a> {
    pub source: &'a str,
    pub visible: &'a str,
    pub references: &'a [ContentReferenceOccurrence],
    pub owner: &'a RecordKey,
    pub owner_archetype: Option<&'a str>,
    pub index: &'a ProgressionIndex,
}

#[derive(Clone, Copy)]
struct Fragment<'a> {
    text: &'a str,
    start: usize,
}

impl<'a> Fragment<'a> {
    fn sub(self, start: usize, end: usize) -> Self {
        let untrimmed = &self.text[start..end];
        Self {
            text: untrimmed.trim(),
            start: self.start + start + untrimmed.len() - untrimmed.trim_start().len(),
        }
    }

    fn range(self) -> TextRange {
        TextRange {
            start: self.start,
            end: self.start + self.text.len(),
        }
    }

    fn unresolved(self) -> Expression {
        Expression::Unresolved {
            text: self.text.to_owned(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Operator {
    And,
    Or,
}

impl Operator {
    fn group(self, children: Vec<Expression>) -> Expression {
        match self {
            Self::And => Expression::AllOf { children },
            Self::Or => Expression::AnyOf { children },
        }
    }
}

impl ClauseParser<'_> {
    pub(super) fn parse(&self) -> Expression {
        let source = Fragment {
            text: self.source.trim_start(),
            start: self.source.len() - self.source.trim_start().len(),
        };
        let fragment = source.sub(0, source.text.trim_end_matches(['.', ' ']).len());
        if let Some(atom) = self.atom(fragment) {
            return atom;
        }

        // Markup other than a supported Foundry reference is presented through
        // the audience-filtered content path, without making links from markup.
        if self.source.contains('<') || self.source.len() > 1024 {
            return self.unresolved();
        }

        let Some((operator, parts)) = split_group(fragment) else {
            return self.unresolved();
        };
        if let Some(children) = shared_rank(operator, &parts).or_else(|| shared_attribute(&parts)) {
            return operator.group(children);
        }
        let children: Vec<_> = parts
            .iter()
            .map(|part| self.atom(*part).unwrap_or_else(|| part.unresolved()))
            .collect();
        let known_count = children
            .iter()
            .filter(|child| !matches!(child, Expression::Unresolved { .. }))
            .count();
        // An unmatched operand can carry a shared qualifier, as in
        // "Medium or Small size". Only preserve partial parsing for a known
        // boundary between a complete threshold and an independent predicate.
        if known_count == children.len()
            || (operator == Operator::Or && standalone_casting_alternative(&children))
        {
            operator.group(children)
        } else {
            self.unresolved()
        }
    }

    fn unresolved(&self) -> Expression {
        // The canonical visible text is retained for unsupported markup too.
        Expression::Unresolved {
            text: self.visible.trim().to_owned(),
        }
    }

    fn atom(&self, fragment: Fragment<'_>) -> Option<Expression> {
        if looks_like_reference(fragment.text) {
            return self.authored_reference(fragment.text);
        }
        if fragment.text.is_empty()
            || fragment
                .text
                .contains(['<', '>', '@', '[', ']', '{', '}', ';'])
        {
            return None;
        }

        if let Some((rank, subject)) = proficiency_parts(fragment.text)
            && let Some(subject) = proficiency_subject(subject)
        {
            return Some(Expression::Proficiency {
                subject,
                minimum: rank,
            });
        }
        if let Some(attribute) = attribute_minimum(fragment.text) {
            return Some(attribute);
        }
        self.named_reference(fragment)
    }

    fn authored_reference(&self, text: &str) -> Option<Expression> {
        let (target, label) = reference_token(text)?;
        let mut matching = self.references.iter().filter(|reference| {
            reference.visible
                && matches!(
                    reference.kind,
                    ContentReferenceKind::Uuid | ContentReferenceKind::Compendium
                )
                && reference.authored_target == target
        });
        let reference = matching.next()?;
        if matching.next().is_some()
            || !reference
                .label
                .item_family
                .as_deref()
                .is_some_and(is_prerequisite_family)
        {
            return None;
        }
        let ContentReferenceResolution::Resolved(ContentReferenceTarget::Record { key }) =
            &reference.resolution
        else {
            return None;
        };
        if key == self.owner {
            return None;
        }
        Some(Expression::Record {
            record_key: key.clone(),
            label: label
                .or(reference.label.target_name.as_deref())
                .unwrap_or(self.visible)
                .to_owned(),
            evidence: PrerequisiteEvidence::AuthoredReference,
            text_range: None,
        })
    }

    fn named_reference(&self, fragment: Fragment<'_>) -> Option<Expression> {
        if let Some(candidate) = self
            .index
            .resolve(fragment.text, Qualification::Unqualified)
            && &candidate.key != self.owner
        {
            return Some(Expression::Record {
                record_key: candidate.key.clone(),
                label: fragment.text.to_owned(),
                evidence: if candidate.dedication
                    && self.owner_archetype.is_some()
                    && self.owner_archetype == candidate.archetype.as_deref()
                {
                    PrerequisiteEvidence::SameArchetypeDedication
                } else {
                    PrerequisiteEvidence::ExactTitle
                },
                text_range: Some(fragment.range()),
            });
        }

        let normalized = normalize_title(fragment.text);
        for (suffix, qualification) in [
            (" class feature", Qualification::ClassFeature),
            (" heritage", Qualification::Family("heritage")),
            (" feat", Qualification::Family("feat")),
        ] {
            let Some(title) = normalized.strip_suffix(suffix) else {
                continue;
            };
            let candidate = self.index.resolve(title, qualification)?;
            if &candidate.key == self.owner {
                return None;
            }
            return Some(Expression::Record {
                record_key: candidate.key.clone(),
                label: fragment.text.to_owned(),
                evidence: PrerequisiteEvidence::QualifiedTitle,
                text_range: Some(fragment.range()),
            });
        }
        None
    }
}

fn looks_like_reference(text: &str) -> bool {
    text.starts_with("@UUID[") || text.starts_with("@Compendium[")
}

fn reference_token(text: &str) -> Option<(&str, Option<&str>)> {
    let after = text
        .strip_prefix("@UUID[")
        .or_else(|| text.strip_prefix("@Compendium["))?;
    let (target, label) = after.split_once(']')?;
    let label = if label.is_empty() {
        None
    } else {
        Some(label.strip_prefix('{')?.strip_suffix('}')?)
    };
    if target.is_empty() || label.is_some_and(|value| value.contains(['{', '}', '<', '>'])) {
        return None;
    }
    Some((target, label))
}

fn strip_either(text: &str) -> &str {
    if text
        .get(..7)
        .is_some_and(|word| word.eq_ignore_ascii_case("either "))
    {
        text[7..].trim_start()
    } else {
        text
    }
}

fn standalone_casting_alternative(children: &[Expression]) -> bool {
    let ([
        Expression::AttributeMinimum { .. },
        Expression::Unresolved { text },
    ]
    | [
        Expression::Unresolved { text },
        Expression::AttributeMinimum { .. },
    ]) = children
    else {
        return false;
    };
    let text = text.to_ascii_lowercase();
    let text = text.strip_prefix("the ").unwrap_or(&text);
    text.strip_prefix("ability to cast ")
        .is_some_and(|tail| !tail.is_empty())
}

fn split_group(fragment: Fragment<'_>) -> Option<(Operator, Vec<Fragment<'_>>)> {
    let text = fragment.text;
    let mut operator = None;
    let mut parts = Vec::new();
    let mut offset = 0;
    let mut chars = text.char_indices().peekable();
    let mut inside = None;
    let mut previous_comma = false;
    while let Some((index, character)) = chars.next() {
        if let Some(close) = inside {
            if character == close {
                inside = None;
            }
            continue;
        }
        match character {
            '[' => inside = Some(']'),
            '{' => inside = Some('}'),
            '(' | ')' | ';' => return None,
            ',' => {
                parts.push(fragment.sub(offset, index));
                offset = index + character.len_utf8();
                previous_comma = true;
            }
            _ if character.is_whitespace() => {
                let after = &text[index + character.len_utf8()..];
                let word = after.split_whitespace().next()?;
                let current = if word.eq_ignore_ascii_case("and") {
                    Operator::And
                } else if word.eq_ignore_ascii_case("or") {
                    Operator::Or
                } else {
                    continue;
                };
                if operator.is_some_and(|previous| previous != current) {
                    return None;
                }
                operator = Some(current);
                let word_start = text.len() - after.trim_start().len();
                if !previous_comma || !text[offset..index].trim().is_empty() {
                    parts.push(fragment.sub(offset, index));
                }
                offset = word_start + word.len();
                while chars.peek().is_some_and(|(position, _)| *position < offset) {
                    chars.next();
                }
                previous_comma = false;
            }
            _ => {}
        }
    }
    if inside.is_some() {
        return None;
    }
    parts.push(fragment.sub(offset, text.len()));
    let operator = operator?;
    let first = parts.first_mut()?;
    if operator == Operator::Or && strip_either(first.text) != first.text {
        *first = first.sub(7, first.text.len());
    }
    let valid = (2..=12).contains(&parts.len()) && parts.iter().all(|part| !part.text.is_empty());
    valid.then_some((operator, parts))
}

fn shared_rank(operator: Operator, parts: &[Fragment<'_>]) -> Option<Vec<Expression>> {
    let (rank, subject) = proficiency_parts(parts.first()?.text)?;
    let subject = if operator == Operator::Or {
        strip_either(subject)
    } else {
        subject
    };
    std::iter::once(subject)
        .chain(parts.iter().skip(1).map(|part| part.text))
        .map(proficiency_subject)
        .map(|subject| {
            Some(Expression::Proficiency {
                subject: subject?,
                minimum: rank,
            })
        })
        .collect()
}

fn proficiency_parts(text: &str) -> Option<(ProficiencyRank, &str)> {
    let (rank, tail) = text.split_once(char::is_whitespace)?;
    let (qualifier, subject) = tail.trim_start().split_once(char::is_whitespace)?;
    if !qualifier.eq_ignore_ascii_case("in") {
        return None;
    }
    let rank = match rank.to_ascii_lowercase().as_str() {
        "trained" => ProficiencyRank::Trained,
        "expert" => ProficiencyRank::Expert,
        "master" => ProficiencyRank::Master,
        "legendary" => ProficiencyRank::Legendary,
        _ => return None,
    };
    Some((rank, subject.trim()))
}

fn proficiency_subject(value: &str) -> Option<ProficiencySubject> {
    const SKILLS: &[&str] = &[
        "Acrobatics",
        "Arcana",
        "Athletics",
        "Crafting",
        "Deception",
        "Diplomacy",
        "Intimidation",
        "Medicine",
        "Nature",
        "Occultism",
        "Performance",
        "Religion",
        "Society",
        "Stealth",
        "Survival",
        "Thievery",
    ];
    if let Some(name) = SKILLS.iter().find(|name| name.eq_ignore_ascii_case(value)) {
        return Some(ProficiencySubject::Skill {
            name: (*name).to_owned(),
        });
    }
    if value.eq_ignore_ascii_case("perception") {
        return Some(ProficiencySubject::Perception);
    }
    // A specific Lore subject uses a named qualifier, as opposed to a choice of
    // Lore or a proficiency requirement qualified by further prose.
    let subject = value.strip_suffix(" Lore")?;
    if !subject.is_empty()
        && subject.split_whitespace().count() <= 4
        && subject.split_whitespace().all(|word| {
            word.chars().next().is_some_and(char::is_uppercase)
                && word
                    .chars()
                    .all(|c| c.is_alphabetic() || matches!(c, '\'' | '’' | '-'))
        })
        && !matches!(subject, "Any" | "Specific" | "One" | "Your" | "A")
    {
        return Some(ProficiencySubject::Lore {
            name: value.to_owned(),
        });
    }
    None
}

fn shared_attribute(parts: &[Fragment<'_>]) -> Option<Vec<Expression>> {
    let (last, leading) = parts.split_last()?;
    let last = attribute_minimum(last.text)?;
    let Expression::AttributeMinimum { minimum, .. } = last else {
        return None;
    };
    let mut requirements = leading
        .iter()
        .map(|part| {
            Some(Expression::AttributeMinimum {
                attribute: attribute_name(part.text)?,
                minimum,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    requirements.push(last);
    Some(requirements)
}

fn attribute_minimum(text: &str) -> Option<Expression> {
    let (name, raw) = text.split_once(char::is_whitespace)?;
    let modifier = raw.trim().strip_prefix('+')?;
    if modifier.is_empty() || !modifier.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    Some(Expression::AttributeMinimum {
        attribute: attribute_name(name)?,
        minimum: modifier.parse().ok()?,
    })
}

fn attribute_name(name: &str) -> Option<Attribute> {
    Some(match name.to_ascii_lowercase().as_str() {
        "strength" | "str" => Attribute::Strength,
        "dexterity" | "dex" => Attribute::Dexterity,
        "constitution" | "con" => Attribute::Constitution,
        "intelligence" | "int" => Attribute::Intelligence,
        "wisdom" | "wis" => Attribute::Wisdom,
        "charisma" | "cha" => Attribute::Charisma,
        _ => return None,
    })
}
