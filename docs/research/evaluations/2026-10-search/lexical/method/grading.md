# Source-reviewer guidance

Status: revised product relevance rubric; calibration required before bulk resumption. Original rubric and opinions remain unchanged.

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

## Name lookup and subject discovery

Ordinary search text is not a strict identity resolver. Capitalization, a title-shaped
query, and an analyst's exact/partial category do not impose an exclusive-identity
requirement. Exact canonical names and verified aliases should find their record
first; that is measured separately from relevance of the surrounding results.

For a specific named item, spell or other record, the exact record is a direct
answer (3). A source-supported close alternative or variant can be useful (2),
without claiming it is the same identity. A broad category or subject query can
receive direct answers (3) from multiple kinds, including dedicated rules, family
abilities, creatures and lore. Respect explicit kind filters and explicit wording
such as creatures or only the exact item. Do not impose those restrictions otherwise.

Distinguish a record ABOUT the subject from a record that merely references or
possesses it. Incidental possession/reference alone generally supports related-only
(1), not useful discovery (2). A dedicated explanation or meaningful usable
relationship may be useful; justify that from source. Explicit requests for owners,
references or creatures with a capability change what is direct. Do not demote an
intrinsic ability merely because Foundry represents it as an embedded item.

For example, leather armor should find Leather Armor first, but Studded Leather
Armor can be a useful related alternative; a creature merely wearing Leather Armor
is not more useful solely because it contains the exact item. Harrowkin family
abilities and Harrowkin creatures can both directly answer unfiltered harrowkin.
A card-named creature can also be direct when its source identifies it as Harrowkin.
A title substring alone never proves relevance; inspect the actual subject.

For truncated names, assess plausible source-supported completions; do not invent
a single hidden target. Multiple completions can be useful. Use null for genuinely
insufficient evidence, not solely because no unique completion exists.

These are relevance principles, not a mandatory ranking-by-kind rule. Do not infer
grades from a policy's result order. Exact identity success is a separate metric.

## Relevance scale

| Grade | Meaning |
|---|---|
|3|A direct, useful answer to the actual intent. Ordinary game limitations do not automatically make an otherwise direct answer indirect.|
|2|A useful qualified or indirect answer with an important limitation; describe that limitation.|
|1|Substantively related but not useful for this intent, including incomplete or opposite-function answers. Explain the relationship and unmet intent.|
|0|Unrelated or merely coincidental vocabulary. A shared broad class alone is insufficient, but a meaningfully related alternative can be useful even when its canonical identity differs.|
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
- Another staff is not useful merely because it is a staff. A substitute needs a
  source-supported relation to the requested function or subject.
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

## Pilot calibration clarification (v2)

Grade the actual query and explicit kind filter, not an inferred evaluation category. A broad word such as fire can reasonably seek useful fire content, not only titles. Do not require hand-held needles for poison needle; a poisoned needle trap is a direct answer. Ordinary operating triggers do not automatically lower a direct answer to grade 2. Record ambiguity rather than invent restrictions. VerifiedAliases in the packet are artifact-backed identity evidence; do not infer rename equivalence from a shared icon or similar mechanics. For alias evidence, cite {document:"context",pointer:"/verifiedAliases/0/alias",quote:"..."}; other evidence still addresses the raw source record.

## Expanded-development evidence requirements

This run follows an audit that found factual mistakes despite valid quotations.
For each judgment, read the exact assigned source; do not carry capabilities between
similarly named or differently leveled variants. Read root description/public notes
before deciding what a creature/hazard is or does. Read actual mechanical descriptions
for capability claims. Names/icons alone are insufficient for those claims.

A casting assertion must cite the spell mechanics and its associated casting entry
(or an explicit exception such as Staff Mastery). A carried item is not by itself
proof its owner can use it. A negative claim must account for root prose and relevant
embedded entries; a missing snippet is not absence. If a record is long, read the
remaining relevant content before grading, and flag unresolved evidence instead of
using a title as a shortcut.

Grade the usefulness of the returned root record. A creature that contains the
requested spell is not automatically as direct an answer as the requested spell
itself. Explain why it answers the actual query; indirect usable references can
receive2. Do not invent stricter requirements than the query states, and do not
convert broad flavor queries into literal mechanical-trait requirements.

For full phrase `ghost with a mask`, a merely masked fey is related, not evidence of
a ghost. For the broader `masked spirit`, mutable masked fey can be qualified useful
flavor. A hidden-needle hazard remains a direct poison-needle answer even if the
needle is described in root details and the reaction is called Purple Worm Sting.

Process small batches; write each finished batch before continuing. Every resolved
entry must have a nonempty literal supporting quote. Validate each batch's IDs and
quotations. Retain uncertainty and rationales; do not force uniform judgments.

## Expanded calibration correction

The first expanded batches are provisional, not accepted quality evidence. Before
resuming bulk work, audit them in a separate corrected file. Preserve originals.

Inspect the complete root `system` as well as prose. Immunities, weaknesses,
resistances, movement and defenses may be structured fields with no prose mention.
Use source-view.ts SOURCE root, then items/pages manifests and relevant individual
entries to read pointer-addressed values without terminal truncation. This is a
navigation aid, not a replacement source or a query-dependent extract. If a claim
requires checking absent capabilities, inspect all plausibly relevant entries;
otherwise use null rather than unsupported absence claims.

A broad category query does not exclude specific instances: a particular snare can
be a direct answer to snare, and a magical mirror can directly answer mirror. Do not
add an unspoken generic-only requirement. For exact item identity queries, an owner
merely carrying that item is not the requested item record. Use grade1 for substantive
possession context alone; grade2 requires an actual useful indirect answer justified
by the query. Apply this consistently across armor, potions, spells and equipment.

A necessary ingredient for an imagined combination is not a useful answer to the
requested combined function without source support. Suppressing noise alone does
not provide an alarm. Inspect recipients and qualifiers for healing versus harming;
related opposite-function evidence normally belongs to grade1, not useful grade2.

Capability grades need capability evidence, including structured fields. Name-only
quotations are sufficient only when identity/category itself answers the query.
Literal quotation validation does not establish correctness of the judgment.

## Entity requests and source representations

When the query explicitly requests an entity (for example a creature with a
capability), that requirement applies even without a kind filter. A spell, feat or
effect that could give some unspecified creature the capability is related content,
not an identified creature satisfying the request. Do not grant useful-answer credit
merely for a hypothetical combination. This differs from a query asking for a means
to achieve a function, where multiple record kinds can directly answer it.

Read spell association according to source representation. Rituals have their own
checks and casting rules and need not have a normal prepared/spontaneous casting
entry. A null ritual location alone is not evidence that the ritual is unavailable.
Likewise distinguish constant personal effects from granting an effect to others.
If the source cannot establish a required distinction, state uncertainty rather than
inventing a rule. These are reading safeguards, not requirements for full runtime
simulation or proof that every resource is currently available.
