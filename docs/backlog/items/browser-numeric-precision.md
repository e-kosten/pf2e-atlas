# Browser Numeric Precision

Status: deferred
Priority: later
Owner: unassigned
Last reviewed: 2026-10-30

Define honest browser transport and display for admitted integers outside
JavaScript's safe range. Source snapshots retain exact numbers, but current
numeric presentation facts use JSON numbers and TypeScript number. For example,
9007199254740993 becomes 9007199254740992 when parsed by JavaScript. No affected
record in the supported corpus was established by the parity audit.

[PR160](https://github.com/e-kosten/pf2e-atlas/pull/160) repaired safe integer
conversion across selected browser response graphs with typed handwritten
converters. Some runtime fields are bigint after decoding; numeric facts remain
numbers. Neither representation recovers precision already lost during JSON
parsing. Compile-time coverage is distinct from runtime numeric validation and
actual endpoint-to-converter association. The repair establishes no server
numeric-string contract; a production decoder generator is not required by the
[architecture review](../history/items/architecture-simplification-review.md).

Choose a shared contract for exact text, bounded numeric values or explicit
unavailability rather than silently rounding. Account for display, editing,
filter literals, generated contracts and CLI compatibility without converting
every source value into a browser DTO. Validate both safe boundaries, negative
values, decimals and source-to-browser round trips. Query SQL already enforces
its own supported numeric precision; preserve that independent guarantee.

Related: [query catalog expansion](source-query-catalog-expansion.md),
[capability recovery](rust-integration-recovery.md).
