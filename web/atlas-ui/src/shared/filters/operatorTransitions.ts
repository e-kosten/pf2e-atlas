import type { FilterOperandView } from "../../generated/atlas";
import { emptyGroup } from "./queryBuilder";

const scalar = (value: unknown): value is string | number | boolean =>
  typeof value === "string" || typeof value === "boolean" || typeof value === "number";

function emptyOperand(operand: FilterOperandView): unknown {
  switch (operand) {
    case "scalar":
    case "state":
      return "";
    case "list":
      return [];
    case "range":
      return ["", ""];
    case "predicate":
      return emptyGroup();
  }
}

/** Adapt only the operand shapes supplied by the authoritative editor catalog. */
export function transitionOperand(
  previous: FilterOperandView | null,
  next: FilterOperandView,
  value: unknown,
): unknown {
  if (previous === next) {
    switch (next) {
      case "scalar":
      case "state":
        return scalar(value) ? value : emptyOperand(next);
      case "list":
        return Array.isArray(value) && value.every(scalar) ? value : emptyOperand(next);
      case "range":
        return Array.isArray(value) && value.length === 2 && value.every(scalar)
          ? value
          : emptyOperand(next);
      case "predicate":
        return value &&
          typeof value === "object" &&
          "rules" in value &&
          Array.isArray(value.rules) &&
          "combinator" in value &&
          (value.combinator === "and" || value.combinator === "or")
          ? value
          : emptyOperand(next);
    }
  }
  if (previous === "scalar" && next === "list" && scalar(value))
    return value === "" ? [] : [value];
  if (
    previous === "list" &&
    next === "scalar" &&
    Array.isArray(value) &&
    value.length === 1 &&
    scalar(value[0])
  )
    return value[0];
  return emptyOperand(next);
}
