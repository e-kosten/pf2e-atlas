use atlas_domain::RecordKey;

use super::*;

#[test]
fn tokenization_normalizes_stop_words_and_deduplicates_terms() {
    assert_eq!(
        tokenize_query("The Reactive Strike and reactive-strike!"),
        vec!["reactive", "strike"]
    );
    assert_eq!(
        normalize_text("Attack of Opportunity"),
        "attack opportunity"
    );
}

#[test]
fn primary_type_intent_vocabulary_matches_strict_query_exclusions() {
    for token in [
        "spell",
        "feat",
        "creature",
        "item",
        "condition",
        "hazard",
        "vehicle",
        "ancestry",
        "companion",
        "familiar",
        "affliction",
    ] {
        assert!(is_primary_type_intent_token(token), "{token}");
    }
    for secondary_equipment_token in ["weapon", "armor", "shield", "potion", "wand", "rune"] {
        assert!(!is_primary_type_intent_token(secondary_equipment_token));
        assert!(is_type_intent_token(secondary_equipment_token));
    }
}

#[test]
fn strict_tier_sorts_before_better_ranked_fallback_hits() {
    let mut hits = [
        hit("actions:testAction2", -10_000.0, FtsMatchTier::Fallback),
        hit("actions:testAction1", 10_000.0, FtsMatchTier::Strict),
    ];

    hits.sort_by(compare_fts_document_hits);

    assert_eq!(hits[0].record_key.to_string(), "actions:testAction1");
}

#[test]
fn record_key_breaks_ties_after_tier_and_rank() {
    let mut hits = [
        hit("actions:testAction2", 10.0, FtsMatchTier::Strict),
        hit("actions:testAction1", 10.0, FtsMatchTier::Strict),
    ];

    hits.sort_by(compare_fts_document_hits);

    assert_eq!(hits[0].record_key.to_string(), "actions:testAction1");
    assert_eq!(hits[1].record_key.to_string(), "actions:testAction2");
}

#[test]
fn negative_column_weights_do_not_create_negative_boosts() {
    let tokens = vec!["needle".to_string()];
    let query_phrase = "needle";
    let document = FtsDocument {
        title: "needle".to_string(),
        aliases: String::new(),
        traits: String::new(),
        taxonomy_terms: String::new(),
        constraint_terms: String::new(),
        mechanic_terms: String::new(),
        source_terms: String::new(),
        metric_terms: String::new(),
        headings: String::new(),
        body: String::new(),
        facts: String::new(),
        reference_terms: String::new(),
        embedded_content: String::new(),
        record_family: "rule".to_string(),
        foundry_record_type: "action".to_string(),
    };
    let hit = FtsDocumentHit {
        record_key: RecordKey::parse("actions:testAction1").expect("record key"),
        base_rank: 0.0,
        rank: 0.0,
        tier: FtsMatchTier::Fallback,
        document,
    };

    let negative_weight_rank = adjusted_rank(
        &tokens,
        query_phrase,
        &hit,
        FtsColumnWeights {
            title: -10.0,
            ..FtsColumnWeights::default()
        },
    );
    let zero_weight_rank = adjusted_rank(
        &tokens,
        query_phrase,
        &hit,
        FtsColumnWeights {
            title: 0.0,
            ..FtsColumnWeights::default()
        },
    );

    assert_eq!(negative_weight_rank, zero_weight_rank);
}

#[test]
fn alias_phrase_boosts_are_stronger_than_body_phrase_boosts() {
    let tokens = vec!["shadow".to_string(), "blast".to_string()];
    let query_phrase = "shadow blast";
    let alias_hit = hit_with_document(
        "actions:testAction1",
        FtsDocument {
            aliases: "Shadow Blast".to_string(),
            ..empty_document()
        },
    );
    let body_hit = hit_with_document(
        "actions:testAction2",
        FtsDocument {
            body: "Shadow Blast".to_string(),
            ..empty_document()
        },
    );

    assert!(
        adjusted_rank(
            &tokens,
            query_phrase,
            &alias_hit,
            FtsColumnWeights::default()
        ) < adjusted_rank(
            &tokens,
            query_phrase,
            &body_hit,
            FtsColumnWeights::default()
        )
    );
}

