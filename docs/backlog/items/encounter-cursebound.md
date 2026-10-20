# Cursebound encounter mechanics

Status: deferred

Deferred by user decision from CF-07. Atlas primarily supports GMs running in-person
encounters; player-focused curse mechanics have lower demonstrated value for that
workflow. This is a prioritization decision, not a claim that NPC use is impossible.

Remove Cursebound from the standard encounter condition picker while deferred;
retain its searchable reference page.
Before implementation, establish concrete GM demand and choose supported sources.
The PF2e 6.12.4 audit identified eleven remaster curse profiles; legacy staged curses
are different effects and must not be inferred from matching display names.

Future scope may include typed source identity, threshold effects, grants, contextual
protection/healing changes and Refocus/consciousness lifecycle. Reuse the shared
condition engine; do not add a parallel class-feature engine or arbitrary Foundry
rule interpreter. Resolve prose/runtime differences and the non-mitigation policy.

Research provenance: design/condition-system, Atlas baseline 1482b02d,
PF2e pin 4cbdaa37d6c33e9519561bae2c59a23e0288cbce;
local evidence `scratch/condition-system/curse-sources.md`.
Related: [CF-07](rust-integration-recovery.md#cf-07--complete-encounter-condition-system).
