use atlas_domain::RecordKey;

use super::FtsColumnWeights;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FtsMatchTier {
    Strict,
    Fallback,
}

#[derive(Debug, Clone)]
pub(crate) struct FtsDocumentHit {
    pub(crate) record_key: RecordKey,
    pub(crate) base_rank: f64,
    pub(crate) rank: f64,
    pub(crate) tier: FtsMatchTier,
    pub(crate) document: FtsDocument,
}

#[derive(Debug, Clone)]
pub(crate) struct FtsDocument {
    pub(crate) title: String,
    pub(crate) aliases: String,
    pub(crate) traits: String,
    pub(crate) taxonomy_terms: String,
    pub(crate) constraint_terms: String,
    pub(crate) mechanic_terms: String,
    pub(crate) source_terms: String,
    pub(crate) metric_terms: String,
    pub(crate) headings: String,
    pub(crate) body: String,
    pub(crate) facts: String,
    pub(crate) reference_terms: String,
    pub(crate) embedded_content: String,
    pub(crate) record_kind: String,
    pub(crate) foundry_record_type: String,
}

const STRICT_MATCH_BONUS: f64 = 35.0;

pub(crate) fn adjusted_rank(
    tokens: &[String],
    query_phrase: &str,
    hit: &FtsDocumentHit,
    weights: FtsColumnWeights,
) -> f64 {
    let document = &hit.document;
    let mut boost = 0.0;
    boost += column_phrase_boost(query_phrase, &document.title, 90.0, 45.0, weights.title);
    boost += column_phrase_boost(query_phrase, &document.aliases, 80.0, 35.0, weights.aliases);
    boost += column_phrase_boost(query_phrase, &document.traits, 20.0, 12.0, weights.traits);
    boost += column_phrase_boost(
        query_phrase,
        &document.taxonomy_terms,
        18.0,
        10.0,
        weights.taxonomy_terms,
    );
    boost += column_phrase_boost(
        query_phrase,
        &document.constraint_terms,
        16.0,
        8.0,
        weights.constraint_terms,
    );
    boost += column_phrase_boost(
        query_phrase,
        &document.mechanic_terms,
        14.0,
        7.0,
        weights.mechanic_terms,
    );
    boost += column_phrase_boost(
        query_phrase,
        &document.source_terms,
        14.0,
        7.0,
        weights.source_terms,
    );
    boost += column_phrase_boost(
        query_phrase,
        &document.metric_terms,
        14.0,
        7.0,
        weights.metric_terms,
    );
    boost += column_phrase_boost(
        query_phrase,
        &document.headings,
        18.0,
        8.0,
        weights.headings,
    );
    boost += column_phrase_boost(query_phrase, &document.body, 5.0, 2.0, weights.body);
    boost += column_phrase_boost(query_phrase, &document.facts, 8.0, 3.0, weights.facts);
    boost += column_phrase_boost(
        query_phrase,
        &document.reference_terms,
        5.0,
        2.0,
        weights.reference_terms,
    );
    boost += column_phrase_boost(
        query_phrase,
        &document.embedded_content,
        4.0,
        1.0,
        weights.embedded_content,
    );

    boost += column_token_coverage_boost(tokens, &document.title, 36.0, weights.title);
    boost += column_token_coverage_boost(tokens, &document.aliases, 32.0, weights.aliases);
    boost += column_token_coverage_boost(tokens, &document.traits, 18.0, weights.traits);
    boost += column_token_coverage_boost(
        tokens,
        &document.taxonomy_terms,
        16.0,
        weights.taxonomy_terms,
    );
    boost += column_token_coverage_boost(
        tokens,
        &document.constraint_terms,
        14.0,
        weights.constraint_terms,
    );
    boost += column_token_coverage_boost(
        tokens,
        &document.mechanic_terms,
        12.0,
        weights.mechanic_terms,
    );
    boost +=
        column_token_coverage_boost(tokens, &document.source_terms, 12.0, weights.source_terms);
    boost +=
        column_token_coverage_boost(tokens, &document.metric_terms, 12.0, weights.metric_terms);
    boost += column_token_coverage_boost(tokens, &document.headings, 12.0, weights.headings);
    boost += column_token_coverage_boost(tokens, &document.body, 4.0, weights.body);
    boost += column_token_coverage_boost(tokens, &document.facts, 6.0, weights.facts);
    boost += column_token_coverage_boost(
        tokens,
        &document.reference_terms,
        4.0,
        weights.reference_terms,
    );
    boost += column_token_coverage_boost(
        tokens,
        &document.embedded_content,
        3.0,
        weights.embedded_content,
    );

    boost += title_containment_boost(tokens, document);
    boost += partial_title_token_boost(tokens, &document.title, 60.0);
    boost += partial_title_token_boost(tokens, &document.aliases, 50.0);
    boost += combined_coverage_boost(tokens, document);
    boost += high_value_token_boost(tokens, document);
    boost += type_intent_boost(tokens, document);
    if matches!(hit.tier, FtsMatchTier::Strict) {
        boost += STRICT_MATCH_BONUS;
    }

    hit.base_rank - boost + implementation_effect_penalty(document)
}

