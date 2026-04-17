import { normalizeText } from "../utils.js";

export type FilterValueOrdering =
  | { kind: "alpha" }
  | { kind: "countDescThenAlpha" }
  | { kind: "numericAsc" }
  | { kind: "canonical"; order: readonly string[]; unknowns?: "tailAlpha" | "headAlpha" };

type FilterValueEntry = {
  value: string;
  count: number;
};

function compareAlphabetically(left: string, right: string): number {
  return normalizeText(left).localeCompare(normalizeText(right));
}

function compareNumerically(left: string, right: string): number {
  const leftNumber = Number(left);
  const rightNumber = Number(right);
  const leftIsNumber = Number.isFinite(leftNumber);
  const rightIsNumber = Number.isFinite(rightNumber);

  if (leftIsNumber || rightIsNumber) {
    if (!leftIsNumber) {
      return 1;
    }
    if (!rightIsNumber) {
      return -1;
    }
    return leftNumber - rightNumber || compareAlphabetically(left, right);
  }

  return compareAlphabetically(left, right);
}

function compareCanonically(
  left: string,
  right: string,
  ordering: Extract<FilterValueOrdering, { kind: "canonical" }>,
): number {
  const canonicalOrder = ordering.order.map((value) => normalizeText(value));
  const leftIndex = canonicalOrder.indexOf(normalizeText(left));
  const rightIndex = canonicalOrder.indexOf(normalizeText(right));

  if (leftIndex >= 0 || rightIndex >= 0) {
    if (leftIndex < 0) {
      return ordering.unknowns === "headAlpha" ? -1 : 1;
    }
    if (rightIndex < 0) {
      return ordering.unknowns === "headAlpha" ? 1 : -1;
    }
    return leftIndex - rightIndex;
  }

  return compareAlphabetically(left, right);
}

export function compareFilterValues(
  left: FilterValueEntry,
  right: FilterValueEntry,
  ordering: FilterValueOrdering = { kind: "countDescThenAlpha" },
): number {
  switch (ordering.kind) {
    case "alpha":
      return compareAlphabetically(left.value, right.value);
    case "numericAsc":
      return compareNumerically(left.value, right.value);
    case "canonical":
      return compareCanonically(left.value, right.value, ordering);
    case "countDescThenAlpha":
    default:
      return right.count - left.count || compareAlphabetically(left.value, right.value);
  }
}

export function orderFilterValues<T extends FilterValueEntry>(
  values: readonly T[],
  ordering?: FilterValueOrdering,
): T[] {
  return [...values].sort((left, right) => compareFilterValues(left, right, ordering));
}
