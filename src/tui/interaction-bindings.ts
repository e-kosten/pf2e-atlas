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
