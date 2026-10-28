# Systematic test-quality review

Status: in progress. Baseline: `ce3f6442` (2026-10-28). Prior milestone: [PR141](https://github.com/e-kosten/pf2e-atlas/pull/141).

This ledger tracks the current project-wide review. A broad scan or a passing suite does not mark a file reviewed. Review requires reading its test scenarios and relevant production boundary, identifying the meaningful faults protected, and recording a disposition. Findings are implemented and validated in logical commits; unchanged tests retain a concrete reason. The condition system is excluded, including condition-specific cases in mixed files. Generated source output and data fixtures are assessed through their owning contract tests.

Statuses: **pending** (not yet reviewed), **reviewed** (no worthwhile change identified), **changed** (improvement implemented; validation recorded), **excluded** (condition scope), **mixed** (eligible cases reviewed; condition cases excluded), **deferred** (a consequential issue requiring an explicit decision). Deferred findings do not silently count as completed work.

Completion requires every inventory entry to have a substantive disposition, every accepted improvement implemented and checked, and final required gates/CI on the published candidate. PR boundaries do not end this review.

## Progress

| Subsystem | Files | Pending | Reviewed/changed/mixed | Excluded |
| --- | ---: | ---: | ---: | ---: |
| crates/atlas-app-model | 6 | 0 | 6 | 0 |
| crates/atlas-app-service | 74 | 74 | 0 | 0 |
| crates/atlas-cli | 23 | 23 | 0 | 0 |
| crates/atlas-cli-support | 2 | 0 | 2 | 0 |
| crates/atlas-dev | 7 | 0 | 7 | 0 |
| crates/atlas-dice | 1 | 0 | 1 | 0 |
| crates/atlas-domain | 11 | 0 | 11 | 0 |
| crates/atlas-embedding | 5 | 0 | 5 | 0 |
| crates/atlas-foundry-model | 11 | 0 | 11 | 0 |
| crates/atlas-index | 17 | 0 | 17 | 0 |
| crates/atlas-ingest | 6 | 0 | 6 | 0 |
| crates/atlas-local-state | 7 | 7 | 0 | 0 |
| crates/atlas-record | 18 | 0 | 18 | 0 |
| crates/atlas-runtime | 7 | 0 | 7 | 0 |
| crates/atlas-search | 15 | 15 | 0 | 0 |
| crates/atlas-storage | 3 | 0 | 3 | 0 |
| crates/atlas-tags | 4 | 0 | 4 | 0 |
| crates/atlas-web | 3 | 3 | 0 | 0 |
| dev-tools/dice-prototype | 1 | 0 | 1 | 0 |
| dev-tools/release | 3 | 0 | 3 | 0 |
| dev-tools/source-contracts | 28 | 0 | 26 | 2 |
| research verification | 1 | 0 | 1 | 0 |
| scripts | 8 | 0 | 8 | 0 |
| web/atlas-ui/e2e | 16 | 16 | 0 | 0 |
| web/atlas-ui/src/api | 3 | 3 | 0 | 0 |
| web/atlas-ui/src/app | 11 | 11 | 0 | 0 |
| web/atlas-ui/src/features | 53 | 53 | 0 | 0 |
| web/atlas-ui/src/shared | 92 | 92 | 0 | 0 |

## Review ledger

### crates/atlas-app-model

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [crates/atlas-app-model/src/encounter.rs](../../crates/atlas-app-model/src/encounter.rs) | mixed | Condition-duration test excluded and unchanged. Reviewed HP correction/omitted-vs-null cases, participant revision and durable receipt shape. Corrected HP revision rejection to start from a valid request, removing only expected_revision rather than also omitting request_id. |
| [crates/atlas-app-model/src/filter.rs](../../crates/atlas-app-model/src/filter.rs) | reviewed | Literal operand shapes, omitted editor context, vocabulary/context rejection and unavailable-count status protect distinct wire contracts; retained. |
| [crates/atlas-app-model/src/lib.rs](../../crates/atlas-app-model/src/lib.rs) | reviewed | Freshness compares the complete generated TypeScript directory including missing/extra files; explicit export remains an ignored maintenance command. Aggregate contract machinery retained, with no condition-semantics changes. |
| [crates/atlas-app-model/src/list.rs](../../crates/atlas-app-model/src/list.rs) | changed | Mutation counts now require literal values and field identities instead of number types alone. Retained independent JSON integer and TypeScript-number assertions across summaries, items, exports and import/batch counts. |
| [crates/atlas-app-model/src/record.rs](../../crates/atlas-app-model/src/record.rs) | reviewed | Strict request admission distinguishes retired owners from supported embedded_path so old links cannot silently select the root; retained. |
| [crates/atlas-app-model/src/similarity.rs](../../crates/atlas-app-model/src/similarity.rs) | changed | Similar criteria now checks selected parts and corresponding-candidate mode as well as decay; retained strict method fields, legacy omitted scopes, flattened sections and identity metadata separation. |

### crates/atlas-app-service

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [crates/atlas-app-service/src/content_roll.rs](../../crates/atlas-app-service/src/content_roll.rs) | pending | — |
| [crates/atlas-app-service/src/content_roll_presentation.rs](../../crates/atlas-app-service/src/content_roll_presentation.rs) | pending | — |
| [crates/atlas-app-service/src/contextual_content_tests.rs](../../crates/atlas-app-service/src/contextual_content_tests.rs) | pending | — |
| [crates/atlas-app-service/src/contextual_references.rs](../../crates/atlas-app-service/src/contextual_references.rs) | pending | — |
| [crates/atlas-app-service/src/discovery.rs](../../crates/atlas-app-service/src/discovery.rs) | pending | — |
| [crates/atlas-app-service/src/embedded_reading_tests.rs](../../crates/atlas-app-service/src/embedded_reading_tests.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/casting_reminders.rs](../../crates/atlas-app-service/src/encounters/casting_reminders.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/condition_events.rs](../../crates/atlas-app-service/src/encounters/condition_events.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/difficulty/tests.rs](../../crates/atlas-app-service/src/encounters/difficulty/tests.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/event_execution.rs](../../crates/atlas-app-service/src/encounters/event_execution.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/hp.rs](../../crates/atlas-app-service/src/encounters/hp.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/hp_events.rs](../../crates/atlas-app-service/src/encounters/hp_events.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/hp_tests.rs](../../crates/atlas-app-service/src/encounters/hp_tests.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/initiative_tests.rs](../../crates/atlas-app-service/src/encounters/initiative_tests.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/persistent_damage_tests.rs](../../crates/atlas-app-service/src/encounters/persistent_damage_tests.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/recovery.rs](../../crates/atlas-app-service/src/encounters/recovery.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/recovery_tests.rs](../../crates/atlas-app-service/src/encounters/recovery_tests.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/reference_tests.rs](../../crates/atlas-app-service/src/encounters/reference_tests.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/rules/applicability.rs](../../crates/atlas-app-service/src/encounters/rules/applicability.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/rules/capabilities.rs](../../crates/atlas-app-service/src/encounters/rules/capabilities.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/rules/causes.rs](../../crates/atlas-app-service/src/encounters/rules/causes.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/rules/condition_effects.rs](../../crates/atlas-app-service/src/encounters/rules/condition_effects.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/rules/damage.rs](../../crates/atlas-app-service/src/encounters/rules/damage.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/rules/effective_immunities.rs](../../crates/atlas-app-service/src/encounters/rules/effective_immunities.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/rules/lifecycle.rs](../../crates/atlas-app-service/src/encounters/rules/lifecycle.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/rules/modifiers.rs](../../crates/atlas-app-service/src/encounters/rules/modifiers.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/rules/numerical_tests.rs](../../crates/atlas-app-service/src/encounters/rules/numerical_tests.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/rules/persistent_damage.rs](../../crates/atlas-app-service/src/encounters/rules/persistent_damage.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/rules/projection.rs](../../crates/atlas-app-service/src/encounters/rules/projection.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/rules/senses.rs](../../crates/atlas-app-service/src/encounters/rules/senses.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/rules/source_ability_state.rs](../../crates/atlas-app-service/src/encounters/rules/source_ability_state.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/rules/source_conditions.rs](../../crates/atlas-app-service/src/encounters/rules/source_conditions.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/rules/source_facts.rs](../../crates/atlas-app-service/src/encounters/rules/source_facts.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/rules/spell_damage_tests.rs](../../crates/atlas-app-service/src/encounters/rules/spell_damage_tests.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/spell_tests.rs](../../crates/atlas-app-service/src/encounters/spell_tests.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/strike_context.rs](../../crates/atlas-app-service/src/encounters/strike_context.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/strike_roll.rs](../../crates/atlas-app-service/src/encounters/strike_roll.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/tests.rs](../../crates/atlas-app-service/src/encounters/tests.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/timing_tests.rs](../../crates/atlas-app-service/src/encounters/timing_tests.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/turn_resource_tests.rs](../../crates/atlas-app-service/src/encounters/turn_resource_tests.rs) | pending | — |
| [crates/atlas-app-service/src/encounters/workflow.rs](../../crates/atlas-app-service/src/encounters/workflow.rs) | pending | — |
| [crates/atlas-app-service/src/executor.rs](../../crates/atlas-app-service/src/executor.rs) | pending | — |
| [crates/atlas-app-service/src/executor_abandonment_tests.rs](../../crates/atlas-app-service/src/executor_abandonment_tests.rs) | pending | — |
| [crates/atlas-app-service/src/executor_lifetime_tests.rs](../../crates/atlas-app-service/src/executor_lifetime_tests.rs) | pending | — |
| [crates/atlas-app-service/src/filters.rs](../../crates/atlas-app-service/src/filters.rs) | pending | — |
| [crates/atlas-app-service/src/flat_check.rs](../../crates/atlas-app-service/src/flat_check.rs) | pending | — |
| [crates/atlas-app-service/src/inline_content_tests.rs](../../crates/atlas-app-service/src/inline_content_tests.rs) | pending | — |
| [crates/atlas-app-service/src/party_tests.rs](../../crates/atlas-app-service/src/party_tests.rs) | pending | — |
| [crates/atlas-app-service/src/physical_labels.rs](../../crates/atlas-app-service/src/physical_labels.rs) | pending | — |
| [crates/atlas-app-service/src/prepared_content.rs](../../crates/atlas-app-service/src/prepared_content.rs) | pending | — |
| [crates/atlas-app-service/src/presentation/hazard_readouts.rs](../../crates/atlas-app-service/src/presentation/hazard_readouts.rs) | pending | — |
| [crates/atlas-app-service/src/presentation/spell_changes.rs](../../crates/atlas-app-service/src/presentation/spell_changes.rs) | pending | — |
| [crates/atlas-app-service/src/presentation/spell_context.rs](../../crates/atlas-app-service/src/presentation/spell_context.rs) | pending | — |
| [crates/atlas-app-service/src/presentation/spell_effect/tests.rs](../../crates/atlas-app-service/src/presentation/spell_effect/tests.rs) | pending | — |
| [crates/atlas-app-service/src/presentation/spell_forms.rs](../../crates/atlas-app-service/src/presentation/spell_forms.rs) | pending | — |
| [crates/atlas-app-service/src/presentation/spell_rank.rs](../../crates/atlas-app-service/src/presentation/spell_rank.rs) | pending | — |
| [crates/atlas-app-service/src/presentation/tests.rs](../../crates/atlas-app-service/src/presentation/tests.rs) | pending | — |
| [crates/atlas-app-service/src/projection.rs](../../crates/atlas-app-service/src/projection.rs) | pending | — |
| [crates/atlas-app-service/src/read_execution.rs](../../crates/atlas-app-service/src/read_execution.rs) | pending | — |
| [crates/atlas-app-service/src/related_tests.rs](../../crates/atlas-app-service/src/related_tests.rs) | pending | — |
| [crates/atlas-app-service/src/relationship_discovery_tests.rs](../../crates/atlas-app-service/src/relationship_discovery_tests.rs) | pending | — |
| [crates/atlas-app-service/src/relationship_search.rs](../../crates/atlas-app-service/src/relationship_search.rs) | pending | — |
| [crates/atlas-app-service/src/result_order.rs](../../crates/atlas-app-service/src/result_order.rs) | pending | — |
| [crates/atlas-app-service/src/result_order_tests.rs](../../crates/atlas-app-service/src/result_order_tests.rs) | pending | — |
| [crates/atlas-app-service/src/search_match_sections.rs](../../crates/atlas-app-service/src/search_match_sections.rs) | pending | — |
| [crates/atlas-app-service/src/search_pages/tests.rs](../../crates/atlas-app-service/src/search_pages/tests.rs) | pending | — |
| [crates/atlas-app-service/src/service.rs](../../crates/atlas-app-service/src/service.rs) | pending | — |
| [crates/atlas-app-service/src/service_abandonment_tests.rs](../../crates/atlas-app-service/src/service_abandonment_tests.rs) | pending | — |
| [crates/atlas-app-service/src/service_lifetime_tests.rs](../../crates/atlas-app-service/src/service_lifetime_tests.rs) | pending | — |
| [crates/atlas-app-service/src/similarity.rs](../../crates/atlas-app-service/src/similarity.rs) | pending | — |
| [crates/atlas-app-service/src/source_consumer_tests.rs](../../crates/atlas-app-service/src/source_consumer_tests.rs) | pending | — |
| [crates/atlas-app-service/src/table_roll.rs](../../crates/atlas-app-service/src/table_roll.rs) | pending | — |
| [crates/atlas-app-service/src/traits.rs](../../crates/atlas-app-service/src/traits.rs) | pending | — |
| [crates/atlas-app-service/tests/source_ability_state.rs](../../crates/atlas-app-service/tests/source_ability_state.rs) | pending | — |

### crates/atlas-cli

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [crates/atlas-cli/src/agent_skills/install.rs](../../crates/atlas-cli/src/agent_skills/install.rs) | pending | — |
| [crates/atlas-cli/src/agent_skills/package.rs](../../crates/atlas-cli/src/agent_skills/package.rs) | pending | — |
| [crates/atlas-cli/src/commands/filter_discovery.rs](../../crates/atlas-cli/src/commands/filter_discovery.rs) | pending | — |
| [crates/atlas-cli/src/commands/filters.rs](../../crates/atlas-cli/src/commands/filters.rs) | pending | — |
| [crates/atlas-cli/src/commands/record.rs](../../crates/atlas-cli/src/commands/record.rs) | pending | — |
| [crates/atlas-cli/src/commands/record/batch.rs](../../crates/atlas-cli/src/commands/record/batch.rs) | pending | — |
| [crates/atlas-cli/src/commands/record/presentation/reference_profiles.rs](../../crates/atlas-cli/src/commands/record/presentation/reference_profiles.rs) | pending | — |
| [crates/atlas-cli/src/commands/record/presentation/tests.rs](../../crates/atlas-cli/src/commands/record/presentation/tests.rs) | pending | — |
| [crates/atlas-cli/src/commands/record/terminal.rs](../../crates/atlas-cli/src/commands/record/terminal.rs) | pending | — |
| [crates/atlas-cli/src/commands/result_order.rs](../../crates/atlas-cli/src/commands/result_order.rs) | pending | — |
| [crates/atlas-cli/src/commands/roll.rs](../../crates/atlas-cli/src/commands/roll.rs) | pending | — |
| [crates/atlas-cli/src/commands/similar.rs](../../crates/atlas-cli/src/commands/similar.rs) | pending | — |
| [crates/atlas-cli/src/commands/similar/recovery.rs](../../crates/atlas-cli/src/commands/similar/recovery.rs) | pending | — |
| [crates/atlas-cli/src/commands/traits.rs](../../crates/atlas-cli/src/commands/traits.rs) | pending | — |
| [crates/atlas-cli/src/commands/web.rs](../../crates/atlas-cli/src/commands/web.rs) | pending | — |
| [crates/atlas-cli/tests/agent_skills.rs](../../crates/atlas-cli/tests/agent_skills.rs) | pending | — |
| [crates/atlas-cli/tests/cli_help.rs](../../crates/atlas-cli/tests/cli_help.rs) | pending | — |
| [crates/atlas-cli/tests/completions.rs](../../crates/atlas-cli/tests/completions.rs) | pending | — |
| [crates/atlas-cli/tests/relationship_filters.rs](../../crates/atlas-cli/tests/relationship_filters.rs) | pending | — |
| [crates/atlas-cli/tests/roll.rs](../../crates/atlas-cli/tests/roll.rs) | pending | — |
| [crates/atlas-cli/tests/source_product.rs](../../crates/atlas-cli/tests/source_product.rs) | pending | — |
| [crates/atlas-cli/tests/tags_validate.rs](../../crates/atlas-cli/tests/tags_validate.rs) | pending | — |
| [crates/atlas-cli/tests/version.rs](../../crates/atlas-cli/tests/version.rs) | pending | — |

### crates/atlas-cli-support

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [crates/atlas-cli-support/src/duration.rs](../../crates/atlas-cli-support/src/duration.rs) | changed | Added 999ms/1000ms unit boundary to existing independent precision and long-duration expectations. |
| [crates/atlas-cli-support/src/progress.rs](../../crates/atlas-cli-support/src/progress.rs) | changed | Consolidated event filtering into a literal target/diagnostics table, retaining all three prior scenarios and excluding third-party diagnostics. Kept terminal modes, actual Indicatif rendering, phase lifecycle and width protection; clock-sensitive rendering cases remain, without a flakiness claim. |

### crates/atlas-dev

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [crates/atlas-dev/src/commands/baseline/tests.rs](../../crates/atlas-dev/src/commands/baseline/tests.rs) | changed | Status failure now requires both exact read-only command argument lists, not two matching prefixes that could repeat one command. Retained missing-only private copies, pinning, tamper/symlink rejection, executable selection, validation failures, busy publisher and launcher forwarding. |
| [crates/atlas-dev/src/commands/similarity/tests.rs](../../crates/atlas-dev/src/commands/similarity/tests.rs) | changed | Percentile inputs are deliberately unsorted to detect missing sorting; subset parsing verifies variants and explicit scenario mode. Kept distinct recall denominators/unknowns and frozen-input tampering before workspace access. |
| [crates/atlas-dev/tests/artifact_read_evidence.rs](../../crates/atlas-dev/tests/artifact_read_evidence.rs) | reviewed | Explicit ignored local timing harness compares repeated warm/fresh read counts for five distinct public operations and disclaims cold OS cache. It is evidence collection, not an independent relevance oracle; no worthwhile test rewrite identified. |
| [crates/atlas-dev/tests/baseline_web.rs](../../crates/atlas-dev/tests/baseline_web.rs) | reviewed | Real child execution distinguishes baseline from explicit candidate with spaces, verifies selected path/banner/exit forwarding and preserves private workspace bytes; retained. |
| [crates/atlas-dev/tests/commands.rs](../../crates/atlas-dev/tests/commands.rs) | changed | Inspection now verifies selected record key, source path, snapshot version/content and the complete requested original payload. Kept no-Node source commands, absence of implicit artifact creation and original-source checksum rejection. |
| [crates/atlas-dev/tests/embedding_storage.rs](../../crates/atlas-dev/tests/embedding_storage.rs) | reviewed | Explicit schema-9 full-corpus probe snapshots into disposable storage and compares saved/attributed/vector digests, kNN output, revision/key, reuse and physical reclamation. Distinct migration proof retained; historical corpus prerequisite unavailable in this task. |
| [crates/atlas-dev/tests/retrieval_evidence.rs](../../crates/atlas-dev/tests/retrieval_evidence.rs) | reviewed | Explicit ignored judged-source harness uses real ingest/native retrieval, scoped expected top-three results and source-passage recovery, retaining unjudged output as observations. Private historical judged sample unavailable; no replacement with synthetic relevance claims. |

### crates/atlas-dice

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [crates/atlas-dice/src/evaluation.rs](../../crates/atlas-dice/src/evaluation.rs) | changed | Nested preview now verifies Negate(Add(d8,2))*d4, not only the outer multiply. Retained bounded formula rejection, independent totals, draw-order/filter/tie table, signed arithmetic and magnitude partial order. |

### crates/atlas-domain

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [crates/atlas-domain/src/categories.rs](../../crates/atlas-domain/src/categories.rs) | changed | Literal canonical vocabulary replaces the encoder-derived input oracle and duplicate character-option serialization; alias admission remains separate from strict wire decoding. |
| [crates/atlas-domain/src/core_sections.rs](../../crates/atlas-domain/src/core_sections.rs) | reviewed | Default All behavior and explicit wire names/retired scope rejection protect distinct public contracts; retained. |
| [crates/atlas-domain/src/name_lookup.rs](../../crates/atlas-domain/src/name_lookup.rs) | reviewed | Unicode normalization cases distinguish canonical composition from accent, punctuation and compatibility folding; retained independent literals and non-equivalence checks. |
| [crates/atlas-domain/src/query.rs](../../crates/atlas-domain/src/query.rs) | reviewed | Generated tagged TypeScript structure is an intentional query contract. QueryTruth helpers serve index truth-table test expectations; assess that coupled oracle with atlas-index. |
| [crates/atlas-domain/src/query_relationship.rs](../../crates/atlas-domain/src/query_relationship.rs) | changed | Retained explicit-ID allocation, negation, ambiguity, duplicate IDs and idempotence; added acceptance at 16 relationship leaves beside rejection at 17. |
| [crates/atlas-domain/src/record_key.rs](../../crates/atlas-domain/src/record_key.rs) | changed | Preserved all four malformed separator/empty-part cases in a table; added ASCII/Unicode whitespace admission through parser and serde. Source identity callers separately enforce authored IDs. |
| [crates/atlas-domain/src/result_sort.rs](../../crates/atlas-domain/src/result_sort.rs) | changed | Random seed endpoints now assert literal JSON rather than encoder/decoder agreement; retained strict malformed seeds, unknown fields and attribute uniqueness. |
| [crates/atlas-domain/src/similarity.rs](../../crates/atlas-domain/src/similarity.rs) | reviewed | Omitted options, method-specific defaults and forbidden/null baseline fields are distinct protocol cases; retained. |
| [crates/atlas-domain/src/similarity_component.rs](../../crates/atlas-domain/src/similarity_component.rs) | changed | Replaced role-delegation self-oracle with literal correspondence expectations, retaining identity/role separation and strict identity wire shape. |
| [crates/atlas-domain/src/source_address.rs](../../crates/atlas-domain/src/source_address.rs) | changed | Retained inverted/out-of-range/start-inside-UTF8 rejection; added end-inside-UTF8 rejection and valid tail/empty-at-end ranges. |
| [crates/atlas-domain/src/trait_key.rs](../../crates/atlas-domain/src/trait_key.rs) | reviewed | Literal namespace/identity serialization and malformed separator/whitespace cases distinguish custom from canonical trait identities; retained. |

### crates/atlas-embedding

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [crates/atlas-embedding/src/centroid.rs](../../crates/atlas-embedding/src/centroid.rs) | changed | Added complete output dimensions and named rejection partitions for empty input/chunks, mismatched dimensions, nonfinite and cancelling vectors; retained independent unequal-magnitude chunk expectation. |
| [crates/atlas-embedding/src/document_input.rs](../../crates/atlas-embedding/src/document_input.rs) | reviewed | Complete shared segmentation digest perturbation preserves its abbreviated prefix and must change every profile reuse identity; retained alongside raw-hash separation. Execution-input sensitivity is tested separately in tests.rs. |
| [crates/atlas-embedding/src/model_cache.rs](../../crates/atlas-embedding/src/model_cache.rs) | reviewed | Known SHA-256 vectors, immutable revision paths, verified no-download reuse, failed/partial repairs, retry and coordinated overlapping publication use real files. Same-target versus external-weights concurrency protects distinct cleanup hazards; retained. |
| [crates/atlas-embedding/src/query_sessions.rs](../../crates/atlas-embedding/src/query_sessions.rs) | changed | Retained nonblocking readiness, one shared native lane and real retained two-model sessions. Added dimensions before zip-based native stability comparison so truncated output cannot pass vacuously. |
| [crates/atlas-embedding/src/tests.rs](../../crates/atlas-embedding/src/tests.rs) | changed | Added nonunit/infinite vector rejection, native batch/output lengths and source-section attribution in byte-coverage checks. Kept independent pinned profile/prefix contracts, UTF-8/body/overlap budgets, full segmentation-digest sensitivity, cache repair and reusable occurrence/error protection. |

### crates/atlas-foundry-model

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [crates/atlas-foundry-model/examples/item_generation_probe.rs](../../crates/atlas-foundry-model/examples/item_generation_probe.rs) | reviewed | Raw ordered-source oracle is exercised against real typed Item slices with complete flags, open namespaces, large numeric/string choices and repeated payload fields. Retained separate raw projection and generated-parser implementations. |
| [crates/atlas-foundry-model/examples/predicate_generation_probe.rs](../../crates/atlas-foundry-model/examples/predicate_generation_probe.rs) | changed | Replaced mutation/comparison of JSON trees with literal complete expected projection results for numeric and string operands. Retained comparison against actual generated parsing for partial operator keys. |
| [crates/atlas-foundry-model/src/source_model/retained_rule.rs](../../crates/atlas-foundry-model/src/source_model/retained_rule.rs) | changed | Ignored conditional grants now require the literal authored predicate instead of one element. Retained source immutability, whole-rule rejection, duplicate specialized fields and authoritative BattleForm bracket parsing; no condition event/effect semantics changed. |
| [crates/atlas-foundry-model/tests/equipment_source_generation.rs](../../crates/atlas-foundry-model/tests/equipment_source_generation.rs) | reviewed | Typed common/physical-owner equivalence is backed by independent values for equipped/HP/price/usage. Distinct missing/null/empty state, ordered legacy data, duplicate structural fields, vocabulary/numeric errors and malformed envelopes retained. |
| [crates/atlas-foundry-model/tests/fixtures/source_model/fidelity_tests.rs](../../crates/atlas-foundry-model/tests/fixtures/source_model/fidelity_tests.rs) | reviewed | Real generated parsers cover scalar/array policy and nested IWR forms. Fidelity checker receives independent omission/coercion/number/order/path/diagnostic faults and real mapped/open values; retained all distinct partitions. |
| [crates/atlas-foundry-model/tests/fixtures/source_model/mod.rs](../../crates/atlas-foundry-model/tests/fixtures/source_model/mod.rs) | reviewed | Compiled generic graph protects numeric domains, exact null/collection/source-key values and errors, recursive/shared owners, templates, tuple/union discrimination, authored inputs and constrained open data. These runtime checks complement emitted API/freshness assertions; retained. |
| [crates/atlas-foundry-model/tests/item_source_generation.rs](../../crates/atlas-foundry-model/tests/item_source_generation.rs) | mixed | Condition-family smoke row retained unchanged and excluded. Eligible flag scenarios now require exact repeated object values, array null/number identity and open namespace payloads. Retained independent family/value, missing/null, keyed grants and malformed nested-context checks. |
| [crates/atlas-foundry-model/tests/predicate_source_generation.rs](../../crates/atlas-foundry-model/tests/predicate_source_generation.rs) | reviewed | All operator owners, recursive comparisons/u64 maximum, empty versus constructor array input, extra repeated fields, competing keys and exact nested/tuple diagnostics protect distinct parser contracts; retained. |
| [crates/atlas-foundry-model/tests/snapshot.rs](../../crates/atlas-foundry-model/tests/snapshot.rs) | reviewed | Round trips intentionally verify the full codec contract, backed by independent hydrated values and corrupt version/identity/tag/type/extra-field/truncation checks. Raw-only roots, retained ordered values and deep/escaped nesting cases retained. |
| [crates/atlas-foundry-model/tests/source_admission.rs](../../crates/atlas-foundry-model/tests/source_admission.rs) | changed | Valid admission now independently asserts ordered duplicate unknown values; unaffected domain values and invalid prepared spell references require exact content. Retained strict/admission equivalence, explicit invalid states, whole-rule exclusion, unknown roots and contextual diagnostics. |
| [crates/atlas-foundry-model/tests/source_portfolio_generation.rs](../../crates/atlas-foundry-model/tests/source_portfolio_generation.rs) | mixed | Condition-family smoke row unchanged and excluded. Eligible cases now verify each pinned rule typed variant, exact retained unknown values and surviving generic Item rules. Kept full family/parser API, authored patches/sentinels, document kinds and bounded enum layout. |

### crates/atlas-index

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [crates/atlas-index/src/core_tests.rs](../../crates/atlas-index/src/core_tests.rs) | reviewed | Synthetic vectors distinguish pre-limit Core eligibility, exact identity/prose/owned masks, private and unmodelled fallbacks, trait label-only seeds and absent identity readiness. Real fault-triggered upgrades compare all retained table values, generation, recovery and idempotence; retained. |
| [crates/atlas-index/src/discovery/value_order.rs](../../crates/atlas-index/src/discovery/value_order.rs) | reviewed | Independent ordered duration vocabulary separates timed values from conditional prose while retaining original authored spelling; retained. |
| [crates/atlas-index/src/embedding_values_tests.rs](../../crates/atlas-index/src/embedding_values_tests.rs) | reviewed | Real workspace tests cover exact native reuse, forced pending progress without published mutation, input/model corruption, occurrence attribution, slot retention, migration rollback and trait/document value transitions. Distinct lifecycle cases retained. |
| [crates/atlas-index/src/foundation_tests.rs](../../crates/atlas-index/src/foundation_tests.rs) | changed | Persisted preparation compares every returned record/input field directly rather than using the same private digest on both sides. Physical price projection requires 200 copper across families; renamed its formerly misleading embedded-query title. Retained snapshot/summary separation, publication rollback, marker/sidecar corruption, numeric precision, source scopes, batching and saved-reference preservation. |
| [crates/atlas-index/src/identity.rs](../../crates/atlas-index/src/identity.rs) | reviewed | Exact internal ID survives unavailable imported/removal/reappearance states, separate keys remain separate, and final title/availability are independently checked; retained. |
| [crates/atlas-index/src/input.rs](../../crates/atlas-index/src/input.rs) | changed | Locale and each locale/English catalog digest independently invalidate preparation identity. Existing corpus/query/profile exclusion from shared preparation and full-generation differentiation remain covered. |
| [crates/atlas-index/src/persisted_inputs.rs](../../crates/atlas-index/src/persisted_inputs.rs) | reviewed | Selected-input EXPLAIN contract requires indexed key search and no corpus scan, then checks returned record/content and missing key. Complete reconstructed inputs are independently checked in the strengthened foundation case; retained. |
| [crates/atlas-index/src/profile_tests.rs](../../crates/atlas-index/src/profile_tests.rs) | reviewed | Equal-dimensional profiles retain distinct KNN witnesses/seeds, complete coverage/native budgets, execution identity and segmentation digest isolation. Profile add/remove preserves exact vectors and deletes retired vec tables; retained. |
| [crates/atlas-index/src/query/catalog.rs](../../crates/atlas-index/src/query/catalog.rs) | reviewed | Non-enum choice discovery fails explicitly while generated enum discovery supplies choices; full catalog/schema binding and exact representative choices are exercised in neighboring query tests. Retained. |
| [crates/atlas-index/src/query/cel_relationship.rs](../../crates/atlas-index/src/query/cel_relationship.rs) | changed | Typed relationship parsing now checks exact canonical key, owner collection/identity, field and optional selection facts instead of owner count alone. Strict duplicate/unknown/invalid literal and direction/scope cases retained. |
| [crates/atlas-index/src/query/cel_semantics.rs](../../crates/atlas-index/src/query/cel_semantics.rs) | reviewed | Dependency conformance cases intentionally execute the real CEL interpreter, documenting Boolean/unknown behavior and order-dependent Exists errors independently of Atlas SQL. Retained distinct SQL truth-table coverage. |
| [crates/atlas-index/src/query/corpus_tests.rs](../../crates/atlas-index/src/query/corpus_tests.rs) | mixed | Reviewed independent raw JSON oracle, source-owner alignment, numeric/state comparisons, emitted trait sets and descriptor SQL execution. Condition projection branch excluded and unchanged. Historical pinned full-corpus counts/prerequisite remain explicit; not claimed rerun. |
| [crates/atlas-index/src/query/tests.rs](../../crates/atlas-index/src/query/tests.rs) | mixed | Complete five-state identity/count partition replaces a vacuous all predicate; paged vocabulary requires the actual flying value. Retained exact SQL root sets for unknown logic, same-child Exists, facets/counterfactuals, scopes, numeric units, validation and parameterization. Condition catalog entries excluded and unchanged. |
| [crates/atlas-index/src/requested_config.rs](../../crates/atlas-index/src/requested_config.rs) | reviewed | Real transactions prove dropped writes roll back, explicit commit persists, rejected replacement cannot overwrite committed intent and unknown/corrupt payloads fail rather than disappear; retained. |
| [crates/atlas-index/src/result_order.rs](../../crates/atlas-index/src/result_order.rs) | reviewed | Literal fixed ordering vectors cover zero, ordinary and maximum random seeds with reverse input; database ordering behavior is reviewed again through retrieval result-order cases. Retained. |
| [crates/atlas-index/src/traits.rs](../../crates/atlas-index/src/traits.rs) | reviewed | Exact trait seeds, independent changed centroids/definitions, complete source coverage, metadata-only versus configured profile requirements, atomic publication, corruption, reference occurrence cardinality, source ownership and source-free reads retain distinct protection. |
| [crates/atlas-index/src/vector_allocation.rs](../../crates/atlas-index/src/vector_allocation.rs) | changed | Each filtered KNN query must return its full requested k before comparing distances, rejecting empty/truncated identical-before/after results. Retained exact vector-byte preservation, real block reclamation, eligibility and transaction rollback/commit checks. |

### crates/atlas-ingest

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [crates/atlas-ingest/src/audit.rs](../../crates/atlas-ingest/src/audit.rs) | reviewed | Retained dynamic damage-key wildcard versus fixed AC member normalization. The public path-audit command is also exercised by the CLI integration inventory, which remains a separate review obligation. |
| [crates/atlas-ingest/src/build_tests.rs](../../crates/atlas-ingest/src/build_tests.rs) | mixed | Incremental reference lifecycle now reads published occurrence destinations and display names after missing, added, duplicate-name, unique-again and renamed states, beyond reuse counters. Retained atomic failures, captures, locale/fingerprint changes, source-free trait output and native reuse/attribution checks. Condition-related alias fixture assertions excluded and unchanged. |
| [crates/atlas-ingest/src/source/localization/tests.rs](../../crates/atlas-ingest/src/source/localization/tests.rs) | reviewed | Independent literal fallback/override expectations, manifest order, catalog bytes/path identity, captured transient changes and invalid paths retain distinct protection. Historical complete-source Localize census retains its explicit pinned-source prerequisite. |
| [crates/atlas-ingest/src/source_compatibility.rs](../../crates/atlas-ingest/src/source_compatibility.rs) | reviewed | System identity and conventional manifest filename both enforce the model version, custom module versions remain independent, and explicit manifest override is checked; retained. |
| [crates/atlas-ingest/src/source_enrichment/tests.rs](../../crates/atlas-ingest/src/source_enrichment/tests.rs) | reviewed | Exact bytes, snapshot, provenance, diagnostics and report partitions cross the consuming handoff. Collision reservation, unavailable-name exclusion in both orders, sparse outcomes and action-case collision invalidation remain distinct; retained. |
| [crates/atlas-ingest/tests/typed_source_loading.rs](../../crates/atlas-ingest/tests/typed_source_loading.rs) | mixed | Reviewed document-kind dispatch, exact source bytes/hash/provenance, invalid/partial members, embedded identities, quarantine classes, stable discovery and read failures. Condition-family dispatch fixture row excluded and unchanged; all generic loading protection retained. |

### crates/atlas-local-state

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [crates/atlas-local-state/src/atomic_action.rs](../../crates/atlas-local-state/src/atomic_action.rs) | pending | — |
| [crates/atlas-local-state/src/encounters/receipt_codec.rs](../../crates/atlas-local-state/src/encounters/receipt_codec.rs) | pending | — |
| [crates/atlas-local-state/src/encounters/tests.rs](../../crates/atlas-local-state/src/encounters/tests.rs) | pending | — |
| [crates/atlas-local-state/src/encounters/turn_resources.rs](../../crates/atlas-local-state/src/encounters/turn_resources.rs) | pending | — |
| [crates/atlas-local-state/src/parties/tests.rs](../../crates/atlas-local-state/src/parties/tests.rs) | pending | — |
| [crates/atlas-local-state/src/saved_lists/tests.rs](../../crates/atlas-local-state/src/saved_lists/tests.rs) | pending | — |
| [crates/atlas-local-state/src/test_support.rs](../../crates/atlas-local-state/src/test_support.rs) | pending | — |

### crates/atlas-record

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [crates/atlas-record/src/source_content/contextual_formula/tests.rs](../../crates/atlas-record/src/source_content/contextual_formula/tests.rs) | mixed | Reviewed bounded arithmetic, rounding, missing context, healing labels and rejection partitions. Condition-specific badge/prose cases excluded and unchanged; other literal expected prose and formulas retained. |
| [crates/atlas-record/src/source_content/inclusion.rs](../../crates/atlas-record/src/source_content/inclusion.rs) | changed | Action heading checks now require the exact retained or generated linked title and namespace, so a heading with only glyph/qualifiers cannot pass. Retained isolated-vs-inline, multiple-inclusion and unheaded cases. |
| [crates/atlas-record/src/source_content/literal_damage.rs](../../crates/atlas-record/src/source_content/literal_damage.rs) | changed | Assert complete component cardinality, each formula and typed damage/healing kind; added material, precision, splash and combined healing/damage label cases with literal dice structure. All prior unsupported/partial formula cases retained. |
| [crates/atlas-record/src/source_content/literal_roll.rs](../../crates/atlas-record/src/source_content/literal_roll.rs) | changed | All three roll modes now require the literal two-dice-plus-three preview tree, beyond copied formula text and a dice flag. Pool/caption handling and every unsupported input retained. |
| [crates/atlas-record/src/source_content/marker_validation.rs](../../crates/atlas-record/src/source_content/marker_validation.rs) | changed | Safe cached markup uses an independent canonical literal including safe-link rel attributes, rather than deriving its input from the sanitizer under test. Missing/duplicate/kind, active markup and unavailable-image partitions retained. |
| [crates/atlas-record/src/source_content/parser_tests.rs](../../crates/atlas-record/src/source_content/parser_tests.rs) | reviewed | Retained exact rendered text plus source target/kind/label evidence, comma-bearing targets, tables/templates and nested/missing/recursive localization. Condition UUID is opaque parser input, with no condition semantics reviewed. |
| [crates/atlas-record/src/source_content/tests.rs](../../crates/atlas-record/src/source_content/tests.rs) | reviewed | Retained reference-binding field mutations, href tampering, source identity, complete visibility evidence, sanitization, localization, nested references, glyph and formula boundaries, checked action navigation and independent literal check/area prose. Opaque condition targets do not exercise condition mechanics. |
| [crates/atlas-record/src/source_content/text_projection.rs](../../crates/atlas-record/src/source_content/text_projection.rs) | reviewed | Canonical words, absent renderer decorations/URLs and preserved block breaks protect the intended undecorated projection without freezing library whitespace; retained. |
| [crates/atlas-record/src/source_content/trait_references.rs](../../crates/atlas-record/src/source_content/trait_references.rs) | reviewed | Repeated occurrences keep exact trait/owner attribution and distinct paths; hidden/unknown visibility, shared-key ambiguity, duplicate same-identity mapping and script exclusion remain distinct protections. |
| [crates/atlas-record/src/source_record/actor_presentation.rs](../../crates/atlas-record/src/source_record/actor_presentation.rs) | changed | Skill visitor must enumerate exactly arcana, preventing its callback-local assertions from passing when no skill is visited. Retained literal ability/special-skill values, adjustment states and duplicate retained-member rejection. |
| [crates/atlas-record/src/source_record/actor_query_tests.rs](../../crates/atlas-record/src/source_record/actor_query_tests.rs) | changed | Applicable populated fields must be Value rather than any non-NotApplicable state; speed entries require exact fly/swim values in authored order. Retained null/invalid/default ancestry, IWR and zero/negative baseline cases. Historical corpus check keeps its explicit source prerequisite. |
| [crates/atlas-record/src/source_record/core_sections_tests.rs](../../crates/atlas-record/src/source_record/core_sections_tests.rs) | mixed | Reviewed exact family/field/owner membership, nested source exclusions, visibility, unavailable-core fallback and foreign identity rejection. Condition family row excluded and unchanged. |
| [crates/atlas-record/src/source_record/declared_grant_tests.rs](../../crates/atlas-record/src/source_record/declared_grant_tests.rs) | reviewed | Static grant targets, owned attribution, occurrence order, applied-vs-declared distinction, rejected/dynamic/ignored rules and escaped ABC feature keys have independent exact expectations; retained. No condition application mechanics. |
| [crates/atlas-record/src/source_record/item_query_tests.rs](../../crates/atlas-record/src/source_record/item_query_tests.rs) | mixed | Condition cases excluded and unchanged. Root/embedded equivalence now has independent rank/tradition/focus/key-order expectations; raw JSON bytes preserve actual z-before-a source order instead of json map sorting. Ammunition price requires 400 copper, not presence alone. Retained units, fractional price, unsafe numbers and availability cases. |
| [crates/atlas-record/src/source_record/search_selection.rs](../../crates/atlas-record/src/source_record/search_selection.rs) | changed | Passage recovery now independently rejects stale canonical text digest and selection version, matching the existing identity contract. Preserved duplicate heading landing, nested markup, UTF8, source address, partitions and historical corpus navigation cases. |
| [crates/atlas-record/src/source_record/similarity_tests.rs](../../crates/atlas-record/src/source_record/similarity_tests.rs) | reviewed | Typed root/embedded families, public/private biography and empty fields, journal/table roles, hazards and prerequisites use literal role/address expectations; retained. |
| [crates/atlas-record/src/source_record/tests.rs](../../crates/atlas-record/src/source_record/tests.rs) | mixed | Reviewed generic source dispatch, traversal, checked snapshots, prepared visibility, relative/qualified/colliding references, source states, grants/provenance and action navigation. Existing independent outcome expectations retained. Condition family entries excluded and unchanged. |
| [crates/atlas-record/src/source_record/trait_memberships.rs](../../crates/atlas-record/src/source_record/trait_memberships.rs) | changed | Require complete three/five membership cardinality as well as exact owned trait identities, labels, nested paths and duplicate-ID positions; surplus memberships can no longer pass find-only checks. |

### crates/atlas-runtime

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [crates/atlas-runtime/src/error.rs](../../crates/atlas-runtime/src/error.rs) | reviewed | Literal error kind/message and target identity protect stable runtime diagnostics; retained. Typed storage/search conversions are also exercised at runtime transport boundaries. |
| [crates/atlas-runtime/src/profile_assets.rs](../../crates/atlas-runtime/src/profile_assets.rs) | changed | Added deterministic file replacement inside verification: stale success is rejected and the next access must verify again. Retained missing-package no-scan, unchanged positive/negative caching, normal replacement recovery and stored-profile scope checks. |
| [crates/atlas-runtime/src/setup_clean.rs](../../crates/atlas-runtime/src/setup_clean.rs) | changed | Cleanup now requires both requested targets and real directory survival/removal in check/execute modes, beyond report status. Retained containment, sidecar, relative nonexistent suffix, symlink and broken-alias protection. |
| [crates/atlas-runtime/src/source_acquisition.rs](../../crates/atlas-runtime/src/source_acquisition.rs) | reviewed | Real local Git repositories prove exact detached pin, offline cache reuse, dirty/wrong revision preservation and absent partial publication after failure; retained without network mocks. |
| [crates/atlas-runtime/src/source_configuration.rs](../../crates/atlas-runtime/src/source_configuration.rs) | changed | Adding Gemma must preserve exactly the sorted Base/Gemma profile set, instead of cardinality plus Base presence. Retained durable locale/model intent, source override precedence, check-mode no-creation and incompatible-file preservation. |
| [crates/atlas-runtime/src/tests.rs](../../crates/atlas-runtime/src/tests.rs) | reviewed | Retained query/lexical/historical upgrade matrix with exact saved rows, vectors, pending progress, revisions and no inference/source access. Setup freshness, failed build/config/profile preservation, absent assets, native retained query session and source compatibility have distinct outcomes. |
| [crates/atlas-runtime/src/workspace.rs](../../crates/atlas-runtime/src/workspace.rs) | mixed | Condition application/system migration cases excluded and unchanged. Fresh migration failure now proves no user tables or schema version remain before recovery; historical baseline upgrade checks complete saved-list identity and encounter key/name. Other migration/history/vector/receipt preservation and idempotent reopening checks retained. |

### crates/atlas-search

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [crates/atlas-search/src/configurable_similarity/tests.rs](../../crates/atlas-search/src/configurable_similarity/tests.rs) | pending | — |
| [crates/atlas-search/src/core_search_tests.rs](../../crates/atlas-search/src/core_search_tests.rs) | pending | — |
| [crates/atlas-search/src/error/tests.rs](../../crates/atlas-search/src/error/tests.rs) | pending | — |
| [crates/atlas-search/src/fusion.rs](../../crates/atlas-search/src/fusion.rs) | pending | — |
| [crates/atlas-search/src/page.rs](../../crates/atlas-search/src/page.rs) | pending | — |
| [crates/atlas-search/src/reference_resolution.rs](../../crates/atlas-search/src/reference_resolution.rs) | pending | — |
| [crates/atlas-search/src/relationship_count_tests.rs](../../crates/atlas-search/src/relationship_count_tests.rs) | pending | — |
| [crates/atlas-search/src/relationship_tests.rs](../../crates/atlas-search/src/relationship_tests.rs) | pending | — |
| [crates/atlas-search/src/result_order_tests.rs](../../crates/atlas-search/src/result_order_tests.rs) | pending | — |
| [crates/atlas-search/src/similarity_evaluation/tests.rs](../../crates/atlas-search/src/similarity_evaluation/tests.rs) | pending | — |
| [crates/atlas-search/src/similarity_scoring/tests.rs](../../crates/atlas-search/src/similarity_scoring/tests.rs) | pending | — |
| [crates/atlas-search/src/tests.rs](../../crates/atlas-search/src/tests.rs) | pending | — |
| [crates/atlas-search/src/text.rs](../../crates/atlas-search/src/text.rs) | pending | — |
| [crates/atlas-search/src/text_inference_tests.rs](../../crates/atlas-search/src/text_inference_tests.rs) | pending | — |
| [crates/atlas-search/src/traits.rs](../../crates/atlas-search/src/traits.rs) | pending | — |

### crates/atlas-storage

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [crates/atlas-storage/src/reclamation.rs](../../crates/atlas-storage/src/reclamation.rs) | reviewed | Real SQLite free-page fixture covers no-op, capacity failure on each filesystem, unknown capacity and exact required capacity; physical preservation is independently tested in storage tests.rs. Retained. |
| [crates/atlas-storage/src/tests.rs](../../crates/atlas-storage/src/tests.rs) | changed | Added rolled-back ledger count, persisted publication title, typed backfill bytes and a post-backfill sentinel proving current-schema startup does not reexecute. Retained migration history, admission, WAL, snapshot/seed, rollback and physical reclamation cases. |
| [crates/atlas-storage/tests/process.rs](../../crates/atlas-storage/tests/process.rs) | changed | Preserved real separate-process locking/abrupt-exit scenario and now verifies reopened committed title and publication revision after recovery. |

### crates/atlas-tags

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [crates/atlas-tags/src/id.rs](../../crates/atlas-tags/src/id.rs) | changed | Preserved namespace/case/hyphen regressions; added empty/empty-segment/leading-digit rejection and YAML admission parity, plus literal valid serialization. |
| [crates/atlas-tags/src/validation.rs](../../crates/atlas-tags/src/validation.rs) | changed | Merged catalog now asserts complete membership and the returned definition identity/label. Kept separate YAML shape, retired metric, duplicate assignment/catalog and display-group validity cases. |
| [crates/atlas-tags/tests/corpus.rs](../../crates/atlas-tags/tests/corpus.rs) | changed | Consolidated equivalent success cases across three public loaders with complete literal catalog membership, record/tag mapping and suggestion identity. Retained cross-file duplicate, unknown assigned tag and missing-directory failures. |
| [crates/atlas-tags/tests/fixtures.rs](../../crates/atlas-tags/tests/fixtures.rs) | changed | Replaced count/nonempty-only fixture assertions with exact assigned and reviewed-empty record identities and all suggestion-to-trigger mappings; catalog order and research flags remain protected. |

### crates/atlas-web

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [crates/atlas-web/src/assets.rs](../../crates/atlas-web/src/assets.rs) | pending | — |
| [crates/atlas-web/src/tests.rs](../../crates/atlas-web/src/tests.rs) | pending | — |
| [crates/atlas-web/src/tests/read_abandonment_tests.rs](../../crates/atlas-web/src/tests/read_abandonment_tests.rs) | pending | — |

### dev-tools/dice-prototype

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [dev-tools/dice-prototype/server/src/main.rs](../../dev-tools/dice-prototype/server/src/main.rs) | reviewed | Exploratory Caith/dicexp comparison is separate from production atlas-dice. Retained mixed-pool arithmetic invariant, bounded high/low/tie seed family, request replay/conflict, forbidden formula and plain breakdown tests; all six passed. |

### dev-tools/release

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [dev-tools/release/tests/nightly.test.ts](../../dev-tools/release/tests/nightly.test.ts) | reviewed | PR141 literal version/lockfile checks and required-command existence guards retained. Scheduled exact-source skip, immutable dated-release retry, validation/install/version failures and no-channel-advance assertions protect separate publication hazards. |
| [dev-tools/release/tests/release.test.ts](../../dev-tools/release/tests/release.test.ts) | changed | Manifest expectations now use literal supported platform/package metadata and the complete checksum list, including both installers and notices. Dependency ordering requires the complete literal Rust notice block. Retained binary hashing, unknown metadata, Unicode serialization, strict CLI exits, archive inspection and runtime-data rejection. |
| [dev-tools/release/tests/smoke-packaged-web.test.ts](../../dev-tools/release/tests/smoke-packaged-web.test.ts) | reviewed | Real child/HTTP fixtures distinguish missing assets, wrong bytes/MIME, early exit and startup timeout, with actual temporary-directory cleanup. Native packaging checks remain separate. Retained all six modes. |

### dev-tools/source-contracts

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [dev-tools/source-contracts/tests/action-definitions.test.ts](../../dev-tools/source-contracts/tests/action-definitions.test.ts) | changed | Complete literal action object now includes slug and source line, retaining export/constructor/variant resolution, deduplication and proof that upstream source is not executed. |
| [dev-tools/source-contracts/tests/collection-generation.test.ts](../../dev-tools/source-contracts/tests/collection-generation.test.ts) | changed | Removed the standalone test of stock JSON.stringify, which invoked no repository code. Retained nullable/undefined collection generation, ordinary-array/map constraints, null-only slots, unsupported indices, bounded shared signatures and recursive unsupported descendants; actual collection values are checked in the Rust generic fixture. |
| [dev-tools/source-contracts/tests/compare-documents.test.ts](../../dev-tools/source-contracts/tests/compare-documents.test.ts) | reviewed | Retained untouched numeric/duplicate payload sampling, accepted-versus-lossy outcomes, normalized rejection grouping with exact contexts, recovered/regressed identities and stale-report invalidation/path isolation. |
| [dev-tools/source-contracts/tests/compare-equipment.test.ts](../../dev-tools/source-contracts/tests/compare-equipment.test.ts) | changed | Large-integer and presence differences now use independent candidates and require exact reported contexts/results. Previously the presence case retained the numeric difference and could pass with presence comparison broken. |
| [dev-tools/source-contracts/tests/compare-traits.test.ts](../../dev-tools/source-contracts/tests/compare-traits.test.ts) | reviewed | Literal identity/membership/description deltas, missing/empty distinctions, source-line insensitivity, conflicting mappings, literal authored fences, exact artifact hashes and CLI output preservation cover distinct contracts; retained. |
| [dev-tools/source-contracts/tests/condition-catalog.test.ts](../../dev-tools/source-contracts/tests/condition-catalog.test.ts) | excluded | Condition catalog extraction, modifier predicates, grants/overrides and HP-loss ownership are excluded by the user. |
| [dev-tools/source-contracts/tests/condition-extraction.test.ts](../../dev-tools/source-contracts/tests/condition-extraction.test.ts) | excluded | Condition expression/trait rule extraction is excluded by the user. |
| [dev-tools/source-contracts/tests/document-inputs.test.ts](../../dev-tools/source-contracts/tests/document-inputs.test.ts) | reviewed | Declaration-preserving bounded projections, sentinel/area/statistic forms, nested overlay/fixed-heightening patches and drift refusal retained. Real generated Rust probes independently verify acceptance, rejection and value fidelity. |
| [dev-tools/source-contracts/tests/document-traits.test.ts](../../dev-tools/source-contracts/tests/document-traits.test.ts) | reviewed | Open trait-string arrays remain separate from closed small vocabularies; nonstring drift and never-element discovery have distinct outcomes. Public generated types and parser admission guards retained. |
| [dev-tools/source-contracts/tests/extract.test.ts](../../dev-tools/source-contracts/tests/extract.test.ts) | reviewed | Real CLI checks persisted incomplete reports, strict nonzero exit, complete retry, registered rule identities and dependency-versus-source digest separation; retained. |
| [dev-tools/source-contracts/tests/generated-output.test.ts](../../dev-tools/source-contracts/tests/generated-output.test.ts) | reviewed | Whole output-set freshness, retired generated file removal, unmanaged-file preservation, missing/directory targets and pre-acquisition source/output overlap refusal retained. |
| [dev-tools/source-contracts/tests/numeric-generation.test.ts](../../dev-tools/source-contracts/tests/numeric-generation.test.ts) | reviewed | Number/string key domains, retained nonnumeric keys, recursive anchors, null-only fields/unusual tokens and shared imports protect generated public shapes. Numeric-key fixture is independently checked against pinned TypeScript; retained. |
| [dev-tools/source-contracts/tests/object-patch.test.ts](../../dev-tools/source-contracts/tests/object-patch.test.ts) | reviewed | Original declarations, replacement collections, merged partial union fields and cycle/collision refusal retained. Generated Rust probe checks literal acceptance/fidelity and exact nested failure path. |
| [dev-tools/source-contracts/tests/open-generation.test.ts](../../dev-tools/source-contracts/tests/open-generation.test.ts) | reviewed | Unknown/object/non-nullish constraints, canonical names across union ordering, indexed typed owners, null tuple guards, unsupported/dangling domains and forbidden-member shape identity retained. |
| [dev-tools/source-contracts/tests/pinned-generation.integration.ts](../../dev-tools/source-contracts/tests/pinned-generation.integration.ts) | mixed | Condition facts case excluded from review and unchanged. Whole generated output freshness, one shared physical owner, complete document/family/rule inventory, baseline identities, emitted field/literal drift, trait-array policy, strict unsupported constructs and source-backed actions retained. |
| [dev-tools/source-contracts/tests/pinned-source.test.ts](../../dev-tools/source-contracts/tests/pinned-source.test.ts) | mixed | Condition-pack invalidation case excluded and unchanged. Reviewed independent declaration digest/count/version admission, isolated local export, warm-cache corruption refusal and real exact-commit Git acquisition/reuse. |
| [dev-tools/source-contracts/tests/portfolio-comparison.test.ts](../../dev-tools/source-contracts/tests/portfolio-comparison.test.ts) | changed | Admission baseline now independently rejects source and corpus identity changes as well as count/outcome changes. Kept retained/partial/raw/fidelity outcome partitions, portfolio rejection-context hashing and non-Actor raw sampling. |
| [dev-tools/source-contracts/tests/recursive-generation.test.ts](../../dev-tools/source-contracts/tests/recursive-generation.test.ts) | reviewed | Retained recursive inline-versus-collection boxing, unanchored alias rejection, tuple arity/nullability, union discrimination, exact source-key mapping/collisions, indexed intersections, compiled-fixture freshness and cross-module canonical owner/import contracts. |
| [dev-tools/source-contracts/tests/rule-inputs.test.ts](../../dev-tools/source-contracts/tests/rule-inputs.test.ts) | reviewed | Ordinary/Strict/Set array policies, shared declaration propagation through recursive collections, provenance drift, raw rule sampling and per-occurrence recovery/loss categories are distinct regression cases; retained. |
| [dev-tools/source-contracts/tests/rule-value-inputs.test.ts](../../dev-tools/source-contracts/tests/rule-value-inputs.test.ts) | reviewed | Bounded constructor projections preserve original schema/provenance, BattleForm-only widening, direct Strike closed types, drift refusal and open scalar-trait policy; retained. |
| [dev-tools/source-contracts/tests/sample-equipment.test.ts](../../dev-tools/source-contracts/tests/sample-equipment.test.ts) | reviewed | Exact raw numeric/duplicate spans, embedded ordinal/context, malformed context/JSON, manifest folder skipping/missing packs and PF2e-versus-SF2e pack isolation retained. |
| [dev-tools/source-contracts/tests/sample-items.test.ts](../../dev-tools/source-contracts/tests/sample-items.test.ts) | reviewed | Recursive Item-only traversal retains exact nested paths and raw duplicate/large-number payloads while rejecting actual duplicate context and malformed roots; retained. |
| [dev-tools/source-contracts/tests/sample-predicates.test.ts](../../dev-tools/source-contracts/tests/sample-predicates.test.ts) | reviewed | Constructor-versus-array shapes, raw large numbers, declared extra predicate contexts, malformed values and repeated authored-member occurrences remain distinct; retained. |
| [dev-tools/source-contracts/tests/source-identity.test.ts](../../dev-tools/source-contracts/tests/source-identity.test.ts) | reviewed | PR141 covers all six input classes, relative-name binding, ignored files/location and invalid version admission independently. Sensitivity assertions are legitimate hash-input invariants; retained. |
| [dev-tools/source-contracts/tests/template-generation.test.ts](../../dev-tools/source-contracts/tests/template-generation.test.ts) | reviewed | Template boundary literals, unsupported interpolation/malformed shapes, distinct generic instantiations, stable shape sharing and name collisions protect emitted type contracts; retained. |
| [dev-tools/source-contracts/tests/trait-catalog.test.ts](../../dev-tools/source-contracts/tests/trait-catalog.test.ts) | reviewed | Real static extraction distinguishes spreads/overrides, explicit description keys, absent localization, trait/tag/rarity namespaces, family evidence, unsupported calls/ranges, deterministic relocation and selector-only lists; retained. |
| [dev-tools/source-contracts/tests/trait-definition-generation.test.ts](../../dev-tools/source-contracts/tests/trait-definition-generation.test.ts) | changed | Shared variants now require the literal authored description key rather than equality to each other. Retained exact variant/membership/source identity, conflicts, incomplete catalogs, exclusion of English prose and pinned attribution. |
| [dev-tools/source-contracts/tests/type-graph.test.ts](../../dev-tools/source-contracts/tests/type-graph.test.ts) | changed | Added-field drift now verifies resolved number-or-undefined semantics, replacing its self-derived expected ref. Kept constructor provenance, portfolio expansion, unresolved/closure bounds, serialization/callback drift, recursive/generic identities and path-independent determinism. |

### research verification

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [docs/research/evaluations/2026-10-search/verify.ts](../../docs/research/evaluations/2026-10-search/verify.ts) | changed | Removed a counter that incremented beside each row push and asserted the resulting row length. Kept independent exported-file digests, judgment/ranking joins, identity/duplicate/grade bounds, cluster coverage and full point-metric replay; frozen evidence and output unchanged. |

### scripts

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [scripts/browser-fixture.test.mjs](../../scripts/browser-fixture.test.mjs) | changed | Build/fallback assertions now require the locked atlas-web example arguments and Linux strip arguments, not just tool names or a broad regex. Retained real quoted executable invocation, content checksums, commit/host admission and dirty-source refusal. |
| [scripts/git-hooks/test-common.sh](../../scripts/git-hooks/test-common.sh) | reviewed | Real isolated Git history protects staged/range classification, deletions/renames/modes, docs/skill/research ownership, fast/full check separation, failure propagation and diagnostic/timing summaries. External compilers are intentionally command-boundary doubles; retained. |
| [scripts/git-hooks/test-pre-push.sh](../../scripts/git-hooks/test-pre-push.sh) | reviewed | Offline notices command checks its exact arguments/environment, outgoing ranges and multi-ref deduplication. Distinct missing dependencies, deletion, unrelated dirt, dirty dependency inputs and checkout/head mismatch retained; exercised through test-common.sh. |
| [scripts/release/test-installer.ps1](../../scripts/release/test-installer.ps1) | changed | Windows nightly installation now requires the complete resolved URL set and Windows archive; retained native executable install, dry-run/overwrite refusal, checksum and rollback scenarios. PowerShell syntax passed locally; native Windows execution awaits the published CI candidate. |
| [scripts/release/test-prepare-release.sh](../../scripts/release/test-prepare-release.sh) | changed | Command-order helpers now fail when the required command is absent before comparing positions; retained prepare/open-PR/publish modes, prompts/RC choices, dry-run forbidden effects, dirty/tag/refusal and post-validation cleanliness checks. |
| [scripts/release/test-release-tools.sh](../../scripts/release/test-release-tools.sh) | changed | Nightly installation now requires the complete resolved URL set and host archive, retaining pointer-first behavior. Kept real archive/hash validation, dry-run, overwrite refusal, malformed pointers and rollback to the previous binary. |
| [scripts/validation/ci-results.test.mjs](../../scripts/validation/ci-results.test.mjs) | reviewed | Selected-job failure/cancel/skip cases, all browser prerequisites, Rust/web-only selection, docs-only and malformed flags plus real CLI exit behavior retained. Expected jobs are literal, independent of production selection map. |
| [scripts/validation/skill.test.ts](../../scripts/validation/skill.test.ts) | reviewed | Temporary package fixtures distinguish malformed frontmatter/body from missing embed and unbundled resources. Structural checks are the explicit shipped single-file package contract; retained. |

### web/atlas-ui/e2e

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [web/atlas-ui/e2e/devices/compatibility.spec.ts](../../web/atlas-ui/e2e/devices/compatibility.spec.ts) | pending | — |
| [web/atlas-ui/e2e/devices/condition-refinement.spec.ts](../../web/atlas-ui/e2e/devices/condition-refinement.spec.ts) | pending | — |
| [web/atlas-ui/e2e/devices/conditions.spec.ts](../../web/atlas-ui/e2e/devices/conditions.spec.ts) | pending | — |
| [web/atlas-ui/e2e/devices/encounter-forms.spec.ts](../../web/atlas-ui/e2e/devices/encounter-forms.spec.ts) | pending | — |
| [web/atlas-ui/e2e/devices/feedback.spec.ts](../../web/atlas-ui/e2e/devices/feedback.spec.ts) | pending | — |
| [web/atlas-ui/e2e/devices/journeys.spec.ts](../../web/atlas-ui/e2e/devices/journeys.spec.ts) | pending | — |
| [web/atlas-ui/e2e/devices/spell-effects.spec.ts](../../web/atlas-ui/e2e/devices/spell-effects.spec.ts) | pending | — |
| [web/atlas-ui/e2e/devices/transitions.spec.ts](../../web/atlas-ui/e2e/devices/transitions.spec.ts) | pending | — |
| [web/atlas-ui/e2e/devices/turn-reminders.spec.ts](../../web/atlas-ui/e2e/devices/turn-reminders.spec.ts) | pending | — |
| [web/atlas-ui/e2e/filter-ux.spec.ts](../../web/atlas-ui/e2e/filter-ux.spec.ts) | pending | — |
| [web/atlas-ui/e2e/list-metadata.spec.ts](../../web/atlas-ui/e2e/list-metadata.spec.ts) | pending | — |
| [web/atlas-ui/e2e/mutable-workflows.spec.ts](../../web/atlas-ui/e2e/mutable-workflows.spec.ts) | pending | — |
| [web/atlas-ui/e2e/presentation-feedback.spec.ts](../../web/atlas-ui/e2e/presentation-feedback.spec.ts) | pending | — |
| [web/atlas-ui/e2e/responsive-search.spec.ts](../../web/atlas-ui/e2e/responsive-search.spec.ts) | pending | — |
| [web/atlas-ui/e2e/source-artifact.spec.ts](../../web/atlas-ui/e2e/source-artifact.spec.ts) | pending | — |
| [web/atlas-ui/e2e/trait-reference.spec.ts](../../web/atlas-ui/e2e/trait-reference.spec.ts) | pending | — |

### web/atlas-ui/src/api

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [web/atlas-ui/src/api/atlasApi.test.ts](../../web/atlas-ui/src/api/atlasApi.test.ts) | pending | — |
| [web/atlas-ui/src/api/availability.test.ts](../../web/atlas-ui/src/api/availability.test.ts) | pending | — |
| [web/atlas-ui/src/api/compatibility.test.ts](../../web/atlas-ui/src/api/compatibility.test.ts) | pending | — |

### web/atlas-ui/src/app

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [web/atlas-ui/src/app/AppShell.test.tsx](../../web/atlas-ui/src/app/AppShell.test.tsx) | pending | — |
| [web/atlas-ui/src/app/AtlasApp.test.tsx](../../web/atlas-ui/src/app/AtlasApp.test.tsx) | pending | — |
| [web/atlas-ui/src/app/BackendCompatibilityGate.test.tsx](../../web/atlas-ui/src/app/BackendCompatibilityGate.test.tsx) | pending | — |
| [web/atlas-ui/src/app/InvalidUrl.test.tsx](../../web/atlas-ui/src/app/InvalidUrl.test.tsx) | pending | — |
| [web/atlas-ui/src/app/documentNavigation.test.ts](../../web/atlas-ui/src/app/documentNavigation.test.ts) | pending | — |
| [web/atlas-ui/src/app/queryClient.test.ts](../../web/atlas-ui/src/app/queryClient.test.ts) | pending | — |
| [web/atlas-ui/src/app/readingUrl.test.ts](../../web/atlas-ui/src/app/readingUrl.test.ts) | pending | — |
| [web/atlas-ui/src/app/routeSearch.test.ts](../../web/atlas-ui/src/app/routeSearch.test.ts) | pending | — |
| [web/atlas-ui/src/app/routerNavigation.test.ts](../../web/atlas-ui/src/app/routerNavigation.test.ts) | pending | — |
| [web/atlas-ui/src/app/routes.test.ts](../../web/atlas-ui/src/app/routes.test.ts) | pending | — |
| [web/atlas-ui/src/app/urlState.test.ts](../../web/atlas-ui/src/app/urlState.test.ts) | pending | — |

### web/atlas-ui/src/features

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [web/atlas-ui/src/features/encounters/DiceLifecycle.test.tsx](../../web/atlas-ui/src/features/encounters/DiceLifecycle.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterAbilityControls.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterAbilityControls.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterActorReading.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterActorReading.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterAddDifficulty.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterAddDifficulty.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterConditionConsequences.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterConditionConsequences.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterConditionEditor.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterConditionEditor.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterConditionLifecycle.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterConditionLifecycle.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterConditionValue.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterConditionValue.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterDifficulty.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterDifficulty.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterHpAmount.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterHpAmount.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterHpControls.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterHpControls.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterHpEditor.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterHpEditor.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterHpValue.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterHpValue.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterInspectorPane.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterInspectorPane.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterLifeStatus.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterLifeStatus.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterLifeStatusPending.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterLifeStatusPending.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterModal.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterModal.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterParticipantSettings.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterParticipantSettings.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterPersistentCondition.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterPersistentCondition.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterPersistentDamageBatch.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterPersistentDamageBatch.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterPersistentRecovery.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterPersistentRecovery.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterPersistentTurnReview.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterPersistentTurnReview.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterRecoveryModal.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterRecoveryModal.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterRecoveryParameters.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterRecoveryParameters.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterSpellControls.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterSpellControls.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterTurnResources.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterTurnResources.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/EncounterTurnReview.test.tsx](../../web/atlas-ui/src/features/encounters/EncounterTurnReview.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/conditionDraft.test.ts](../../web/atlas-ui/src/features/encounters/conditionDraft.test.ts) | pending | — |
| [web/atlas-ui/src/features/encounters/encounterViews.test.tsx](../../web/atlas-ui/src/features/encounters/encounterViews.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/participantEdits.test.ts](../../web/atlas-ui/src/features/encounters/participantEdits.test.ts) | pending | — |
| [web/atlas-ui/src/features/encounters/useConditionEventPreview.test.tsx](../../web/atlas-ui/src/features/encounters/useConditionEventPreview.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/useEncounterMutations.test.tsx](../../web/atlas-ui/src/features/encounters/useEncounterMutations.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/useEncounterStrike.test.tsx](../../web/atlas-ui/src/features/encounters/useEncounterStrike.test.tsx) | pending | — |
| [web/atlas-ui/src/features/encounters/useEncounterTurnReview.test.tsx](../../web/atlas-ui/src/features/encounters/useEncounterTurnReview.test.tsx) | pending | — |
| [web/atlas-ui/src/features/lists/ListEntryNote.test.tsx](../../web/atlas-ui/src/features/lists/ListEntryNote.test.tsx) | pending | — |
| [web/atlas-ui/src/features/lists/ListItemsPane.test.tsx](../../web/atlas-ui/src/features/lists/ListItemsPane.test.tsx) | pending | — |
| [web/atlas-ui/src/features/lists/ListItemsReorder.test.tsx](../../web/atlas-ui/src/features/lists/ListItemsReorder.test.tsx) | pending | — |
| [web/atlas-ui/src/features/lists/ListModalLifecycle.test.tsx](../../web/atlas-ui/src/features/lists/ListModalLifecycle.test.tsx) | pending | — |
| [web/atlas-ui/src/features/lists/ListPaneStates.test.tsx](../../web/atlas-ui/src/features/lists/ListPaneStates.test.tsx) | pending | — |
| [web/atlas-ui/src/features/lists/ListTagSummary.test.tsx](../../web/atlas-ui/src/features/lists/ListTagSummary.test.tsx) | pending | — |
| [web/atlas-ui/src/features/lists/collectionUrlState.test.tsx](../../web/atlas-ui/src/features/lists/collectionUrlState.test.tsx) | pending | — |
| [web/atlas-ui/src/features/lists/savedListViews.test.tsx](../../web/atlas-ui/src/features/lists/savedListViews.test.tsx) | pending | — |
| [web/atlas-ui/src/features/parties/parties.test.tsx](../../web/atlas-ui/src/features/parties/parties.test.tsx) | pending | — |
| [web/atlas-ui/src/features/record-actions/AddToRecordButton.test.tsx](../../web/atlas-ui/src/features/record-actions/AddToRecordButton.test.tsx) | pending | — |
| [web/atlas-ui/src/features/records/RecordViews.test.tsx](../../web/atlas-ui/src/features/records/RecordViews.test.tsx) | pending | — |
| [web/atlas-ui/src/features/records/TraitView.test.tsx](../../web/atlas-ui/src/features/records/TraitView.test.tsx) | pending | — |
| [web/atlas-ui/src/features/search/AddRelationshipFilter.test.tsx](../../web/atlas-ui/src/features/search/AddRelationshipFilter.test.tsx) | pending | — |
| [web/atlas-ui/src/features/search/RelationshipFilters.test.tsx](../../web/atlas-ui/src/features/search/RelationshipFilters.test.tsx) | pending | — |
| [web/atlas-ui/src/features/search/ResultPaneHeader.test.tsx](../../web/atlas-ui/src/features/search/ResultPaneHeader.test.tsx) | pending | — |
| [web/atlas-ui/src/features/search/pageMetrics.test.ts](../../web/atlas-ui/src/features/search/pageMetrics.test.ts) | pending | — |
| [web/atlas-ui/src/features/search/relationshipConstraints.test.ts](../../web/atlas-ui/src/features/search/relationshipConstraints.test.ts) | pending | — |
| [web/atlas-ui/src/features/search/useSearchWorkspace.test.tsx](../../web/atlas-ui/src/features/search/useSearchWorkspace.test.tsx) | pending | — |
| [web/atlas-ui/src/features/search/workspaceState.test.ts](../../web/atlas-ui/src/features/search/workspaceState.test.ts) | pending | — |

### web/atlas-ui/src/shared

| File | Status | Evidence and disposition |
| --- | --- | --- |
| [web/atlas-ui/src/shared/branding/AtlasMark.test.tsx](../../web/atlas-ui/src/shared/branding/AtlasMark.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/browser/startupReload.test.ts](../../web/atlas-ui/src/shared/browser/startupReload.test.ts) | pending | — |
| [web/atlas-ui/src/shared/dice/ContentRoll.test.tsx](../../web/atlas-ui/src/shared/dice/ContentRoll.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/dice/RollResult.test.tsx](../../web/atlas-ui/src/shared/dice/RollResult.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/dice/useDicePopoverFocus.test.tsx](../../web/atlas-ui/src/shared/dice/useDicePopoverFocus.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/dice/useDiceRoll.test.tsx](../../web/atlas-ui/src/shared/dice/useDiceRoll.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/feedback/ErrorNotice.test.tsx](../../web/atlas-ui/src/shared/feedback/ErrorNotice.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/feedback/GlobalFeedback.test.tsx](../../web/atlas-ui/src/shared/feedback/GlobalFeedback.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/feedback/errorMessages.test.ts](../../web/atlas-ui/src/shared/feedback/errorMessages.test.ts) | pending | — |
| [web/atlas-ui/src/shared/filters/CollectionValueInput.test.tsx](../../web/atlas-ui/src/shared/filters/CollectionValueInput.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/filters/CommonChoicePicker.test.tsx](../../web/atlas-ui/src/shared/filters/CommonChoicePicker.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/filters/FilterPanel.test.tsx](../../web/atlas-ui/src/shared/filters/FilterPanel.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/filters/FilterValueInput.test.tsx](../../web/atlas-ui/src/shared/filters/FilterValueInput.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/filters/SearchEntry.test.tsx](../../web/atlas-ui/src/shared/filters/SearchEntry.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/filters/builderEdits.test.ts](../../web/atlas-ui/src/shared/filters/builderEdits.test.ts) | pending | — |
| [web/atlas-ui/src/shared/filters/filterControls.test.ts](../../web/atlas-ui/src/shared/filters/filterControls.test.ts) | pending | — |
| [web/atlas-ui/src/shared/filters/filterUrl.test.ts](../../web/atlas-ui/src/shared/filters/filterUrl.test.ts) | pending | — |
| [web/atlas-ui/src/shared/filters/operatorTransitions.test.ts](../../web/atlas-ui/src/shared/filters/operatorTransitions.test.ts) | pending | — |
| [web/atlas-ui/src/shared/filters/relationshipUrl.test.ts](../../web/atlas-ui/src/shared/filters/relationshipUrl.test.ts) | pending | — |
| [web/atlas-ui/src/shared/filters/searchState.test.ts](../../web/atlas-ui/src/shared/filters/searchState.test.ts) | pending | — |
| [web/atlas-ui/src/shared/filters/useFilterDiscovery.test.ts](../../web/atlas-ui/src/shared/filters/useFilterDiscovery.test.ts) | pending | — |
| [web/atlas-ui/src/shared/filters/useFilterEditor.test.tsx](../../web/atlas-ui/src/shared/filters/useFilterEditor.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/filters/useFilterUrlRecovery.test.tsx](../../web/atlas-ui/src/shared/filters/useFilterUrlRecovery.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/layout/PaneLayout.test.tsx](../../web/atlas-ui/src/shared/layout/PaneLayout.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/layout/WorkspaceLayout.test.tsx](../../web/atlas-ui/src/shared/layout/WorkspaceLayout.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/queries/immutableReferenceQueries.test.ts](../../web/atlas-ui/src/shared/queries/immutableReferenceQueries.test.ts) | pending | — |
| [web/atlas-ui/src/shared/records/ActionCost.test.tsx](../../web/atlas-ui/src/shared/records/ActionCost.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/ActorRuntimeHints.test.tsx](../../web/atlas-ui/src/shared/records/ActorRuntimeHints.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/ContextualActivityPreview.test.tsx](../../web/atlas-ui/src/shared/records/ContextualActivityPreview.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/CreatureReference.test.tsx](../../web/atlas-ui/src/shared/records/CreatureReference.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/EmbeddedAbilityReading.test.tsx](../../web/atlas-ui/src/shared/records/EmbeddedAbilityReading.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/FeedbackFamilyLayout.test.tsx](../../web/atlas-ui/src/shared/records/FeedbackFamilyLayout.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/HazardReference.test.tsx](../../web/atlas-ui/src/shared/records/HazardReference.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/JournalContents.test.tsx](../../web/atlas-ui/src/shared/records/JournalContents.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/PhysicalDetails.test.tsx](../../web/atlas-ui/src/shared/records/PhysicalDetails.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/PreparedContent.test.tsx](../../web/atlas-ui/src/shared/records/PreparedContent.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/PreparedTraitContent.test.tsx](../../web/atlas-ui/src/shared/records/PreparedTraitContent.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/PreviewReadingHistory.test.tsx](../../web/atlas-ui/src/shared/records/PreviewReadingHistory.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/RecordBrowseSummary.test.tsx](../../web/atlas-ui/src/shared/records/RecordBrowseSummary.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/RecordDetailPane.test.tsx](../../web/atlas-ui/src/shared/records/RecordDetailPane.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/RecordGrants.test.tsx](../../web/atlas-ui/src/shared/records/RecordGrants.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/RecordPresentation.test.tsx](../../web/atlas-ui/src/shared/records/RecordPresentation.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/RecordRarity.test.tsx](../../web/atlas-ui/src/shared/records/RecordRarity.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/RecordReadingWorkspace.test.tsx](../../web/atlas-ui/src/shared/records/RecordReadingWorkspace.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/RecordReferenceDrawer.test.tsx](../../web/atlas-ui/src/shared/records/RecordReferenceDrawer.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/RecordReferenceDrawerFocus.test.tsx](../../web/atlas-ui/src/shared/records/RecordReferenceDrawerFocus.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/RecordReferenceDrawerIdentity.test.tsx](../../web/atlas-ui/src/shared/records/RecordReferenceDrawerIdentity.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/RecordReferenceDrawerOpeningFocus.test.tsx](../../web/atlas-ui/src/shared/records/RecordReferenceDrawerOpeningFocus.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/ReferenceBrowser.test.tsx](../../web/atlas-ui/src/shared/records/ReferenceBrowser.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/ReferenceCompleteness.test.tsx](../../web/atlas-ui/src/shared/records/ReferenceCompleteness.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/ReferenceFactPolicy.test.tsx](../../web/atlas-ui/src/shared/records/ReferenceFactPolicy.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/ReferenceOptionalFacts.test.tsx](../../web/atlas-ui/src/shared/records/ReferenceOptionalFacts.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/RelatedRecords.test.tsx](../../web/atlas-ui/src/shared/records/RelatedRecords.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/RelationshipEvidence.test.tsx](../../web/atlas-ui/src/shared/records/RelationshipEvidence.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/RollTableReading.test.tsx](../../web/atlas-ui/src/shared/records/RollTableReading.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/SearchMatchContext.test.tsx](../../web/atlas-ui/src/shared/records/SearchMatchContext.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/SourceSectionSelection.test.tsx](../../web/atlas-ui/src/shared/records/SourceSectionSelection.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/SpellEffectPreview.test.tsx](../../web/atlas-ui/src/shared/records/SpellEffectPreview.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/SpellRankControl.test.tsx](../../web/atlas-ui/src/shared/records/SpellRankControl.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/SpellRankValue.test.tsx](../../web/atlas-ui/src/shared/records/SpellRankValue.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/SpellRuntimeReference.test.tsx](../../web/atlas-ui/src/shared/records/SpellRuntimeReference.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/TraitBadge.test.tsx](../../web/atlas-ui/src/shared/records/TraitBadge.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/documentHeadings.test.ts](../../web/atlas-ui/src/shared/records/documentHeadings.test.ts) | pending | — |
| [web/atlas-ui/src/shared/records/presentationFacts.test.tsx](../../web/atlas-ui/src/shared/records/presentationFacts.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/readingUrlNavigation.test.ts](../../web/atlas-ui/src/shared/records/readingUrlNavigation.test.ts) | pending | — |
| [web/atlas-ui/src/shared/records/recordDetailQuery.test.ts](../../web/atlas-ui/src/shared/records/recordDetailQuery.test.ts) | pending | — |
| [web/atlas-ui/src/shared/records/recordNavigationIdentity.test.ts](../../web/atlas-ui/src/shared/records/recordNavigationIdentity.test.ts) | pending | — |
| [web/atlas-ui/src/shared/records/relationshipSearch.test.ts](../../web/atlas-ui/src/shared/records/relationshipSearch.test.ts) | pending | — |
| [web/atlas-ui/src/shared/records/statusReasonLabel.test.ts](../../web/atlas-ui/src/shared/records/statusReasonLabel.test.ts) | pending | — |
| [web/atlas-ui/src/shared/records/useBrowseHeaderAlignment.test.tsx](../../web/atlas-ui/src/shared/records/useBrowseHeaderAlignment.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/usePreviewContentFocus.test.tsx](../../web/atlas-ui/src/shared/records/usePreviewContentFocus.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/usePreviewReadingPosition.test.tsx](../../web/atlas-ui/src/shared/records/usePreviewReadingPosition.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/useRecordDetail.test.tsx](../../web/atlas-ui/src/shared/records/useRecordDetail.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/useRecordPreview.test.tsx](../../web/atlas-ui/src/shared/records/useRecordPreview.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/useReferenceReading.test.tsx](../../web/atlas-ui/src/shared/records/useReferenceReading.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/records/useRetainedRecord.test.tsx](../../web/atlas-ui/src/shared/records/useRetainedRecord.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/search/ResultSortControl.test.tsx](../../web/atlas-ui/src/shared/search/ResultSortControl.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/search/SearchProfileContext.test.tsx](../../web/atlas-ui/src/shared/search/SearchProfileContext.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/search/SearchSettings.test.tsx](../../web/atlas-ui/src/shared/search/SearchSettings.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/search/SearchSettingsFlow.test.tsx](../../web/atlas-ui/src/shared/search/SearchSettingsFlow.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/search/SimilaritySourceDocument.test.tsx](../../web/atlas-ui/src/shared/search/SimilaritySourceDocument.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/search/searchIdentity.test.ts](../../web/atlas-ui/src/shared/search/searchIdentity.test.ts) | pending | — |
| [web/atlas-ui/src/shared/search/searchPreferences.test.ts](../../web/atlas-ui/src/shared/search/searchPreferences.test.ts) | pending | — |
| [web/atlas-ui/src/shared/search/similarityState.test.ts](../../web/atlas-ui/src/shared/search/similarityState.test.ts) | pending | — |
| [web/atlas-ui/src/shared/theme/themeDefinitions.test.ts](../../web/atlas-ui/src/shared/theme/themeDefinitions.test.ts) | pending | — |
| [web/atlas-ui/src/shared/ui/HelpTooltip.test.tsx](../../web/atlas-ui/src/shared/ui/HelpTooltip.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/ui/LoadingStatus.test.tsx](../../web/atlas-ui/src/shared/ui/LoadingStatus.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/ui/actions/DangerActionButton.test.tsx](../../web/atlas-ui/src/shared/ui/actions/DangerActionButton.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/ui/actions/PaneAction.test.tsx](../../web/atlas-ui/src/shared/ui/actions/PaneAction.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/ui/forms/EditableCommitField.test.tsx](../../web/atlas-ui/src/shared/ui/forms/EditableCommitField.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/ui/pickers/SearchPickerModal.test.tsx](../../web/atlas-ui/src/shared/ui/pickers/SearchPickerModal.test.tsx) | pending | — |
| [web/atlas-ui/src/shared/ui/tables/IndexTable.test.tsx](../../web/atlas-ui/src/shared/ui/tables/IndexTable.test.tsx) | pending | — |

## Validation and delivery

- Initial inventory is test-entry-point based: Rust test-bearing modules, JS/TS test/spec/integration entry points, executable shell/PowerShell test scripts, and research verifiers. Supporting fixture/harness consumers are examined with their owning cases. Case counts are descriptive runner metadata, not review targets.
- PR141 strengthened selected cases and fixed a tooltip checkpoint. Its changes are retained, but those files still receive the same systematic review as the rest of the inventory.
- No condition rule, lifecycle, application, or presentation behavior is authorized for change in this review. A mixed file can be reviewed without changing its condition-specific cases.

### Core primitives and storage milestone

- Baseline `cargo test --workspace` passed at `ce3f6442` before candidate test execution. Existing ignored native-model tests remain ignored under their documented requirements.
- Focused `cargo test -p atlas-domain -p atlas-dice -p atlas-tags -p atlas-cli-support -p atlas-storage -p atlas-app-model` passed, including the final membership-ordering refinements. Tag cardinality assertions also passed in a subsequent focused rerun.
- Plausible faults now rejected include accepting Unicode whitespace keys, rejecting the maximum valid relationship budget, matching Other roles to themselves, dropping preview negation, swapping tag assignments, and rerunning an already-applied backfill. These are reasoned counterexamples; no mutation campaign is claimed.
- `atlas-sqlite-vec` has no local test entry point: registration is exercised against real vec0 tables and rollback in storage, and vector capabilities are covered again with atlas-index. No synthetic registration-failure seam added.

- Application-model checks passed with `cargo test -p atlas-app-model` and the combined focused run. The HP revision test now isolates one missing-field failure instead of accepting an unrelated malformed request as evidence.

### Embedding milestone

- `cargo test -p atlas-embedding -- --include-ignored --test-threads=1`: 24 passed, including all three native BGE/Gemma execution cases. Reused verified shared model assets through a task-local cache link; no asset downloads or installed workspace changes.
- The unchanged frontend baseline also passed: 159 files / 1,585 tests. Browser checks remain separate.
- Native batch lengths are now checked before elementwise tolerance comparisons; this rejects an empty/truncated output that a zip-only check could accept. Cache checks remain real filesystem operations with bounded synchronization, without retries or weaker tolerances.

### Developer tooling milestone

- `cargo test -p atlas-dev` passed after supplying the installed Apple Clang runtime through `LIBRARY_PATH`; ONNX static linking advertises a removed Clang 17 directory. The first attempt failed at linking, before test execution. No repository/build-policy change was made.
- `cargo test --manifest-path dev-tools/dice-prototype/server/Cargo.toml --target-dir target --locked`: six passed. Reused the task target rather than allocating another compiler cache.
- The three developer evidence collectors remain explicitly ignored with their existing fixture prerequisites; source inspection supports their retained purpose, without claiming those historical full-corpus experiments were rerun.

### Release and validation infrastructure milestone

- `npm --prefix dev-tools/release run verify`: 23 passed. `node --test scripts/browser-fixture.test.mjs scripts/validation/ci-results.test.mjs scripts/validation/skill.test.ts`, `sh scripts/git-hooks/test-common.sh`, `sh scripts/release/test-release-tools.sh` and `sh scripts/release/test-prepare-release.sh` passed, with affected shell suites rerun after final assertion edits.
- Disposable compiled-release probes first passed unchanged, then rejected wrong platform metadata and an omitted first dependency notice. Missing-command probes demonstrated both old order helpers returning success with no match and both candidate helpers rejecting that input.
- PowerShell test source parsed successfully. Its native Windows executable fixture requires Windows and remains a CI validation obligation; no cross-platform replacement is claimed.

### Source-contract tooling milestone

- Unchanged offline fixture baseline: 133 passed. Final `npm --prefix dev-tools/source-contracts run verify` with pinned Rust 1.95: 132 passed. One removed test exercised only JavaScript built-ins; all repository scenarios remain.
- `npm --prefix dev-tools/source-contracts run verify-generated`: 11 passed against fresh pinned upstream extraction, with no generated artifact changes. Condition-specific assertions ran unchanged as part of that gate but remain excluded from this review.
- A disposable presence-blind comparator passed the previous equipment test and failed the candidate presence case. Both unchanged test copies passed before mutation. Temporary probe modules were removed afterward.
- The first added-field assertion assumed a primitive reference and failed: the compiler correctly retains an optional number as number-or-undefined. The final assertion verifies those literal resolved meanings without binding the structural reference hash.

### Rust source-model milestone

- `cargo test -p atlas-foundry-model --all-targets`: 91 passed, including the generic compiled fixture and developer example tests. The final rule-variant check also passed a focused dispatch rerun before the final all-target check.
- Production source parsing, admission, dispatch, snapshots and generated output remain unchanged. The strengthened assertions distinguish correct-length output from exact retained duplicate values/references and verify typed dispatch independently of the common rule key.

### Archived evaluation verifier

- `sh scripts/validation/research.sh` passed before and after removal: 34 exported files and 22,533 point-metric values. Original bootstrap intervals, source interpretations and frozen datasets were not modified or claimed rerun.

### Source-backed records and content milestone

- `cargo test -p atlas-record`: 145 passed, two existing corpus tests ignored, and both compile-fail doctests passed. Condition-specific cases ran unchanged but remain excluded from the review.
- Stronger fixtures exposed sorted JSON-map setup in the original order test; raw authored bytes now test z-before-a order directly. The independent safe HTML fixture includes the sanitizer-required link relationship rather than silently sanitizing its own expected input.
- Reasoned counterexamples include dropped/surplus damage or trait components, wrong executable kind/AST, missing skill callback, missing values misreported as applicable, lost action heading title and stale passage identity. No production mutation probe is claimed for this milestone.

### Ingest milestone

- `cargo test -p atlas-ingest`: 29 library and eight loader integration tests passed. Five additional bounded native tests passed with `--ignored --skip actual_complete_source --skip complete_artifact_snapshots --skip pinned_english --test-threads=1`, using the verified BGE/Gemma caches.
- The four full-source/archive checks retain their existing prerequisites; this milestone does not claim their historical corpora were rebuilt. Actual target addition, ambiguity, recovery and rename publication now have independent destination/label expectations. A plausible counterexample is advancing reuse statistics while publishing stale references; no mutation probe is claimed.
- Compiler output reached approximately 15 GiB: inspected 9.5 GiB dependency output, 3.9 GiB incremental state and 216 MiB build scripts. Free space remained 129 GiB; retained the active task cache rather than rebuilding it.

### Runtime milestone

- `cargo test -p atlas-runtime`: 53 passed, one existing native test ignored. `cargo test -p atlas-runtime -- --ignored --test-threads=1`: the retained Gemma query-session recovery test passed with verified assets.
- The cleanup assertions reject a lying Removed report, the verification-race fixture changes file length deterministically, and rollback is inspected before a later initialization could repair partial state. These are discriminating inputs/reasoned counterexamples; no production mutation probe is claimed.
- CI on the earlier published source-tooling head passed native Windows release tooling, Windows publication, macOS/Linux release checks, frontend unit/bundle checks, source contracts and research evidence. Rust and one browser shard were still running at inspection; this is not final-head readiness evidence.

### Index milestone

- `cargo test -p atlas-index`: 104 passed, two existing corpus/evidence tests ignored under their documented prerequisites. The final simplified state-partition assertion also passed a focused rerun.
- Complete key/owner selections, exact persisted inputs, literal prices/catalog dependencies, all five discovery states and full KNN cardinality now reject plausible partial or stale outputs. Existing SQLite mutation/fault-trigger tests still validate rollback and rejection; no new production mutation probe is claimed.
