# Additional CLI Content Formats

Status: deferred

## Current Surface

Human record reading supports summary, standard and full detail. Record JSON
returns the complete selected Atlas reference response. A separate source command
exports the checked Foundry DTO and provenance; developer original-file inspection
reads the configured source checkout with hash verification.

Prepared sanitized HTML is the shared display input, with terminal rendering
performed at read time. These surfaces follow
[ADR 0047](../../architecture/decisions/0047-semantic-record-presentation.md).

## Open Question

Identify a concrete consumer need before adding another content format. Possible
needs include selected prepared HTML, authored markup fragments, or plain text
for shell workflows. Decide what information the existing reference/source
responses lack and how callers would request the additional output.

Keep the human detail axis, reference JSON contract, source inspection and any new
format clearly distinguished. Use the existing checked source and preparation
outputs when they satisfy the request. This item selects no new format or model.
