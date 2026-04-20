import React from "react";

import {
  buildDerivedTagTerminalActionTargetLine,
  createDerivedTagTerminalActionTargetState,
  getDerivedTagTerminalActionTargetInteractionActions,
  reduceDerivedTagTerminalActionTargetState,
  shouldRenderDerivedTagTerminalActionTarget,
  type DerivedTagTerminalActionTargetOption,
  type DerivedTagTerminalActionTargetState,
  type DerivedTagTerminalActionTargetVisibility,
} from "./action-target.js";
import { TerminalTwoPaneScreen, TerminalTextScreen, getTerminalPaneBodyHeight } from "./framework/rendering.js";
import { useDerivedTagTerminalSize } from "./framework/context.js";
import type { DerivedTagTerminalLine } from "./framework/types.js";
import {
  buildTerminalInteractionHelpLines,
  formatTerminalFooterBindings,
  formatTerminalInteractionFooter,
  type TerminalFooterBinding,
  type TerminalInteractionAction,
  type TerminalInteractionHelpSection,
} from "./interaction-bindings.js";
import { showTerminalReturnDialog, useTerminalInteractionContextAdapters } from "./interaction-context-adapters.js";
import {
  createTerminalActionTargetInteractionContext,
  createTerminalListInteractionContext,
  useTerminalInteractionContextRouter,
} from "./interaction-context-router.js";
import { buildScrollableLines } from "./list-utils.js";

export function TerminalBusyScreen({
  title = "PF2E Terminal",
  message,
}: {
  title?: string;
  message: string;
}): React.JSX.Element {
  return (
    <TerminalTextScreen
      title={title}
      body={[{ text: message, tone: "section" }]}
      footer={[{ text: "Working...", tone: "dim" }]}
    />
  );
}

export function TerminalMessageScreen({
  title,
  subtitle,
  body,
  interactionActions,
  footer,
  helpTitle,
  helpBody,
  onBack,
}: {
  title: string;
  subtitle?: string;
  body: DerivedTagTerminalLine[];
  interactionActions: TerminalInteractionAction[];
  footer?: DerivedTagTerminalLine[];
  helpTitle?: string;
  helpBody?: DerivedTagTerminalLine[];
  onBack: () => void;
}): React.JSX.Element {
  const adapters = useTerminalInteractionContextAdapters();

  useTerminalInteractionContextRouter({
    contexts: [{ id: "message", kind: "message", interactionActions }],
    onRoute: ({ message }) => {
      if (message.interactionAction?.id === "back" || message.interactionAction?.id === "quit") {
        onBack();
        return;
      }

      if (message.interactionAction?.id === "help" && helpTitle && helpBody) {
        void showTerminalReturnDialog(adapters, helpTitle, helpBody);
      }
    },
  });

  return (
    <TerminalTextScreen
      title={title}
      subtitle={subtitle}
      body={body}
      footer={
        footer ?? [
          {
            text: formatTerminalInteractionFooter(interactionActions),
            tone: "dim",
          },
        ]
      }
    />
  );
}

export type TerminalMenuScreenItem = {
  label: string;
};

export type TerminalMenuScreenInteractions = {
  actions: TerminalInteractionAction[];
  footerBindings?: TerminalFooterBinding[];
  help: {
    title: string;
    sections: TerminalInteractionHelpSection[];
    appendix?: DerivedTagTerminalLine[];
  };
};

const TERMINAL_MENU_NAVIGATION_FOOTER_BINDINGS: TerminalFooterBinding[] = [
  { kind: "action", action: { id: "move" } },
  { kind: "action", action: { id: "jump" } },
  { kind: "action", action: { id: "page" } },
  { kind: "action", action: { id: "edge" } },
];

function buildTerminalMenuScreenFooterText(interactions: TerminalMenuScreenInteractions): string {
  return formatTerminalFooterBindings([
    ...TERMINAL_MENU_NAVIGATION_FOOTER_BINDINGS,
    ...(interactions.footerBindings ??
      interactions.actions.map((action) => ({
        kind: "action",
        action,
      } satisfies TerminalFooterBinding))),
  ]);
}

function buildTerminalMenuScreenHelpBody(interactions: TerminalMenuScreenInteractions): DerivedTagTerminalLine[] {
  const lines = buildTerminalInteractionHelpLines(interactions.help.sections);
  const appendix = interactions.help.appendix ?? [];
  if (appendix.length === 0) {
    return lines;
  }
  if (lines.length === 0) {
    return [...appendix];
  }
  return [...lines, { text: "" }, ...appendix];
}

