//! Typed content correspondence; field names never classify prose semantics.
use super::{ItemSourceView, SourceBackedRecord, SourceContentRole, SourceNodeView};
use crate::source_content::SourceContentLocator;
use atlas_domain::SimilarityRole;

pub fn similarity_content_role(
    record: &SourceBackedRecord,
    locator: &SourceContentLocator,
    content_role: SourceContentRole,
) -> Option<SimilarityRole> {
    let node = record.node_at(&locator.owners)?;
    use SimilarityRole as R;
    use SourceContentRole as C;
    Some(match content_role {
        C::Description if matches!(node, SourceNodeView::JournalPage(_)) => R::JournalText,
        C::Description
            if matches!(
                node,
                SourceNodeView::Item(
                    ItemSourceView::AbilitySource(_)
                        | ItemSourceView::FeatSource(_)
                        | ItemSourceView::EffectSource(_)
                )
            ) =>
        {
            R::Ability
        }
        C::Description => R::Description,
        C::Notes
            if matches!(
                locator.field.as_str(),
                "/system/details/publicNotes" | "/system/details/blurb"
            ) =>
        {
            R::Description
        }
        C::Biography
            if matches!(
                locator.field.as_str(),
                "/system/details/biography/appearance" | "/system/details/biography/backstory"
            ) =>
        {
            R::Description
        }
        C::Notes | C::Biography => R::Notes,
        C::Routine => R::Routine,
        C::Disable => R::Disable,
        C::Reset => R::Reset,
        C::Stealth => R::Stealth,
        C::Prerequisite => R::Prerequisites,
        C::TableResult => R::TableResult,
        C::Name | C::Caption => R::Other,
    })
}
