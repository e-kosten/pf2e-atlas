//! Existing section metadata for source navigation, without bodies or vectors.
use crate::{IndexError, SqliteIndexReader};
use atlas_domain::{RecordKey, SimilarityPartSelector, SourceByteRange, SourcePassageAddress};
use std::collections::BTreeMap;

impl SqliteIndexReader {
    /// Recover complete section addresses from lexical rows and active passage
    /// locations across profiles. Unlabeled lexical-only sections have no address.
    pub fn similarity_part_addresses(
        &self,
        record: &RecordKey,
    ) -> Result<BTreeMap<SimilarityPartSelector, SourcePassageAddress>, IndexError> {
        let select = |table: &str, active: &str| {
            format!(
                "SELECT sp.selector,l.section_json FROM records r JOIN similarity_parts sp USING(record_id) JOIN {table} l ON l.record_id=sp.record_id AND l.owners_json=sp.owners_json AND l.field_path=sp.field_path AND json_extract(l.section_json,'$.section_ordinal')=sp.section_ordinal AND coalesce(json_extract(l.section_json,'$.canonical_text_sha256'),json_extract(l.section_json,'$.source_text_sha256'))=sp.body_hash WHERE r.key=?1 AND r.document_kind<>'Macro'{active}"
            )
        };
        let sql = format!(
            "{} UNION ALL {} LIMIT 100001",
            select("lexical_units", ""),
            select(
                "semantic_locations",
                " AND EXISTS(SELECT 1 FROM semantic_units su WHERE su.location_id=l.location_id)"
            )
        );
        self.count_statement();
        let c = self.native()?;
        let mut statement = c.prepare(&sql)?;
        let mut rows = statement.query([record.to_string()])?;
        let mut sections = BTreeMap::<_, (SourcePassageAddress, Vec<SourceByteRange>)>::new();
        let mut count = 0;
        while let Some(row) = rows.next()? {
            count += 1;
            if count > 100_000 {
                return Err(IndexError::Limit(
                    "section navigation exceeds 100000 source locations".into(),
                ));
            }
            let selector = SimilarityPartSelector(row.get(0)?);
            let address = serde_json::from_str(&row.get::<_, String>(1)?)?;
            add_address(&mut sections, selector, address)?;
        }
        sections
            .into_iter()
            .map(|(selector, (mut address, mut ranges))| {
                ranges.sort_by_key(|range| (range.start, range.end));
                let mut end = 0;
                for range in ranges {
                    if range.start > end || range.start > range.end {
                        return Err(IndexError::Invalid(
                            "section navigation has incomplete passage coverage".into(),
                        ));
                    }
                    end = end.max(range.end);
                }
                *range_mut(&mut address)? = SourceByteRange { start: 0, end };
                Ok((selector, address))
            })
            .collect()
    }
}

fn range_mut(address: &mut SourcePassageAddress) -> Result<&mut SourceByteRange, IndexError> {
    match address {
        SourcePassageAddress::HtmlSection {
            selection_version,
            chunk_bytes,
            ..
        }
        | SourcePassageAddress::PlainSection {
            selection_version,
            chunk_bytes,
            ..
        } if selection_version == atlas_record::source_record::SOURCE_CONTENT_SELECTION_VERSION => {
            Ok(chunk_bytes)
        }
        _ => Err(IndexError::Invalid(
            "section navigation has an incompatible source address".into(),
        )),
    }
}

fn add_address(
    sections: &mut BTreeMap<SimilarityPartSelector, (SourcePassageAddress, Vec<SourceByteRange>)>,
    selector: SimilarityPartSelector,
    mut address: SourcePassageAddress,
) -> Result<(), IndexError> {
    let range = *range_mut(&mut address)?;
    *range_mut(&mut address)? = SourceByteRange { start: 0, end: 0 };
    let (canonical, ranges) = sections
        .entry(selector)
        .or_insert_with(|| (address.clone(), vec![]));
    if *canonical != address {
        return Err(IndexError::Invalid(
            "section navigation source identities disagree".into(),
        ));
    }
    ranges.push(range);
    Ok(())
}
