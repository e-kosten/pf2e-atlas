import type { FilterEditorFieldView } from "../../generated/atlas";

/** Format the display label without changing the literal used by the filter. */
export function filterValueLabel(field: FilterEditorFieldView, value: string): string {
  const choice = field.choices.find((entry) => String(entry.value) === value);
  if (choice) return choice.label;
  if (field.definition.path !== "physical.base_type") return value;
  return value
    .split(/[-_]/)
    .map((word) => word.charAt(0).toLocaleUpperCase("en") + word.slice(1))
    .join(" ");
}
