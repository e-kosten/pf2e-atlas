import React from "react";

import type { OntologyDomainModel, OntologyNodeQuery } from "../../types.js";
import {
  TerminalPaneScreen,
  TerminalTwoPaneScreen,
  useDerivedTagTerminalApp,
} from "../terminal-ui.js";
import { TERMINAL_DIALOG_RETURN_FOOTER } from "../interaction-bindings.js";
import { useOntologyExplorerController } from "./controller.js";
import {
  buildOntologyBrowserHelpLines,
  buildOntologyBrowserScreenModel,
  buildOntologyCommandEntries,
  getOntologyBrowserInteractionActions,
} from "./screen-models.js";
import { buildOntologyBrowserListLines } from "./ui.js";

export function OntologyBrowserScreen({
  model,
  onExit,
  onOpenQuery,
}: {
  model: OntologyDomainModel;
  onExit: () => void;
  onOpenQuery?: (query: OntologyNodeQuery) => void;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const controller = useOntologyExplorerController({
    model,
    onExit,
    onOpenQuery,
    onConfirm: ({ currentNode, currentNodeHasChildren }) => {
      if (!currentNodeHasChildren && currentNode?.query?.kind === "listRecords" && model.id !== "derivedTags") {
        onOpenQuery?.(currentNode.query);
        return true;
      }
      return false;
    },
    getInteractionActions: getOntologyBrowserInteractionActions,
    onAction: (action, keyContext) => {
      if (action.id === "commands") {
        const commandEntries = buildOntologyCommandEntries(keyContext, onOpenQuery);
        if (commandEntries.length === 0) {
          return true;
        }
        void terminal
          .promptCommandPalette({
            title: "Ontology Commands",
            prompt: "Filter ontology commands",
            entries: commandEntries,
          })
          .then((selected) => {
            if (selected === "openQuery" && keyContext.selectedQuery) {
              onOpenQuery?.(keyContext.selectedQuery);
            }
          });
        return true;
      }
      if (action.id !== "help") {
        return false;
      }
      void terminal.showDialog({
        title: "Ontology Browser Help",
        body: buildOntologyBrowserHelpLines(keyContext, onOpenQuery),
        footer: [{ text: TERMINAL_DIALOG_RETURN_FOOTER, tone: "dim" }],
      });
      return true;
    },
  });

  const screen = buildOntologyBrowserScreenModel({
    model,
    controller,
    leftLines: buildOntologyBrowserListLines(model, controller.effectiveState, controller.bodyHeight),
  });

  if (screen.kind === "detail-only") {
    return <TerminalPaneScreen {...screen.props} />;
  }

  return <TerminalTwoPaneScreen {...screen.props} />;
}
