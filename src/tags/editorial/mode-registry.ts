import type { DerivedTagWorkbenchMode } from "./types.js";
import { DERIVED_TAG_WORKBENCH } from "./types.js";

const DERIVED_TAG_WORKBENCH_MODE_SET = new Set<DerivedTagWorkbenchMode>(DERIVED_TAG_WORKBENCH.MODES);

export const DERIVED_TAG_WORKBENCH_MODE_ALIASES = {
  new_tagging: "proposal_review",
} as const;

export const DERIVED_TAG_WORKBENCH_MODE_ALIASES_LIST = Object.entries(DERIVED_TAG_WORKBENCH_MODE_ALIASES).map(
  ([alias, canonical]) => `"${alias}" -> "${canonical}"`,
);

export function parseDerivedTagWorkbenchMode(value: string | undefined): DerivedTagWorkbenchMode | undefined {
  if (value === undefined) {
    return undefined;
  }

  const normalized = value.trim();
  if (!normalized) {
    return undefined;
  }

  if (normalized in DERIVED_TAG_WORKBENCH_MODE_ALIASES) {
    return DERIVED_TAG_WORKBENCH_MODE_ALIASES[normalized as keyof typeof DERIVED_TAG_WORKBENCH_MODE_ALIASES];
  }

  return DERIVED_TAG_WORKBENCH_MODE_SET.has(normalized as DerivedTagWorkbenchMode)
    ? (normalized as DerivedTagWorkbenchMode)
    : undefined;
}
