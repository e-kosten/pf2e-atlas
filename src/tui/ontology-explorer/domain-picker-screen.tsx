import React from "react";

import type { OntologyDomainSummary } from "../../types.js";
import {
  buildTerminalInteractionHelpLines,
  formatTerminalInteractionFooter,
  type TerminalInteractionAction,
} from "../interaction-bindings.js";
import type { DerivedTagTerminalLine } from "../terminal-ui.js";
import { TerminalMenuScreen } from "../shared-screens.js";

function getOntologyDomainPickerInteractionActions(): TerminalInteractionAction[] {
  return [{ id: "select" }, { id: "back", label: "back" }, { id: "help" }, { id: "quit", label: "back" }];
}

function buildOntologyDomainPickerHelpLines(): DerivedTagTerminalLine[] {
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

function buildOntologyDomainDetailLines(selectedDomain: OntologyDomainSummary | undefined): DerivedTagTerminalLine[] {
  return selectedDomain
    ? [{ text: selectedDomain.label, tone: "section" }, { text: selectedDomain.description }]
    : [{ text: "No domain selected.", tone: "dim" }];
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
  const selectedDomain = domains[selectedIndex];

  return (
    <TerminalMenuScreen
      title="Ontology Search"
      subtitle="Choose an ontology-backed browse domain"
      leftTitle="Domains"
      rightTitle="Domain Details"
      items={domains}
      selectedIndex={selectedIndex}
      interactionActions={getOntologyDomainPickerInteractionActions()}
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
      ]}
      status={{ text: selectedDomain?.label ?? "-", tone: "accent" }}
      helpTitle="Ontology Domains"
      helpBody={buildOntologyDomainPickerHelpLines()}
      buildDetailLines={buildOntologyDomainDetailLines}
      onMove={onMove}
      onSelect={onOpenSelected}
      onBack={onBack}
    />
  );
}
