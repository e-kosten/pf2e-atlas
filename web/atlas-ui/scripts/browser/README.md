# Record disclosure Safari regression

Use `assert-disclosure-visible.js` in the exact corrected, owned Safari preview document after an actual pointer or keyboard action. The file is a browser function expression. Remove its final semicolon, append `({ label: "exact header", expectedText: "expected body text" })`, then evaluate the combined source using Safari's JavaScript tool. Do not substitute jsdom geometry or `aria-expanded` for these results.

For both an H5 occurrence (False Door's Purple Worm Venom or Torchbearer's occurrence) and an ordinary standalone detail disclosure (Source & provenance):

1. Record the immutable commit/tree, executable and served UI hashes, preview URL, viewport and theme.
2. Open with the pointer. Run the probe using the current exact header label (including summary facts) and an expected sentence/value from that disclosure's API-backed body. Capture the returned evidence and screenshot.
3. Close, Tab to the same header and activate with Enter. Run the same probe and capture evidence. Confirm focus stays on the header and closing works.
4. For a sensitivity check in that dedicated test tab, temporarily set the opened content element's inline height to `0px` and opacity to `0`. Run the probe and require failure despite `aria-expanded="true"`; restore the original inline style immediately. This changes only the inspected DOM, not application or saved state.

The probe waits for settling, then requires nonzero real geometry, visible body text, visible ancestors and absence of a stuck motion start class. Repeat for occurrence and ordinary detail disclosures. Retain missing/unsupported fixture expectations; do not invent body content to satisfy the probe. Fixed-fixture checks do not establish empty/loading/error behavior. Independent147 must repeat real pointer and keyboard checks against the sealed corrected candidate.
