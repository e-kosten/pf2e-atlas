import type { DerivedTagMigrationDecision, DerivedTagMigrationSession } from "./types.js";
import { getDerivedTagMigrationReviewItems } from "./review-session.js";
import { terminalTheme } from "./terminal-ui.js";

function describeDecision(decision: DerivedTagMigrationDecision): string {
  if (decision.kind === "assignment") {
    return `${decision.family}.${decision.tag} ${decision.mode}`;
  }
  if (decision.kind === "exemplar") {
    const current = decision.currentPolarity ? ` from ${decision.currentPolarity}` : "";
    return `${decision.tag} exemplar${current} -> ${decision.action === "drop" ? "drop" : decision.polarity}`;
  }
  return `${decision.tag} rule ${decision.decision}`;
}

function renderStatus(value: string): string {
  if (value === "approved" || value === "complete") {
    return terminalTheme.successBadge(value);
  }
  if (value === "rejected") {
    return terminalTheme.dangerBadge(value);
  }
  return terminalTheme.warningBadge(value);
}

export function renderDerivedTagMigrationSessionSummary(session: DerivedTagMigrationSession): string {
  const items = getDerivedTagMigrationReviewItems(session);
  const resolvedCount = session.decisions.filter((decision) => decision.resolutionStatus === "complete").length;
  return [
    terminalTheme.heading(`Session: ${session.manifest.id}`),
    `Mode: ${terminalTheme.accent(session.manifest.mode)}`,
    `Category: ${session.manifest.category ?? "(all)"}`,
    `Tag: ${session.manifest.tag ?? "(any)"}`,
    `Records: ${session.manifest.recordCount}`,
    `Resolved records: ${resolvedCount}/${session.decisions.length}`,
    `Visible review items: ${items.length}`,
    `Unresolved only: ${session.reviewState.unresolvedOnly ? terminalTheme.warningBadge("yes") : terminalTheme.dim("no")}`,
  ].join("\n");
}

export function renderDerivedTagMigrationReviewItem(
  session: DerivedTagMigrationSession,
  itemIndex: number,
  actionBar?: string,
): string {
  const items = getDerivedTagMigrationReviewItems(session);
  if (items.length === 0) {
    return [
      renderDerivedTagMigrationSessionSummary(session),
      "",
      "No review items matched the current filters.",
      ...(actionBar ? ["", actionBar] : []),
    ].join("\n");
  }

  const item = items[itemIndex]!;
  const recordDecision = session.decisions[item.recordIndex]!;
  const record = session.records.find((entry) => entry.recordKey === recordDecision.recordKey)!;
  const decision = recordDecision.decisions[item.decisionIndex]!;

  return [
    renderDerivedTagMigrationSessionSummary(session),
    "",
    terminalTheme.section(`Item ${itemIndex + 1}/${items.length}`),
    terminalTheme.accent(`${record.name} (${record.recordKey})`),
    `Scope: ${record.category}${record.subcategory ? `/${record.subcategory}` : ""} | level ${record.level ?? "-"}`,
    `Decision: ${describeDecision(decision)}`,
    `Status: ${renderStatus(decision.status)}`,
    `Confidence: ${"confidence" in decision ? (decision.confidence ?? "unspecified") : "n/a"}`,
    `Current tags: ${record.currentDerivedTags.join(", ") || "(none)"}`,
    `Selection: ${record.selectionReasons.map((reason) => reason.note).join(" | ") || "(none)"}`,
    `Rationale: ${decision.rationale}`,
    ...(actionBar ? ["", actionBar] : []),
  ].join("\n");
}
