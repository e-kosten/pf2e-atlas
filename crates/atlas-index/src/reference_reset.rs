//! Reset only generated reference state inside the rebuild publication transaction.
use crate::IndexError;
use rusqlite::Connection;

pub(crate) fn clear(connection: &Connection) -> Result<(), IndexError> {
    // Model metadata may itself be damaged. Only consider virtual vec0 tables
    // with the executable's native table name shape, never their shadow tables.
    let tables = connection
        .prepare("SELECT name FROM sqlite_schema WHERE type='table' AND sql LIKE 'CREATE VIRTUAL TABLE%USING vec0(%'")?
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    for table in tables {
        let valid_name = table.strip_prefix("native_vectors_").is_some_and(|digest| {
            digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
        });
        if !valid_name {
            return Err(IndexError::Invalid("unexpected native vector table".into()));
        }
        connection.execute_batch(&format!("DROP TABLE {table}"))?;
    }

    // Deleting lexical rows via the ordinary trigger would use their old
    // postings. Replace the FTS index independently of those postings.
    connection.execute_batch(
        "DROP TRIGGER lexical_delete;
         DROP TABLE lexical_fts;
         DELETE FROM core_parts;
         DELETE FROM core_lexical_units;
         DELETE FROM core_semantic_locations;
         DELETE FROM core_roots;
         DELETE FROM trait_references;
         DELETE FROM trait_members;
         DELETE FROM reference_entries;
         DELETE FROM reference_notices;
         DELETE FROM owned_ability_evidence;
         DELETE FROM semantic_units;
         DELETE FROM semantic_locations;
         DELETE FROM lexical_units;
         DELETE FROM selected_sections;
         DELETE FROM search_profiles;
         DELETE FROM pending_embedding_values;
         DELETE FROM embedding_inputs;
         DELETE FROM embedding_models;
         DELETE FROM record_remaster_context;
         DELETE FROM remaster_pairs;
         DELETE FROM verified_aliases;
         DELETE FROM verified_duplicate_records;
         DELETE FROM reference_component_digests;
         DELETE FROM relationship_occurrences;
         DELETE FROM developer_diagnostics;
         DELETE FROM prepared_content;
         DELETE FROM record_bodies;
         DELETE FROM projection_allocations;
         DELETE FROM actor_items;
         DELETE FROM actor_iwr_entries;
         DELETE FROM actor_languages;
         DELETE FROM actor_speeds;
         DELETE FROM actor_senses;
         DELETE FROM actor_projection;
         DELETE FROM spell_damage_kinds;
         DELETE FROM spell_damage_entries;
         DELETE FROM spell_traditions;
         DELETE FROM spell_projection;
         DELETE FROM weapon_projection;
         DELETE FROM armor_projection;
         DELETE FROM shield_projection;
         DELETE FROM physical_projection;
         DELETE FROM ability_projection;
         DELETE FROM heritage_projection;
         DELETE FROM effect_projection;
         DELETE FROM condition_projection;
         DELETE FROM deity_domains;
         DELETE FROM deity_fonts;
         DELETE FROM deity_projection;
         DELETE FROM records;
         DELETE FROM packs;
         DELETE FROM artifact_context;
         UPDATE record_identity SET availability='unavailable' WHERE availability<>'unavailable';
         CREATE VIRTUAL TABLE lexical_fts USING fts5(
           identity_terms,alias_terms,structured_terms,definition_terms,
           content='lexical_units',content_rowid='unit_id',
           tokenize='unicode61 remove_diacritics 2'
         );
         CREATE TRIGGER lexical_delete AFTER DELETE ON lexical_units BEGIN
           INSERT INTO lexical_fts(lexical_fts,rowid,identity_terms,alias_terms,structured_terms,definition_terms)
           VALUES('delete',old.unit_id,old.identity_terms,old.alias_terms,old.structured_terms,old.definition_terms);
         END;",
    )?;
    Ok(())
}
