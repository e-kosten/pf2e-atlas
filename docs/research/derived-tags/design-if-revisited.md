# Concept labeling design if revisited

Status: research preserved; implementation deferred. This consolidates the
October 9 discussion, including later clarifications. The [original discussion
plan](sources/2026-10-09-design-discussion.md) is retained as inert evidence.
Start with a demonstrated retrieval gap as described in the [entry point](README.md).
None of the mechanisms below is a commitment to build the feature.

## Concepts, claims and browse collections

A concept needs a stable identity, a clear user question, meaning, boundaries and
positive/near-negative examples. Its short label may summarize a richer claim:

| Claim | Why the distinction matters |
| --- | --- |
| Provides a way to counteract active magic | Supplies a solution |
| Can be resolved through counteracting magic | Accepts that solution |
| Characteristically attacks from concealment | Describes actor behavior |
| Has a strong underground association | Describes setting rather than movement capability |
| Contains or generates folk-hero stories | Can apply to a RollTable or other unexpected family |

Explore small relation vocabularies such as provides, inflicts, protects against,
resolved by, associated with and contains/generates only where they clarify useful
questions. A complete formal graph, RDF implementation and fixed concept schema
were not selected.

Browse collections organize discovery independently of meaning. A concept may
have several homes, and placement changes should not invalidate assignments.
A selectable collection still needs an explicit filter meaning, such as union
of members; visual placement alone does not imply inheritance or entailment.

Named native-filter expressions and authored interpretive labels solve different
problems. If DTO-backed facts already answer the question precisely, improve their
discoverability instead of asking an LLM to duplicate them. Historical coverage
checklists included settings, encounter/social roles, problem response, exploration,
combat effects, themes, scene challenges, character support, usage patterns and
status/affliction behavior. They are prompts for gap analysis, not a mandate to
populate every group. Consider actions, feats, character options, deities, rules
and RollTables as well as the old five Node categories.

## Two connected loops

**Ontology development:** agents research native overlap and real records,
propose definitions/applicability and prepare positive/negative examples. A
coordinator prepares a review packet. The human approves or corrects it; an
approved revision can release assessment work. Research may inspect records
without first completing corpus classification or a universal ontology.

**Record assessment:** workers apply an approved concept revision to source
context, returning decisions and evidence. Validation, reconciliation and review
lead to accepted assignments. Repeated ambiguity, missing concepts or unjustified
family exclusions produce ontology proposals. Workers cannot silently expand
meaning or invent live tags during an assessment run. Unaffected approved concepts
can continue while another concept is revised.

## Applicability and complete coverage

Restrictions should follow what a claim requires, rather than the families in
which its first examples happened to occur. A theme can cross families; a behavior
claim may require an actor subject. Unknown facts and uncertain combinations
remain eligible for assessment. Each exclusion needs one concise, versioned
policy justification and can be challenged through examples or an audit.

If broad labeling is approved, construct an exact inventory of every eligible
record/concept pair. Search, embeddings, names and keyword rules may prioritize
work or discover exemplars, but the user preferred extra LLM passes over candidate
pruning that silently loses positives. Batching concepts is an execution choice,
independent of browse grouping. Every required pair needs a disposition; missing,
malformed, failed or truncated output remains unfinished.

Determine the assignment subjects before implementing: roots, navigable owned
items, prose definitions or a selected combination. A creature's ability can
support a creature claim without inheriting every label from every referenced
record. Containment and generated content require explicit claim semantics.

Context should be bounded and sufficient: labelled source facts, relevant prepared
content, owned abilities, conditions, references and qualifications, with checked
source addresses. Do not scrape terminal output, assume an LLM summary is complete,
or equate a valid packet schema with sufficient evidence. Audience policy must
govern both labeling context and published explanations.

## Negatives, revisions and reassessment

Candidate dispositions discussed were positive, weak negative, strong negative
and insufficient context; policy-excluded and unfinished work are different.
Exact names and representation remain open. Sparse positive assignments can
coexist with compact coverage tracking.

Weak/strong is intended to describe support for a negative, not just self-reported
model confidence. A compact basis code or criterion reference may help. Routine
negatives need no prose essay; retain detailed explanations for disputes, boundary
examples, audits and human corrections. Invalid or missing context cannot support
a strong negative. Policy exclusion explanations belong once per rule, not on
every excluded record.

Weak negatives can be reassessment priorities after a refinement. A strong
negative is still relative to its old definition: changing “performs X” to
“contains or generates X” can make an unrelated RollTable relevant. Expanding
applicability schedules newly eligible records. Negative strength alone cannot
authorize skipping reassessment.

The safe initial policy is reassessing all potentially affected eligible pairs
after decision-affecting changes. Selective carry-forward remains an optimization
to validate. It would need the old decision and basis, old/new revisions, explicit
impact justification, acceptance and random audits of carried-forward strong
negatives. Reuse must not masquerade as a fresh LLM evaluation or recreate an
overfit semantic rule engine.

