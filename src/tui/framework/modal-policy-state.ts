import { createEmptyPolicySelection } from "./modal-helpers.js";
import type {
  DerivedTagTerminalPolicySelection,
  DerivedTagTerminalPolicyState,
  DerivedTagTerminalSelectOption,
} from "./types.js";

export function getPolicyStateForValue(
  value: string,
  valueStates: Record<string, DerivedTagTerminalPolicyState | undefined>,
): DerivedTagTerminalPolicyState | undefined {
  return valueStates[value];
}

export function createValueStateLookup(
  selection: Partial<DerivedTagTerminalPolicySelection<string>> | undefined,
): Record<string, DerivedTagTerminalPolicyState | undefined> {
  const valueStates: Record<string, DerivedTagTerminalPolicyState | undefined> = {};

  for (const value of selection?.exclude ?? []) {
    valueStates[value] = "exclude";
  }
  for (const value of selection?.all ?? []) {
    valueStates[value] = "all";
  }
  for (const value of selection?.any ?? []) {
    valueStates[value] = "any";
  }

  return valueStates;
}

export function buildPolicySelection(
  entries: DerivedTagTerminalSelectOption<string>[],
  valueStates: Record<string, DerivedTagTerminalPolicyState | undefined>,
): DerivedTagTerminalPolicySelection<string> {
  const selection = createEmptyPolicySelection<string>();

  for (const entry of entries) {
    const state = getPolicyStateForValue(entry.value, valueStates);
    if (!state) {
      continue;
    }
    selection[state].push(entry.value);
  }

  selection.any.sort((left, right) => left.localeCompare(right));
  selection.all.sort((left, right) => left.localeCompare(right));
  selection.exclude.sort((left, right) => left.localeCompare(right));
  return selection;
}

export function cyclePolicyState(
  currentState: DerivedTagTerminalPolicyState | undefined,
  allowedStates: DerivedTagTerminalPolicyState[],
  direction: 1 | -1 = 1,
): DerivedTagTerminalPolicyState | undefined {
  const stateOrder: Array<DerivedTagTerminalPolicyState | undefined> = [undefined, ...allowedStates];
  const currentIndex = stateOrder.findIndex((state) => state === currentState);
  const nextIndex = (((currentIndex + direction) % stateOrder.length) + stateOrder.length) % stateOrder.length;
  return stateOrder[nextIndex];
}
