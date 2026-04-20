import React from "react";

import type { OntologyDomainSummary } from "../../domain/ontology-types.js";
import {
  type TerminalInteractionAction,
} from "../interaction-bindings.js";
import {
  createMergedReturnFooterBinding,
  createSharedReturnInteractionActions,
} from "../shell-navigation-copy.js";
import type { DerivedTagTerminalLine } from "../framework/types.js";
import { TerminalMenuScreen, type TerminalMenuScreenInteractions } from "../shared-screens.js";

function getOntologyDomainPickerInteractionActions(): TerminalInteractionAction[] {
  return [
    { id: "select", helpText: "open the selected domain" },
    ...createSharedReturnInteractionActions().map((action) => ({
      ...action,
      helpText: "return to the previous area",
    })),
    { id: "help", helpText: "show this help" },
  ];
}

function createOntologyDomainPickerInteractions(): TerminalMenuScreenInteractions {
  return {
    actions: getOntologyDomainPickerInteractionActions(),
    footerBindings: [
      { kind: "action", action: { id: "select" } },
      { kind: "action", action: { id: "help" } },
      createMergedReturnFooterBinding(),
    ],
    help: {
      title: "Ontology Domains",
      sections: [
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
          actions: getOntologyDomainPickerInteractionActions(),
        },
        {
          title: "Domains",
          lines: [
            { text: "Categories: public catalog category and subcategory browsing." },
            { text: "Search Semantics: metadata field and search-vocabulary discovery." },
          ],
        },
      ],
    },
  };
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
  domains: readonly OntologyDomainSummary[];
  selectedIndex: number;
  onBack: () => void;
  onMove: (delta: number, itemCount: number) => void;
  onOpenSelected: () => void;
}): React.JSX.Element {
  const selectedDomain = domains[selectedIndex];
  const interactions = createOntologyDomainPickerInteractions();

  return (
    <TerminalMenuScreen
      title="Ontology Browser"
      subtitle="Choose an ontology-backed browse domain"
      leftTitle="Domains"
      rightTitle="Domain Details"
      items={domains}
      selectedIndex={selectedIndex}
      interactions={interactions}
      status={{ text: selectedDomain?.label ?? "-", tone: "accent" }}
      buildDetailLines={buildOntologyDomainDetailLines}
      onMove={onMove}
      onSelect={onOpenSelected}
      onBack={onBack}
    />
  );
}
