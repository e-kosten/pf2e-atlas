# Execution task packets

No fresh queries or model outputs have been opened while preparing these tasks.
Coordinator freezes protocol, rubric, corpus identity and source-discovery inputs
before initiating the tasks. No model outputs are provided to discovery agents.

## Source discovery, three Luna agents

Partition the24 concepts across three agents, balanced by six categories. Input:
`{sourceRoot, corpusFingerprint, concepts:[{id,category,evaluationRole,
observations:[{id,text,args,scope,intent,criteria}]}]}`. Provide the exact pinned
source checkout and a read-only record-key/source-file inventory. No model
rankings, returned candidates, embeddings, old answer labels or model-based search.

Task: inspect the assigned natural intents and locate plausible source-backed
reference records using source filenames, text search and actual JSON mechanics.
Use the authored wording; do not invent stricter criteria. Prefer up to three
substantively different useful candidates per concept, including a qualified
alternative where appropriate. This is a bounded reference search, not exhaustive
corpus recall. Spend at most five minutes per concept; failure to find a reference
is an explicit search gap, not proof none exists. Do not force a candidate.
Check actual owners, spellcasting association and negation. Report at root-record
granularity and list only observation scopes in which the candidate is eligible.
Do not infer eligibility solely from kind; the coordinator verifies visibility,
pack and other filters against the same eligible root inventory used by ranking.

Output:
```
{"references":[{
  "key":"pack:id", "name":"record name", "record_kind":"creature",
  "source_path":"packs/pack/file.json", "observationIds":["concept-open"],
  "proposalRationale":"why this may be useful, with limitations",
  "evidence":[{"pointer":"/system/details/publicNotes","quote":"literal substring"}]
}], "coverage":[{"concept":"concept-id","status":"proposed|none-found|uncertain",
"searchSummary":"bounded paths/terms examined and unresolved limitations"}]}
```

Write only your assigned reference proposal file. Do not edit sources or run
inference/builds. Do not give your proposals privileged grades. Coordinator checks
key/path identity, literal citations and exact scope eligibility, records hashes,
combines proposals, and freezes that reference file before inspecting model output.
The blinder consumes its `references` array; proposal rationale/evidence and
reference status are never forwarded as answer hints to relevance reviewers.

## Primary source grading, three Luna agents

Assign `panel-N.json`, the frozen GRADING.md and one exclusive output file. Read
all assigned relevant source mechanics and needed embedded context. Return every
ID exactly once using the GRADING.md schema, with pointer/quote evidence only.
Use null with rationale for insufficient evidence. No model/rank/score/channel,
reference status, other grades, private bundle or metrics access. No external
research, inference, changes to sources, or unassigned writes. Source familiarity
must be disclosed. Compatible paired scopes share the same ID and judgment.
Coordinator runs the shared source-bound validator; evidence formatting errors
are corrected in a new version, leaving originals intact.

## Independent overlap, rotated Luna assignment

Assign each primary reviewer the next panel's independently selected overlap,
without primary grades. Same rubric/evidence contract. Every grade or opposite
flag disagreement goes to source adjudication, including0/1 and2/3. The coordinator
examines evidence without model membership or joined scores. Adjudication output
uses the same grade schema plus `adjudicationReason`; original opinions remain
immutable. Do not settle truth by majority vote. Unresolved cases block final
point conclusions; exploratory provisional metrics are clearly labeled.

## User review and reporting

Pregrade `user-review-sample.json` selects one concept per category. After grading,
prepare its sources, consequential gain/loss examples and all unresolved disputes,
with a concise first page and complete appendix. User corrections create a new
adjudication version applied consistently to every configuration. Show known
reference ranks only when the reference is useful and dispute-resolved; rank
absence from complete semantic results differs from unavailable rank evidence.
All references remain nonexhaustive and may be rejected by blinded source review.
