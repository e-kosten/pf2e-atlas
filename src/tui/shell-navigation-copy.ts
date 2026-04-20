import type {
  TerminalFooterBinding,
  TerminalInteractionAction,
  TerminalInteractionLine,
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

export function buildMergedReturnHelpLine(helpText: string): TerminalInteractionLine {
  return {
    text: `Escape / q / \u2190 or h / Backspace: ${helpText}`,
  };
}
