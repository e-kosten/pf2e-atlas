import type { DerivedTagTerminalInputEvent } from "./framework/types.js";

export type TerminalInteractionTone = "default" | "section" | "dim" | "accent";

export type TerminalInteractionLine = {
  text: string;
  tone?: TerminalInteractionTone;
  indent?: number;
};

export type TerminalInteractionActionId =
  | "move"
  | "moveHorizontal"
  | "scroll"
  | "jump"
  | "page"
  | "edge"
  | "select"
  | "open"
  | "preview"
  | "edit"
  | "toggle"
  | "cycle"
  | "cancel"
  | "back"
  | "return"
  | "focus"
  | "layout"
  | "search"
  | "help"
  | "quit"
  | "commands"
  | "actions"
  | "apply"
  | "close"
  | "execute";

export type TerminalInteractionAction = {
  id: TerminalInteractionActionId;
  label?: string;
  helpText?: string;
};

export type TerminalFooterBindingKeyStyle = "compact" | "expanded";

export type TerminalTextEntryBindingId = "submit" | "cancel" | "deleteBackward";

export type TerminalFooterBinding =
  | { kind: "text"; text: string }
  | { kind: "action"; action: TerminalInteractionAction; keyStyle?: TerminalFooterBindingKeyStyle }
  | {
      kind: "actionGroup";
      label: string;
      actions: TerminalInteractionAction[];
      keyStyle?: TerminalFooterBindingKeyStyle;
    }
  | { kind: "textEntry"; textEntry: TerminalTextEntryBindingId; label?: string };

export type TerminalInteractionCommand = {
  label: string;
  description: string;
  aliases?: string[];
};

export type TerminalInteractionHelpSection = {
  title: string;
  actions?: TerminalInteractionAction[];
  commands?: TerminalInteractionCommand[];
  lines?: TerminalInteractionLine[];
};

export type TerminalTextEntryIntent =
  | { kind: "submit" }
  | { kind: "cancel" }
  | { kind: "deleteBackward" }
  | { kind: "append"; text: string };

export const TERMINAL_DIALOG_RETURN_FOOTER = "Press any key to return.";
export const TERMINAL_DIALOG_CONTINUE_FOOTER = "Press any key to continue.";

type TerminalInteractionDefinition = {
  footerKeys: string;
  expandedFooterKeys?: string;
  helpKeys: string;
  defaultLabel: string;
};

type TerminalTextEntryDefinition = {
  footerKeys: string;
  defaultLabel: string;
};

const TERMINAL_INTERACTION_DEFINITIONS: Record<TerminalInteractionActionId, TerminalInteractionDefinition> = {
  move: {
    footerKeys: "↑/↓",
    helpKeys: "↑ / ↓ or j / k",
    defaultLabel: "move",
  },
  moveHorizontal: {
    footerKeys: "←/→",
    helpKeys: "← / → or h / l",
    defaultLabel: "move",
  },
  scroll: {
    footerKeys: "↑/↓",
    helpKeys: "↑ / ↓ or j / k",
    defaultLabel: "scroll",
  },
  jump: {
    footerKeys: "Ctrl-U/D",
    helpKeys: "Ctrl-U / Ctrl-D",
    defaultLabel: "jump",
  },
  page: {
    footerKeys: "b/f",
    helpKeys: "b / f or PageUp / PageDown",
    defaultLabel: "page",
  },
  edge: {
    footerKeys: "gg/G",
    helpKeys: "gg / G or Home / End",
    defaultLabel: "edge",
  },
  select: {
    footerKeys: "Enter/→",
    helpKeys: "Enter / → or l",
    defaultLabel: "select",
  },
  open: {
    footerKeys: "Enter/→",
    helpKeys: "Enter / → or l",
    defaultLabel: "open",
  },
  preview: {
    footerKeys: "Enter/→",
    helpKeys: "Enter / → or l",
    defaultLabel: "preview",
  },
  edit: {
    footerKeys: "Enter/→/Space",
    helpKeys: "Enter / → or l / Space",
    defaultLabel: "edit",
  },
  toggle: {
    footerKeys: "Enter/Space",
    helpKeys: "Enter / Space",
    defaultLabel: "toggle",
  },
  cycle: {
    footerKeys: "Enter/Space",
    helpKeys: "Enter / Space",
    defaultLabel: "cycle",
  },
  cancel: {
    footerKeys: "Esc",
    helpKeys: "Escape",
    defaultLabel: "cancel",
  },
  back: {
    footerKeys: "←",
    expandedFooterKeys: "Backspace/←",
    helpKeys: "← or h / Backspace",
    defaultLabel: "back",
  },
  return: {
    footerKeys: "←",
    expandedFooterKeys: "Backspace/←",
    helpKeys: "← or h / Backspace",
    defaultLabel: "return",
  },
  focus: {
    footerKeys: "Tab",
    helpKeys: "Tab / Shift-Tab or w",
    defaultLabel: "focus",
  },
  layout: {
    footerKeys: "z",
    helpKeys: "z",
    defaultLabel: "toggle layout",
  },
  search: {
    footerKeys: "/",
    helpKeys: "/",
    defaultLabel: "search",
  },
  help: {
    footerKeys: "?",
    helpKeys: "?",
    defaultLabel: "help",
  },
  quit: {
    footerKeys: "q",
    helpKeys: "q",
    defaultLabel: "back",
  },
  commands: {
    footerKeys: ":",
    helpKeys: ":",
    defaultLabel: "commands",
  },
  actions: {
    footerKeys: ":",
    helpKeys: ":",
    defaultLabel: "actions",
  },
  apply: {
    footerKeys: "Enter",
    helpKeys: "Enter",
    defaultLabel: "apply",
  },
  close: {
    footerKeys: "Esc",
    helpKeys: "Escape",
    defaultLabel: "close",
  },
  execute: {
    footerKeys: "Tab",
    helpKeys: "Tab",
    defaultLabel: "execute",
  },
};

