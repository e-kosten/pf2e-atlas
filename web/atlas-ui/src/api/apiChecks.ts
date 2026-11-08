import type {
  ConditionParametersView,
  EncounterDetailView,
  EncounterTurnReviewView,
  FilterCountsView,
  FilterValueListView,
  PersistentDamageGroupView,
  SearchPageView,
  SearchResultsPage,
} from "../generated/http";
import { AtlasApiError } from "./apiError";
import { isRecord } from "./atlasTransport";

// Orval serializes bodies before calling the Fetch hook.
export function checkRequest<T>(value: T): T {
  if (
    typeof value === "bigint" ||
    (typeof value === "number" &&
      (!Number.isFinite(value) ||
        (Number.isInteger(value) && !Number.isSafeInteger(value))))
  ) {
    throw new AtlasApiError(0, "Request numeric field exceeds JSON safe numeric range");
  }
  if (Array.isArray(value)) value.forEach(checkRequest);
  else if (isRecord(value)) Object.values(value).forEach(checkRequest);
  return value;
}

function checkInteger(value: unknown): void {
  if (!Number.isSafeInteger(value))
    throw new AtlasApiError(200, "Invalid numeric field in API response");
}

export function checkSearchPage<T extends { page: SearchPageView }>(value: T): T {
  if (!isRecord(value) || !isRecord(value.page))
    throw new AtlasApiError(200, "Invalid search-page response");
  checkInteger(value.page.total);
  return value;
}

export function checkSearchResults(value: SearchResultsPage): SearchResultsPage {
  checkSearchPage(value);
  if (Array.isArray(value.rows)) {
    for (const row of value.rows) {
      if (!isRecord(row) || !Array.isArray(row.relationship_evidence)) continue;
      for (const evidence of row.relationship_evidence) checkSearchPage(evidence);
    }
  }
  return value;
}

function checkOptionCounts(value: {
  total_values: number;
  options: { distinct_roots: number }[];
}): void {
  checkInteger(value.total_values);
  if (Array.isArray(value.options)) {
    for (const option of value.options) checkInteger(option.distinct_roots);
  }
}

export function checkFilterCounts(value: FilterCountsView): FilterCountsView {
  if (value.counts.status === "available") checkOptionCounts(value.counts.value);
  return value;
}

export function checkFilterValues(value: FilterValueListView): FilterValueListView {
  if (isRecord(value)) {
    if (value.kind === "vocabulary" && isRecord(value.values)) {
      checkInteger(value.values.total_values);
      return value;
    }
    if (value.kind === "contextual" && isRecord(value.outcome)) {
      if (value.outcome.status === "unavailable") return value;
      if (value.outcome.status === "available" && isRecord(value.outcome.value)) {
        checkOptionCounts(value.outcome.value);
        return value;
      }
    }
  }
  throw new AtlasApiError(200, "Invalid filter-value response");
}

export function checkEncounterResponse<T extends object>(value: T): T {
  if (!isRecord(value)) throw new AtlasApiError(200, "Invalid encounter response");
  return value;
}

function checkConditionParameters(value: ConditionParametersView): void {
  if (!isRecord(value)) throw new AtlasApiError(200, "Invalid condition parameters");
  if (value.kind === "persistent_damage") {
    if (!isRecord(value.recovery))
      throw new AtlasApiError(200, "Invalid persistent recovery");
    if ("dc" in value.recovery) checkInteger(value.recovery.dc);
  }
}

export function checkPersistentGroup(value: PersistentDamageGroupView): void {
  if (!isRecord(value) || !Array.isArray(value.sources))
    throw new AtlasApiError(200, "Invalid persistent damage group");
  checkInteger(value.selected_source_id);
  for (const source of value.sources) {
    if (!isRecord(source))
      throw new AtlasApiError(200, "Invalid persistent damage source");
    if (source.condition_id !== null) checkInteger(source.condition_id);
    if (source.recovery_dc !== null) checkInteger(source.recovery_dc);
  }
}

export function checkEncounterDetail(value: EncounterDetailView): EncounterDetailView {
  checkEncounterResponse(value);
  if (!Array.isArray(value.participants))
    throw new AtlasApiError(200, "Invalid encounter participants");
  if (!isRecord(value.encounter))
    throw new AtlasApiError(200, "Invalid encounter summary");
  for (const participant of value.participants) {
    if (!isRecord(participant) || !Array.isArray(participant.conditions))
      throw new AtlasApiError(200, "Invalid encounter participant");
    if (!Number.isSafeInteger(participant.revision) || participant.revision < 0)
      throw new AtlasApiError(200, "Invalid participant revision in API response");
    for (const field of [
      participant.initiative,
      participant.current_hp,
      participant.max_hp,
      participant.level_override,
      participant.base_death_threshold,
    ]) {
      if (field !== undefined) checkInteger(field);
    }
    for (const condition of participant.conditions) {
      if (!isRecord(condition))
        throw new AtlasApiError(200, "Invalid encounter condition");
      if (condition.parameters !== undefined)
        checkConditionParameters(condition.parameters);
      if (condition.duration !== undefined && !isRecord(condition.duration))
        throw new AtlasApiError(200, "Invalid condition duration");
    }
    if (Array.isArray(participant.persistent_damage_groups))
      participant.persistent_damage_groups.forEach(checkPersistentGroup);
    if (Array.isArray(participant.ability_states)) {
      for (const ability of participant.ability_states) {
        if (!isRecord(ability) || !isRecord(ability.state))
          throw new AtlasApiError(200, "Invalid ability state");
      }
    }
  }
  return value;
}

export function checkTurnReview(
  value: EncounterTurnReviewView,
): EncounterTurnReviewView {
  checkEncounterResponse(value);
  for (const turn of [value.outgoing, value.incoming]) {
    if (turn == null) continue;
    if (
      !isRecord(turn) ||
      !Array.isArray(turn.reminders) ||
      !Array.isArray(turn.persistent_damage_groups)
    )
      throw new AtlasApiError(200, "Invalid turn boundary");
    checkInteger(turn.source_revision);
    checkInteger(turn.round_number);
    for (const reminder of turn.reminders) {
      if (!isRecord(reminder)) throw new AtlasApiError(200, "Invalid turn reminder");
      if (reminder.kind === "tracker_elapsed") continue;
      if (reminder.kind === "condition_recovery") {
        if (!Array.isArray(reminder.condition_ids))
          throw new AtlasApiError(200, "Invalid condition recovery reminder");
        reminder.condition_ids.forEach(checkInteger);
      } else {
        checkInteger(reminder.condition_id);
      }
    }
    turn.persistent_damage_groups.forEach(checkPersistentGroup);
  }
  return value;
}
