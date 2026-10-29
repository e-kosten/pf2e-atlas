# Responsive UI validation

Status: done
Last reviewed: 2026-10-16

Delivered through [PR59](https://github.com/e-kosten/pf2e-atlas/pull/59): responsive
guidance, a model-free real-app browser matrix in CI, the short-landscape picker
repair and caption cleanup. Local gates, independent/Astra review and hosted CI
passed before merge. Human previews use the full-corpus dual-model artifact.
Physical-device/software-keyboard and actual browser text-zoom acceptance remain
unverified; the documented emulation/proxy boundaries still apply.

Establish repeatable device expectations and browser acceptance after the shared
UI work in PR58. Preserve the current Ant/shared-component ownership and product
interactions while checking task completion across narrow and short screens,
touch/keyboard, themes, open-reader transitions and constrained forms/overlays.

The implementation defines responsive guidance, a deterministic model-free real
app fixture and a bounded Chromium/WebKit phone/tablet/desktop suite integrated
with CI. It covers filter browsing, exact reference history, list editing/Add,
encounter recovery, pending writes and native links. Existing corpus-backed
browser exploration and semantic inference quality remain separate.

Short-landscape acceptance also requires selecting a fresh result after rotation.
The shared Add picker keeps a readable result area and scrolls its body when
filters, notices and feature controls exceed the available height.

Completion requires passing applicable gates and browser matrix, independent
source/test review and Astra visual/interaction review. Record actual device,
software-keyboard and browser-enlargement coverage separately; emulation does
not establish those outcomes. Broad accessibility remediation or new mobile
interaction designs need their own evidence and agreed scope.

See [frontend guidance](../../../../web/atlas-ui/docs/frontend-guidelines.md#responsive-design-and-acceptance)
and [contributor validation](../../../development/validation.md).