const TERMINAL_TEXT_ENTRY_DEFINITIONS: Record<TerminalTextEntryBindingId, TerminalTextEntryDefinition> = {
  submit: {
    footerKeys: "Enter",
    defaultLabel: "submit",
  },
  cancel: {
    footerKeys: "Esc",
    defaultLabel: "cancel",
  },
  deleteBackward: {
    footerKeys: "Backspace",
    defaultLabel: "edit",
  },
};

function getInteractionDefinition(action: TerminalInteractionAction): TerminalInteractionDefinition {
  return TERMINAL_INTERACTION_DEFINITIONS[action.id];
}

function getInteractionLabel(action: TerminalInteractionAction): string {
  return action.label ?? getInteractionDefinition(action).defaultLabel;
}

function findDeclaredInteractionAction(
  actions: TerminalInteractionAction[],
  targetId: TerminalInteractionActionId,
): TerminalInteractionAction | undefined {
  return actions.find((action) => action.id === targetId);
}

function getEscapeFallbackAction(actions: TerminalInteractionAction[]): TerminalInteractionAction | undefined {
  return (
    findDeclaredInteractionAction(actions, "cancel") ??
    findDeclaredInteractionAction(actions, "close") ??
    findDeclaredInteractionAction(actions, "back") ??
    findDeclaredInteractionAction(actions, "return") ??
    findDeclaredInteractionAction(actions, "quit")
  );
}

function isEscapeFallbackTarget(action: TerminalInteractionAction, actions: TerminalInteractionAction[]): boolean {
  return getEscapeFallbackAction(actions)?.id === action.id;
}

function getInteractionDisplayKeys(
  action: TerminalInteractionAction,
  actions: TerminalInteractionAction[],
): Pick<TerminalInteractionDefinition, "footerKeys" | "helpKeys"> {
  const definition = getInteractionDefinition(action);

  if (action.id === "back" || action.id === "return") {
    if (isEscapeFallbackTarget(action, actions)) {
      return {
        footerKeys: `${definition.footerKeys}/Esc`,
        helpKeys: `${definition.helpKeys} / Escape`,
      };
    }
    return {
      footerKeys: definition.footerKeys,
      helpKeys: definition.helpKeys,
    };
  }

  if (action.id === "quit" && isEscapeFallbackTarget(action, actions)) {
    return {
      footerKeys: `Esc/${definition.footerKeys}`,
      helpKeys: `Escape / ${definition.helpKeys}`,
    };
  }

  return {
    footerKeys: definition.footerKeys,
    helpKeys: definition.helpKeys,
  };
}

