import type {
  TerminalFooterBinding,
  TerminalInteractionAction,
} from "./interaction-bindings.js";

export function createSharedReturnInteractionActions(label = "return"): TerminalInteractionAction[] {
  return [
    { id: "back", label },
    { id: "quit", label },
  ];
}

export function createMergedReturnFooterBinding(label = "return"): TerminalFooterBinding {
  return {
    kind: "actionGroup",
    label,
    actions: createSharedReturnInteractionActions(),
    keyStyle: "expanded",
  };
}