| Versioned concern | Why distinguish it |
| --- | --- |
| Concept meaning and applicability | Defines what decision was asked and which subjects were eligible |
| Catalog membership | Defines the complete assessment scope |
| Browse organization | Can change without changing meaning |
| Record context and dependencies | Source/reference changes can invalidate evidence |
| Procedure, prompts and models | Changes require calibration and impact review |
| Accepted publication | Identifies what users can actually filter against |

Retain actual versioned inputs or retrievable snapshots, not only hashes. Material
meaning changes may need new identities, splits or retirement. Reviewed-empty
means no accepted positives for a completed scope/revision, not permanently
reviewed. Product exclusion filters need an explicit policy for unknown and
incomplete coverage. Coherent accepted editions were proposed; mixed partial
publication and its user behavior were unresolved.

For scale only, 25,000 records × 300 concepts gives 7.5 million pairs. At 20
concepts/request and two reviewers it gives 750,000 requests before exclusions,
retries or disputes. One byte/disposition is 7.5 MB/reviewer/edition before keys,
metadata, indexes and evidence. These are arithmetic examples, not approved
catalog sizes, measured storage, performance or cost estimates. Pilot complete
context and batching first; optimize resumability, shared prefixes, validated
reuse and incremental work before reducing semantic coverage. Retention of raw
responses and older context snapshots needs a separate decision.

## Human review and coordinator feedback

Agents should assemble the calibration set and review proposals. The human wants
to approve them through a developer surface, rather than create datasets or edit
files. A separate developer UI, possibly hosted through `atlas-dev`, was suggested;
hosting, frontend/service ownership and durable review storage remain unresolved.
Shared record rendering could be reused without putting editorial controls into
ordinary product flows.

| Interaction | Useful human input and retained outcome |
| --- | --- |
| Concept review | Approve/edit meaning, boundaries, applicability and prepared examples |
| Record or disagreement review | Correct verdict, selected evidence, qualifications and explanation |
| Applicability challenge | Attach a case showing that a family exclusion is unjustified |
| Missing concept | State the user question and attach a triggering record |
| Calibration curation | Approve teaching/evaluation use or reject an ambiguous candidate |
| Negative review | Challenge strength/basis, flag omissions or request sufficient context |
| Revision preview | Review changed meaning, affected records and proposed reuse/rerun scope |
| Coverage/drift review | Inspect pending/stale cases and recurring errors |
| Defer | Capture missing information without forcing a verdict |

Correction scope matters: one record, a procedure/instruction issue, or a general
definition/applicability change. Case approval does not silently become a general
rule. Optional notes, source selections and wording edits supplement prepared
proposals.

A completed review should produce a durable session ID, readable digest and
machine-readable packet retrievable by a coordinator. Preserve exact concept,
context and procedure revisions, reviewed proposal, human action, evidence/edits,
scope and unresolved questions. Handle stale revisions and duplicate imports
explicitly. The coordinator applies authorized case decisions, groups systemic
findings into reviewable proposals, requests ontology approval where needed,
schedules reassessment and reports accepted versus pending versus published state.

Published positive evidence could power “why this matched”: a short claim,
important prerequisites/limits and navigation to the actual field or owned
ability. Validate attribution and audience scope. Developer proposals and rejected
explanations should not become product evidence.

## Calibration and future architecture

[Research and evaluation](research-and-evaluation.md) preserves the panel research
and proposed pilot. Neither Luna/Sol participation nor reviewer counts or
unanimity were selected by measurement. A full job would follow calibration, not
serve as its substitute.

Recheck ownership against current source-backed contracts before resuming:
`atlas-tags` owns definitions/validation today; record policies provide pure source
facts/content; ingest prepares inputs; index owns reference schema and SQL;
search owns retrieval; runtime constructs bounded workspace operations; app-service
composes workflows and app-model exports intentional contracts. CLI/HTTP/UI stay
thin. Developer assessment storage/services need a real owner, independent of
replaceable product projections. Do not restore normalized AtlasRecord, metric
EAV, persisted RichDocument, old facades or query-time LLM filtering.

If approved, develop a small ontology pilot, revision/assessment contracts,
developer review flow and panel experiment before whole-corpus processing and
product publication. Add authoritative assignment rows and query integration
together through normal catalog-bound SQL eligibility. Define missing-coverage,
composed native/label filters, publication and source-change behavior explicitly.
Update architecture/ADRs and relevant contributor guidance with the agreed
implementation. Historical schemas and rollout slices are not execution authority.

## Open choices

- Whether any gap warrants authored labels at all, and which small concepts test it.
- Formal claim structure versus defined labels; named native filters versus new truth.
- Assignment subjects and contained/generated meaning.
- Applicability boundaries and what makes a negative strong.
- Safe reuse/invalidation, assessment editions and incomplete/exclusion queries.
- Reviewer mix/count, acceptance policy, batching and reconsideration limits.
- Developer UI, durable feedback/run storage and service ownership.
- Authoring/storage format, publication workflow and evidence retention.

YAML, the old closed group enums, single-home display organization, minimal
provenance and unanimous acceptance remain historical choices to reassess, not
requirements for a revived design. Existing file contracts are unchanged by this
research preservation task.
