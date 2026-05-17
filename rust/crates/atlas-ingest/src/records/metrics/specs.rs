use atlas_record::metrics as metric_definitions;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MetricCoercion {
    Number,
    NumberLike,
    DamageDieFaces,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct MetricPathCandidate {
    pub path: &'static str,
    pub coercion: MetricCoercion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct MetricPathTemplateCandidate {
    pub prefix: &'static str,
    pub suffix: &'static str,
    pub coercion: MetricCoercion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CaptureNormalize {
    Slug,
    SaveKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct StaticMetricSourceSpec {
    pub definition: metric_definitions::MetricDefinition,
    pub paths: &'static [MetricPathCandidate],
}

#[derive(Debug, Clone, Copy)]
pub(super) enum DynamicMetricSourceSpec {
    FixedCapture {
        definition: metric_definitions::MetricDefinition,
        capture: &'static str,
        key_builder: fn(&str) -> String,
        paths: &'static [MetricPathCandidate],
    },
    ClosedVocabulary {
        definition: metric_definitions::MetricDefinition,
        captures: &'static [&'static str],
        key_builder: fn(&str) -> String,
        path_templates: &'static [MetricPathTemplateCandidate],
    },
    ObjectEntries {
        definition: metric_definitions::MetricDefinition,
        collection_path: &'static str,
        capture_normalize: CaptureNormalize,
        key_builder: fn(&str) -> String,
        value_paths: &'static [MetricPathCandidate],
    },
    ArrayEntries {
        definition: metric_definitions::MetricDefinition,
        collection_path: &'static str,
        capture_path: &'static str,
        capture_normalize: CaptureNormalize,
        key_builder: fn(&str) -> String,
        value_paths: &'static [MetricPathCandidate],
    },
}

pub(super) const ACTOR_STATIC_SPECS: &[StaticMetricSourceSpec] = &[
    StaticMetricSourceSpec {
        definition: metric_definitions::actor::PERCEPTION_MOD,
        paths: &[
            number("/system/perception/mod"),
            number("/system/perception/modifier"),
            number("/system/perception/value"),
        ],
    },
    StaticMetricSourceSpec {
        definition: metric_definitions::actor::ARMOR_CLASS,
        paths: &[number("/system/attributes/ac/value")],
    },
    StaticMetricSourceSpec {
        definition: metric_definitions::actor::HARDNESS,
        paths: &[number("/system/attributes/hardness")],
    },
    StaticMetricSourceSpec {
        definition: metric_definitions::actor::HP_VALUE,
        paths: &[number("/system/attributes/hp/value")],
    },
    StaticMetricSourceSpec {
        definition: metric_definitions::actor::HP_MAX,
        paths: &[number("/system/attributes/hp/max")],
    },
    StaticMetricSourceSpec {
        definition: metric_definitions::actor::HP_BROKEN_THRESHOLD,
        paths: &[
            number("/system/attributes/hp/brokenThreshold"),
            number("/system/attributes/hp/broken"),
            number("/system/attributes/hp/bt"),
        ],
    },
    StaticMetricSourceSpec {
        definition: metric_definitions::actor::STEALTH_MOD,
        paths: &[
            number("/system/attributes/stealth/value"),
            number("/system/attributes/stealth/mod"),
            number("/system/attributes/stealth/modifier"),
        ],
    },
];

pub(super) const ACTOR_DYNAMIC_SPECS: &[DynamicMetricSourceSpec] = &[
    DynamicMetricSourceSpec::ClosedVocabulary {
        definition: metric_definitions::actor::ability::MOD,
        captures: &["str", "dex", "con", "int", "wis", "cha"],
        key_builder: metric_definitions::actor::ability::mod_key,
        path_templates: &[
            number_template("/system/abilities/", "/mod"),
            number_template("/system/abilities/", "/modifier"),
        ],
    },
    DynamicMetricSourceSpec::ObjectEntries {
        definition: metric_definitions::actor::save::MOD,
        collection_path: "/system/saves",
        capture_normalize: CaptureNormalize::SaveKey,
        key_builder: metric_definitions::actor::save::mod_key,
        value_paths: &[
            number("/mod"),
            number("/modifier"),
            number("/value"),
            number("/totalModifier"),
        ],
    },
    DynamicMetricSourceSpec::ObjectEntries {
        definition: metric_definitions::actor::skill::MOD,
        collection_path: "/system/skills",
        capture_normalize: CaptureNormalize::Slug,
        key_builder: metric_definitions::actor::skill::mod_key,
        value_paths: &[number("/mod"), number("/modifier"), number("/value")],
    },
    DynamicMetricSourceSpec::ObjectEntries {
        definition: metric_definitions::actor::skill::RANK,
        collection_path: "/system/skills",
        capture_normalize: CaptureNormalize::Slug,
        key_builder: metric_definitions::actor::skill::rank_key,
        value_paths: &[number("/rank")],
    },
    DynamicMetricSourceSpec::FixedCapture {
        definition: metric_definitions::actor::speed::VALUE,
        capture: "land",
        key_builder: metric_definitions::actor::speed::value_key,
        paths: &[number_like("/system/attributes/speed/value")],
    },
    DynamicMetricSourceSpec::ArrayEntries {
        definition: metric_definitions::actor::speed::VALUE,
        collection_path: "/system/attributes/speed/otherSpeeds",
        capture_path: "/type",
        capture_normalize: CaptureNormalize::Slug,
        key_builder: metric_definitions::actor::speed::value_key,
        value_paths: &[number_like("/value")],
    },
    DynamicMetricSourceSpec::ArrayEntries {
        definition: metric_definitions::actor::sense::RANGE,
        collection_path: "/system/perception/senses",
        capture_path: "/type",
        capture_normalize: CaptureNormalize::Slug,
        key_builder: metric_definitions::actor::sense::range_key,
        value_paths: &[number_like("/range")],
    },
];

pub(super) const WEAPON_STATIC_SPECS: &[StaticMetricSourceSpec] = &[
    StaticMetricSourceSpec {
        definition: metric_definitions::item::weapon::RANGE_INCREMENT,
        paths: &[
            number_like("/system/range/increment"),
            number_like("/system/range/value"),
            number_like("/system/range"),
        ],
    },
    StaticMetricSourceSpec {
        definition: metric_definitions::item::weapon::RELOAD,
        paths: &[
            number_like("/system/reload/value"),
            number_like("/system/reload"),
        ],
    },
    StaticMetricSourceSpec {
        definition: metric_definitions::item::weapon::DAMAGE_DICE,
        paths: &[number("/system/damage/dice")],
    },
    StaticMetricSourceSpec {
        definition: metric_definitions::item::weapon::DAMAGE_DIE_FACES,
        paths: &[damage_die_faces("/system/damage/die")],
    },
];

pub(super) const ARMOR_STATIC_SPECS: &[StaticMetricSourceSpec] = &[
    StaticMetricSourceSpec {
        definition: metric_definitions::item::armor::AC_BONUS,
        paths: &[number("/system/acBonus")],
    },
    StaticMetricSourceSpec {
        definition: metric_definitions::item::armor::DEX_CAP,
        paths: &[number("/system/dexCap")],
    },
    StaticMetricSourceSpec {
        definition: metric_definitions::item::armor::STRENGTH,
        paths: &[number("/system/strength")],
    },
    StaticMetricSourceSpec {
        definition: metric_definitions::item::armor::CHECK_PENALTY,
        paths: &[number("/system/checkPenalty")],
    },
    StaticMetricSourceSpec {
        definition: metric_definitions::item::armor::SPEED_PENALTY,
        paths: &[number("/system/speedPenalty")],
    },
];

pub(super) const SHIELD_STATIC_SPECS: &[StaticMetricSourceSpec] = &[
    StaticMetricSourceSpec {
        definition: metric_definitions::item::shield::AC_BONUS,
        paths: &[number("/system/acBonus")],
    },
    StaticMetricSourceSpec {
        definition: metric_definitions::item::shield::HARDNESS,
        paths: &[number("/system/hardness")],
    },
    StaticMetricSourceSpec {
        definition: metric_definitions::item::shield::HP,
        paths: &[number("/system/hp/value"), number("/system/hp/max")],
    },
    StaticMetricSourceSpec {
        definition: metric_definitions::item::shield::BROKEN_THRESHOLD,
        paths: &[
            number("/system/hp/brokenThreshold"),
            number("/system/hp/broken"),
            number("/system/hp/bt"),
        ],
    },
];

const fn number(path: &'static str) -> MetricPathCandidate {
    MetricPathCandidate {
        path,
        coercion: MetricCoercion::Number,
    }
}

const fn number_like(path: &'static str) -> MetricPathCandidate {
    MetricPathCandidate {
        path,
        coercion: MetricCoercion::NumberLike,
    }
}

const fn damage_die_faces(path: &'static str) -> MetricPathCandidate {
    MetricPathCandidate {
        path,
        coercion: MetricCoercion::DamageDieFaces,
    }
}

const fn number_template(
    prefix: &'static str,
    suffix: &'static str,
) -> MetricPathTemplateCandidate {
    MetricPathTemplateCandidate {
        prefix,
        suffix,
        coercion: MetricCoercion::Number,
    }
}