function getInteractionFooterKeys(
  action: TerminalInteractionAction,
  actions: TerminalInteractionAction[],
  keyStyle: TerminalFooterBindingKeyStyle,
): string {
  if (keyStyle === "compact") {
    return getInteractionDisplayKeys(action, actions).footerKeys;
  }

  const definition = getInteractionDefinition(action);
  const expandedKeys = definition.expandedFooterKeys ?? definition.footerKeys;

  if (action.id === "back" || action.id === "return") {
    return isEscapeFallbackTarget(action, actions) ? `Esc/${expandedKeys}` : expandedKeys;
  }

  if (action.id === "quit" && isEscapeFallbackTarget(action, actions)) {
    return `Esc/${expandedKeys}`;
  }

  return expandedKeys;
}

function collectDeclaredFooterActions(bindings: TerminalFooterBinding[]): TerminalInteractionAction[] {
  return bindings.flatMap((binding) => {
    switch (binding.kind) {
      case "action":
        return [binding.action];
      case "actionGroup":
        return binding.actions;
      case "text":
      case "textEntry":
        return [];
    }
  });
}

function joinFooterKeySets(keySets: string[]): string {
  const seenKeys = new Set<string>();
  const orderedKeys: string[] = [];

  for (const keySet of keySets) {
    for (const key of keySet.split("/")) {
      if (seenKeys.has(key)) {
        continue;
      }
      seenKeys.add(key);
      orderedKeys.push(key);
    }
  }

  return orderedKeys.join("/");
}

function formatTerminalFooterBinding(
  binding: TerminalFooterBinding,
  declaredActions: TerminalInteractionAction[],
): string {
  switch (binding.kind) {
    case "text":
      return binding.text;
    case "textEntry": {
      const definition = TERMINAL_TEXT_ENTRY_DEFINITIONS[binding.textEntry];
      return `${definition.footerKeys} ${binding.label ?? definition.defaultLabel}`;
    }
    case "action": {
      const keyStyle = binding.keyStyle ?? "compact";
      return `${getInteractionFooterKeys(binding.action, declaredActions, keyStyle)} ${getInteractionLabel(binding.action)}`;
    }
    case "actionGroup": {
      const keyStyle = binding.keyStyle ?? "compact";
      const groupedKeys = joinFooterKeySets(
        binding.actions.map((action) => getInteractionFooterKeys(action, declaredActions, keyStyle)),
      );
      return `${groupedKeys} ${binding.label}`;
    }
  }
}

export function formatTerminalInteractionFooter(actions: TerminalInteractionAction[]): string {
  return actions
    .map((action) => {
      const displayKeys = getInteractionDisplayKeys(action, actions);
      return `${displayKeys.footerKeys} ${getInteractionLabel(action)}`;
    })
    .join("  ");
}

export function formatTerminalFooterBindings(bindings: TerminalFooterBinding[]): string {
  const declaredActions = collectDeclaredFooterActions(bindings);
  return bindings.map((binding) => formatTerminalFooterBinding(binding, declaredActions)).join("  ");
}

export const TERMINAL_TEXT_INPUT_FOOTER = formatTerminalFooterBindings([
  { kind: "text", text: "Type text" },
  { kind: "textEntry", textEntry: "submit" },
  { kind: "textEntry", textEntry: "deleteBackward" },
  { kind: "textEntry", textEntry: "cancel" },
]);

export const TERMINAL_COMMAND_PALETTE_FILTER_FOOTER = formatTerminalFooterBindings([
  { kind: "text", text: "Type to filter" },
  { kind: "action", action: { id: "select" } },
  { kind: "textEntry", textEntry: "deleteBackward" },
  { kind: "textEntry", textEntry: "cancel" },
]);

export const TERMINAL_COMMAND_PALETTE_EMPTY_FILTER_FOOTER = formatTerminalFooterBindings([
  { kind: "text", text: "Type to filter" },
  { kind: "textEntry", textEntry: "deleteBackward" },
  { kind: "textEntry", textEntry: "cancel" },
]);

export const TERMINAL_SELECT_EMPTY_FOOTER = formatTerminalFooterBindings([
  {
    kind: "actionGroup",
    label: "cancel",
    actions: [{ id: "back" }, { id: "quit" }],
    keyStyle: "expanded",
  },
]);