export function TerminalMenuScreen<TItem extends TerminalMenuScreenItem>({
  title,
  subtitle,
  leftTitle,
  rightTitle,
  items,
  selectedIndex,
  interactions,
  interactionActions,
  footer,
  status,
  helpTitle,
  helpBody,
  buildDetailLines,
  onMove,
  onSelect,
  onBack,
}: {
  title: string;
  subtitle?: string;
  leftTitle: string;
  rightTitle: string;
  items: readonly TItem[];
  selectedIndex: number;
  interactions?: TerminalMenuScreenInteractions;
  interactionActions?: TerminalInteractionAction[];
  footer?: DerivedTagTerminalLine[];
  status: DerivedTagTerminalLine;
  helpTitle?: string;
  helpBody?: DerivedTagTerminalLine[];
  buildDetailLines: (item: TItem | undefined) => DerivedTagTerminalLine[];
  onMove: (delta: number, itemCount: number) => void;
  onSelect: () => void;
  onBack: () => void;
}): React.JSX.Element {
  const adapters = useTerminalInteractionContextAdapters();
  const size = useDerivedTagTerminalSize();
  const bodyHeight = Math.max(
    1,
    getTerminalPaneBodyHeight(size.height, {
      hasSubtitle: true,
      footerLineCount: 2,
    }),
  );
  const resolvedInteractionActions = interactions?.actions ?? interactionActions ?? [];
  const resolvedHelpTitle = interactions?.help.title ?? helpTitle;
  const resolvedHelpBody = interactions ? buildTerminalMenuScreenHelpBody(interactions) : helpBody;
  const resolvedFooter =
    interactions !== undefined
      ? [
          {
            text: buildTerminalMenuScreenFooterText(interactions),
            tone: "dim" as const,
          },
        ]
      : footer ?? [];

  useTerminalInteractionContextRouter({
    contexts: [
      createTerminalListInteractionContext("menu", {
        interactionActions: resolvedInteractionActions,
        pageSize: Math.max(1, bodyHeight - 1),
        jumpSize: Math.max(1, Math.floor(bodyHeight / 2)),
        includeConfirmKeys: true,
        includeHorizontalConfirmKeys: true,
      }),
    ],
    onRoute: ({ menu }) => {
      if (menu.interactionAction?.id === "back" || menu.interactionAction?.id === "quit") {
        onBack();
        return;
      }
      if (menu.navigationAction?.kind === "move") {
        onMove(menu.navigationAction.delta, items.length);
        return;
      }
      if (menu.navigationAction?.kind === "boundary") {
        onMove(
          menu.navigationAction.boundary === "start" ? -selectedIndex : items.length - 1 - selectedIndex,
          items.length,
        );
        return;
      }
      if (menu.interactionAction?.id === "select") {
        onSelect();
        return;
      }
      if (menu.interactionAction?.id === "help" && resolvedHelpTitle && resolvedHelpBody) {
        void showTerminalReturnDialog(adapters, resolvedHelpTitle, resolvedHelpBody);
      }
    },
  });

  return (
    <TerminalTwoPaneScreen
      title={title}
      subtitle={subtitle}
      left={{
        title: leftTitle,
        lines: buildScrollableLines(items, selectedIndex, bodyHeight),
        active: true,
      }}
      right={{
        title: rightTitle,
        lines: buildDetailLines(items[selectedIndex]),
      }}
      footer={[...resolvedFooter, status]}
      leftWidth={32}
    />
  );
}

