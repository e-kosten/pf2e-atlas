//! Whole-section navigation is independent of lexical labels and stored vectors.
use crate::{IndexError, SqliteIndexReader};
use atlas_domain::{RecordKey, SimilarityPartSelector, SourcePassageAddress};
use std::collections::BTreeMap;
impl SqliteIndexReader {
    pub fn similarity_part_addresses(
        &self,
        record: &RecordKey,
    ) -> Result<BTreeMap<SimilarityPartSelector, SourcePassageAddress>, IndexError> {
        self.count_statement();
        let mut statement=self.native()?.prepare(&format!("SELECT {} FROM selected_sections ss JOIN records r USING(record_id) WHERE r.key=? AND r.document_kind<>'Macro' ORDER BY ss.section_id LIMIT 100001",crate::selected_sections::COLUMNS))?;
        let mut rows = statement.query([record.to_string()])?;
        let mut output = BTreeMap::new();
        while let Some(row) = rows.next()? {
            if output.len() == 100_000 {
                return Err(IndexError::Limit(
                    "section navigation exceeds100000 sections".into(),
                ));
            }
            let section = crate::selected_sections::SelectedSection::read(row, 0)?;
            if output
                .insert(section.part.selector, section.address)
                .is_some()
            {
                return Err(IndexError::Invalid(
                    "duplicate selected section selector".into(),
                ));
            }
        }
        Ok(output)
    }
}
