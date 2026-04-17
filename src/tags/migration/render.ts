import type { DerivedTagMigrationDecision, DerivedTagMigrationSession } from "./types.js";
import { getCurrentDerivedTagMigrationAuthoredState } from "./authored-state.js";
import { getDerivedTagMigrationReviewItems } from "./review-session.js";

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

function toManagedCategory(category: string): "affliction" | "creature" | "equipment" | "hazard" | "spell" | null {
  if (
    category === "affliction"
    || category === "creature"
    || category === "equipment"
    || category === "hazard"
    || category === "spell"
  ) {
    return category;
  }
  return null;
}

function renderLiveAssignments(category: string, recordKey: string): string {
  const managedCategory = toManagedCategory(category);
  if (!managedCategory) {
    return "(n/a)";
  }

  const state = getCurrentDerivedTagMigrationAuthoredState();
  const assignment = state.assignments[managedCategory].find((entry) => entry.recordKey === recordKey);
  if (!assignment) {
    return "(none)";
  }

  const renderedApplied = Object.entries(assignment.applied ?? {})
    .flatMap(([family, decisions]) => decisions.map((decision) => `${family}.${decision.tag}`));
  const renderedExcluded = Object.entries(assignment.excluded ?? {})
    .flatMap(([family, decisions]) => decisions.map((decision) => `!${family}.${decision.tag}`));
  const rendered = [...renderedApplied, ...renderedExcluded];
  return rendered.length > 0 ? rendered.join(", ") : "(none)";
}

function renderAssignmentMemory(category: string, recordKey: string): string {
  const managedCategory = toManagedCategory(category);
  if (!managedCategory) {
    return "(n/a)";
  }

  const state = getCurrentDerivedTagMigrationAuthoredState();
  const decisions = state.assignmentMemory[managedCategory].decisions
    .filter((decision) => decision.recordKey === recordKey)
    .map((decision) => `${decision.mode === "exclude" ? "!" : ""}${decision.family}.${decision.tag}`);

  return decisions.length > 0 ? decisions.join(", ") : "(none)";
}

function renderStatus(value: string): string {
  return value;
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
    `Item ${itemIndex + 1}/${items.length}`,
    `${record.name} (${record.recordKey})`,
    `Scope: ${record.category}${record.subcategory ? `/${record.subcategory}` : ""} | level ${record.level ?? "-"}`,
    `Decision: ${describeDecision(decision)}`,
    `Status: ${renderStatus(decision.status)}`,
    `Confidence: ${"confidence" in decision ? (decision.confidence ?? "unspecified") : "n/a"}`,
    `Current tags: ${record.currentDerivedTags.join(", ") || "(none)"}`,
    `Live assignments: ${renderLiveAssignments(record.category, record.recordKey)}`,
    `Rejected memory: ${renderAssignmentMemory(record.category, record.recordKey)}`,
    `Selection: ${record.selectionReasons.map((reason) => reason.note).join(" | ") || "(none)"}`,
    `Rationale: ${decision.rationale}`,
    ...(actionBar ? ["", actionBar] : []),
  ].join("\n");
}
