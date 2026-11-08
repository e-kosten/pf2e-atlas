import { AtlasApiError } from "../../api/apiError";

export function savedEditConflictFields(error: unknown): string[] | null {
  if (
    !(error instanceof AtlasApiError) ||
    error.appError?.code !== "saved_edit_conflict"
  ) {
    return null;
  }
  const details = error.appError.details;
  if (
    !details ||
    typeof details !== "object" ||
    !("fields" in details) ||
    !Array.isArray(details.fields)
  ) {
    return null;
  }
  const fields = details.fields.filter(
    (field): field is string => typeof field === "string" && field.length > 0,
  );
  return fields.length ? fields : null;
}
