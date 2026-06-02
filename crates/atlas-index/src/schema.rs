diesel::table! {
    artifact_metadata (key) {
        key -> Text,
        value -> Text,
    }
}

diesel::table! {
    packs (name) {
        name -> Text,
        label -> Text,
        document_type -> Text,
        declared_path -> Text,
        resolved_path -> Text,
        record_count -> BigInt,
    }
}

diesel::table! {
    records (record_key) {
        record_key -> Text,
        id -> Text,
        name -> Text,
        normalized_name -> Text,
        record_kind -> Text,
        pack_name -> Text,
        pack_label -> Text,
        foundry_document_type -> Text,
        foundry_record_type -> Text,
        level -> Nullable<BigInt>,
        rarity -> Nullable<Text>,
        traits_json -> Text,
        prerequisites_json -> Text,
        system_category -> Nullable<Text>,
        system_group -> Nullable<Text>,
        system_base_item -> Nullable<Text>,
        system_usage -> Nullable<Text>,
        system_price_json -> Nullable<Text>,
        system_actions_value -> Nullable<BigInt>,
        system_time_value -> Nullable<Text>,
        system_duration_value -> Nullable<Text>,
        price_cp -> Nullable<BigInt>,
        activation_time_kind -> Nullable<Text>,
        activation_time_actions -> Nullable<BigInt>,
        activation_time_duration_value -> Nullable<BigInt>,
        activation_time_duration_unit -> Nullable<Text>,
        activation_time_text -> Nullable<Text>,
        duration_kind -> Nullable<Text>,
        duration_value -> Nullable<BigInt>,
        duration_unit -> Nullable<Text>,
        duration_text -> Nullable<Text>,
        publication_title -> Nullable<Text>,
        publication_remaster -> Bool,
        description_json -> Nullable<Text>,
        blurb_json -> Nullable<Text>,
        publication_family -> Text,
        folder_id -> Nullable<Text>,
        taxonomy_families_json -> Text,
        variant_group_key -> Nullable<Text>,
        variant_base_name -> Nullable<Text>,
        variant_label -> Nullable<Text>,
        variant_axes_json -> Text,
        variant_confidence -> Nullable<Double>,
        variant_source -> Text,
        source_path -> Text,
        is_default_visible -> Bool,
        raw_json -> Text,
    }
}

diesel::table! {
    record_content (record_key, content_key) {
        record_key -> Text,
        content_key -> Text,
        ordinal -> BigInt,
        source_kind -> Text,
        visibility -> Text,
        contributes_to_search -> Bool,
        contributes_to_references -> Bool,
        label -> Nullable<Text>,
        content_json -> Text,
    }
}

diesel::table! {
    record_traits (record_key, trait_) {
        record_key -> Text,
        #[sql_name = "trait"]
        trait_ -> Text,
    }
}

diesel::table! {
    reference_edges (from_record_key, to_record_key, reference_text, source_kind) {
        from_record_key -> Text,
        to_record_key -> Text,
        display_text -> Nullable<Text>,
        reference_text -> Text,
        source_kind -> Text,
        visibility -> Text,
    }
}

diesel::table! {
    reference_occurrences (record_key, content_key, occurrence_ordinal) {
        record_key -> Text,
        content_key -> Text,
        occurrence_ordinal -> BigInt,
        target_record_key -> Text,
        source_kind -> Text,
        visibility -> Text,
        display_text -> Nullable<Text>,
        reference_text -> Text,
    }
}

diesel::table! {
    record_aliases (canonical_record_key, normalized_alias, source_kind, source_ref) {
        canonical_record_key -> Text,
        alias_text -> Text,
        normalized_alias -> Text,
        source_kind -> Text,
        source_ref -> Text,
    }
}

diesel::table! {
    remaster_links (remaster_record_key, legacy_record_key, source_kind, source_ref) {
        remaster_record_key -> Text,
        legacy_record_key -> Text,
        source_kind -> Text,
        source_ref -> Text,
    }
}

diesel::table! {
    record_metrics (record_key, metric_domain, metric_key) {
        record_key -> Text,
        metric_domain -> Text,
        metric_key -> Text,
        value_type -> Text,
        number_value -> Nullable<Double>,
        text_value -> Nullable<Text>,
        bool_value -> Nullable<Bool>,
    }
}

diesel::table! {
    metric_key_catalog (metric_domain, record_kind, metric_key) {
        metric_domain -> Text,
        record_kind -> Nullable<Text>,
        namespace_prefix -> Text,
        metric_key -> Text,
        value_type -> Text,
        catalog_count -> BigInt,
        numeric_min -> Nullable<Double>,
        numeric_max -> Nullable<Double>,
    }
}

