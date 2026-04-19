import React from "react";

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
