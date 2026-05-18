# Rust CLI Typo Tolerant Discovery

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-05-18

## Problem

Strict record resolution intentionally misses misspelled names, but the fallback search path is not currently useful for simple typo recovery. A misspelled query such as `Treet Wonds` can return unrelated ranked results instead of surfacing likely nearby canonical names such as `Treat Wounds`.

This is not a short-term blocker for agent workflows because agents usually issue copied or canonical names. It is still worth tracking for human CLI use and for resilience when user-provided names contain small mistakes.

## Desired Outcome

The CLI offers a deliberate typo-tolerant discovery path without weakening strict resolution semantics.

Potential solutions include:

- A name-suggestion mode for `record resolve` misses.
- A dedicated fuzzy name search over canonical names and verified aliases.
- Better miss handling in JSON errors, with suggested nearby records when confidence is high.

## Constraints

- Keep `record resolve` strict by default.
- Do not let fuzzy name behavior silently replace ranked semantic search.
- Avoid broad synonym maps or hand-maintained typo rules.

## Notes

This item is separate from general semantic search quality tuning. It is about canonical-name and alias recovery, not improving natural-language relevance.
