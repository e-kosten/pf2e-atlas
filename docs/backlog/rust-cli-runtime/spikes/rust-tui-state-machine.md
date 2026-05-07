# Rust TUI State Machine Spike

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-05-06

## Question

Can Ratatui-style explicit state machines model the important PF2e Atlas TUI workflows with better correctness and acceptable development cost?

## Motivation

The current terminal/editorial workflows have complex navigation, list/detail panes, prompts, modals, structured search editing, and review state. Rust may improve these by making invalid interaction states harder to represent, but the development cost must be tested.

## Prototype Scope

- Build a small Rust TUI prototype with one list/detail search flow.
- Add one modal or prompt workflow.
- Model state as explicit enums and transition functions.
- Use mocked backend data, but realistic result rows and detail text.
- Test keyboard navigation, focus changes, modal lifecycle, and render stability.

## Do Not Mock

- interaction state
- navigation transitions
- key handling
- modal lifecycle
- pane sizing
- text wrapping constraints
- render/update loop behavior

## Outputs

- state-machine design notes
- ergonomics comparison against Ink/React
- test strategy recommendation
- decision: Ratatui primary TUI, defer TUI, or consider local web instead

## Migration Dependency

The runtime migration can start CLI-first, but replacing the current Ink workbench should wait for this spike or an explicit decision to pursue a local web workbench instead.
