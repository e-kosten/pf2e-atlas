# Browser Numeric Precision

Status: deferred
Priority: later
Owner: unassigned
Last reviewed: 2026-10-27

Define honest browser transport and display for admitted integers outside
JavaScript's safe range. Source snapshots retain exact numbers, but current
numeric presentation facts use JSON numbers and TypeScript number. For example,
9007199254740993 becomes 9007199254740992 when parsed by JavaScript. No affected
record in the supported corpus was established by the parity audit.

Choose a shared contract for exact text, bounded numeric values or explicit
unavailability rather than silently rounding. Account for display, editing,
filter literals, generated contracts and CLI compatibility without converting
every source value into a browser DTO. Validate both safe boundaries, negative
values, decimals and source-to-browser round trips. Query SQL already enforces
its own supported numeric precision; preserve that independent guarantee.

Related: [query catalog expansion](source-query-catalog-expansion.md),
[capability recovery](rust-integration-recovery.md).
