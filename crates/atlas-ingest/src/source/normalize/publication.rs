use atlas_domain::PublicationCategory;

use crate::source::normalize::normalize_text;

pub(crate) fn publication_family(
    pack_name: &str,
    publication_title: Option<&str>,
) -> PublicationCategory {
    if is_core_publication(publication_title) {
        return PublicationCategory::Core;
    }
    if is_adventure_publication(publication_title) || is_adventure_pack(pack_name) {
        return PublicationCategory::Adventure;
    }
    if publication_title.is_some_and(|title| !title.trim().is_empty()) {
        return PublicationCategory::Rules;
    }
    PublicationCategory::Unknown
}

fn is_core_publication(publication_title: Option<&str>) -> bool {
    matches!(
        normalize_text(publication_title.unwrap_or_default()).as_str(),
        "pathfinder player core"
            | "player core"
            | "pathfinder player core 2"
            | "player core 2"
            | "pathfinder gm core"
            | "gm core"
            | "pathfinder monster core"
            | "monster core"
            | "pathfinder monster core 2"
            | "monster core 2"
            | "pathfinder beginner box"
    )
}

fn is_adventure_publication(publication_title: Option<&str>) -> bool {
    let normalized = normalize_text(publication_title.unwrap_or_default());
    !normalized.is_empty()
        && (normalized.contains("adventure path")
            || normalized.contains("pathfinder society")
            || normalized.contains("quest")
            || normalized.contains("one shot")
            || normalized.contains("special")
            || normalized.starts_with("pathfinder adventure ")
            || is_pathfinder_numbered_adventure(&normalized))
}

fn is_pathfinder_numbered_adventure(value: &str) -> bool {
    let mut parts = value.split_whitespace();
    matches!(parts.next(), Some("pathfinder"))
        && parts.next().is_some_and(|part| part.parse::<u16>().is_ok())
}

fn is_adventure_pack(pack_name: &str) -> bool {
    let normalized = normalize_text(pack_name);
    normalized.starts_with("pfs ")
        || normalized.contains("one shot")
        || normalized.contains("quest")
}