#[test]
fn phrase_contains_boost_prefers_longer_alias_phrase_matches() {
    assert_eq!(
        phrase_boost("shadow blast", "Greater Shadow Blast", 80.0, 35.0),
        35.0
    );

    let tokens = vec!["shadow".to_string(), "blast".to_string()];
    let query_phrase = "shadow blast";
    let partial_hit = hit_with_document(
        "actions:testAction1",
        FtsDocument {
            aliases: "Shadow".to_string(),
            body: "blast".to_string(),
            ..empty_document()
        },
    );
    let contained_phrase_hit = hit_with_document(
        "actions:testAction2",
        FtsDocument {
            aliases: "Greater Shadow Blast".to_string(),
            ..empty_document()
        },
    );

    assert!(
        adjusted_rank(
            &tokens,
            query_phrase,
            &contained_phrase_hit,
            FtsColumnWeights::default()
        ) < adjusted_rank(
            &tokens,
            query_phrase,
            &partial_hit,
            FtsColumnWeights::default()
        )
    );
}

#[test]
fn alias_partial_title_boost_prefers_alias_contained_in_query_tokens() {
    let tokens = vec![
        "reactive".to_string(),
        "strike".to_string(),
        "prevention".to_string(),
    ];
    let query_phrase = "reactive strike prevention";
    let body_hit = hit_with_document(
        "actions:testAction1",
        FtsDocument {
            body: "reactive strike".to_string(),
            ..empty_document()
        },
    );
    let alias_hit = hit_with_document(
        "actions:testAction2",
        FtsDocument {
            aliases: "Reactive Strike".to_string(),
            ..empty_document()
        },
    );

    assert!(
        adjusted_rank(&tokens, query_phrase, &alias_hit, zero_column_weights())
            < adjusted_rank(&tokens, query_phrase, &body_hit, zero_column_weights())
    );
}

#[test]
fn combined_coverage_boost_prefers_documents_matching_all_query_tokens() {
    let tokens = vec!["spell".to_string(), "action".to_string()];
    let query_phrase = "spell action";
    let partial_hit = hit_with_document(
        "actions:testAction1",
        FtsDocument {
            body: "spell".to_string(),
            ..empty_document()
        },
    );
    let full_hit = hit_with_document(
        "actions:testAction2",
        FtsDocument {
            body: "spell action".to_string(),
            ..empty_document()
        },
    );

    assert!(
        adjusted_rank(&tokens, query_phrase, &full_hit, zero_column_weights())
            < adjusted_rank(&tokens, query_phrase, &partial_hit, zero_column_weights())
    );
}

#[test]
fn high_value_token_boost_prefers_documents_matching_specific_terms() {
    let tokens = vec!["alpha".to_string(), "spell".to_string()];
    let query_phrase = "alpha spell";
    let generic_hit = hit_with_document(
        "actions:testAction1",
        FtsDocument {
            body: "spell".to_string(),
            ..empty_document()
        },
    );
    let high_value_hit = hit_with_document(
        "actions:testAction2",
        FtsDocument {
            body: "alpha".to_string(),
            ..empty_document()
        },
    );

    assert!(
        adjusted_rank(
            &tokens,
            query_phrase,
            &high_value_hit,
            zero_column_weights()
        ) < adjusted_rank(&tokens, query_phrase, &generic_hit, zero_column_weights())
    );
}

#[test]
fn type_intent_boost_covers_supported_record_families_and_types() {
    let cases = [
        ("spell", "spell", "spell"),
        ("feat", "feat", "feat"),
        ("creature", "creature", "npc"),
        ("item", "equipment", "equipment"),
        ("weapon", "equipment", "weapon"),
        ("armor", "equipment", "armor"),
        ("shield", "equipment", "shield"),
        ("potion", "equipment", "consumable"),
        ("condition", "rule", "condition"),
        ("hazard", "hazard", "hazard"),
        ("vehicle", "vehicle", "vehicle"),
        ("ancestry", "character_option", "ancestry"),
        ("companion", "companion", "animal"),
        ("familiar", "companion", "familiar"),
        ("affliction", "affliction", "affliction"),
    ];

    for (token, family, foundry_type) in cases {
        let tokens = vec![token.to_string()];
        let boosted = hit_with_document(
            "actions:testAction1",
            FtsDocument {
                record_family: family.to_string(),
                foundry_record_type: foundry_type.to_string(),
                ..empty_document()
            },
        );
        let unboosted = hit_with_document("actions:testAction2", empty_document());

        assert!(
            adjusted_rank(&tokens, token, &boosted, FtsColumnWeights::default())
                < adjusted_rank(&tokens, token, &unboosted, FtsColumnWeights::default()),
            "{token}"
        );
    }
}

