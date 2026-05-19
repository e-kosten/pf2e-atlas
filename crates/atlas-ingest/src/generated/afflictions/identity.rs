use crate::generated::afflictions::AfflictionFamily;
use crate::records::variants;
use crate::source::normalize::normalize_text;

pub(super) fn choose_affliction_canonical_identity_key(candidate_keys: &[String]) -> String {
    let mut keys = variants::sorted_unique(candidate_keys.to_vec());
    keys.sort_by(|left, right| {
        affliction_identity_key_rank(left)
            .cmp(&affliction_identity_key_rank(right))
            .then_with(|| left.cmp(right))
    });
    keys.into_iter().next().unwrap_or_default()
}

pub(super) fn build_affliction_occurrence_candidate_keys(
    family: AfflictionFamily,
    name: &str,
    slug: Option<&str>,
    compendium_source: Option<&str>,
    source_record_key: Option<&str>,
) -> Vec<String> {
    variants::sorted_unique(
        [
            source_record_key.map(|value| format!("record:{value}")),
            compendium_source.map(|value| format!("compendium:{}", normalize_text(value))),
            slug.map(|value| {
                format!(
                    "slug:{}:{}",
                    affliction_family_label(family),
                    normalize_text(value)
                )
            }),
            Some(format!(
                "name:{}:{}",
                affliction_family_label(family),
                normalize_text(name)
            )),
        ]
        .into_iter()
        .flatten()
        .collect(),
    )
}

pub(super) fn hash_text(value: &str) -> String {
    let mut hash: u32 = 2_166_136_261;
    for byte in value.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(16_777_619);
    }
    format!("{hash:x}")
}

fn affliction_identity_key_rank(value: &str) -> u8 {
    if value.starts_with("record:") {
        0
    } else if value.starts_with("compendium:") {
        1
    } else if value.starts_with("slug:") {
        2
    } else {
        3
    }
}

fn affliction_family_label(family: AfflictionFamily) -> &'static str {
    match family {
        AfflictionFamily::Curse => "curse",
        AfflictionFamily::Disease => "disease",
        AfflictionFamily::Poison => "poison",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_identity_prefers_record_keys_over_looser_aliases() {
        assert_eq!(
            choose_affliction_canonical_identity_key(&[
                "name:curse:theft of thought".to_string(),
                "slug:curse:theft-of-thought".to_string(),
                "record:equipment:abc123".to_string(),
            ]),
            "record:equipment:abc123"
        );
    }

    #[test]
    fn candidate_keys_include_all_available_identity_evidence() {
        assert_eq!(
            build_affliction_occurrence_candidate_keys(
                AfflictionFamily::Disease,
                "Blinding Sickness",
                Some("blinding-sickness"),
                Some("Compendium.pf2e.equipment-srd.Item.abc123"),
                Some("equipment-srd:abc123"),
            ),
            vec![
                "compendium:compendium.pf2e.equipment-srd.item.abc123".to_string(),
                "name:disease:blinding sickness".to_string(),
                "record:equipment-srd:abc123".to_string(),
                "slug:disease:blinding-sickness".to_string(),
            ]
        );
    }
}