pub(crate) fn compare_fts_document_hits(
    left: &FtsDocumentHit,
    right: &FtsDocumentHit,
) -> std::cmp::Ordering {
    match_tier_order(left.tier)
        .cmp(&match_tier_order(right.tier))
        .then_with(|| left.rank.total_cmp(&right.rank))
        .then_with(|| left.record_key.cmp(&right.record_key))
}

fn match_tier_order(tier: FtsMatchTier) -> u8 {
    match tier {
        FtsMatchTier::Strict => 0,
        FtsMatchTier::Fallback => 1,
    }
}

fn phrase_boost(
    normalized_query_phrase: &str,
    text: &str,
    exact_boost: f64,
    contains_boost: f64,
) -> f64 {
    if normalized_query_phrase.is_empty() || text.trim().is_empty() {
        return 0.0;
    }
    let normalized_text = normalize_text(text);
    if normalized_text == normalized_query_phrase {
        exact_boost
    } else if normalized_text.contains(normalized_query_phrase) {
        contains_boost
    } else {
        0.0
    }
}

fn column_phrase_boost(
    normalized_query_phrase: &str,
    text: &str,
    exact_boost: f64,
    contains_boost: f64,
    weight: f64,
) -> f64 {
    phrase_boost(normalized_query_phrase, text, exact_boost, contains_boost) * weight.max(0.0)
}

fn token_coverage_boost(tokens: &[String], text: &str, full_boost: f64) -> f64 {
    if tokens.is_empty() || text.trim().is_empty() {
        return 0.0;
    }
    let text_tokens = tokenize_query(text);
    if text_tokens.is_empty() {
        return 0.0;
    }
    let matched = tokens
        .iter()
        .filter(|query_token| {
            text_tokens
                .iter()
                .any(|text_token| text_token == *query_token)
        })
        .count();
    if matched == 0 {
        return 0.0;
    }
    let coverage = matched as f64 / tokens.len() as f64;
    full_boost * coverage * coverage
}

fn column_token_coverage_boost(tokens: &[String], text: &str, full_boost: f64, weight: f64) -> f64 {
    token_coverage_boost(tokens, text, full_boost) * weight.max(0.0)
}

fn title_containment_boost(query_tokens: &[String], document: &FtsDocument) -> f64 {
    if query_tokens.is_empty() || document.title.trim().is_empty() {
        return 0.0;
    }
    let has_type_intent = query_has_type_intent(query_tokens);
    if has_type_intent && type_intent_boost(query_tokens, document) == 0.0 {
        return 0.0;
    }
    let title_tokens = tokenize_query(&document.title);
    if title_tokens.is_empty() || title_tokens.len() > query_tokens.len() {
        return 0.0;
    }
    if !has_type_intent && title_tokens.len() < query_tokens.len() {
        return 0.0;
    }
    if title_tokens.iter().all(|title_token| {
        query_tokens
            .iter()
            .any(|query_token| query_token == title_token)
    }) {
        130.0
    } else {
        0.0
    }
}

