// Durable plain-line presentation path for non-page ontology/editorial consumers.
// Structured record-page TUI consumers must go through services.user.entityPages
// and src/tui/page-document.
export {
  buildOntologyExplorerEntityDetailLines,
  buildOntologyExplorerEntitySummary,
} from "./entity-page.js";