#[test]
fn type_intent_boost_covers_plural_and_synonym_tokens() {
    let cases = [
        ("spells", "spell", "spell"),
        ("cantrip", "spell", "spell"),
        ("cantrips", "spell", "spell"),
        ("feats", "feat", "feat"),
        ("monsters", "creature", "npc"),
        ("npc", "creature", "npc"),
        ("npcs", "creature", "npc"),
        ("items", "equipment", "equipment"),
        ("equipment", "equipment", "equipment"),
        ("monster", "creature", "npc"),
        ("weapons", "equipment", "weapon"),
        ("armour", "equipment", "armor"),
        ("shields", "equipment", "shield"),
        ("potions", "equipment", "consumable"),
        ("wand", "equipment", "wand"),
        ("wands", "equipment", "wand"),
        ("rune", "equipment", "rune"),
        ("runes", "equipment", "rune"),
        ("conditions", "rule", "condition"),
        ("hazards", "hazard", "hazard"),
        ("vehicles", "vehicle", "vehicle"),
        ("ancestries", "character_option", "ancestry"),
        ("companions", "companion", "animal"),
        ("familiars", "companion", "familiar"),
        ("afflictions", "affliction", "affliction"),
    ];

    for (token, family, foundry_type) in cases {
        let tokens = vec![token.to_string()];
        let boosted = hit_with_document(
            "actions:testAction1",
            FtsDocument {
                record_family: family.to_string(),
                foundry_record_type: foundry_type.to_string(),
                ..empty_document()
            },
        );
        let unboosted = hit_with_document("actions:testAction2", empty_document());

        assert!(
            adjusted_rank(&tokens, token, &boosted, FtsColumnWeights::default())
                < adjusted_rank(&tokens, token, &unboosted, FtsColumnWeights::default()),
            "{token}"
        );
    }
}

fn hit(record_key: &str, rank: f64, tier: FtsMatchTier) -> FtsDocumentHit {
    hit_with_document(record_key, empty_document())
        .with_rank(rank)
        .with_tier(tier)
}

fn hit_with_document(record_key: &str, document: FtsDocument) -> FtsDocumentHit {
    FtsDocumentHit {
        record_key: RecordKey::parse(record_key).expect("record key"),
        base_rank: 0.0,
        rank: 0.0,
        tier: FtsMatchTier::Fallback,
        document,
    }
}

trait HitBuilder {
    fn with_rank(self, rank: f64) -> Self;
    fn with_tier(self, tier: FtsMatchTier) -> Self;
}

impl HitBuilder for FtsDocumentHit {
    fn with_rank(mut self, rank: f64) -> Self {
        self.base_rank = rank;
        self.rank = rank;
        self
    }

    fn with_tier(mut self, tier: FtsMatchTier) -> Self {
        self.tier = tier;
        self
    }
}

fn empty_document() -> FtsDocument {
    FtsDocument {
        title: String::new(),
        aliases: String::new(),
        traits: String::new(),
        taxonomy_terms: String::new(),
        constraint_terms: String::new(),
        mechanic_terms: String::new(),
        source_terms: String::new(),
        metric_terms: String::new(),
        headings: String::new(),
        body: String::new(),
        facts: String::new(),
        reference_terms: String::new(),
        embedded_content: String::new(),
        record_family: String::new(),
        foundry_record_type: String::new(),
    }
}

fn zero_column_weights() -> FtsColumnWeights {
    FtsColumnWeights {
        title: 0.0,
        aliases: 0.0,
        traits: 0.0,
        taxonomy_terms: 0.0,
        constraint_terms: 0.0,
        mechanic_terms: 0.0,
        source_terms: 0.0,
        metric_terms: 0.0,
        headings: 0.0,
        body: 0.0,
        facts: 0.0,
        reference_terms: 0.0,
        embedded_content: 0.0,
    }
}
