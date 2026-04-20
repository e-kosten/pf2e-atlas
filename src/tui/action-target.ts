import type {
  DerivedTagTerminalInputEvent,
  DerivedTagTerminalLine,
  DerivedTagTerminalSegment,
} from "./framework/types.js";
import type { TerminalInteractionAction, TerminalInteractionHelpSection } from "./interaction-bindings.js";
import { buildTerminalInteractionHelpLines } from "./interaction-bindings.js";

export type DerivedTagTerminalActionTargetFocus = "content" | "actions";
export type DerivedTagTerminalActionTargetOrientation = "horizontal" | "vertical";
export type DerivedTagTerminalActionTargetVisibility = "persistent" | "onDemand";

export type DerivedTagTerminalActionTargetOption<T extends string = string> = {
  id: T;
  label: string;
  description?: string;
};

export type DerivedTagTerminalActionTargetState = {
  activeTarget: DerivedTagTerminalActionTargetFocus;
  selectedActionIndex: number;
};

export type DerivedTagTerminalActionTargetAction =
  | { type: "toggle_target" }
  | { type: "leave_actions" }
  | { type: "move_action"; delta: number; actionCount: number };

export type DerivedTagTerminalActionTargetIntent =
  | { kind: "toggle_target" }
  | { kind: "leave_actions" }
  | { kind: "move_action"; delta: number }
  | { kind: "apply_action" };

function moveSelectionWrapped(currentIndex: number, delta: number, itemCount: number): number {
  if (itemCount <= 0) {
    return 0;
  }
  const rawIndex = currentIndex + delta;
  return ((rawIndex % itemCount) + itemCount) % itemCount;
}

export function createDerivedTagTerminalActionTargetState(
  selectedActionIndex = 0,
): DerivedTagTerminalActionTargetState {
  return {
    activeTarget: "content",
    selectedActionIndex,
  };
}

export function reduceDerivedTagTerminalActionTargetState<TState extends DerivedTagTerminalActionTargetState>(
  state: TState,
  action: DerivedTagTerminalActionTargetAction,
): TState {
  switch (action.type) {
    case "toggle_target":
      return {
        ...state,
        activeTarget: state.activeTarget === "content" ? "actions" : "content",
      };
    case "leave_actions":
      return {
        ...state,
        activeTarget: "content",
      };
    case "move_action":
      return {
        ...state,
        selectedActionIndex: moveSelectionWrapped(state.selectedActionIndex, action.delta, action.actionCount),
      };
    default:
      return state;
  }
}

export function resolveDerivedTagTerminalActionTargetIntent(
  event: DerivedTagTerminalInputEvent,
  state: DerivedTagTerminalActionTargetState,
  orientation: DerivedTagTerminalActionTargetOrientation,
): DerivedTagTerminalActionTargetIntent | undefined {
  if (event.isCommandPaletteKey()) {
    return { kind: "toggle_target" };
  }
  if (state.activeTarget !== "actions") {
    return undefined;
  }
  if (event.textInputAction === "cancel") {
    return { kind: "leave_actions" };
  }
  if (event.isConfirmKey()) {
    return { kind: "apply_action" };
  }

  if (orientation === "horizontal") {
    if (event.isMoveLeftKey()) {
      return { kind: "move_action", delta: -1 };
    }
    if (event.isMoveRightKey()) {
      return { kind: "move_action", delta: 1 };
    }
    return undefined;
  }

  if (event.isMoveUpKey()) {
    return { kind: "move_action", delta: -1 };
  }
  if (event.isMoveDownKey()) {
    return { kind: "move_action", delta: 1 };
  }
  return undefined;
}

export function formatDerivedTagTerminalActionTargetBar<T extends string>(
  actions: readonly DerivedTagTerminalActionTargetOption<T>[],
  state: DerivedTagTerminalActionTargetState,
): string {
  return [
    "Actions:",
    ...actions.map((action, index) => (index === state.selectedActionIndex ? ` ${action.label} ` : action.label)),
  ].join("  ");
}

export function buildDerivedTagTerminalActionTargetLine<T extends string>(
  actions: readonly DerivedTagTerminalActionTargetOption<T>[],
  state: DerivedTagTerminalActionTargetState,
): DerivedTagTerminalLine {
  const segments: DerivedTagTerminalSegment[] = [
    {
      text: "Actions:",
      tone: state.activeTarget === "actions" ? "section" : "dim",
    },
  ];

  actions.forEach((action, index) => {
    segments.push({ text: "  ", tone: "default" });
    segments.push({
      text: index === state.selectedActionIndex ? ` ${action.label} ` : action.label,
      tone: index === state.selectedActionIndex ? "selected" : state.activeTarget === "actions" ? "accent" : "default",
    });
  });

  return {
    text: "",
    segments,
    noWrap: true,
  };
}

export function shouldRenderDerivedTagTerminalActionTarget(
  state: DerivedTagTerminalActionTargetState,
  visibility: DerivedTagTerminalActionTargetVisibility,
): boolean {
  return visibility === "persistent" || state.activeTarget === "actions";
}

export function getDerivedTagTerminalActionTargetInteractionActions(
  state: DerivedTagTerminalActionTargetState,
  orientation: DerivedTagTerminalActionTargetOrientation,
): TerminalInteractionAction[] {
  if (state.activeTarget === "actions") {
    return [
      { id: orientation === "horizontal" ? "moveHorizontal" : "move" },
      { id: "apply" },
      { id: "actions", label: "leave actions" },
      { id: "close", label: "leave actions" },
    ];
  }

  return [{ id: "actions", label: "focus actions" }];
}

export function buildDerivedTagTerminalActionTargetHelpLines(options: {
  contentHelpText?: string;
  orientation: DerivedTagTerminalActionTargetOrientation;
  visibility: DerivedTagTerminalActionTargetVisibility;
  actions: readonly DerivedTagTerminalActionTargetOption[];
}): Array<{ text: string; tone?: "default" | "section" | "dim" | "accent"; indent?: number }> {
  const sections: TerminalInteractionHelpSection[] = [
    {
      title: "Enter Actions",
      actions: [{ id: "actions", label: "focus actions" }],
      lines: [
        {
          text:
            options.visibility === "persistent"
              ? "The action rail stays visible, but it only takes focus after you press :."
              : "Press : to open the action rail, then move inside it with the action-target keys.",
        },
      ],
    },
    {
      title: "While Actions Are Focused",
      actions: [
        { id: options.orientation === "horizontal" ? "moveHorizontal" : "move" },
        { id: "apply" },
        { id: "actions", label: "leave actions" },
        { id: "close", label: "leave actions" },
      ],
      lines: options.contentHelpText ? [{ text: options.contentHelpText }] : [],
    },
    {
      title: "Available Actions",
      lines: options.actions.map((action) => ({
        text: `${action.label}${action.description ? `  ${action.description}` : ""}`,
      })),
    },
  ];

  return buildTerminalInteractionHelpLines(sections);
}