export const TERMINAL_LIVE_FILTER_FOOTER = formatTerminalFooterBindings([
  { kind: "text", text: "Type to filter live" },
  { kind: "textEntry", textEntry: "deleteBackward" },
  { kind: "textEntry", textEntry: "submit", label: "keep filter" },
  { kind: "textEntry", textEntry: "cancel", label: "clear and back out" },
]);

function matchesTerminalInteractionAction(
  actionId: TerminalInteractionActionId,
  event: DerivedTagTerminalInputEvent,
): boolean {
  switch (actionId) {
    case "move":
    case "scroll":
    case "jump":
    case "page":
    case "edge":
      return false;
    case "select":
    case "open":
    case "preview":
      return event.isConfirmKey() || event.isMoveRightKey();
    case "edit":
      return event.isConfirmOrToggleKey() || event.isMoveRightKey();
    case "toggle":
      return event.isConfirmOrToggleKey();
    case "cycle":
      return Boolean(event.getCycleDirection());
    case "cancel":
      return false;
    case "moveHorizontal":
      return event.isMoveLeftKey() || event.isMoveRightKey();
    case "back":
    case "return":
      return event.isBackNavigationKey() && event.textInputAction !== "cancel";
    case "focus":
      return event.isFocusToggleKey();
    case "layout":
      return event.isLayoutToggleKey();
    case "search":
      return event.isSearchKey();
    case "help":
      return event.isHelpKey();
    case "quit":
      return event.isTerminalQuitKey();
    case "commands":
    case "actions":
      return event.isCommandPaletteKey();
    case "apply":
      return event.isConfirmKey();
    case "close":
      return event.textInputAction === "cancel";
    case "execute":
      return event.isExecuteKey();
  }
}

export function resolveTerminalInteractionAction(
  event: DerivedTagTerminalInputEvent,
  actions: TerminalInteractionAction[],
): TerminalInteractionAction | undefined {
  if (event.textInputAction === "cancel") {
    return getEscapeFallbackAction(actions);
  }

  return actions.find((action) => matchesTerminalInteractionAction(action.id, event));
}

export function resolveTerminalTextEntryIntent(
  event: DerivedTagTerminalInputEvent,
): TerminalTextEntryIntent | undefined {
  if (event.textInputAction === "submit") {
    return { kind: "submit" };
  }
  if (event.textInputAction === "cancel") {
    return { kind: "cancel" };
  }
  if (event.textInputAction === "deleteBackward") {
    return { kind: "deleteBackward" };
  }
  if (event.printable) {
    return { kind: "append", text: event.printable };
  }
  return undefined;
}

export function getTerminalInteractionCycleDirection(
  event: DerivedTagTerminalInputEvent,
  action: TerminalInteractionAction | undefined,
): 1 | -1 | undefined {
  if (action?.id === "cycle") {
    return event.getCycleDirection();
  }
  return undefined;
}

function buildActionHelpLine(
  action: TerminalInteractionAction,
  actions: TerminalInteractionAction[],
): TerminalInteractionLine {
  const displayKeys = getInteractionDisplayKeys(action, actions);
  return {
    text: `${displayKeys.helpKeys}: ${action.helpText ?? getInteractionLabel(action)}`,
  };
}

function buildCommandHelpLine(command: TerminalInteractionCommand): TerminalInteractionLine {
  const aliasLabel = command.aliases && command.aliases.length > 0 ? `  aliases: ${command.aliases.join(", ")}` : "";
  return {
    text: `${command.label}${aliasLabel}  ${command.description}`,
  };
}

export function buildTerminalInteractionHelpLines(
  sections: TerminalInteractionHelpSection[],
): TerminalInteractionLine[] {
  const lines: TerminalInteractionLine[] = [];
  const declaredActions = sections.flatMap((section) => section.actions ?? []);

  for (const section of sections) {
    if (lines.length > 0) {
      lines.push({ text: "" });
    }
    lines.push({ text: section.title, tone: "section" });
    for (const action of section.actions ?? []) {
      lines.push(buildActionHelpLine(action, declaredActions));
    }
    for (const command of section.commands ?? []) {
      lines.push(buildCommandHelpLine(command));
    }
    for (const line of section.lines ?? []) {
      lines.push(line);
    }
  }

  return lines;
}