export function TerminalActionMenuScreen<TItem extends TerminalMenuScreenItem, TAction extends string = string>({
  title,
  subtitle,
  leftTitle,
  rightTitle,
  leftWidth = 32,
  items,
  selectedIndex,
  interactions,
  actionEntries,
  actionTargetVisibility = "onDemand",
  buildRightLines,
  buildStatusLine,
  onMove,
  onSelect,
  onBack,
  onAction,
}: {
  title: string;
  subtitle?: string;
  leftTitle: string;
  rightTitle: string;
  leftWidth?: number;
  items: readonly TItem[];
  selectedIndex: number;
  interactions: TerminalMenuScreenInteractions;
  actionEntries: DerivedTagTerminalActionTargetOption<TAction>[];
  actionTargetVisibility?: DerivedTagTerminalActionTargetVisibility;
  buildRightLines: (item: TItem | undefined) => DerivedTagTerminalLine[];
  buildStatusLine: (context: {
    actionTargetState: DerivedTagTerminalActionTargetState;
    selectedItem: TItem | undefined;
  }) => DerivedTagTerminalLine;
  onMove: (delta: number, itemCount: number) => void;
  onSelect: () => void;
  onBack: () => void;
  onAction: (actionId: TAction) => void;
}): React.JSX.Element {
  const adapters = useTerminalInteractionContextAdapters();
  const size = useDerivedTagTerminalSize();
  const [actionTargetState, dispatchActionTarget] = React.useReducer(
    reduceDerivedTagTerminalActionTargetState<DerivedTagTerminalActionTargetState>,
    undefined,
    () => createDerivedTagTerminalActionTargetState(),
  );
  const bodyHeight = Math.max(
    1,
    getTerminalPaneBodyHeight(size.height, {
      hasSubtitle: true,
      footerLineCount: 2,
    }),
  );
  const selectedItem = items[selectedIndex];

  useTerminalInteractionContextRouter({
    contexts: [
      createTerminalListInteractionContext("menu", {
        interactionActions: interactions.actions,
        pageSize: Math.max(1, bodyHeight - 1),
        jumpSize: Math.max(1, Math.floor(bodyHeight / 2)),
        includeConfirmKeys: true,
      }),
      createTerminalActionTargetInteractionContext("actionTarget", {
        interactionActions: [
          ...getDerivedTagTerminalActionTargetInteractionActions(actionTargetState, "horizontal"),
          { id: "help" },
        ],
        state: actionTargetState,
        orientation: "horizontal",
      }),
    ],
    onRoute: ({ actionTarget, menu }) => {
      if (actionTarget.actionTargetIntent?.kind === "toggle_target") {
        dispatchActionTarget({ type: "toggle_target" });
        return;
      }
      if (actionTarget.actionTargetIntent?.kind === "leave_actions") {
        dispatchActionTarget({ type: "leave_actions" });
        return;
      }
      if (actionTarget.actionTargetIntent?.kind === "move_action") {
        dispatchActionTarget({
          type: "move_action",
          delta: actionTarget.actionTargetIntent.delta,
          actionCount: actionEntries.length,
        });
        return;
      }
      if (actionTarget.actionTargetIntent?.kind === "apply_action") {
        const selectedAction = actionEntries[actionTargetState.selectedActionIndex];
        if (selectedAction) {
          onAction(selectedAction.id);
        }
        return;
      }
      if (actionTargetState.activeTarget === "actions") {
        if (actionTarget.interactionAction?.id === "help") {
          void showTerminalReturnDialog(
            adapters,
            interactions.help.title,
            buildTerminalMenuScreenHelpBody(interactions),
          );
        }
        return;
      }
      if (menu.interactionAction?.id === "back" || menu.interactionAction?.id === "quit") {
        onBack();
        return;
      }
      if (menu.navigationAction?.kind === "move") {
        onMove(menu.navigationAction.delta, items.length);
        return;
      }
      if (menu.navigationAction?.kind === "boundary") {
        onMove(
          menu.navigationAction.boundary === "start" ? -selectedIndex : items.length - 1 - selectedIndex,
          items.length,
        );
        return;
      }
      if (menu.interactionAction?.id === "select") {
        onSelect();
        return;
      }
      if (menu.interactionAction?.id === "help") {
        void showTerminalReturnDialog(
          adapters,
          interactions.help.title,
          buildTerminalMenuScreenHelpBody(interactions),
        );
      }
    },
  });

  const footerActions: TerminalInteractionAction[] =
    actionTargetState.activeTarget === "actions"
      ? [...getDerivedTagTerminalActionTargetInteractionActions(actionTargetState, "horizontal"), { id: "help" }]
      : [{ id: "move" }, { id: "jump" }, { id: "page" }, { id: "edge" }, ...interactions.actions];
  const footerText =
    actionTargetState.activeTarget === "actions"
      ? formatTerminalInteractionFooter(footerActions)
      : buildTerminalMenuScreenFooterText(interactions);

  return (
    <TerminalTwoPaneScreen
      title={title}
      subtitle={subtitle}
      left={{
        title: leftTitle,
        lines: buildScrollableLines(items, selectedIndex, bodyHeight),
        active: true,
      }}
      right={{
        title: rightTitle,
        lines: buildRightLines(selectedItem),
      }}
      footer={[
        {
          text: footerText,
          tone: "dim",
        },
        shouldRenderDerivedTagTerminalActionTarget(actionTargetState, actionTargetVisibility)
          ? buildDerivedTagTerminalActionTargetLine(actionEntries, actionTargetState)
          : buildStatusLine({ actionTargetState, selectedItem }),
      ]}
      leftWidth={leftWidth}
    />
  );
}
