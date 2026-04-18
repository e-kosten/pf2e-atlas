import React from "react";

import type { OntologyDomainSummary } from "../../types.js";
import {
  TerminalTwoPaneScreen,
  createDerivedTagTerminalListNavigationState,
  getNormalizedKeyName,
  getTerminalPaneBodyHeight,
  resolveDerivedTagTerminalListNavigationAction,
  useDerivedTagTerminalApp,
  useDerivedTagTerminalInput,
  useDerivedTagTerminalSize,
} from "../terminal-ui.js";
import {
  TERMINAL_DIALOG_RETURN_FOOTER,
  buildTerminalInteractionHelpLines,
  formatTerminalInteractionFooter,
  resolveTerminalInteractionAction,
  type TerminalInteractionAction,
} from "../interaction-bindings.js";
import { buildScrollableLines } from "../list-utils.js";

function getOntologyDomainPickerInteractionActions(): TerminalInteractionAction[] {
  return [{ id: "select" }, { id: "back", label: "back" }, { id: "help" }, { id: "quit", label: "back" }];
}

function buildOntologyDomainPickerHelpLines() {
  return buildTerminalInteractionHelpLines([
    {
      title: "Navigation",
      actions: [
        { id: "move", helpText: "move between ontology domains" },
        { id: "jump", helpText: "jump through the domain list" },
        { id: "page", helpText: "page through the domain list" },
        { id: "edge", helpText: "jump to the first or last domain" },
      ],
    },
    {
      title: "Actions",
      actions: getOntologyDomainPickerInteractionActions().map((action) => ({
        ...action,
        helpText:
          action.id === "select"
            ? "open the selected domain"
            : action.id === "help"
              ? "show this help"
              : "return to the previous area",
      })),
    },
    {
      title: "Domains",
      lines: [
        { text: "Derived Tags: authored tag ontology with live record coverage." },
        { text: "Categories: public catalog category and subcategory browsing." },
        { text: "Search Semantics: metadata field and search-vocabulary discovery." },
      ],
    },
  ]);
}

export function OntologyDomainPickerScreen({
  domains,
  selectedIndex,
  onBack,
  onMove,
  onOpenSelected,
}: {
  domains: OntologyDomainSummary[];
  selectedIndex: number;
  onBack: () => void;
  onMove: (delta: number, itemCount: number) => void;
  onOpenSelected: () => void;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const size = useDerivedTagTerminalSize();
  const navigationStateRef = React.useRef(createDerivedTagTerminalListNavigationState());
  const selectedDomain = domains[selectedIndex];
  const bodyHeight = Math.max(
    1,
    getTerminalPaneBodyHeight(size.height, {
      hasSubtitle: true,
      footerLineCount: 2,
    }),
  );

  useDerivedTagTerminalInput((input, key) => {
    const normalized = getNormalizedKeyName(input, key);
    const navigation = resolveDerivedTagTerminalListNavigationAction(
      input,
      key,
      {
        pageSize: Math.max(1, bodyHeight - 1),
        jumpSize: Math.max(1, Math.floor(bodyHeight / 2)),
        includeConfirmKeys: true,
        includeHorizontalConfirmKeys: true,
      },
      navigationStateRef.current,
    );
    navigationStateRef.current = navigation.state;
    const interactionAction = resolveTerminalInteractionAction(normalized, getOntologyDomainPickerInteractionActions());

    if (normalized === "ctrl_c" || interactionAction?.id === "back" || interactionAction?.id === "quit") {
      onBack();
      return;
    }
    if (navigation.action?.kind === "move") {
      onMove(navigation.action.delta, domains.length);
      return;
    }
    if (navigation.action?.kind === "boundary") {
      onMove(
        navigation.action.boundary === "start" ? -selectedIndex : domains.length - 1 - selectedIndex,
        domains.length,
      );
      return;
    }
    if (interactionAction?.id === "select") {
      onOpenSelected();
      return;
    }
    if (interactionAction?.id === "help") {
      void terminal.showDialog({
        title: "Ontology Domains",
        body: buildOntologyDomainPickerHelpLines(),
        footer: [{ text: TERMINAL_DIALOG_RETURN_FOOTER, tone: "dim" }],
      });
      return;
    }
  });

  return (
    <TerminalTwoPaneScreen
      title="Ontology Search"
      subtitle="Choose an ontology-backed browse domain"
      left={{
        title: "Domains",
        lines: buildScrollableLines(domains, selectedIndex, bodyHeight),
        active: true,
      }}
      right={{
        title: "Domain Details",
        lines: selectedDomain
          ? [{ text: selectedDomain.label, tone: "section" }, { text: selectedDomain.description }]
          : [{ text: "No domain selected.", tone: "dim" }],
      }}
      footer={[
        {
          text: formatTerminalInteractionFooter([
            { id: "move" },
            { id: "jump" },
            { id: "page" },
            { id: "edge" },
            ...getOntologyDomainPickerInteractionActions(),
          ]),
          tone: "dim",
        },
        { text: selectedDomain?.label ?? "-", tone: "accent" },
      ]}
      leftWidth={32}
    />
  );
}
