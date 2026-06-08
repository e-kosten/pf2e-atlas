# Rust Web Filter State Policy Hardening

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-06-07

## Problem

The web UI filter state helpers now support tri-state include/exclude behavior through backend-derived operator policy, but `setValuesForField` still has a compatibility fallback when callers omit that policy. Today the production tri-state picker passes policy derived from `FilterEditorFieldView.default_operator` and `allowed_operators`, so behavior is correct for the current UI path.

The remaining risk is future frontend callers using `setValuesForField(search, fieldId, values)` directly and silently falling back to `include_any`, bypassing backend-owned operator semantics.

## Desired Outcome

Make option-field value mutation require an explicit backend-derived operator policy, or split helpers so only field types with fixed semantics can use policy-free convenience functions.

The follow-up should:

- audit direct `setValuesForField` callers;
- make backend-derived operator policy mandatory for option-field mutation paths;
- preserve simple boolean/range/metric helpers where field semantics are already fixed;
- keep `FilterClause` as the authored UI search-state shape;
- avoid reintroducing field-specific include/exclude buckets.

## Constraints

- Backend/app-service remains the owner of field operator policy.
- Frontend code may derive a small policy object from generated `FilterEditorFieldView`, but must not hard-code product-specific operator support.
- Do not broaden this into backend filter discovery changes unless a concrete UI caller requires it.

## Related

- [Architecture overview](../../architecture/overview.md)
- [Rust web filter UX expansion](./rust-web-filter-ux-expansion.md)
