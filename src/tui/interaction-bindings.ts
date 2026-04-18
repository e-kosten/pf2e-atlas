import {
  isBackNavigationKey,
  isCommandPaletteKey,
  isConfirmKey,
  isConfirmOrToggleKey,
  isFocusToggleKey,
  isHelpKey,
  isLayoutToggleKey,
  isMoveRightKey,
  isSearchKey,
} from "./keymap.js";

export type TerminalInteractionTone =
  | "default"
  | "section"
  | "dim"
  | "accent";

export type TerminalInteractionLine = {
  text: string;
  tone?: TerminalInteractionTone;
  indent?: number;
};

export type TerminalInteractionActionId =
  | "move"
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
  | "back"
  | "return"
  | "focus"
  | "layout"
  | "search"
  | "help"
  | "quit"
  | "commands"
  | "execute";

export type TerminalInteractionAction = {
  id: TerminalInteractionActionId;
  label?: string;
  helpText?: string;
};

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

export const TERMINAL_DIALOG_RETURN_FOOTER = "Press any key to return.";
export const TERMINAL_DIALOG_CONTINUE_FOOTER = "Press any key to continue.";
export const TERMINAL_TEXT_INPUT_FOOTER = "Type text  Enter submit  Backspace edit  Esc cancel";
export const TERMINAL_COMMAND_PALETTE_FILTER_FOOTER = "Type to filter  Enter/Right select  Backspace edit  Esc cancel";
export const TERMINAL_LIVE_FILTER_FOOTER = "Type to filter live  Backspace edit  Enter keep filter  Esc clear and back out";

type TerminalInteractionDefinition = {
  footerKeys: string;
  helpKeys: string;
  defaultLabel: string;
};

const TERMINAL_INTERACTION_DEFINITIONS: Record<TerminalInteractionActionId, TerminalInteractionDefinition> = {
  move: {
    footerKeys: "Up/Down",
    helpKeys: "Up / Down or j / k",
    defaultLabel: "move",
  },
  scroll: {
    footerKeys: "Up/Down",
    helpKeys: "Up / Down or j / k",
    defaultLabel: "scroll",
  },
  jump: {
    footerKeys: "Ctrl-U/D",
    helpKeys: "Ctrl-U / Ctrl-D",
    defaultLabel: "jump",
  },
  page: {
    footerKeys: "PgUp/PgDn or b/f",
    helpKeys: "PageUp / PageDown or b / f",
    defaultLabel: "page",
  },
  edge: {
    footerKeys: "gg/G or Home/End",
    helpKeys: "gg / G or Home / End",
    defaultLabel: "edge",
  },
  select: {
    footerKeys: "Enter/Right",
    helpKeys: "Enter / Right or l",
    defaultLabel: "select",
  },
  open: {
    footerKeys: "Enter/Right",
    helpKeys: "Enter / Right or l",
    defaultLabel: "open",
  },
  preview: {
    footerKeys: "Enter/Right",
    helpKeys: "Enter / Right or l",
    defaultLabel: "preview",
  },
  edit: {
    footerKeys: "Enter/Right/Space",
    helpKeys: "Enter / Right or l / Space",
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
  back: {
    footerKeys: "Left/Esc/backspace",
    helpKeys: "Left / h or Escape / Backspace",
    defaultLabel: "back",
  },
  return: {
    footerKeys: "Left/Esc/backspace",
    helpKeys: "Left / h or Escape / Backspace",
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
  execute: {
    footerKeys: "Tab",
    helpKeys: "Tab",
    defaultLabel: "execute",
  },
};

function getInteractionDefinition(action: TerminalInteractionAction): TerminalInteractionDefinition {
  return TERMINAL_INTERACTION_DEFINITIONS[action.id];
}

function getInteractionLabel(action: TerminalInteractionAction): string {
  return action.label ?? getInteractionDefinition(action).defaultLabel;
}

export function formatTerminalInteractionFooter(actions: TerminalInteractionAction[]): string {
  return actions
    .map((action) => {
      const definition = getInteractionDefinition(action);
      return `${definition.footerKeys} ${getInteractionLabel(action)}`;
    })
    .join("  ");
}

function matchesTerminalInteractionAction(actionId: TerminalInteractionActionId, normalizedKey: string): boolean {
  switch (actionId) {
    case "select":
    case "open":
    case "preview":
      return isConfirmKey(normalizedKey) || isMoveRightKey(normalizedKey);
    case "edit":
      return isConfirmOrToggleKey(normalizedKey) || isMoveRightKey(normalizedKey);
    case "toggle":
    case "cycle":
      return isConfirmOrToggleKey(normalizedKey);
    case "back":
    case "return":
      return isBackNavigationKey(normalizedKey);
    case "focus":
      return isFocusToggleKey(normalizedKey);
    case "layout":
      return isLayoutToggleKey(normalizedKey);
    case "search":
      return isSearchKey(normalizedKey);
    case "help":
      return isHelpKey(normalizedKey);
    case "quit":
      return normalizedKey === "q";
    case "commands":
      return isCommandPaletteKey(normalizedKey);
    case "execute":
      return normalizedKey === "tab" || normalizedKey === "shift_tab";
    default:
      return false;
  }
}

export function resolveTerminalInteractionAction(
  normalizedKey: string,
  actions: TerminalInteractionAction[],
): TerminalInteractionAction | undefined {
  return actions.find((action) => matchesTerminalInteractionAction(action.id, normalizedKey));
}

function buildActionHelpLine(action: TerminalInteractionAction): TerminalInteractionLine {
  const definition = getInteractionDefinition(action);
  return {
    text: `${definition.helpKeys}: ${action.helpText ?? getInteractionLabel(action)}`,
  };
}

function buildCommandHelpLine(command: TerminalInteractionCommand): TerminalInteractionLine {
  const aliasLabel = command.aliases && command.aliases.length > 0
    ? `  aliases: ${command.aliases.join(", ")}`
    : "";
  return {
    text: `${command.label}${aliasLabel}  ${command.description}`,
  };
}

export function buildTerminalInteractionHelpLines(
  sections: TerminalInteractionHelpSection[],
): TerminalInteractionLine[] {
  const lines: TerminalInteractionLine[] = [];

  for (const section of sections) {
    if (lines.length > 0) {
      lines.push({ text: "" });
    }
    lines.push({ text: section.title, tone: "section" });
    for (const action of section.actions ?? []) {
      lines.push(buildActionHelpLine(action));
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