fn partial_title_token_boost(query_tokens: &[String], text: &str, full_boost: f64) -> f64 {
    if query_tokens.len() < 2 || query_has_type_intent(query_tokens) || text.trim().is_empty() {
        return 0.0;
    }
    let text_tokens = tokenize_query(text);
    if text_tokens.is_empty() || text_tokens.len() >= query_tokens.len() {
        return 0.0;
    }
    if text_tokens.iter().all(|title_token| {
        query_tokens
            .iter()
            .any(|query_token| query_token == title_token)
    }) {
        full_boost
    } else {
        0.0
    }
}

fn combined_coverage_boost(tokens: &[String], document: &FtsDocument) -> f64 {
    if tokens.is_empty() {
        return 0.0;
    }
    let combined_tokens = combined_document_tokens(document);
    let matched = tokens
        .iter()
        .filter(|query_token| {
            combined_tokens
                .iter()
                .any(|text_token| text_token == *query_token)
        })
        .count();
    match matched {
        0 => 0.0,
        matched if matched == tokens.len() => 28.0,
        matched => {
            let coverage = matched as f64 / tokens.len() as f64;
            8.0 * coverage * coverage
        }
    }
}

fn high_value_token_boost(tokens: &[String], document: &FtsDocument) -> f64 {
    let high_value_tokens = tokens
        .iter()
        .filter(|token| !is_domain_generic_token(token))
        .collect::<Vec<_>>();
    if high_value_tokens.is_empty() {
        return 0.0;
    }
    let combined_tokens = combined_document_tokens(document);
    let matched = high_value_tokens
        .iter()
        .filter(|query_token| {
            combined_tokens
                .iter()
                .any(|text_token| text_token == **query_token)
        })
        .count();
    if matched == high_value_tokens.len() {
        45.0
    } else {
        18.0 * (matched as f64 / high_value_tokens.len() as f64)
    }
}

fn combined_document_tokens(document: &FtsDocument) -> Vec<String> {
    tokenize_query(
        &[
            document.title.as_str(),
            document.aliases.as_str(),
            document.traits.as_str(),
            document.taxonomy_terms.as_str(),
            document.constraint_terms.as_str(),
            document.mechanic_terms.as_str(),
            document.source_terms.as_str(),
            document.metric_terms.as_str(),
            document.headings.as_str(),
            document.body.as_str(),
            document.facts.as_str(),
            document.reference_terms.as_str(),
            document.embedded_content.as_str(),
        ]
        .join(" "),
    )
}

fn type_intent_boost(tokens: &[String], document: &FtsDocument) -> f64 {
    let mut boost = 0.0;
    for token in tokens {
        boost += match token.as_str() {
            "spell" | "spells" | "cantrip" | "cantrips" => {
                record_kind_boost(document, "spell", 55.0)
                    + foundry_record_type_boost(document, "spell", 12.0)
            }
            "feat" | "feats" => {
                record_kind_boost(document, "feat", 55.0)
                    + foundry_record_type_boost(document, "feat", 12.0)
            }
            "creature" | "creatures" | "monster" | "monsters" | "npc" | "npcs" => {
                record_kind_boost(document, "creature", 55.0)
                    + foundry_record_type_boost(document, "npc", 12.0)
            }
            "item" | "items" | "equipment" => record_kind_boost(document, "equipment", 45.0),
            "weapon" | "weapons" => {
                record_kind_boost(document, "equipment", 35.0)
                    + foundry_record_type_boost(document, "weapon", 18.0)
            }
            "armor" | "armour" => {
                record_kind_boost(document, "equipment", 35.0)
                    + foundry_record_type_boost(document, "armor", 18.0)
            }
            "shield" | "shields" => {
                record_kind_boost(document, "equipment", 35.0)
                    + foundry_record_type_boost(document, "shield", 18.0)
            }
            "potion" | "potions" | "wand" | "wands" | "rune" | "runes" => {
                record_kind_boost(document, "equipment", 35.0)
            }
            "condition" | "conditions" => {
                record_kind_boost(document, "rule", 35.0)
                    + foundry_record_type_boost(document, "condition", 35.0)
            }
            "hazard" | "hazards" => {
                record_kind_boost(document, "hazard", 55.0)
                    + foundry_record_type_boost(document, "hazard", 12.0)
            }
            "vehicle" | "vehicles" => {
                record_kind_boost(document, "vehicle", 55.0)
                    + foundry_record_type_boost(document, "vehicle", 12.0)
            }
            "ancestry" | "ancestries" => {
                record_kind_boost(document, "character_option", 35.0)
                    + foundry_record_type_boost(document, "ancestry", 35.0)
            }
            "companion" | "companions" => record_kind_boost(document, "companion", 55.0),
            "familiar" | "familiars" => {
                record_kind_boost(document, "companion", 35.0)
                    + foundry_record_type_boost(document, "familiar", 35.0)
            }
            "affliction" | "afflictions" => record_kind_boost(document, "affliction", 55.0),
            _ => 0.0,
        };
    }
    boost
}

