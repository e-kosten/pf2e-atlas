import React from "react";

import type { OntologyDomainSummary } from "../../types.js";
import {
  TerminalTwoPaneScreen,
  getDerivedTagTerminalListNavigationAction,
  getNormalizedKeyName,
  getTerminalPaneBodyHeight,
  useDerivedTagTerminalApp,
  useDerivedTagTerminalInput,
  useDerivedTagTerminalSize,
} from "../terminal-ui.js";
import { buildScrollableLines } from "../list-utils.js";

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
  const selectedDomain = domains[selectedIndex];
  const bodyHeight = Math.max(1, getTerminalPaneBodyHeight(size.height, {
    hasSubtitle: true,
    footerLineCount: 2,
  }));

  useDerivedTagTerminalInput((input, key) => {
    const normalized = getNormalizedKeyName(input, key);
    const navigation = getDerivedTagTerminalListNavigationAction(normalized, {
      pageSize: Math.max(1, bodyHeight - 1),
      jumpSize: Math.max(1, Math.floor(bodyHeight / 2)),
      includeConfirmKeys: true,
      includeHorizontalConfirmKeys: true,
      includeVimHorizontalConfirmKeys: true,
    });
    if (normalized === "ctrl_c" || normalized === "q" || normalized === "escape" || normalized === "backspace") {
      onBack();
      return;
    }
    if (navigation?.kind === "move") {
      onMove(navigation.delta, domains.length);
      return;
    }
    if (navigation?.kind === "boundary") {
      onMove(navigation.boundary === "start" ? -selectedIndex : domains.length - 1 - selectedIndex, domains.length);
      return;
    }
    if (navigation?.kind === "confirm") {
      onOpenSelected();
      return;
    }
    if (normalized === "?") {
      void terminal.showDialog({
        title: "Ontology Domains",
        body: [
          { text: "Choose which ontology-backed browse surface to open.", tone: "section" },
          { text: "Derived Tags: authored tag ontology with live record coverage." },
          { text: "Categories: public catalog category and subcategory browsing." },
          { text: "Search Semantics: metadata field and search-vocabulary discovery." },
        ],
        footer: [{ text: "Press any key to return.", tone: "dim" }],
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
          ? [
            { text: selectedDomain.label, tone: "section" },
            { text: selectedDomain.description },
          ]
          : [{ text: "No domain selected.", tone: "dim" }],
      }}
      footer={[
        { text: "Up/Down move  Ctrl-U/D jump  PgUp/PgDn page  Home/End edge  Enter/right/l select  ? help  q back", tone: "dim" },
        { text: selectedDomain?.label ?? "-", tone: "accent" },
      ]}
      leftWidth={32}
    />
  );
}
