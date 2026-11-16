//! CommonMark source and soft links to exact Atlas records.
use atlas_domain::RecordKey;
use pulldown_cmark::{Event, Parser, Tag};

#[derive(Clone, Copy)]
pub(crate) struct Markdown<'a>(pub(crate) &'a str);

impl<'a> Markdown<'a> {
    pub(crate) fn record_keys(self) -> impl Iterator<Item = RecordKey> + 'a {
        Parser::new(self.0).filter_map(|event| {
            let Event::Start(Tag::Link { dest_url, .. }) = event else {
                return None;
            };
            let target = dest_url.strip_prefix("atlas-record:")?;
            let key = percent_encoding::percent_decode_str(target)
                .decode_utf8()
                .ok()?;
            RecordKey::parse(&key).ok()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_markdown_links_refer_to_records() {
        let markdown =
            Markdown("[Harm](atlas-record:spells:harm) and `[Heal](atlas-record:spells%3Aheal)`");
        let keys: Vec<_> = markdown.record_keys().map(|key| key.to_string()).collect();
        assert_eq!(keys, ["spells:harm"]);
    }
}
