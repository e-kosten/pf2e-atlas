use super::{
    ContentInteraction, ContentReferenceKind, ContentReferenceOccurrence,
    ContentReferenceResolution, ContentReferenceTarget, OwnedContentIdentity, OwnedContentLocator,
    SourceContentLocator,
};
use scraper::{Html, Selector};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

/// Versioned canonical digest: byte-length-framed UTF-8 fields, with enum tags,
/// ordered owner/audience sequences and sorted Embed options. This establishes
/// cache coherence, not authentication against coordinated artifact tampering.
pub(super) fn reference_binding(
    locator: &SourceContentLocator,
    authored_hash: &str,
    reference: &ContentReferenceOccurrence,
) -> String {
    fn text(h: &mut Sha256, s: &str) {
        h.update((s.len() as u64).to_be_bytes());
        h.update(s.as_bytes());
    }
    fn owners(h: &mut Sha256, chain: &[OwnedContentLocator]) {
        text(h, &chain.len().to_string());
        for owner in chain {
            text(h, &owner.collection);
            match &owner.identity {
                OwnedContentIdentity::Stable(id) => {
                    text(h, "stable");
                    text(h, id);
                }
                OwnedContentIdentity::SnapshotLocal { index } => {
                    text(h, "snapshot_local");
                    text(h, &index.to_string());
                }
            }
        }
    }
    let mut hash = Sha256::new();
    text(&mut hash, "atlas-reference-binding/v1");
    text(&mut hash, &locator.record.to_string());
    owners(&mut hash, &locator.owners);
    text(&mut hash, &locator.field);
    text(&mut hash, authored_hash);
    text(&mut hash, &reference.ordinal.to_string());
    text(&mut hash, &reference.path);
    match &reference.kind {
        ContentReferenceKind::Uuid => text(&mut hash, "uuid"),
        ContentReferenceKind::Compendium => text(&mut hash, "compendium"),
        ContentReferenceKind::HtmlLink => text(&mut hash, "html_link"),
        ContentReferenceKind::Embed { options } => {
            text(&mut hash, "embed");
            text(&mut hash, &options.len().to_string());
            for (k, v) in options {
                text(&mut hash, k);
                text(&mut hash, v);
            }
        }
    }
    text(&mut hash, &reference.authored_target);
    match &reference.resolution {
        ContentReferenceResolution::Resolved(ContentReferenceTarget::Record { key }) => {
            text(&mut hash, "record");
            text(&mut hash, &key.to_string());
        }
        ContentReferenceResolution::Resolved(ContentReferenceTarget::OwnedNode {
            key,
            owners: chain,
        }) => {
            text(&mut hash, "owned");
            text(&mut hash, &key.to_string());
            owners(&mut hash, chain);
        }
        ContentReferenceResolution::Resolved(ContentReferenceTarget::Url { url }) => {
            text(&mut hash, "resolved_url");
            text(&mut hash, url);
        }
        ContentReferenceResolution::UnverifiedUrl { url } => {
            text(&mut hash, "unverified_url");
            text(&mut hash, url);
        }
        ContentReferenceResolution::Unresolved => text(&mut hash, "unresolved"),
        ContentReferenceResolution::Blocked => text(&mut hash, "blocked"),
    }
    text(
        &mut hash,
        if reference.visible {
            "visible"
        } else {
            "hidden"
        },
    );
    text(&mut hash, &reference.audiences.len().to_string());
    for audience in &reference.audiences {
        text(&mut hash, audience);
    }
    format!("{:x}", hash.finalize())
}

