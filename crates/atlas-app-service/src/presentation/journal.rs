use super::*;
use atlas_foundry_model::generated::SourceFromSchemaJournalEntrySchemaPagesEntrySourceFromSchema as JournalPageSource;
use atlas_record::source_record::{SourceContentFormat, SourceContentRole};

pub(crate) fn page_presentation(page: &JournalPageSource) -> JournalPagePresentationView {
    JournalPagePresentationView {
        show_body_heading: fact(
            SourceFieldView::from(&page.title)
                .and_then(|title| (&title.show).into())
                .map(|show| *show),
        ),
        media_notice: match page.r#type.as_value().map(String::as_str) {
            Some("text") => None,
            Some(kind) => Some(format!("Media preview unavailable ({kind}).")),
            None => Some("Page type unavailable.".into()),
        },
    }
}

// The checked source selection distinguishes HTML bodies from unsupported formats.
// A caption, media source, or nonempty markup also gives a page content of its own.
fn empty_text_page(page: &JournalPageSource) -> bool {
    let absent_or_blank = |value: SourceFieldView<'_, &String>| match value {
        SourceFieldView::Missing | SourceFieldView::Null => true,
        SourceFieldView::Value(text) => text.trim().is_empty(),
        _ => false,
    };
    if !absent_or_blank((&page.src).into()) {
        return false;
    }
    let mut empty_body = false;
    for field in SourceNodeView::JournalPage(page).content_selections() {
        match field.role {
            SourceContentRole::Description => {
                empty_body = matches!(
                    field.format,
                    SourceFieldView::Value(SourceContentFormat::Html)
                ) && field
                    .text
                    .value()
                    .is_some_and(|text| text.trim().is_empty());
            }
            SourceContentRole::Caption if !absent_or_blank(field.text) => return false,
            _ => {}
        }
    }
    empty_body
}

pub(super) fn contents(
    record: &SourceBackedRecord,
    fingerprint: &str,
) -> FactView<Vec<JournalContentsEntryView>> {
    struct Page<'a> {
        source: &'a JournalPageSource,
        link: EmbeddedRecordLinkView,
        sort: Option<i64>,
        level: Option<u64>,
    }
    let mut pages = Vec::new();
    let state = record.visit_immediate_journal_pages(|owner, node| {
        let SourceNodeView::JournalPage(page) = node else {
            return;
        };
        let title = page.title.as_value();
        pages.push(Page {
            source: page,
            link: owned_link(record, owner, node, fingerprint),
            sort: page.sort.as_value().and_then(|sort| sort.as_i64()),
            level: title
                .and_then(|title| title.level.as_value()?.as_u64())
                .filter(|&level| level > 0),
        });
    });
    fact(state.map(|()| {
        // Ordinal locators are already bound to original positions. Ties retain source order.
        if pages.iter().all(|page| page.sort.is_some()) {
            pages.sort_by_key(|page| page.sort);
        }
        let mut outline = Vec::with_capacity(pages.len());
        let mut ancestors: Vec<(u64, usize)> = Vec::new();
        let mut pages = pages.into_iter().peekable();
        while let Some(page) = pages.next() {
            let mut parent_index = None;
            if let Some(level) = page.level {
                while ancestors
                    .last()
                    .is_some_and(|&(parent_level, _)| parent_level >= level)
                {
                    ancestors.pop();
                }
                parent_index = ancestors.last().map(|&(_, index)| index);
                ancestors.push((level, outline.len()));
            } else {
                ancestors.clear();
            }
            let has_children = page
                .level
                .zip(pages.peek().and_then(|p| p.level))
                .is_some_and(|(level, next)| next > level);
            outline.push(JournalContentsEntryView {
                link: page.link,
                parent_index,
                is_heading_only: has_children
                    && page.source.title.as_value().and_then(|t| t.show.as_value()) == Some(&true)
                    && empty_text_page(page.source),
            });
        }
        outline
    }))
}
