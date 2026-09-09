# RollTable result dispositions

This is a human-readable companion to the complete `roll-table.yaml` machine
ledger. It is not an `atlas-source-leaf-coverage/v1` ledger and does not extend
or weaken that schema. The machine ledger owns all 17 representable declarations:
eight parent fields and all nine supported scalar descendants of `results[]`.
There is no extra terminal-container inventory entry for RollTable, so its
machine declarations account for all 17 currently observed paths.

The machine ledger is bound only to the accepted parent registry selector
`rolltable--root--top-level--root--root--root`
(`RollTable|*|top_level|{}`). Child structure is separately bound to the
registered `TableResult` variants under parent context
`{document_class: RollTable, type_discriminator: "*", relationship_path: RollTable.results}`:

| Embedded registry identity | Variant | Declared count | H8 child owner |
| --- | --- | ---: | --- |
| `tableresult--text--embedded--rolltable--root--rolltable-results` | `text` | 388 observed | `TableResultEntry::Result(TableResult)` under `RollTableRecord.results` |
| `tableresult--pack--embedded--rolltable--root--rolltable-results` | `pack` | 1,077 observed | same parent-scoped result owner |
| `tableresult--document--embedded--rolltable--root--rolltable-results` | `document` | 0 declared | same parent-scoped result owner; metadata only |

The result declarations are `_id`, `documentCollection`, `documentId`, `drawn`,
`img`, ordered `range[]`, `text`, `type`, and `weight`. Their exact owners are
the corresponding `TableResult` H8 facts under the parent
`RollTableRecord.results`; result identity stays parent-scoped and independent
from authored order.

| Child or variant case | Exact policy | Failure-sensitive evidence |
| --- | --- | --- |
| Duplicate result `img` | Convert only that child to `UnsupportedResult` at its existing locator, retaining both values in source order and all valid siblings; never select first/last and never reject the table | `source::h8_tests::h8_roll_table_localizes_duplicate_result_image_and_never_selects_a_value` |
| Duplicate parent `$.img` | Reject the RollTable body because the parent fixed field has no unambiguous owner | `source::h8_tests::h8_duplicate_parent_image_and_duplicate_dispatch_fail_closed` |
| `$.results` container | `RollTableSource.results: SourcePresence<Vec<SerializedSourceValue>>` then `RollTableRecord.results: H8Fact<Vec<TableResultEntry>>`; Missing, Null, exact empty array, and populated order stay distinct; wrong shape and duplicate parent members reject the body | `source::h8_tests::h8_parent_child_containers_preserve_presence_and_reject_malformed_shapes` checks lossless tree, DTO, and canonical states; populated codec/hydration remains covered by `index_build_input::tests::h8_mixed_artifact_round_trips_journals_tables_and_typed_child_references` |
| Text and pack results | Retain the exact typed result facts and ordered child identity; pack/document targets require their authored pair and never use label lookup | `source::h8_tests::h8_real_loader_resolves_hero_point_result_to_exact_journal_page`; `index_build_input::tests::h8_mixed_artifact_round_trips_journals_tables_and_typed_child_references` |
| Document result | Zero pinned observations. The source-shaped document case remains a contract test, not a corpus claim; it retains typed metadata without execution | `source::h8_tests::h8_zero_observed_variants_retain_metadata_without_media_or_roll_execution` |

The full H8 evidence reconciliation remains independent from receipt success:
the two machine ledgers contribute 33 v1-admissible scalar identities and must
exactly equal the accepted parent-selector prevalence partition. The Journal
companion separately binds two independently observed exact-empty container
states to typed structural evidence; they are not prevalence entries, machine
leaves, or receipt counts. Machine and companion totals must always be reported
separately.
