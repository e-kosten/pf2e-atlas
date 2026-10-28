import type { AppError, AppErrorCode } from "../../generated/atlas";
import { AtlasCompatibilityError } from "../../api/compatibility";
import { AtlasConnectionError, isReferenceFailure } from "../../api/availability";

// AppError.message and Error.message are diagnostics, not reviewed display copy.
// Keep expected product failures specific without exposing response bodies, IDs,
// paths, SQL, decoder messages, or native fetch errors.
const explanations: Partial<Record<AppErrorCode, string>> = {
  record_not_found: "This record is no longer in the reference collection.",
  record_resolution_miss: "Choose another record.",
  record_resolution_ambiguous:
    "More than one record matches. Choose a specific record.",
  saved_list_not_found: "This list is no longer available. Return to Lists.",
  saved_list_already_exists:
    "A list with this name already exists. Choose another name.",
  party_not_found: "This party is no longer available.",
  party_conflict: "This party changed. Review its current values before trying again.",
  encounter_not_found: "This encounter is no longer available. Return to Encounters.",
  encounter_already_exists:
    "An encounter with this name already exists. Choose another name.",
  encounter_participant_not_found: "This participant is no longer in the encounter.",
  encounter_participant_conflict:
    "This participant changed. Review the current values before trying again.",
  encounter_turn_conflict:
    "The encounter changed. Review the current turn before trying again.",
  artifact_generation_changed:
    "The reference collection changed. Refresh before continuing.",
  filter_invalid: "Review the selected filters before searching again.",
  filter_field_invalid:
    "One of these filters is no longer available. Choose another filter.",
  filter_option_invalid:
    "One of these filter values is no longer available. Choose another value.",
  filter_field_not_applicable: "This filter does not apply to the selected records.",
  filter_editor_conflict: "The available filters changed. Refresh before continuing.",
  vector_readiness_required:
    "This search mode is unavailable. Choose another search mode.",
  embedding_model_unavailable:
    "This search model is unavailable. Choose another model.",
  service_busy: "Atlas is busy. Try again shortly.",
  setup_in_progress: "Atlas is updating its reference collection. Try again shortly.",
  operation_timeout:
    "Atlas took too long to respond. Check the current values before trying again.",
};

function appError(error: unknown): AppError | undefined {
  if (!error || typeof error !== "object") return undefined;
  const value = "appError" in error ? error.appError : error;
  return value &&
    typeof value === "object" &&
    "code" in value &&
    typeof value.code === "string"
    ? (value as AppError)
    : undefined;
}

export function isSharedFailure(error: unknown): boolean {
  return (
    error instanceof AtlasCompatibilityError ||
    error instanceof AtlasConnectionError ||
    isReferenceFailure(error)
  );
}

export function errorExplanation(error: unknown): string | undefined {
  if (error instanceof AtlasConnectionError)
    return error.write
      ? "The change could not be confirmed. Check the current values after reconnecting before trying again."
      : undefined;
  const code = appError(error)?.code;
  return code && Object.prototype.hasOwnProperty.call(explanations, code)
    ? explanations[code]
    : undefined;
}

/** Use when a local action stores display text rather than the original error. */
export function userErrorMessage(
  error: unknown,
  fallback = "Couldn’t confirm this change. Check the current values before trying again.",
): string {
  if (error instanceof AtlasCompatibilityError) return "";
  return errorExplanation(error) ?? fallback;
}
