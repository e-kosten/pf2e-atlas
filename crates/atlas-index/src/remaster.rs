use crate::{IndexError, PublicationSpool, SqliteIndexReader};
use atlas_domain::RecordKey;
use atlas_record::remaster::RecordRemasterContext;
use rusqlite::{Connection, params};
use std::collections::BTreeMap;

pub(crate) fn publish(
    c: &Connection,
    input: &PublicationSpool,
    products: &BTreeMap<RecordKey, i64>,
) -> Result<(), IndexError> {
    let mut rows = BTreeMap::<i64, Vec<String>>::new();
    input.remaster_context.visit(&mut |context| {
        let id = crate::writer::product_id(products, &context.record)?;
        let json = serde_json::to_string(&context)?;
        let values = rows.entry(id).or_default();
        if !values.contains(&json) {
            values.push(json);
        }
        Ok(())
    })?;
    let old=c.prepare("SELECT record_id,ordinal,payload_json FROM record_remaster_context ORDER BY record_id,ordinal")?.query_map([],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,usize>(1)?,r.get::<_,String>(2)?)))?.collect::<Result<Vec<_>,_>>()?;
    let mut expected = BTreeMap::new();
    for (id, values) in rows {
        for (ordinal, value) in values.into_iter().enumerate() {
            expected.insert((id, ordinal), value);
        }
    }
    for (id, ordinal, json) in old {
        if expected.get(&(id, ordinal)) != Some(&json) {
            c.execute(
                "DELETE FROM record_remaster_context WHERE record_id=? AND ordinal=?",
                params![id, ordinal],
            )?;
        } else {
            expected.remove(&(id, ordinal));
        }
    }
    for ((id, ordinal), json) in expected {
        c.execute(
            "INSERT INTO record_remaster_context VALUES(?,?,?)",
            params![id, ordinal, json],
        )?;
    }
    validate(c)
}
pub(crate) fn validate(c: &Connection) -> Result<(), IndexError> {
    let mut query=c.prepare("SELECT r.key,c.payload_json FROM record_remaster_context c JOIN records r USING(record_id)")?;
    for row in query.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))? {
        let (key, json) = row?;
        let context: RecordRemasterContext = serde_json::from_str(&json)?;
        if context.record.to_string() != key
            || context.targets.iter().any(|t| t.name.trim().is_empty())
        {
            return Err(IndexError::Invalid(
                "invalid record remaster context".into(),
            ));
        }
    }
    Ok(())
}
impl SqliteIndexReader {
    pub fn read_remaster_context(
        &self,
        key: &RecordKey,
        owners: &[atlas_domain::OwnedContentLocator],
    ) -> Result<Vec<RecordRemasterContext>, IndexError> {
        let c = self.native()?;
        self.count_statement();
        c.prepare("SELECT c.payload_json FROM record_remaster_context c JOIN records r USING(record_id) WHERE r.key=? AND json_extract(c.payload_json,'$.owners')=json(?) ORDER BY c.ordinal LIMIT 101")?.query_map([key.to_string(),serde_json::to_string(owners)?],|r|r.get::<_,String>(0))?.map(|row|Ok(serde_json::from_str(&row?)?)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atlas_record::remaster::{RemasterRelation, RemasterTarget};
    #[test]
    fn exact_owner_selection_precedes_limit_and_publication_reconciles_context() {
        let mut input = crate::foundation_tests::fixture(false);
        let key = input.records[0].record.key().clone();
        let selected = vec![atlas_domain::OwnedContentLocator {
            collection: "/items".into(),
            identity: atlas_domain::OwnedContentIdentity::Stable("bbbbbbbbbbbbbbbb".into()),
        }];
        let context = RecordRemasterContext {
            record: key.clone(),
            owners: vec![],
            relation: RemasterRelation::FormerName,
            targets: vec![RemasterTarget {
                name: "Old name".into(),
                record: None,
                owners: vec![],
            }],
            journal: None,
            scope: "fixture".into(),
        };
        input.remaster_context = (0..110)
            .map(|i| RecordRemasterContext {
                scope: format!("other-{i}"),
                ..context.clone()
            })
            .chain([RecordRemasterContext {
                owners: selected.clone(),
                ..context.clone()
            }])
            .collect();
        let (_dir, path) = crate::foundation_tests::build(&input);
        let reader = crate::test_support::open_reader(&path).unwrap();
        assert_eq!(reader.read_remaster_context(&key, &[]).unwrap().len(), 101);
        assert_eq!(
            reader.read_remaster_context(&key, &selected).unwrap().len(),
            1
        );
        drop(reader);
        input.remaster_context = vec![RecordRemasterContext {
            targets: vec![RemasterTarget {
                name: "Revised name".into(),
                record: None,
                owners: vec![],
            }],
            ..context
        }];
        crate::test_support::publish_input(&path, &input).unwrap();
        let reader = crate::test_support::open_reader(&path).unwrap();
        assert!(
            reader
                .read_remaster_context(&key, &selected)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            reader.read_remaster_context(&key, &[]).unwrap()[0].targets[0].name,
            "Revised name"
        );
        drop(reader);
        let c = Connection::open(path).unwrap();
        assert!(c.prepare("SELECT * FROM remaster_declarations").is_err());
        assert!(c.prepare("SELECT * FROM verified_aliases").is_ok());
        assert!(c.prepare("SELECT * FROM remaster_pairs").is_ok());
    }
}
