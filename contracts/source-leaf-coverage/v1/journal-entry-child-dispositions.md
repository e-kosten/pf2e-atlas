# JournalEntry child and terminal-container dispositions

This is a human-readable companion to the complete `journal-entry.yaml` machine
ledger. It is not an `atlas-source-leaf-coverage/v1` ledger and does not extend
or weaken that schema. The parent ledger owns all 16 representable scalar
declarations: three parent fields plus all 13 supported scalar descendants of
`pages[]`.

The independently observed source inventory has two additional exact terminal
objects. They cannot be represented as scalar ledger declarations, so this
companion accounts for them separately and does not call the 16 machine
declarations complete coverage of the 18 observed JournalEntry paths. The
unchanged generic ledger linter remains clean because these containers are not
machine-leaf prevalence entries; their identities and counts are bound here to
separate typed structural evidence.

The machine ledger is bound only to the accepted parent registry selector
`journalentry--root--top-level--root--root--root`
(`JournalEntry|*|top_level|{}`). Child structure is separately bound to the
registered `JournalEntryPage` variants under parent context
`{document_class: JournalEntry, type_discriminator: "*", relationship_path: JournalEntry.pages}`:

| Embedded registry identity | Variant | Declared count | H8 child owner |
| --- | --- | ---: | --- |
| `journalentrypage--text--embedded--journalentry--root--journalentry-pages` | `text` | 587 observed | `JournalPageEntry::Page(JournalPage)` under `JournalRecord.pages` |
| `journalentrypage--image--embedded--journalentry--root--journalentry-pages` | `image` | 0 declared | same parent-scoped page owner; metadata only |
| `journalentrypage--pdf--embedded--journalentry--root--journalentry-pages` | `pdf` | 0 declared | same parent-scoped page owner; metadata only |
| `journalentrypage--video--embedded--journalentry--root--journalentry-pages` | `video` | 0 declared | same parent-scoped page owner; metadata only |

| Observed path | Inventory identity and prevalence | Exact canonical owner | Failure-sensitive evidence |
| --- | --- | --- | --- |
| `$.pages[].image` | `journal-entry-page-pages-image@4cbdaa37`; 113 records / 587 occurrences; exact `{}` in the pinned inventory | `JournalPage.image_source: H8Fact<H8ExactSourceObject>` owns Missing, Null, `Known({})`, and wrong-shape exact Unsupported. A duplicate fixed member instead makes the smallest page owner `JournalPageEntry::Unsupported(H8UnsupportedChild.exact_source)`, retaining both values. | `source::h8_tests::h8_journal_container_facts_preserve_four_states_and_duplicate_evidence` directly checks the lossless tree, `JournalSource` DTO, and canonical owner; `index_build_input::tests::h8_mixed_artifact_round_trips_journals_tables_and_typed_child_references` checks codec, hydration, and public JSON. |
| `$.pages[].system` | `journal-entry-page-pages-system@4cbdaa37`; 113 records / 587 occurrences; exact `{}` in the pinned inventory | `JournalPage.source_system: H8Fact<H8ExactSourceObject>` owns Missing, Null, `Known({})`, and populated/wrong-shape exact Unsupported. A duplicate fixed member instead makes the smallest page owner `JournalPageEntry::Unsupported(H8UnsupportedChild.exact_source)`, retaining both values. | `source::h8_tests::h8_journal_container_facts_preserve_four_states_and_duplicate_evidence` directly checks the lossless tree, `JournalSource` DTO, and canonical owner; `source::h8_tests::h8_journal_page_system_distinguishes_empty_known_from_populated_unsupported`; `index_build_input::tests::h8_mixed_artifact_round_trips_journals_tables_and_typed_child_references` checks codec, hydration, and public JSON. |
| `$.pages` container | Structural companion only; it is not a v1 leaf or receipt count | `JournalSource.pages: SourcePresence<Vec<SerializedSourceValue>>` then `JournalRecord.pages: H8Fact<Vec<JournalPageEntry>>`; Missing, Null, exact empty array, and populated order remain distinct; wrong shape and duplicate parent members reject the body | `source::h8_tests::h8_parent_child_containers_preserve_presence_and_reject_malformed_shapes` checks lossless tree, DTO, and canonical states; populated codec/hydration remains covered by `index_build_input::tests::h8_mixed_artifact_round_trips_journals_tables_and_typed_child_references` |
| Populated inactive `$.pages[].video` object | Not a separate terminal inventory leaf; its observed scalar descendants remain in the machine ledger | `JournalPage.video: H8Fact<JournalPageVideo>` owns exact Unsupported aggregate evidence while the supported scalar descendants retain their facts | `source::h8_tests::h8_journal_preserves_four_states_exact_objects_and_local_child_failure` |
| Zero-observed image/PDF/video page variants | Synthetic source-shaped contract cases only; no prevalence identity or count is invented | `JournalPage.page_kind` plus the existing typed media/text facts retain supplied metadata; unsupported members remain exact at the page owner | `source::h8_tests::h8_zero_observed_variants_retain_metadata_without_media_or_roll_execution` |

All child scalar descendants remain in `journal-entry.yaml`: `_id`,
`flags.core.sourceId`, `name`, `sort`, `src`, `text.content`, `text.format`,
`text.markdown`, `title.level`, `title.show`, `type`, `video.controls`, and
`video.volume`. The authenticated text-page fixtures serialize populated video
metadata; those scalar declarations retain the values while
`JournalPage.video` retains the inactive variant as exact Unsupported evidence.
`source::h8_tests::h8_journal_preserves_four_states_exact_objects_and_local_child_failure`
proves that no inactive object is promoted to active media semantics.

Image, PDF, and video page variants have zero pinned observations. The
source-shaped cases in
`source::h8_tests::h8_zero_observed_variants_retain_metadata_without_media_or_roll_execution`
are contract tests, not corpus evidence. They prove typed metadata retention and
the inclusive `video.volume` range without authorizing fetch, display, playback,
ranking, or embedding.
