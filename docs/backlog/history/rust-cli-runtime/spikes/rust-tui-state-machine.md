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
- Include pane-scoped pointer interaction in the list/detail flow:
  - draggable pane sizing with explicit min/max constraints
  - mouse press/drag/release classification for click, resize, scroll, and text-selection gestures
  - mouse drag text selection bounded to the originating pane or modal region
  - auto-copy-on-release for non-empty text selections
- Model state as explicit enums and transition functions.
- Use mocked backend data, but realistic result rows and detail text.
- Test keyboard navigation, focus changes, pointer gestures, modal lifecycle, clipboard-copy outcomes, and render stability.

## Do Not Mock

- interaction state
- navigation transitions
- key handling
- mouse event handling
- modal lifecycle
- pane sizing
- text wrapping constraints
- rendered-row selection and visible-text serialization
- clipboard-copy command integration, including terminal or multiplexer failure behavior
- render/update loop behavior

## Text Selection Contract

The prototype should treat copy-oriented text selection as part of the shared TUI interaction model, not as a feature-local behavior of one screen.

The contract under test is:

- Selection starts from a mouse press in selectable rendered text.
- A press/release below the drag threshold remains a click and may focus or activate the target under the pointer.
- Movement past the threshold promotes the gesture into text selection and suppresses the click.
- Selection stays scoped to the originating pane, modal, footer, or structured region.
- Releasing a non-empty selection serializes the visible rendered text and copies it automatically.
- Wrapped, clipped, and scrolled content serializes according to the visible rendered rows, not according to unwrapped source data.
- Resize handles, overlays, explicit pointer targets, wheel scrolling, and text selection have documented precedence.
- Clipboard writes should use the Rust terminal stack's OSC 52 path where possible, such as Crossterm clipboard support, with a clear fallback state when the terminal or multiplexer does not accept the copy.

## Outputs

- state-machine design notes
- ergonomics comparison against Ink/React
- test strategy recommendation
- selection-contract recommendation for the backlog item [Mouse Text Selection for Copy/Paste](../../items/mouse-text-selection-copy-paste.md)
- decision: Ratatui primary TUI, defer TUI, or consider local web instead

## Migration Dependency

The runtime migration can start CLI-first, but replacing the current Ink workbench should wait for this spike or an explicit decision to pursue a local web workbench instead.
