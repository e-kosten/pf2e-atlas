use atlas_record::metrics as metric_definitions;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MetricCoercion {
    Number,
    NumberLike,
    DamageDieFaces,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MetricPathShape {
    Pointer(&'static str),
    Template {
        prefix: &'static str,
        suffix: &'static str,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct MetricValuePath {
    pub shape: MetricPathShape,
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
    pub paths: &'static [MetricValuePath],
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DynamicMetricSourceSpec {
    pub definition: metric_definitions::MetricDefinition,
    pub key_builder: fn(&str) -> String,
    pub capture_source: MetricCaptureSource,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum MetricCaptureSource {
    FixedCapture {
        capture: &'static str,
        paths: &'static [MetricValuePath],
    },
    ClosedVocabulary {
        captures: &'static [&'static str],
        paths: &'static [MetricValuePath],
    },
    ObjectEntries {
        collection_path: &'static str,
        capture_normalize: CaptureNormalize,
        value_paths: &'static [MetricValuePath],
    },
    ArrayEntries {
        collection_path: &'static str,
        capture_path: &'static str,
        capture_normalize: CaptureNormalize,
        value_paths: &'static [MetricValuePath],
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
    DynamicMetricSourceSpec {
        definition: metric_definitions::actor::ability::MOD,
        key_builder: metric_definitions::actor::ability::mod_key,
        capture_source: MetricCaptureSource::ClosedVocabulary {
            captures: &["str", "dex", "con", "int", "wis", "cha"],
            paths: &[
                number_template("/system/abilities/", "/mod"),
                number_template("/system/abilities/", "/modifier"),
            ],
        },
    },
    DynamicMetricSourceSpec {
        definition: metric_definitions::actor::save::MOD,
        key_builder: metric_definitions::actor::save::mod_key,
        capture_source: MetricCaptureSource::ObjectEntries {
            collection_path: "/system/saves",
            capture_normalize: CaptureNormalize::SaveKey,
            value_paths: &[
                number("/mod"),
                number("/modifier"),
                number("/value"),
                number("/totalModifier"),
            ],
        },
    },
    DynamicMetricSourceSpec {
        definition: metric_definitions::actor::skill::MOD,
        key_builder: metric_definitions::actor::skill::mod_key,
        capture_source: MetricCaptureSource::ObjectEntries {
            collection_path: "/system/skills",
            capture_normalize: CaptureNormalize::Slug,
            value_paths: &[number("/mod"), number("/modifier"), number("/value")],
        },
    },
    DynamicMetricSourceSpec {
        definition: metric_definitions::actor::skill::RANK,
        key_builder: metric_definitions::actor::skill::rank_key,
        capture_source: MetricCaptureSource::ObjectEntries {
            collection_path: "/system/skills",
            capture_normalize: CaptureNormalize::Slug,
            value_paths: &[number("/rank")],
        },
    },
    DynamicMetricSourceSpec {
        definition: metric_definitions::actor::speed::VALUE,
        key_builder: metric_definitions::actor::speed::value_key,
        capture_source: MetricCaptureSource::FixedCapture {
            capture: "land",
            paths: &[number_like("/system/attributes/speed/value")],
        },
    },
    DynamicMetricSourceSpec {
        definition: metric_definitions::actor::speed::VALUE,
        key_builder: metric_definitions::actor::speed::value_key,
        capture_source: MetricCaptureSource::ArrayEntries {
            collection_path: "/system/attributes/speed/otherSpeeds",
            capture_path: "/type",
            capture_normalize: CaptureNormalize::Slug,
            value_paths: &[number_like("/value")],
        },
    },
    DynamicMetricSourceSpec {
        definition: metric_definitions::actor::sense::RANGE,
        key_builder: metric_definitions::actor::sense::range_key,
        capture_source: MetricCaptureSource::ArrayEntries {
            collection_path: "/system/perception/senses",
            capture_path: "/type",
            capture_normalize: CaptureNormalize::Slug,
            value_paths: &[number_like("/range")],
        },
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

const fn number(path: &'static str) -> MetricValuePath {
    MetricValuePath {
        shape: MetricPathShape::Pointer(path),
        coercion: MetricCoercion::Number,
    }
}

const fn number_like(path: &'static str) -> MetricValuePath {
    MetricValuePath {
        shape: MetricPathShape::Pointer(path),
        coercion: MetricCoercion::NumberLike,
    }
}

const fn damage_die_faces(path: &'static str) -> MetricValuePath {
    MetricValuePath {
        shape: MetricPathShape::Pointer(path),
        coercion: MetricCoercion::DamageDieFaces,
    }
}

const fn number_template(prefix: &'static str, suffix: &'static str) -> MetricValuePath {
    MetricValuePath {
        shape: MetricPathShape::Template { prefix, suffix },
        coercion: MetricCoercion::Number,
    }
}
