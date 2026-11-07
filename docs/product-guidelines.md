# Product presentation

These rules apply to user-visible behavior and copy, including text projected by
Rust app-service into web and CLI responses. Atlas supports GM decisions at the
table; rules reminders must leave outcomes resolved at the table usable.

For browser implementation, read the [frontend guidance](../web/atlas-ui/AGENTS.md)
and its [design principles](../web/atlas-ui/docs/frontend-guidelines.md#design-principles).

## Product UI Controls

- Default to a direct action, then inline or anchored context for small frequent
  tasks. Reserve modals for substantial editing or decisions that require focus.
  Keep the pointer, keyboard focus and reading position near the initiating control.
- Rules reminders support play; they must not become mandatory completion checklists.
  Accept outcomes resolved at the table without demanding a raw die result or
  creating fictitious roll history. Optional rolls use the shared dice surface.
- Keep condition identity and essential mechanics visible. Use disclosures only
  for genuinely secondary details such as provenance or additional sources.
- Show predictable consequences and supported opt-outs before the first submission.
  One task has one submission; moving a second step into the same modal is not a
  single-step flow. Compute previews while editing rather than after clicking Apply.
- Prevent invalid input at the control. Disabled actions need an understandable,
  actionable reason; never strand the user behind an unexplained disabled button.
- Adjacent buttons in the same action group must use the same size and aligned
  height. Express primary/secondary importance with variant or emphasis, not a
  smaller secondary button. Dense inline actions may use small buttons only when
  the whole group uses that size. Align adjacent inputs and buttons consistently.
- Keep encounter workflows visually stable. Do not use expanding disclosures as
  the default container for unrelated controls or introduce animated layout shifts
  around frequently used actions. Place contextual information beside its action;
  choose an overlay only for a focused task that needs one.

Browser error changes must follow the [error feedback and recovery policy](../web/atlas-ui/docs/frontend-guidelines.md#error-feedback-and-recovery):
one owner per shared failure, reviewed task-specific copy, local persistent action
feedback, preserved drafts and truthful save/retry outcomes. Read that policy
before changing API error rendering or recovery behavior.

## Legacy and remaster context

Show a selected entry's supported remaster relationship as a compact inline
sentence above its rules, wrapping naturally on narrow screens. Display each name
once and link only actual available entries. Keep source section qualifiers and
generic mechanics-change annotations in the linked explanation rather than
appending them to relationship notices. Combine supporting source statements
into readable logical relationships without merging different scopes or inventing
record identity. Place a compact information action beside the reading context to
open a readable change explanation in the reference drawer, deduplicating repeated
citations to the same journal page. Use the shared
[inline context action treatment](../web/atlas-ui/docs/frontend-guidelines.md#inline-context-actions)
for text-adjacent information controls. Keep software-provider
labels, internal revisions and diagnostic evidence out of ordinary presentation.

Explore Versions appears for available counterparts or suggested variants, with
a visible Compare button for supported pairs. A former name or explanation link
alone does not warrant that tab. Omit the reader notice throughout Explore; each
tab owns its content. In Versions, place comparison actions below their relationship
line. Comparison columns retain their own identities and rules without reciprocal
notices. Technical provenance belongs in developer diagnostics rather than these reader interactions.

Search and discovery show every independently matching legacy and remaster entry.
Counterpart links explain relationships and enable comparison; they do not hide
results. Publication-era badges alone do not establish a replacement. Label an owned
entry's origin information as source context without substituting its rules for
the selected content. [ADR0067](architecture/decisions/0067-remaster-relationships.md)
owns relationship authority and consumer policy.

## Product UI Copy

These rules apply to all user-visible copy, including text projected by Rust into
web and CLI responses.

- Prefer icons for familiar compact actions when their meaning is clear in context,
  with accessible names and concise hover/focus tooltips. This is a contextual
  preference, not an icons-only rule: use text when it improves comprehension,
  distinguishes choices, or makes an unfamiliar action clear.
- Prefer no helper text when the label, value, control or visible state already
  communicates the behavior. Do not add a sentence merely to explain the feature.
- Every note must give a concrete fact, consequence or useful next action. Avoid
  vague phrases such as “needs manual handling,” “resolve explicitly,” or
  “unsupported automation.” State the actual restriction or action when known;
  do not invent one when the source does not establish it.
- Do not narrate implementation boundaries or add blanket automation disclaimers.
  Surface a limitation only where it changes how the user should interpret a
  displayed value or perform the current task.
- Keep essential status beside the affected fact or control. Put useful optional
  calculation details in its existing explanation popover. Do not create a generic
  notes section or move redundant prose into tooltips merely to hide it.
- Review the literal rendered copy before sharing a preview. Delete repetition
  first, then use short, direct wording. Short but vague copy is not an improvement.

For example, omit “Action loss is shown in the action budget; spending actions is
tracked at the table.” The displayed forecast and its explanation already cover
that distinction. Omit “The additional action's permitted uses need manual
handling.” Show a specific restriction such as “Stride or Strike only” only when
supported by the granting effect.
