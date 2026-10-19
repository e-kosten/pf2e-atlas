# Rust Web Form Accessibility Audit

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-06-23

## Problem

The web UI has grown several Ant Design forms and compact inline editable controls. Some controls rely on local visual labels, placeholders, or ad hoc layout wrappers instead of a consistent accessible labeling pattern.

This can leave forms harder to navigate with assistive technology and makes lint fixes reactive instead of systematic.

## Desired Outcome

Audit web form and editable-control usage and standardize on accessible label patterns.

The pass should cover:

- Ant Design `Form.Item` usage for modal and panel forms;
- explicit `id`/`htmlFor` associations where `Form.Item` is not appropriate;
- placeholder-only fields that need visible labels;
- compact inline editors in list, encounter, filter, and detail views;
- lint coverage for label association regressions.

## Previously reported case to recheck

A prior encounter review found duplicate `name` DOM IDs after opening/canceling
Add PC and then opening encounter settings, so clicking the visible Name label
did not focus its textbox. Reproduce against current forms before claiming it
still fails; if present, use distinct Ant Form namespaces and test focus with
both dialogs previously opened. This report is retained from the earlier runner
review, not a newly verified failure.

## Constraints

- Prefer Ant Design-native form semantics where controls already live in form contexts.
- Keep compact operational UIs dense, but do not trade away accessible names for space.
- Avoid broad visual redesign; this item is about semantic form correctness and consistency.

## Related

- [Architecture overview](../../architecture/overview.md)
- [Rust web keyboard navigation and focus](./rust-web-keyboard-navigation-focus.md)
