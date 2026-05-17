use atlas_artifact::schema::{records, reference_edges};
use atlas_domain::RecordKey;
use atlas_record::{
    ReferenceGraphMode, ReferenceGraphPolicy, ReferenceVisibilityPolicy, reference_graph_policy,
};

use super::FilterCompiler;
use super::error::FilterCompileError;
use super::sql_render::{REFERENCE_EDGES_ALIAS, aliased_column, record_column};

impl FilterCompiler {
    pub(super) fn links_to_predicate(
        &mut self,
        target: &RecordKey,
    ) -> Result<String, FilterCompileError> {
        Ok(format!(
            "EXISTS (SELECT 1 FROM {table} {alias} WHERE {from_key} = {record_key} AND {to_key} = {} AND {default_predicate})",
            self.text(&target.to_string()),
            table = reference_edges::TABLE.name(),
            alias = REFERENCE_EDGES_ALIAS,
            from_key = aliased_column(
                REFERENCE_EDGES_ALIAS,
                reference_edges::columns::FROM_RECORD_KEY
            ),
            record_key = record_column(records::columns::RECORD_KEY),
            to_key = aliased_column(
                REFERENCE_EDGES_ALIAS,
                reference_edges::columns::TO_RECORD_KEY
            ),
            default_predicate = default_reference_edge_sql_predicate(REFERENCE_EDGES_ALIAS),
        ))
    }

    pub(super) fn linked_from_predicate(
        &mut self,
        source: &RecordKey,
    ) -> Result<String, FilterCompileError> {
        Ok(format!(
            "EXISTS (SELECT 1 FROM {table} {alias} WHERE {from_key} = {} AND {to_key} = {record_key} AND {default_predicate})",
            self.text(&source.to_string()),
            table = reference_edges::TABLE.name(),
            alias = REFERENCE_EDGES_ALIAS,
            from_key = aliased_column(
                REFERENCE_EDGES_ALIAS,
                reference_edges::columns::FROM_RECORD_KEY
            ),
            to_key = aliased_column(
                REFERENCE_EDGES_ALIAS,
                reference_edges::columns::TO_RECORD_KEY
            ),
            record_key = record_column(records::columns::RECORD_KEY),
            default_predicate = default_reference_edge_sql_predicate(REFERENCE_EDGES_ALIAS),
        ))
    }
}

fn default_reference_edge_sql_predicate(alias: &str) -> String {
    reference_edge_sql_predicate(alias, reference_graph_policy(ReferenceGraphMode::Default))
}

fn reference_edge_sql_predicate(alias: &str, policy: ReferenceGraphPolicy) -> String {
    let visibility = aliased_column(alias, reference_edges::columns::VISIBILITY);
    let source_kind = aliased_column(alias, reference_edges::columns::SOURCE_KIND);
    let mut predicates = Vec::new();
    match policy.visibility {
        ReferenceVisibilityPolicy::Only(required) => {
            predicates.push(format!("{visibility} = '{}'", required.as_str()));
        }
        ReferenceVisibilityPolicy::Exclude(excluded) => {
            predicates.push(format!("{visibility} != '{}'", excluded.as_str()));
        }
        ReferenceVisibilityPolicy::Any => {}
    }
    if !policy.excluded_source_kinds.is_empty() {
        let excluded_source_kinds = policy
            .excluded_source_kinds
            .iter()
            .map(|kind| format!("'{}'", kind.as_str()))
            .collect::<Vec<_>>()
            .join(", ");
        predicates.push(format!("{source_kind} NOT IN ({excluded_source_kinds})"));
    }
    predicates.join(" AND ")
}

#[cfg(test)]
mod tests {
    use atlas_record::{
        ContentSourceKind, ContentVisibility, ReferenceEdgeFacts, ReferenceGraphMode,
        reference_edge_matches_mode,
    };

    use super::*;

    #[test]
    fn default_reference_sql_is_lowered_from_shared_policy() {
        let predicate = default_reference_edge_sql_predicate("re");
        assert_eq!(
            predicate,
            "re.visibility = 'public' AND re.source_kind NOT IN ('embedded_item_description', 'embedded_spell_description')"
        );
    }

    #[test]
    fn default_reference_sql_policy_matches_representative_edges() {
        let policy = reference_graph_policy(ReferenceGraphMode::Default);
        for (source_kind, visibility) in [
            (ContentSourceKind::Description, ContentVisibility::Public),
            (
                ContentSourceKind::EmbeddedItemDescription,
                ContentVisibility::Public,
            ),
            (ContentSourceKind::Description, ContentVisibility::Private),
            (
                ContentSourceKind::GeneratedAffliction,
                ContentVisibility::Public,
            ),
        ] {
            let edge = ReferenceEdgeFacts {
                source_kind,
                visibility,
            };
            assert_eq!(
                policy.matches(edge),
                reference_edge_matches_mode(edge, ReferenceGraphMode::Default)
            );
        }
    }
}
