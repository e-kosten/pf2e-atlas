import React from "react";

import {
  buildDerivedTagTerminalActionTargetLine,
  createDerivedTagTerminalActionTargetState,
  getDerivedTagTerminalActionTargetInteractionActions,
  reduceDerivedTagTerminalActionTargetState,
  resolveDerivedTagTerminalActionTargetIntent,
  shouldRenderDerivedTagTerminalActionTarget,
  type DerivedTagTerminalActionTargetOption,
  type DerivedTagTerminalActionTargetState,
  type DerivedTagTerminalActionTargetVisibility,
} from "./action-target.js";
import {
  TerminalTwoPaneScreen,
  TerminalTextScreen,
  createDerivedTagTerminalListNavigationState,
  getTerminalPaneBodyHeight,
  resolveDerivedTagTerminalListNavigationAction,
  useDerivedTagTerminalApp,
  useDerivedTagTerminalInput,
  useDerivedTagTerminalSize,
  type DerivedTagTerminalLine,
} from "./terminal-ui.js";
import {
  TERMINAL_DIALOG_RETURN_FOOTER,
  formatTerminalInteractionFooter,
  resolveTerminalInteractionAction,
  type TerminalInteractionAction,
} from "./interaction-bindings.js";
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
  const terminal = useDerivedTagTerminalApp();

  useDerivedTagTerminalInput((event) => {
    const interactionAction = resolveTerminalInteractionAction(event, interactionActions);

    if (interactionAction?.id === "back" || interactionAction?.id === "quit") {
      onBack();
      return;
    }

    if (interactionAction?.id === "help" && helpTitle && helpBody) {
      void terminal.showDialog({
        title: helpTitle,
        body: helpBody,
        footer: [{ text: TERMINAL_DIALOG_RETURN_FOOTER, tone: "dim" }],
      });
    }
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

export function TerminalMenuScreen<TItem extends TerminalMenuScreenItem>({
  title,
  subtitle,
  leftTitle,
  rightTitle,
  items,
  selectedIndex,
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
  items: TItem[];
  selectedIndex: number;
  interactionActions: TerminalInteractionAction[];
  footer: DerivedTagTerminalLine[];
  status: DerivedTagTerminalLine;
  helpTitle: string;
  helpBody: DerivedTagTerminalLine[];
  buildDetailLines: (item: TItem | undefined) => DerivedTagTerminalLine[];
  onMove: (delta: number, itemCount: number) => void;
  onSelect: () => void;
  onBack: () => void;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const size = useDerivedTagTerminalSize();
  const navigationStateRef = React.useRef(createDerivedTagTerminalListNavigationState());
  const bodyHeight = Math.max(
    1,
    getTerminalPaneBodyHeight(size.height, {
      hasSubtitle: true,
      footerLineCount: 2,
    }),
  );

  useDerivedTagTerminalInput((event) => {
    const navigation = resolveDerivedTagTerminalListNavigationAction(
      event,
      {
        pageSize: Math.max(1, bodyHeight - 1),
        jumpSize: Math.max(1, Math.floor(bodyHeight / 2)),
        includeConfirmKeys: true,
        includeHorizontalConfirmKeys: true,
      },
      navigationStateRef.current,
    );
    navigationStateRef.current = navigation.state;
    const interactionAction = resolveTerminalInteractionAction(event, interactionActions);

    if (interactionAction?.id === "back" || interactionAction?.id === "quit") {
      onBack();
      return;
    }
    if (navigation.action?.kind === "move") {
      onMove(navigation.action.delta, items.length);
      return;
    }
    if (navigation.action?.kind === "boundary") {
      onMove(navigation.action.boundary === "start" ? -selectedIndex : items.length - 1 - selectedIndex, items.length);
      return;
    }
    if (interactionAction?.id === "select") {
      onSelect();
      return;
    }
    if (interactionAction?.id === "help") {
      void terminal.showDialog({
        title: helpTitle,
        body: helpBody,
        footer: [{ text: TERMINAL_DIALOG_RETURN_FOOTER, tone: "dim" }],
      });
    }
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
      footer={[...footer, status]}
      leftWidth={32}
    />
  );
}

export function TerminalActionMenuScreen<
  TItem extends TerminalMenuScreenItem,
  TAction extends string = string,
>({
  title,
  subtitle,
  leftTitle,
  rightTitle,
  leftWidth = 32,
  items,
  selectedIndex,
  interactionActions,
  actionEntries,
  actionTargetVisibility = "onDemand",
  helpTitle,
  helpBody,
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
  items: TItem[];
  selectedIndex: number;
  interactionActions: TerminalInteractionAction[];
  actionEntries: DerivedTagTerminalActionTargetOption<TAction>[];
  actionTargetVisibility?: DerivedTagTerminalActionTargetVisibility;
  helpTitle: string;
  helpBody: DerivedTagTerminalLine[];
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
  const terminal = useDerivedTagTerminalApp();
  const size = useDerivedTagTerminalSize();
  const navigationStateRef = React.useRef(createDerivedTagTerminalListNavigationState());
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

  useDerivedTagTerminalInput((event) => {
    const actionTargetIntent = resolveDerivedTagTerminalActionTargetIntent(event, actionTargetState, "horizontal");
    const navigation = resolveDerivedTagTerminalListNavigationAction(
      event,
      {
        pageSize: Math.max(1, bodyHeight - 1),
        jumpSize: Math.max(1, Math.floor(bodyHeight / 2)),
        includeConfirmKeys: true,
      },
      navigationStateRef.current,
    );
    navigationStateRef.current = navigation.state;
    const interactionAction = resolveTerminalInteractionAction(event, interactionActions);

    if (actionTargetIntent?.kind === "toggle_target") {
      dispatchActionTarget({ type: "toggle_target" });
      navigationStateRef.current = createDerivedTagTerminalListNavigationState();
      return;
    }
    if (actionTargetIntent?.kind === "leave_actions") {
      dispatchActionTarget({ type: "leave_actions" });
      return;
    }
    if (actionTargetIntent?.kind === "move_action") {
      dispatchActionTarget({ type: "move_action", delta: actionTargetIntent.delta, actionCount: actionEntries.length });
      return;
    }
    if (actionTargetIntent?.kind === "apply_action") {
      const selectedAction = actionEntries[actionTargetState.selectedActionIndex];
      if (selectedAction) {
        onAction(selectedAction.id);
      }
      return;
    }
    if (actionTargetState.activeTarget === "actions") {
      if (interactionAction?.id === "help") {
        void terminal.showDialog({
          title: helpTitle,
          body: helpBody,
          footer: [{ text: TERMINAL_DIALOG_RETURN_FOOTER, tone: "dim" }],
        });
      }
      return;
    }
    if (interactionAction?.id === "back" || interactionAction?.id === "quit") {
      onBack();
      return;
    }
    if (navigation.action?.kind === "move") {
      onMove(navigation.action.delta, items.length);
      return;
    }
    if (navigation.action?.kind === "boundary") {
      onMove(navigation.action.boundary === "start" ? -selectedIndex : items.length - 1 - selectedIndex, items.length);
      return;
    }
    if (interactionAction?.id === "select") {
      onSelect();
      return;
    }
    if (interactionAction?.id === "help") {
      void terminal.showDialog({
        title: helpTitle,
        body: helpBody,
        footer: [{ text: TERMINAL_DIALOG_RETURN_FOOTER, tone: "dim" }],
      });
    }
  });

  const footerActions: TerminalInteractionAction[] =
    actionTargetState.activeTarget === "actions"
      ? [
          ...getDerivedTagTerminalActionTargetInteractionActions(actionTargetState, "horizontal"),
          { id: "help" },
        ]
      : [{ id: "move" }, { id: "jump" }, { id: "page" }, { id: "edge" }, ...interactionActions];

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
          text: formatTerminalInteractionFooter(footerActions),
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
