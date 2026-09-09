use std::num::NonZeroU32;

use crate::{
    ContentChildLocator, FactValue, H8FieldValue, RollTableRecord, TableResult, TableResultEntry,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableRollUnavailableReason {
    FormulaMissing,
    FormulaNull,
    FormulaUnsupported,
    FormulaInvalid,
    ResultsMissing,
    ResultsNull,
    ResultsUnsupported,
    UnsupportedResult,
    RangeMissing,
    RangeNull,
    RangeUnsupported,
    RangeOutOfDomain,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableRollUnavailable {
    pub reason: TableRollUnavailableReason,
    pub child_locator: Option<ContentChildLocator>,
    pub source_ordinal: Option<u32>,
}

impl TableRollUnavailable {
    fn table(reason: TableRollUnavailableReason) -> Self {
        Self {
            reason,
            child_locator: None,
            source_ordinal: None,
        }
    }

    fn child(reason: TableRollUnavailableReason, result: &TableResult) -> Self {
        Self {
            reason,
            child_locator: Some(result.locator.clone()),
            source_ordinal: Some(result.source_ordinal),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ValidatedRollTable<'a> {
    sides: NonZeroU32,
    results: &'a [TableResultEntry],
}

impl<'a> ValidatedRollTable<'a> {
    pub fn sides(self) -> NonZeroU32 {
        self.sides
    }

    pub fn normalized_formula(self) -> String {
        format!("1d{}", self.sides)
    }

    pub fn results_for_total(self, total: u32) -> Option<Vec<&'a TableResult>> {
        if total == 0 || total > self.sides.get() {
            return None;
        }
        let mut matching = Vec::new();
        for entry in self.results {
            let TableResultEntry::Result(result) = entry else {
                return None;
            };
            let range = result.range.as_value().and_then(H8FieldValue::known)?;
            if range.first <= i64::from(total) && i64::from(total) <= range.last {
                matching.push(result.as_ref());
            }
        }
        Some(matching)
    }
}

impl RollTableRecord {
    pub fn validate_rollability(&self) -> Result<ValidatedRollTable<'_>, TableRollUnavailable> {
        let formula = match &self.formula {
            FactValue::Missing => {
                return Err(TableRollUnavailable::table(
                    TableRollUnavailableReason::FormulaMissing,
                ));
            }
            FactValue::Null => {
                return Err(TableRollUnavailable::table(
                    TableRollUnavailableReason::FormulaNull,
                ));
            }
            FactValue::Value(H8FieldValue::Unsupported(_)) => {
                return Err(TableRollUnavailable::table(
                    TableRollUnavailableReason::FormulaUnsupported,
                ));
            }
            FactValue::Value(H8FieldValue::Known(formula)) => formula,
        };
        let sides = parse_formula(formula).ok_or_else(|| {
            TableRollUnavailable::table(TableRollUnavailableReason::FormulaInvalid)
        })?;

        let results = match &self.results {
            FactValue::Missing => {
                return Err(TableRollUnavailable::table(
                    TableRollUnavailableReason::ResultsMissing,
                ));
            }
            FactValue::Null => {
                return Err(TableRollUnavailable::table(
                    TableRollUnavailableReason::ResultsNull,
                ));
            }
            FactValue::Value(H8FieldValue::Unsupported(_)) => {
                return Err(TableRollUnavailable::table(
                    TableRollUnavailableReason::ResultsUnsupported,
                ));
            }
            FactValue::Value(H8FieldValue::Known(results)) => results,
        };

        for entry in results {
            let result = match entry {
                TableResultEntry::Result(result) => result.as_ref(),
                TableResultEntry::Unsupported(result) => {
                    return Err(TableRollUnavailable {
                        reason: TableRollUnavailableReason::UnsupportedResult,
                        child_locator: Some(result.locator.clone()),
                        source_ordinal: Some(result.source_ordinal),
                    });
                }
            };
            let range = match &result.range {
                FactValue::Missing => {
                    return Err(TableRollUnavailable::child(
                        TableRollUnavailableReason::RangeMissing,
                        result,
                    ));
                }
                FactValue::Null => {
                    return Err(TableRollUnavailable::child(
                        TableRollUnavailableReason::RangeNull,
                        result,
                    ));
                }
                FactValue::Value(H8FieldValue::Unsupported(_)) => {
                    return Err(TableRollUnavailable::child(
                        TableRollUnavailableReason::RangeUnsupported,
                        result,
                    ));
                }
                FactValue::Value(H8FieldValue::Known(range)) => range,
            };
            if range.first < 1
                || range.first > range.last
                || u32::try_from(range.last).map_or(true, |last| last > sides.get())
            {
                return Err(TableRollUnavailable::child(
                    TableRollUnavailableReason::RangeOutOfDomain,
                    result,
                ));
            }
        }

        Ok(ValidatedRollTable { sides, results })
    }
}

fn parse_formula(formula: &str) -> Option<NonZeroU32> {
    let sides = formula.strip_prefix("1d")?;
    if sides.is_empty()
        || (sides.len() > 1 && sides.starts_with('0'))
        || !sides.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    NonZeroU32::new(sides.parse().ok()?)
}

#[cfg(test)]
mod tests {
    use atlas_domain::RecordKey;

    use super::{TableRollUnavailableReason, parse_formula};
    use crate::{
        ContentChildIdentity, ContentChildKind, ContentChildLocator, FactValue,
        H8ExactSourceObject, H8FieldValue, H8Identity, H8Provenance, H8RecordSourceMetadata,
        H8UnsupportedChild, OwnedRichContent, RollTableRecord, SourceDocumentId, TableResult,
        TableResultEntry, TableResultRange, TableResultSourceMetadata, TableResultTarget,
        UnsupportedSourceReason, UnsupportedSourceShape, UnsupportedSourceValue,
    };

    #[test]
    fn table_roll_formula_accepts_only_bounded_exact_one_die_syntax() {
        assert_eq!(parse_formula("1d52").map(|value| value.get()), Some(52));
        for rejected in [
            "",
            "d52",
            "2d52",
            "1d0",
            "1d052",
            "1D52",
            "1d52+1",
            " 1d52",
            "1d52 ",
            "1d4294967296",
            "1d@level",
            "1d[52]",
            "(1d52)",
        ] {
            assert_eq!(parse_formula(rejected), None, "{rejected}");
        }
    }

    #[test]
    fn table_roll_requires_every_result_range_before_matching_gaps_and_overlaps() {
        let mut table = fixture_table(vec![result(0, 1, 2), result(1, 2, 2)]);
        let validated = table.validate_rollability().expect("valid table");
        assert_eq!(validated.normalized_formula(), "1d4");
        assert_eq!(
            validated
                .results_for_total(2)
                .expect("in-domain total")
                .iter()
                .map(|result| result.source_ordinal)
                .collect::<Vec<_>>(),
            vec![0, 1]
        );
        assert!(
            validated
                .results_for_total(4)
                .expect("valid gap")
                .is_empty()
        );
        assert_eq!(validated.results_for_total(0), None);
        assert_eq!(validated.results_for_total(5), None);

        table.results = FactValue::Value(H8FieldValue::Known(vec![unsupported_result(3)]));
        let unavailable = table
            .validate_rollability()
            .expect_err("unsupported result");
        assert_eq!(
            unavailable.reason,
            TableRollUnavailableReason::UnsupportedResult
        );
        assert_eq!(unavailable.source_ordinal, Some(3));

        table.results = FactValue::Value(H8FieldValue::Known(vec![result(4, 0, 1)]));
        let unavailable = table.validate_rollability().expect_err("invalid range");
        assert_eq!(
            unavailable.reason,
            TableRollUnavailableReason::RangeOutOfDomain
        );
        assert_eq!(unavailable.source_ordinal, Some(4));

        let mut missing_range = result(5, 1, 1);
        let TableResultEntry::Result(range_result) = &mut missing_range else {
            unreachable!()
        };
        range_result.range = FactValue::Missing;
        table.results = FactValue::Value(H8FieldValue::Known(vec![missing_range]));
        assert_eq!(
            table
                .validate_rollability()
                .expect_err("missing range")
                .reason,
            TableRollUnavailableReason::RangeMissing
        );

        let mut null_range = result(6, 1, 1);
        let TableResultEntry::Result(range_result) = &mut null_range else {
            unreachable!()
        };
        range_result.range = FactValue::Null;
        table.results = FactValue::Value(H8FieldValue::Known(vec![null_range]));
        assert_eq!(
            table.validate_rollability().expect_err("null range").reason,
            TableRollUnavailableReason::RangeNull
        );
    }

    #[test]
    fn table_roll_distinguishes_non_known_formula_and_result_states() {
        let mut table = fixture_table(Vec::new());
        table.formula = FactValue::Missing;
        assert_eq!(
            table
                .validate_rollability()
                .expect_err("missing formula")
                .reason,
            TableRollUnavailableReason::FormulaMissing
        );
        table.formula = FactValue::Null;
        assert_eq!(
            table
                .validate_rollability()
                .expect_err("null formula")
                .reason,
            TableRollUnavailableReason::FormulaNull
        );
        table.formula = FactValue::Value(H8FieldValue::Known("2d4".to_string()));
        assert_eq!(
            table
                .validate_rollability()
                .expect_err("invalid formula")
                .reason,
            TableRollUnavailableReason::FormulaInvalid
        );
        table.formula = FactValue::Value(H8FieldValue::Unsupported(unsupported()));
        assert_eq!(
            table
                .validate_rollability()
                .expect_err("unsupported formula")
                .reason,
            TableRollUnavailableReason::FormulaUnsupported
        );
        table.formula = FactValue::Value(H8FieldValue::Known("1d4".to_string()));
        table.results = FactValue::Missing;
        assert_eq!(
            table
                .validate_rollability()
                .expect_err("missing results")
                .reason,
            TableRollUnavailableReason::ResultsMissing
        );
        table.results = FactValue::Null;
        assert_eq!(
            table
                .validate_rollability()
                .expect_err("null results")
                .reason,
            TableRollUnavailableReason::ResultsNull
        );
        table.results = FactValue::Value(H8FieldValue::Unsupported(unsupported()));
        assert_eq!(
            table
                .validate_rollability()
                .expect_err("unsupported results")
                .reason,
            TableRollUnavailableReason::ResultsUnsupported
        );

        let mut unsupported_range = result(6, 1, 1);
        if let TableResultEntry::Result(result) = &mut unsupported_range {
            result.range = FactValue::Value(H8FieldValue::Unsupported(unsupported()));
        }
        table.results = FactValue::Value(H8FieldValue::Known(vec![unsupported_range]));
        assert_eq!(
            table
                .validate_rollability()
                .expect_err("unsupported range")
                .reason,
            TableRollUnavailableReason::RangeUnsupported
        );
    }

    fn fixture_table(results: Vec<TableResultEntry>) -> RollTableRecord {
        RollTableRecord {
            identity: H8Identity {
                record_key: RecordKey::parse("rollable-tables:test").expect("record key"),
                source_id: SourceDocumentId::new("table-id").expect("source id"),
                name: "Test table".to_string(),
            },
            description: FactValue::Missing,
            results: FactValue::Value(H8FieldValue::Known(results)),
            formula: FactValue::Value(H8FieldValue::Known("1d4".to_string())),
            replacement: FactValue::Missing,
            display_roll: FactValue::Missing,
            image: FactValue::Missing,
            source_metadata: H8RecordSourceMetadata {
                folder: FactValue::Missing,
                sort: FactValue::Missing,
                ownership: FactValue::Missing,
                flags: FactValue::Missing,
                stats: FactValue::Missing,
            },
            content: OwnedRichContent::default(),
            unsupported_fields: Vec::new(),
            provenance: H8Provenance {
                source_path: "fixture.json".to_string(),
                source_contract_version: "h8.v1".to_string(),
                source_system_version: "fixture".to_string(),
                source_upstream_commit: "fixture".to_string(),
            },
        }
    }

    fn result(source_ordinal: u32, first: i64, last: i64) -> TableResultEntry {
        let parent = RecordKey::parse("rollable-tables:test").expect("record key");
        TableResultEntry::Result(Box::new(TableResult {
            locator: ContentChildLocator {
                parent,
                kind: ContentChildKind::TableResult,
                identity: ContentChildIdentity::Stable(
                    SourceDocumentId::new(format!("result-{source_ordinal}")).expect("source id"),
                ),
            },
            source_id: FactValue::Missing,
            source_ordinal,
            result_kind: FactValue::Missing,
            text: FactValue::Missing,
            target: TableResultTarget {
                collection: FactValue::Missing,
                document_id: FactValue::Missing,
            },
            weight: FactValue::Missing,
            range: FactValue::Value(H8FieldValue::Known(TableResultRange { first, last })),
            drawn: FactValue::Missing,
            image: FactValue::Missing,
            source_metadata: TableResultSourceMetadata {
                flags: FactValue::Missing,
            },
            unsupported_fields: Vec::new(),
        }))
    }

    fn unsupported_result(source_ordinal: u32) -> TableResultEntry {
        TableResultEntry::Unsupported(H8UnsupportedChild {
            locator: ContentChildLocator {
                parent: RecordKey::parse("rollable-tables:test").expect("record key"),
                kind: ContentChildKind::TableResult,
                identity: ContentChildIdentity::Unstable { source_ordinal },
            },
            source_id: FactValue::Missing,
            source_ordinal,
            exact_source: H8ExactSourceObject {
                compact_json: "{}".to_string(),
            },
            reason: "unsupported result".to_string(),
        })
    }

    fn unsupported() -> UnsupportedSourceValue {
        UnsupportedSourceValue {
            shape: UnsupportedSourceShape::String,
            value: "\"unsupported\"".to_string(),
            reason: UnsupportedSourceReason::SourceFieldDrift,
        }
    }
}