diesel::table! {
    metric_value_catalog (metric_domain, record_kind, metric_key, value) {
        metric_domain -> Text,
        record_kind -> Nullable<Text>,
        metric_key -> Text,
        value -> Text,
        catalog_count -> BigInt,
    }
}

diesel::table! {
    filter_field_catalog (field, record_kind) {
        field -> Text,
        record_kind -> Nullable<Text>,
        field_type -> Text,
        field_group -> Text,
        value_policy -> Text,
        operators_json -> Text,
        cli_flags_json -> Text,
        applicable_kinds_json -> Text,
        value_count -> BigInt,
        matching_record_count -> BigInt,
        null_count -> BigInt,
        distinct_count -> BigInt,
        singleton_count -> BigInt,
        singleton_ratio -> Nullable<Double>,
        observation_singleton_ratio -> Nullable<Double>,
        policy_reason -> Text,
    }
}

diesel::table! {
    filter_value_catalog (field, record_kind, value) {
        field -> Text,
        record_kind -> Nullable<Text>,
        value -> Text,
        catalog_count -> BigInt,
    }
}

diesel::table! {
    filter_sample_catalog (field, record_kind, value) {
        field -> Text,
        record_kind -> Nullable<Text>,
        value -> Text,
        catalog_count -> BigInt,
        sample_rank -> BigInt,
    }
}

diesel::table! {
    filter_numeric_catalog (field, record_kind, metric_domain, metric_key) {
        field -> Text,
        record_kind -> Nullable<Text>,
        metric_domain -> Nullable<Text>,
        metric_key -> Nullable<Text>,
        catalog_count -> BigInt,
        null_count -> BigInt,
        min -> Nullable<Double>,
        p05 -> Nullable<Double>,
        p25 -> Nullable<Double>,
        p50 -> Nullable<Double>,
        mean -> Nullable<Double>,
        p75 -> Nullable<Double>,
        p95 -> Nullable<Double>,
        max -> Nullable<Double>,
    }
}

diesel::table! {
    actor_records (record_key) {
        record_key -> Text,
        size -> Nullable<Text>,
        languages_json -> Text,
        speed_types_json -> Text,
        senses_json -> Text,
        immunities_json -> Text,
        resistances_json -> Text,
        weaknesses_json -> Text,
        disable_text -> Nullable<Text>,
        disable_skills_json -> Text,
        is_complex -> Bool,
    }
}

diesel::table! {
    item_records (record_key) {
        record_key -> Text,
        system_category -> Nullable<Text>,
        system_base_item -> Nullable<Text>,
        system_group -> Nullable<Text>,
        system_usage -> Nullable<Text>,
        system_price_json -> Nullable<Text>,
        price_cp -> Nullable<BigInt>,
        bulk_value -> Nullable<Double>,
        hands_requirement -> Nullable<Text>,
        damage_types_json -> Text,
    }
}

diesel::table! {
    spell_records (record_key) {
        record_key -> Text,
        traditions_json -> Text,
        spell_kinds_json -> Text,
        range_text -> Nullable<Text>,
        range_value -> Nullable<Double>,
        target_text -> Nullable<Text>,
        area_type -> Nullable<Text>,
        area_value -> Nullable<Double>,
        save_type -> Nullable<Text>,
        sustained -> Bool,
        basic_save -> Bool,
        damage_types_json -> Text,
    }
}

diesel::table! {
    records_fts (record_key) {
        record_key -> Text,
        title -> Nullable<Text>,
        aliases -> Nullable<Text>,
        traits -> Nullable<Text>,
        taxonomy_terms -> Nullable<Text>,
        constraint_terms -> Nullable<Text>,
        mechanic_terms -> Nullable<Text>,
        source_terms -> Nullable<Text>,
        metric_terms -> Nullable<Text>,
        headings -> Nullable<Text>,
        body -> Nullable<Text>,
        facts -> Nullable<Text>,
        reference_terms -> Nullable<Text>,
        embedded_content -> Nullable<Text>,
    }
}

diesel::table! {
    document_embedding_cache (embedding_unit_key) {
        embedding_unit_key -> Text,
        record_key -> Text,
        unit_kind -> Text,
        label -> Nullable<Text>,
        ordinal -> BigInt,
        semantic_input_hash -> Text,
        dimensions -> BigInt,
        vector_blob -> Binary,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    actor_records,
    artifact_metadata,
    document_embedding_cache,
    filter_field_catalog,
    filter_numeric_catalog,
    filter_sample_catalog,
    filter_value_catalog,
    item_records,
    metric_key_catalog,
    metric_value_catalog,
    packs,
    record_aliases,
    record_content,
    record_metrics,
    record_traits,
    records,
    records_fts,
    reference_edges,
    reference_occurrences,
    remaster_links,
    spell_records,
);
