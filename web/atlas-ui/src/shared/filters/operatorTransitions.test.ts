import { transitionOperand } from "./operatorTransitions";
import type { FilterOperandView } from "../../generated/atlas";

describe("catalog operand transitions", () => {
  it.each(["Companion", "fire, cold", 0, false])(
    "safely promotes and demotes a singleton literal %s",
    (value) => {
      const list = transitionOperand("scalar", "list", value);
      expect(list).toEqual([value]);
      expect(transitionOperand("list", "scalar", list)).toBe(value);
    },
  );
  it("requires an explicit choice rather than selecting one of multiple values", () => {
    expect(transitionOperand("list", "scalar", ["creature", "hazard"])).toBe("");
    expect(transitionOperand("list", "scalar", [])).toBe("");
    expect(transitionOperand("scalar", "list", "")).toEqual([]);
  });
  const examples: Record<FilterOperandView, unknown> = {
    scalar: "fire, cold",
    list: ["fire, cold", "water"],
    range: [0, 12],
    state: "missing",
    predicate: {
      id: "group",
      combinator: "or",
      not: true,
      rules: [{ id: "draft", field: "item.name", operator: "eq", value: "" }],
    },
  };
  const defaults: Record<FilterOperandView, unknown> = {
    scalar: "",
    list: [],
    range: ["", ""],
    state: "",
    predicate: { combinator: "and", rules: [] },
  };
  it.each(Object.keys(examples) as FilterOperandView[])(
    "retains the same %s operand without replacing its editing structure",
    (operand) => {
      expect(transitionOperand(operand, operand, examples[operand])).toBe(
        examples[operand],
      );
    },
  );
  it("resets incompatible shapes before a different value editor renders", () => {
    for (const previous of Object.keys(examples) as FilterOperandView[])
      for (const next of Object.keys(examples) as FilterOperandView[]) {
        if (
          previous === next ||
          (previous === "scalar" && next === "list") ||
          (previous === "list" && next === "scalar")
        )
          continue;
        expect(transitionOperand(previous, next, examples[previous])).toEqual(
          defaults[next],
        );
      }
  });
  it("never passes a scalar or malformed group to a predicate editor", () => {
    for (const value of ["missing", ["missing"], {}, { rules: [] }, null])
      expect(transitionOperand("predicate", "predicate", value)).toEqual(
        defaults.predicate,
      );
    expect(transitionOperand("state", "predicate", "missing")).toEqual(
      defaults.predicate,
    );
  });
});