fn reference_markers(
    document: &Html,
) -> Result<BTreeMap<usize, (String, Option<String>)>, PreparedMarkerError> {
    let selector = Selector::parse("[data-atlas-reference], [data-atlas-reference-binding]")
        .map_err(|e| PreparedMarkerError(e.to_string()))?;
    let mut markers = BTreeMap::new();
    for element in document.select(&selector) {
        let ordinal = element
            .attr("data-atlas-reference")
            .ok_or_else(|| PreparedMarkerError("reference binding has no ordinal".into()))?
            .parse::<usize>()
            .map_err(|_| {
                PreparedMarkerError("reference ordinal is not an unsigned integer".into())
            })?;
        let binding = element
            .attr("data-atlas-reference-binding")
            .ok_or_else(|| PreparedMarkerError("reference marker has no binding".into()))?;
        if binding.len() != 64
            || !binding
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(PreparedMarkerError(
                "reference binding is not a canonical SHA256 digest".into(),
            ));
        }
        if markers
            .insert(
                ordinal,
                (binding.into(), element.attr("href").map(str::to_owned)),
            )
            .is_some()
        {
            return Err(PreparedMarkerError(
                "reference ordinal is duplicated".into(),
            ));
        }
    }
    Ok(markers)
}

/// Exactly one rendered marker per visible field-local occurrence. Hidden
/// occurrences may occupy ordinal gaps, but never gain markers through this check.
pub fn validate_prepared_reference_bindings(
    html: &str,
    locator: &SourceContentLocator,
    authored_hash: &str,
    references: &[ContentReferenceOccurrence],
) -> Result<(), PreparedMarkerError> {
    let mut markers = reference_markers(&Html::parse_fragment(html))?;
    let mut seen = BTreeSet::new();
    for reference in references.iter().filter(|r| r.visible) {
        if !seen.insert(reference.ordinal) {
            return Err(PreparedMarkerError(
                "visible reference sidecar ordinal is duplicated".into(),
            ));
        }
        let marker = markers
            .remove(&reference.ordinal)
            .ok_or_else(|| PreparedMarkerError("visible reference sidecar has no marker".into()))?;
        if marker.0 != reference_binding(locator, authored_hash, reference) {
            return Err(PreparedMarkerError(
                "reference marker and visible sidecar binding disagree".into(),
            ));
        }
        if let Some(href) = marker.1 {
            let destination = match &reference.resolution {
                ContentReferenceResolution::Resolved(ContentReferenceTarget::Url { url })
                | ContentReferenceResolution::UnverifiedUrl { url } => Some(url.as_str()),
                _ => None,
            };
            if destination != Some(href.as_str()) {
                return Err(PreparedMarkerError(
                    "reference href disagrees with sidecar destination".into(),
                ));
            }
        }
    }
    if !markers.is_empty() {
        return Err(PreparedMarkerError(
            "reference marker has no visible sidecar occurrence".into(),
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedMarkerError(pub String);
impl fmt::Display for PreparedMarkerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
impl std::error::Error for PreparedMarkerError {}

/// Validate field-local marker ordinals against their interaction sidecar.
/// Parser ownership stays with scraper; no markup or mechanics are reinterpreted.
pub fn validate_prepared_markers(
    html: &str,
    interactions: &[ContentInteraction],
) -> Result<(), PreparedMarkerError> {
    // A prepared cache must already conform to the preparation owner's exact
    // HTML/URL policy. DOM serialization tolerates harmless quote spelling.
    let sanitized = super::preparation::sanitizer().clean(html).to_string();
    let canonical = |markup: &str| Html::parse_fragment(markup).root_element().inner_html();
    if canonical(html) != canonical(&sanitized) {
        return Err(PreparedMarkerError(
            "prepared HTML violates the sanitizer policy".into(),
        ));
    }
    let selector = Selector::parse("[data-atlas-interaction], [data-atlas-interaction-kind]")
        .map_err(|error| PreparedMarkerError(error.to_string()))?;
    let document = Html::parse_fragment(html);
    reference_markers(&document)?;
    let images = Selector::parse("img").map_err(|error| PreparedMarkerError(error.to_string()))?;
    if document.select(&images).any(|image| {
        !super::preparation::available_remote_image(image.attr("src").unwrap_or_default())
    }) {
        return Err(PreparedMarkerError(
            "prepared image asset is unavailable under the preparation policy".into(),
        ));
    }
    let mut seen = BTreeSet::new();
    for element in document.select(&selector) {
        if element.value().name() != "span" {
            return Err(PreparedMarkerError(
                "interaction marker must be a span".into(),
            ));
        }
        let ordinal = element
            .value()
            .attr("data-atlas-interaction")
            .ok_or_else(|| PreparedMarkerError("interaction marker has no ordinal".into()))?
            .parse::<usize>()
            .map_err(|_| {
                PreparedMarkerError("interaction ordinal is not an unsigned integer".into())
            })?;
        let interaction = interactions
            .get(ordinal)
            .ok_or_else(|| PreparedMarkerError("interaction ordinal is out of range".into()))?;
        if interaction.ordinal != ordinal || !seen.insert(ordinal) {
            return Err(PreparedMarkerError(
                "interaction ordinal is duplicated or disagrees with its sidecar".into(),
            ));
        }
        if element.value().attr("data-atlas-interaction-kind")
            != Some(interaction.kind.marker_kind())
        {
            return Err(PreparedMarkerError(
                "interaction marker kind disagrees with its sidecar".into(),
            ));
        }
    }
    if seen.len() != interactions.len() {
        return Err(PreparedMarkerError(
            "interaction sidecar has no corresponding marker".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_content::ContentInteractionKind;
    #[test]
    fn rejects_missing_duplicate_out_of_range_and_wrong_kind_markers() {
        let interactions = vec![ContentInteraction {
            ordinal: 0,
            path: "/0".into(),
            kind: ContentInteractionKind::Damage {
                formula: "2d6".into(),
                options: Default::default(),
            },
        }];
        let valid =
            "<span data-atlas-interaction='0' data-atlas-interaction-kind='damage'>2d6</span>";
        assert!(validate_prepared_markers(valid, &interactions).is_ok());
        for bad in [
            "",
            "<span data-atlas-interaction='1' data-atlas-interaction-kind='damage'>2d6</span>",
            "<span data-atlas-interaction='0' data-atlas-interaction-kind='check'>2d6</span>",
            "<span data-atlas-interaction-kind='damage'>2d6</span>",
        ] {
            assert!(validate_prepared_markers(bad, &interactions).is_err());
        }
        assert!(validate_prepared_markers(&format!("{valid}{valid}"), &interactions).is_err());
    }
    #[test]
    fn rejects_active_and_unsafe_cached_markup() {
        for html in [
            "<script>alert(1)</script>",
            "<p onclick='evil()'>text</p>",
            "<a href='javascript:evil()'>link</a>",
            "<img src='javascript:evil()'>",
        ] {
            assert!(validate_prepared_markers(html, &[]).is_err(), "{html}");
        }
        let legitimate = super::super::preparation::sanitizer().clean("<table><tbody><tr><td><strong>Value</strong></td></tr></tbody></table><a href='https://example.com'>safe</a>").to_string();
        assert!(validate_prepared_markers(&legitimate, &[]).is_ok());
    }
    #[test]
    fn local_foundry_images_are_unavailable_placeholders_with_diagnostics() {
        let locator = super::super::SourceContentLocator {
            record: "test:aaaaaaaaaaaaaaaa".parse().unwrap(),
            owners: vec![],
            field: "/system/description/value".into(),
        };
        let prepared=super::super::prepare_source_content(locator,"<p>Before<img src='systems/pf2e/local.webp' alt='Portrait'><img src='file:///private/image.png'><img src='https://example.com/chart.png' alt='Chart'>After</p>",super::super::ContentAudience {include_gm:true,include_owner:true,implicit_check_dc:super::super::ContentVisibilityRule::Gm},super::super::ContentVisibilityRule::All,None,None).unwrap();
        assert!(prepared.html.contains("[image unavailable: Portrait]"));
        assert!(!prepared.html.contains("systems/pf2e"));
        assert!(!prepared.html.contains("file:"));
        assert!(prepared.html.contains("https://example.com/chart.png"));
        assert_eq!(
            prepared
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code
                    == super::super::ContentDiagnosticCode::UnavailableImageAsset)
                .count(),
            2
        );
        assert!(validate_prepared_markers(&prepared.html, &prepared.interactions).is_ok());
        assert!(validate_prepared_markers("<img src='systems/pf2e/local.webp'>", &[]).is_err());
    }
}
