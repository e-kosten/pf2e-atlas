# Code and refactor guidance

## Modules and entry points

This codebase is Rust. Keep modules single-purpose and put helpers beside the concern that owns their policy. Avoid generic `utils.rs` or `helpers.rs` modules. Promote shared helpers only when at least two existing owners need the same stable primitive and there is no clearer domain owner.

For crate ownership, read the [architecture overview](../architecture/overview.md).
For private TypeScript package layout and developer-command boundaries, read
[ADR0033](../architecture/decisions/0033-developer-command-surfaces.md).

Use `cargo fmt` for formatting. Keep Clippy clean under the workspace gate.

Keep crate entry points focused on composition:

- `lib.rs` should declare modules, expose the crate's intentional public API with explicit re-exports, and carry only small crate-level glue that has no clearer module owner. Put public model definitions, parsing, projection, storage, and behavior logic in named modules such as `model.rs`, `reader.rs`, `presentation.rs`, or domain-specific submodules, then re-export the public surface from `lib.rs`.
- `main.rs` should wire CLI/runtime entry behavior and delegate command logic to modules. Avoid accumulating command implementations, business logic, or reusable helpers in `main.rs`.
- When a crate root grows large, inspect whether it is large because of public API exports or because it owns real definitions/logic. Prefer moving definitions and logic to cohesive modules while preserving a clear public API at the root.

## Validation ownership and test seams

- Before adding a runtime check, identify the input boundary, the invariant and
  the failure the check prevents. Reuse the owning parser or constructor and
  pass its checked value through the same immutable context. For example,
  search can use a prepared embedding after admitting its model against the
  current read snapshot. It still checks that the requested text and profile
  match the prepared query.
- Check again when new input arrives or when a different snapshot, persisted
  value or transaction can invalidate the earlier guarantee. Ingest validates
  the admission summary's internal counts once and separately checks that the
  summary matches the publication's actual root count. Explicit artifact audits
  inspect persisted data and can detect corruption after publication.
- Exercise real entry points in tests. Keep fixture builders, alternate
  comparison implementations and test controls in test code or behind
  test-only compilation. Before adding a seam to a production path, identify
  the specific regression the normal entry point cannot exercise and use the
  narrowest seam that reaches it. Preserve independent expected results and
  deliberate malformed-input cases.

## Refactor Completion Policy

Refactors must land as finished end-state changes, not as transitional scaffolding.

- Default to removing the old path outright and updating all call sites in the same task.
- Do not add compatibility layers, shim modules, fallback code paths, version bridges, or adapter wrappers unless the user explicitly asks for an incremental strategy.
- Do not treat a refactor as complete if it leaves behind intermediate shims, compatibility wrappers, transitional files, partial migrations, or mixed old-and-new implementations across the codebase.
- Validation for refactor work must explicitly check for leftover uses of the old implementation, leftover shim files, and any other signs of an intermediate state.
