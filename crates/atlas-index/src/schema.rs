// Named relational schema; numeric ANY uses a checked integer/real reader.
diesel::allow_tables_to_appear_in_same_query!(records, record_bodies);
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    packs (pack_id) {
        pack_id -> Text,
        label -> Text,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    records (record_id) {
        record_id -> BigInt,
        key -> Text,
        pack_id -> Text,
        document_kind -> Text,
        source_path -> Text,
        content_hash -> Text,
        name_state -> Text,
        name -> Nullable<Text>,
        name_lookup_key -> Nullable<Text>,
        source_type_state -> Text,
        source_type -> Nullable<Text>,
        record_kind_state -> Text,
        record_kind -> Nullable<Text>,
        rarity_state -> Text,
        rarity -> Nullable<Text>,
        publication_title_state -> Text,
        publication_title -> Nullable<Text>,
        publication_remaster_state -> Text,
        publication_remaster -> Nullable<BigInt>,
        traits_state -> Text,
        level_state -> Text,
        level -> Nullable<SourceNumberSql>,
        size_state -> Text,
        size -> Nullable<Text>,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    record_traits (record_id, value) {
        record_id -> BigInt,
        value -> Text,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    actor_projection (record_id) {
        record_id -> BigInt,
        armor_class_state -> Text,
        armor_class -> Nullable<SourceNumberSql>,
        hp_maximum_state -> Text,
        hp_maximum -> Nullable<SourceNumberSql>,
        hardness_state -> Text,
        hardness -> Nullable<SourceNumberSql>,
        complexity_state -> Text,
        complexity -> Nullable<BigInt>,
        items_state -> Text,
        fortitude_state -> Text,
        fortitude -> Nullable<SourceNumberSql>,
        reflex_state -> Text,
        reflex -> Nullable<SourceNumberSql>,
        will_state -> Text,
        will -> Nullable<SourceNumberSql>,
        immunities_state -> Text,
        weaknesses_state -> Text,
        resistances_state -> Text,
        perception_state -> Text,
        perception -> Nullable<SourceNumberSql>,
        land_speed_state -> Text,
        land_speed -> Nullable<SourceNumberSql>,
        languages_state -> Text,
        speeds_state -> Text,
        senses_state -> Text,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    actor_items (id) {
        id -> BigInt,
        record_id -> BigInt,
        original_index -> BigInt,
        owner_selector_json -> Text,
        authored_id_state -> Text,
        authored_id -> Nullable<Text>,
        source_type_state -> Text,
        source_type -> Nullable<Text>,
        traits_state -> Text,
        rarity_state -> Text,
        rarity -> Nullable<Text>,
        level_state -> Text,
        level -> Nullable<SourceNumberSql>,
        size_state -> Text,
        size -> Nullable<Text>,
        publication_title_state -> Text,
        publication_title -> Nullable<Text>,
        publication_remaster_state -> Text,
        publication_remaster -> Nullable<BigInt>,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    actor_item_traits (item_id, value) {
        item_id -> BigInt,
        value -> Text,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    actor_iwr_entries (entry_id) {
        entry_id -> BigInt,
        record_id -> BigInt,
        kind -> Text,
        original_index -> BigInt,
        type_state -> Text,
        r#type -> Nullable<Text>,
        value_state -> Text,
        value -> Nullable<SourceNumberSql>,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    actor_languages (record_id, value) {
        record_id -> BigInt,
        value -> Text,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    actor_speeds (id) {
        id -> BigInt,
        record_id -> BigInt,
        original_index -> BigInt,
        type_state -> Text,
        r#type -> Nullable<Text>,
        value_state -> Text,
        value -> Nullable<SourceNumberSql>,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    actor_senses (id) {
        id -> BigInt,
        record_id -> BigInt,
        original_index -> BigInt,
        type_state -> Text,
        r#type -> Nullable<Text>,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    spell_projection (id) {
        id -> BigInt,
        root_record_id -> Nullable<BigInt>,
        actor_item_id -> Nullable<BigInt>,
        rank_state -> Text,
        rank -> Nullable<SourceNumberSql>,
        traditions_state -> Text,
        focus_state -> Text,
        focus -> Nullable<BigInt>,
        ritual_state -> Text,
        ritual -> Nullable<BigInt>,
        casting_time_state -> Text,
        casting_time -> Nullable<Text>,
        casting_form_state -> Text,
        casting_form -> Nullable<Text>,
        save_state -> Text,
        save -> Nullable<Text>,
        basic_save_state -> Text,
        basic_save -> Nullable<BigInt>,
        passive_defense_state -> Text,
        passive_defense -> Nullable<Text>,
        area_type_state -> Text,
        area_type -> Nullable<Text>,
        area_size_state -> Text,
        area_size -> Nullable<SourceNumberSql>,
        duration_text_state -> Text,
        duration_text -> Nullable<Text>,
        sustained_state -> Text,
        sustained -> Nullable<BigInt>,
        damage_state -> Text,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    spell_traditions (spell_id, value) {
        spell_id -> BigInt,
        value -> Text,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    spell_damage_entries (id) {
        id -> BigInt,
        spell_id -> BigInt,
        original_index -> BigInt,
        type_state -> Text,
        r#type -> Nullable<Text>,
        kinds_state -> Text,
        original_key -> Text,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    spell_damage_kinds (entry_id, value) {
        entry_id -> BigInt,
        value -> Text,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    physical_projection (id) {
        id -> BigInt,
        root_record_id -> Nullable<BigInt>,
        actor_item_id -> Nullable<BigInt>,
        price_per_item_cp_state -> Text,
        price_per_item_cp -> Nullable<SourceNumberSql>,
        bulk_state -> Text,
        bulk -> Nullable<SourceNumberSql>,
        usage_state -> Text,
        usage -> Nullable<Text>,
        consumable_category_state -> Text,
        consumable_category -> Nullable<Text>,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    weapon_projection (physical_id) {
        physical_id -> BigInt,
        category_state -> Text,
        category -> Nullable<Text>,
        weapon_group_state -> Text,
        weapon_group -> Nullable<Text>,
        damage_type_state -> Text,
        damage_type -> Nullable<Text>,
        range_state -> Text,
        range -> Nullable<SourceNumberSql>,
        reload_state -> Text,
        reload -> Nullable<Text>,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    armor_projection (physical_id) {
        physical_id -> BigInt,
        category_state -> Text,
        category -> Nullable<Text>,
        ac_bonus_state -> Text,
        ac_bonus -> Nullable<SourceNumberSql>,
        dex_cap_state -> Text,
        dex_cap -> Nullable<SourceNumberSql>,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    shield_projection (physical_id) {
        physical_id -> BigInt,
        hardness_state -> Text,
        hardness -> Nullable<SourceNumberSql>,
        hp_maximum_state -> Text,
        hp_maximum -> Nullable<SourceNumberSql>,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    ability_projection (id) {
        id -> BigInt,
        root_record_id -> Nullable<BigInt>,
        actor_item_id -> Nullable<BigInt>,
        action_type_state -> Text,
        action_type -> Nullable<Text>,
        action_count_state -> Text,
        action_count -> Nullable<SourceNumberSql>,
        category_state -> Text,
        category -> Nullable<Text>,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    heritage_projection (id) {
        id -> BigInt,
        root_record_id -> Nullable<BigInt>,
        actor_item_id -> Nullable<BigInt>,
        ancestry_uuid_state -> Text,
        ancestry_uuid -> Nullable<Text>,
        ancestry_slug_state -> Text,
        ancestry_slug -> Nullable<Text>,
        versatile_state -> Text,
        versatile -> Nullable<BigInt>,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    effect_projection (id) {
        id -> BigInt,
        root_record_id -> Nullable<BigInt>,
        actor_item_id -> Nullable<BigInt>,
        duration_unit_state -> Text,
        duration_unit -> Nullable<Text>,
        duration_value_state -> Text,
        duration_value -> Nullable<SourceNumberSql>,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    condition_projection (id) {
        id -> BigInt,
        root_record_id -> Nullable<BigInt>,
        actor_item_id -> Nullable<BigInt>,
        is_valued_state -> Text,
        is_valued -> Nullable<BigInt>,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    deity_projection (id) {
        id -> BigInt,
        root_record_id -> Nullable<BigInt>,
        actor_item_id -> Nullable<BigInt>,
        primary_domains_state -> Text,
        alternate_domains_state -> Text,
        fonts_state -> Text,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    deity_domains (deity_id, kind, value) {
        deity_id -> BigInt,
        kind -> Text,
        value -> Text,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    deity_fonts (deity_id, value) {
        deity_id -> BigInt,
        value -> Text,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    artifact_context (singleton) {
        singleton -> BigInt,
        format_version -> BigInt,
        context_hash -> Text,
        context_json -> Text,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    record_bodies (record_id) {
        record_id -> BigInt,
        codec_version -> BigInt,
        encoding -> Text,
        snapshot -> Binary,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    prepared_content (content_id) {
        content_id -> BigInt,
        record_id -> BigInt,
        owners_json -> Text,
        field_path -> Text,
        role -> Text,
        visibility -> Text,
        authored_markup_sha256 -> Text,
        preparation_context_hash -> Text,
        outcome -> Text,
        html_gzip -> Nullable<Binary>,
        interactions_json -> Text,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    developer_diagnostics (diagnostic_id) {
        diagnostic_id -> BigInt,
        record_id -> BigInt,
        owners_json -> Nullable<Text>,
        field_path -> Nullable<Text>,
        stage -> Text,
        details_json -> Text,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    relationship_occurrences (occurrence_id) {
        occurrence_id -> BigInt,
        record_id -> BigInt,
        owners_json -> Text,
        field_path -> Text,
        ordinal -> BigInt,
        origin -> Text,
        kind -> Text,
        authored_target -> Nullable<Text>,
        occurrence_path -> Nullable<Text>,
        details_json -> Nullable<Text>,
        resolution -> Text,
        target_record_id -> Nullable<BigInt>,
        target_owners_json -> Nullable<Text>,
        target_url -> Nullable<Text>,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    verified_aliases (record_id, alias) {
        record_id -> BigInt,
        alias -> Text,
        alias_lookup_key -> Text,
        evidence_json -> Text,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    remaster_pairs (legacy_record_id) {
        legacy_record_id -> BigInt,
        remaster_record_id -> BigInt,
        evidence_json -> Text,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    query_field_catalog (field) {
        field -> Text,
        definition_json -> Text,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    lexical_units (unit_id) {
        unit_id -> BigInt,
        record_id -> BigInt,
        owners_json -> Text,
        field_path -> Nullable<Text>,
        section_json -> Nullable<Text>,
        unit_kind -> Text,
        identity_terms -> Text,
        alias_terms -> Text,
        structured_terms -> Text,
        definition_terms -> Text,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    semantic_models (model_id) {
        model_id -> BigInt,
        identity_json -> Text,
        dimensions -> BigInt,
        unit_policy_version -> BigInt,
    }
}
diesel::table! {
    use diesel::sql_types::*;
    use crate::numeric::SourceNumberSql;
    semantic_units (unit_id) {
        unit_id -> BigInt,
        record_id -> BigInt,
        model_id -> BigInt,
        owners_json -> Text,
        unit_kind -> Text,
        field_path -> Nullable<Text>,
        section_json -> Text,
        chunk_ordinal -> BigInt,
        input_token_count -> BigInt,
        input_hash -> Text,
    }
}
