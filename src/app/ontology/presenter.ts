// Quarantined line-detail compatibility seam. Structured record-page TUI
// consumers must go through services.user.entityPages and src/tui/page-document.
export {
  buildEntityPageDocument,
  buildOntologyExplorerEntityDetailLines,
  buildOntologyExplorerEntitySummary,
  renderEntityPageDocument,
} from "./entity-page.js";
