# Device visual reference consistency

Status: proposed
Priority: later
Owner: unassigned
Last reviewed: 2026-10-27

Investigate the macOS glyph-raster discrepancies in the maintained condition
refinement screenshots without weakening zero-pixel comparison or refreshing
references merely to pass an implementation change.

On clean main `27b6e0ae7cbd6aec88ea8d42dbdd8bc1a04f0606`, Playwright 1.62.1
reproduced failures at `precedence-editor.png`, `preview-pending.png`, and
`quickened-restrictions.png` across desktop/phone Chromium and phone/tablet WebKit.
The candidate and unchanged-main actual images were pixel-identical in all twelve
cases, with 33 rejected glyph-edge pixels total against the tracked references.
Independent visual review found no content or geometry change. Tests stop at these
checkpoints, so the remainder of each scenario still needs a passing comparison.

Check reference provenance against matching browser, font and OS inputs. Any
deliberate replacement must follow the existing rendered and independent review
workflow and retain the strict regression guard.

See [visual reference guidance](../../development/validation.md#visual-references) and
[condition refinement acceptance](../../../web/atlas-ui/e2e/devices/condition-refinement.spec.ts).
