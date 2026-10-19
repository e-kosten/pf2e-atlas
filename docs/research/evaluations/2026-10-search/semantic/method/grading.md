# Source-reviewer guidance

Status: approved rubric; freeze this file hash with each judgment packet.

Judge whether the supplied record is a useful answer to the supplied query and
its authored intent/criteria. Do not rewrite that intent to fit results. Several
different records may be good answers. Do not infer relevance from title or shared
words alone, except when the request actually is a name lookup.

Read the full relevant source record and its necessary embedded context. Do not
grade a search snippet or assume a matching owned item is an ability of its owner.
For a claimed caster capability, inspect authored spellcasting association,
preparation/usage and relevant conditions. Possession and practical use are
different claims. Preserve qualifiers, negation, immunity/opposition, targets,
trigger conditions and material limitations.

## Relevance scale

| Grade | Meaning |
|---|---|
|3|A direct, useful answer to the actual intent. Ordinary game limitations do not automatically make an otherwise direct answer indirect.|
|2|A useful qualified or indirect answer with an important limitation; describe that limitation.|
|1|Substantively related but not useful for this intent, including incomplete or opposite-function answers. Explain the relationship and unmet intent.|
|0|Unrelated or merely coincidental vocabulary. For exact-name requests, another canonical identity is not a partial answer just because it shares a broad item class.|
|null|Source evidence or intent is insufficient to make a defensible judgment. Explain the uncertainty.|

Grades2 and3 count as useful. Graded ranking also distinguishes them: nDCG uses
gains0,1,3,7 for grades0,1,2,3. Consequently2-versus3 judgments deserve care, even
though both count as useful. State the reason instead of using personal preference
as an unstated requirement.

A related opposite can receive graded-relevance credit while remaining a failure
in the useful-answer measure. A word coincidence alone is insufficient for grade1.

Do not invent strict class, exclusivity, level or safety requirements absent from
the intent. Conversely, a weak hint is not a strong thematic characterization.
Decide requested entity type from the query/criteria, not simply whether the
search happened to have a kind filter. Cross-family answers can be useful when
they actually fulfill the intent.

## Approved calibration clarifications

These examples are exposed development cases. Fresh cases remain unknown.

- A creature with an actual prepared cold cantrip is a qualified answer to
  `frost sorcerer` when the rest of its spells/flavor are unrelated to cold:
  grade2. This does not require the sorcerer class or exclusive cold
  specialization. For a literal `caster with a cold spell` intent it could instead
  be direct; follow the actual wording/criteria.
- An unrelated Staff of Control is grade0 for an exact Staff of Fire lookup.
  Being another staff is not a partial answer to the requested canonical identity.
- An ordinary creature carrying mundane leather armor is grade0 for haunted
  armor without haunting/possession evidence. An inventory noun is not proof of
  the requested concept.
- A standalone heatless illumination spell does not supply a wizard characterized
  by fire magic. Grade1 requires a substantive source-backed thematic connection;
  a title-word coincidence alone is0. Neither qualifies as a useful caster answer.
- Creating a different hazardous wall does not fulfill passage through an existing
  wall. It can be1 when materially related to barriers/traversal; a word coincidence
  alone is0. The opposite-function flag requires actual reversal of the requested
  outcome, not merely a different purpose.

## Separate opposite-function flag

Use `contradictory:true` only for source-backed reversal/negation of the requested
function, explaining the affected recipient/outcome. A source that actively removes
breath can oppose an airless-survival request; a source that extinguishes flames
can oppose producing fire. Ordinary irrelevance or a wrong record name is not
automatically an opposite-function result. Use null when genuinely uncertain.
This flag is a diagnostic tradeoff, not an automatic model veto.

Reference candidates may be mixed into the pool without identification. Judge
every record normally; do not assume every candidate was retrieved or that an
assumed reference must receive a perfect grade.

## Evidence and output

Return every assigned ID exactly once:

```json
{
  "grades": [{
    "id": "<assigned blind ID>",
    "grade": 2,
    "contradictory": false,
    "rationale": "<source-backed fit and important limitation>",
    "evidence": [{
      "pointer": "/items/17/name",
      "quote": "<literal substring at that JSON pointer>"
    }]
  }]
}
```

The coordinator binds each assigned ID to its source file and fingerprint once.
Reviewers need not produce or repeat SHA256 hashes. `pointer` addresses a value
inside that assigned file. For a string quote a literal substring; for a nonstring
quote a substring of JavaScript `JSON.stringify(value)`, without inserting spaces.
Every resolved relevance grade needs evidence. Cite the relevant mechanics and
ownership association, not only a matching word. Quotes establish provenance;
your rationale must establish why the evidence supports the judgment.

Do not inspect model/configuration names, ranks, scores, membership counts, other
reviewers' grades, aggregate metrics or hidden answer keys. Do not browse externally,
run models/builds, install tools, modify source files or change the assigned packet.
If a rule requires evidence beyond the assigned source, report that gap rather
than inventing it. The coordinator can source-review it separately.

Validate the complete ID set, grades/flags and every pointer/literal quote
with the shared coordinator validator before reporting completion. Keep original
grades intact when correcting evidence formatting. Substantive changes require
a separate version or explicit adjudication record.

An independent overlap reviewer follows the same instructions without reading
the primary grades. The coordinator source-reviews disagreements and preserves
both original opinions. Agreement or majority vote is not proof of correctness.
