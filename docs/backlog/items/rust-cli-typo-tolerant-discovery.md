# Query Spelling and Name Suggestions

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-10-13

## Problem

Strict record resolution intentionally misses misspelled names, but the fallback search path is not currently useful for simple typo recovery. A misspelled query such as `Treet Wonds` can return unrelated ranked results instead of surfacing likely nearby canonical names such as `Treat Wounds`.

Offer useful spelling assistance in human CLI and browser search, while keeping canonical-name lookup reliable for agents.

Treat typo checking as shared query analysis over corpus vocabulary, with explicit suggestions available to CLI and browser consumers. Evaluate it alongside lexical attempts without making correction behavior a storage-backend responsibility.

## Desired Outcome

CLI and browser search offer deliberate spelling/name assistance while record resolution stays strict.

Potential solutions include:

- A name-suggestion mode for `record resolve` misses.
- A dedicated fuzzy name search over canonical names and verified aliases.
- Better miss handling in JSON errors, with suggested nearby records when confidence is high.
- A corpus-backed token dictionary built from canonical names, aliases, traits, conditions, actions, spells, creatures, mechanics terms, and high-signal indexed text tokens.
- Query-side typo checking that recommends corrections when strict or conjunctive FTS finds no useful hits, rather than silently rewriting the user's query.
- A small deterministic acronym expansion dictionary for common Pathfinder/RPG terms, applied as query expansion rather than replacement. Examples include `aoe` -> `area of effect`, `dc` -> `difficulty class`, `ac` -> `armor class`, and `hp` -> `hit points`.
- Evaluation of Rust libraries such as `fst` for compact corpus dictionaries with Levenshtein lookup, `strsim` for candidate reranking, or SymSpell-style crates for suggestion generation.

## Constraints

- Keep `record resolve` strict by default.
- Do not let fuzzy name behavior silently replace ranked semantic search.
- Avoid broad synonym maps or hand-maintained typo rules.
- Keep spelling/name assistance in shared product behavior with a clear vocabulary source.
- Do not auto-apply corrections by default. PF2e has many source-specific proper nouns and invented terms, so correction should generally be presented as a suggestion unless a future confidence model proves safe.
- Keep acronym expansion small, explicit, and product-owned. It should improve known abbreviations without becoming a general synonym system.

## Notes

This item is separate from general semantic search quality tuning. It is about typo recovery, acronym expansion, and high-confidence lexical assistance, not improving natural-language semantic relevance. Concept queries such as "low level spell that makes enemies afraid" should still be handled primarily by semantic or hybrid retrieval, not by fuzzy FTS.