fn record_kind_boost(document: &FtsDocument, record_kind: &str, boost: f64) -> f64 {
    if document.record_kind == record_kind {
        boost
    } else {
        0.0
    }
}

fn foundry_record_type_boost(document: &FtsDocument, foundry_record_type: &str, boost: f64) -> f64 {
    if document.foundry_record_type == foundry_record_type {
        boost
    } else {
        0.0
    }
}

fn implementation_effect_penalty(document: &FtsDocument) -> f64 {
    if document.foundry_record_type == "effect" {
        60.0
    } else {
        0.0
    }
}

pub(crate) fn is_primary_type_intent_token(token: &str) -> bool {
    matches!(
        token,
        "spell"
            | "spells"
            | "cantrip"
            | "cantrips"
            | "feat"
            | "feats"
            | "creature"
            | "creatures"
            | "monster"
            | "monsters"
            | "npc"
            | "npcs"
            | "item"
            | "items"
            | "equipment"
            | "condition"
            | "conditions"
            | "hazard"
            | "hazards"
            | "vehicle"
            | "vehicles"
            | "ancestry"
            | "ancestries"
            | "companion"
            | "companions"
            | "familiar"
            | "familiars"
            | "affliction"
            | "afflictions"
    )
}

fn is_type_intent_token(token: &str) -> bool {
    is_primary_type_intent_token(token)
        || matches!(
            token,
            "weapon"
                | "weapons"
                | "armor"
                | "armour"
                | "shield"
                | "shields"
                | "potion"
                | "potions"
                | "wand"
                | "wands"
                | "rune"
                | "runes"
        )
}

fn is_domain_generic_token(token: &str) -> bool {
    matches!(
        token,
        "ability"
            | "action"
            | "actions"
            | "activity"
            | "condition"
            | "creature"
            | "effect"
            | "equipment"
            | "feat"
            | "item"
            | "level"
            | "low"
            | "rank"
            | "rule"
            | "spell"
            | "spells"
    )
}

fn query_has_type_intent(tokens: &[String]) -> bool {
    tokens.iter().any(|token| is_type_intent_token(token))
}

pub(crate) fn normalize_text(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .filter(|token| !is_stop_word(token))
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn tokenize_query(value: &str) -> Vec<String> {
    let mut tokens = value
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .filter(|token| !is_stop_word(token))
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    tokens.sort();
    tokens.dedup();
    tokens
}

fn is_stop_word(token: &str) -> bool {
    matches!(
        token,
        "a" | "an"
            | "and"
            | "are"
            | "as"
            | "at"
            | "be"
            | "by"
            | "for"
            | "from"
            | "in"
            | "is"
            | "of"
            | "on"
            | "or"
            | "that"
            | "the"
            | "to"
            | "which"
            | "with"
    )
}

#[cfg(test)]
mod tests;
