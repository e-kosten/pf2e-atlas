import type { DerivedTagMigrationDecision, DerivedTagMigrationSession } from "./types.js";
import { getDerivedTagMigrationReviewItems } from "./review-session.js";

function describeDecision(decision: DerivedTagMigrationDecision): string {
  if (decision.kind === "assignment") {
    return `${decision.family}.${decision.tag} ${decision.mode}`;
  }
  if (decision.kind === "exemplar") {
    return `${decision.tag} exemplar ${decision.polarity} ${decision.action}`;
  }
  return `${decision.tag} rule ${decision.decision}`;
}

export function renderDerivedTagMigrationSessionSummary(session: DerivedTagMigrationSession): string {
  const items = getDerivedTagMigrationReviewItems(session);
  const resolvedCount = session.decisions.filter((decision) => decision.resolutionStatus === "complete").length;
  return [
    `Session: ${session.manifest.id}`,
    `Mode: ${session.manifest.mode}`,
    `Category: ${session.manifest.category ?? "(all)"}`,
    `Tag: ${session.manifest.tag ?? "(any)"}`,
    `Records: ${session.manifest.recordCount}`,
    `Resolved records: ${resolvedCount}/${session.decisions.length}`,
    `Visible review items: ${items.length}`,
    `Unresolved only: ${session.reviewState.unresolvedOnly ? "yes" : "no"}`,
  ].join("\n");
}

export function renderDerivedTagMigrationReviewItem(
  session: DerivedTagMigrationSession,
  itemIndex: number,
): string {
  const items = getDerivedTagMigrationReviewItems(session);
  if (items.length === 0) {
    return `${renderDerivedTagMigrationSessionSummary(session)}\n\nNo review items matched the current filters.`;
  }

  const item = items[itemIndex]!;
  const recordDecision = session.decisions[item.recordIndex]!;
  const record = session.records.find((entry) => entry.recordKey === recordDecision.recordKey)!;
  const decision = recordDecision.decisions[item.decisionIndex]!;

  return [
    renderDerivedTagMigrationSessionSummary(session),
    "",
    `Item ${itemIndex + 1}/${items.length}`,
    `${record.name} (${record.recordKey})`,
    `Scope: ${record.category}${record.subcategory ? `/${record.subcategory}` : ""} | level ${record.level ?? "-"}`,
    `Decision: ${describeDecision(decision)}`,
    `Status: ${decision.status}`,
    `Confidence: ${"confidence" in decision ? (decision.confidence ?? "unspecified") : "n/a"}`,
    `Current tags: ${record.currentDerivedTags.join(", ") || "(none)"}`,
    `Selection: ${record.selectionReasons.map((reason) => reason.note).join(" | ") || "(none)"}`,
    `Rationale: ${decision.rationale}`,
    "",
    "Commands: [a] approve  [r] reject  [n] needs_review  [j] next  [k] previous  [t] toggle unresolved  [q] save+quit",
  ].join("\n");
}
