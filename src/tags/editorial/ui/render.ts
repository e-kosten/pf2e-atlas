import type { DerivedTagReviewDecision, DerivedTagReviewSession } from "../types.js";
import { getCurrentDerivedTagAuthoredState } from "../state/authored-state.js";
import { getDerivedTagReviewItems, summarizeDerivedTagReviewProgress } from "../sessions/review-session.js";
import { buildDerivedTagMigrationRecordPageTextLines } from "./review-detail-content.js";

function describeDecision(decision: DerivedTagReviewDecision): string {
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
    category === "affliction" ||
    category === "creature" ||
    category === "equipment" ||
    category === "hazard" ||
    category === "spell"
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

  const state = getCurrentDerivedTagAuthoredState();
  const assignment = state.assignments[managedCategory].find((entry) => entry.recordKey === recordKey);
  if (!assignment) {
    return "(none)";
  }

  const renderedApplied = Object.entries(assignment.applied ?? {}).flatMap(([family, decisions]) =>
    decisions.map((decision) => `${family}.${decision.tag}`),
  );
  const renderedExcluded = Object.entries(assignment.excluded ?? {}).flatMap(([family, decisions]) =>
    decisions.map((decision) => `!${family}.${decision.tag}`),
  );
  const rendered = [...renderedApplied, ...renderedExcluded];
  return rendered.length > 0 ? rendered.join(", ") : "(none)";
}

function renderAssignmentMemory(category: string, recordKey: string): string {
  const managedCategory = toManagedCategory(category);
  if (!managedCategory) {
    return "(n/a)";
  }

  const state = getCurrentDerivedTagAuthoredState();
  const decisions = state.assignmentMemory[managedCategory].decisions
    .filter((decision) => decision.recordKey === recordKey)
    .map((decision) => `${decision.mode === "exclude" ? "!" : ""}${decision.family}.${decision.tag}`);

  return decisions.length > 0 ? decisions.join(", ") : "(none)";
}

function renderStatus(value: string): string {
  return value;
}

export function renderDerivedTagReviewSessionSummary(session: DerivedTagReviewSession): string {
  const progress = summarizeDerivedTagReviewProgress(session);
  const actionableSummary =
    progress.actionableRecordCount > 0
      ? `Actionable records resolved: ${progress.resolvedActionableRecordCount}/${progress.actionableRecordCount}`
      : "Actionable review items: 0";
  return [
    `Session: ${session.manifest.id}`,
    `Mode: ${session.manifest.mode}`,
    `Category: ${session.manifest.category ?? "(all)"}`,
    `Tag: ${session.manifest.tag ?? "(any)"}`,
    `Candidate records: ${progress.candidateRecordCount}`,
    actionableSummary,
    `Visible review items: ${progress.visibleItemCount}`,
    `Unresolved only: ${session.reviewState.unresolvedOnly ? "yes" : "no"}`,
  ].join("\n");
}

export function renderDerivedTagReviewItem(
  session: DerivedTagReviewSession,
  itemIndex: number,
  actionBar?: string,
): string {
  const itemsForRender = getDerivedTagReviewItems(session);
  if (itemsForRender.length === 0) {
    return [
      renderDerivedTagReviewSessionSummary(session),
      "",
      "No review items matched the current filters.",
      ...(actionBar ? ["", actionBar] : []),
    ].join("\n");
  }

  const item = itemsForRender[itemIndex]!;
  const recordDecision = session.decisions[item.recordIndex]!;
  const record = session.records.find((entry) => entry.entityRecord.recordKey === recordDecision.recordKey)!;
  const entityRecord = record.entityRecord;
  const decision = recordDecision.decisions[item.decisionIndex]!;

  return [
    renderDerivedTagReviewSessionSummary(session),
    "",
    `Item ${itemIndex + 1}/${itemsForRender.length}`,
    `${entityRecord.name} (${entityRecord.recordKey})`,
    `Scope: ${entityRecord.category}${entityRecord.subcategory ? `/${entityRecord.subcategory}` : ""} | level ${entityRecord.level ?? "-"}`,
    `Decision: ${describeDecision(decision)}`,
    `Status: ${renderStatus(decision.status)}`,
    `Confidence: ${"confidence" in decision ? (decision.confidence ?? "unspecified") : "n/a"}`,
    `Live assignments: ${renderLiveAssignments(entityRecord.category, entityRecord.recordKey)}`,
    `Rejected memory: ${renderAssignmentMemory(entityRecord.category, entityRecord.recordKey)}`,
    `Selection: ${record.selectionReasons.map((reason) => reason.note).join(" | ") || "(none)"}`,
    `Rationale: ${decision.rationale}`,
    "",
    ...buildDerivedTagMigrationRecordPageTextLines(record),
    ...(actionBar ? ["", actionBar] : []),
  ].join("\n");
}
