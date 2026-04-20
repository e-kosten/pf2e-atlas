import React from "react";

import type { OntologyDomainModel, OntologyNodeQuery } from "../../domain/ontology-types.js";
import { TerminalPaneScreen, TerminalTwoPaneScreen } from "../framework/rendering.js";
import { showTerminalReturnDialog, useTerminalInteractionContextAdapters } from "../interaction-context-adapters.js";
import { createOntologyBrowserSnapshot, useOntologyExplorerController } from "./controller.js";
import {
  buildOntologyBrowserHelpLines,
  buildOntologyBrowserScreenModel,
  buildOntologyCommandEntries,
  getOntologyBrowserInteractionActions,
} from "./screen-models.js";
import type { OntologyBrowserSnapshot } from "./ui.js";
import { buildOntologyBrowserListLines } from "./ui.js";

type OntologyResultReaderLaunchQuery = OntologyNodeQuery & {
  openInResults?: boolean;
};

function markQueryToOpenInResults(query: OntologyNodeQuery): OntologyNodeQuery {
  return {
    ...query,
    openInResults: true,
  } as OntologyResultReaderLaunchQuery;
}

export function OntologyBrowserScreen({
  initialSnapshot,
  model,
  onExit,
  onOpenQuery,
  mode = "browse",
}: {
  initialSnapshot?: OntologyBrowserSnapshot;
  model: OntologyDomainModel;
  onExit: () => void;
  onOpenQuery?: (query: OntologyNodeQuery, snapshot: OntologyBrowserSnapshot) => void;
  mode?: "browse" | "inspect-and-open";
}): React.JSX.Element {
  const adapters = useTerminalInteractionContextAdapters();
  const controller = useOntologyExplorerController({
    initialSnapshot,
    model,
    onExit,
    onOpenQuery,
    onConfirm: (context) => {
      const { currentNode, currentNodeHasChildren } = context;
      if (currentNode?.query?.kind === "listRecords" && mode === "inspect-and-open" && !currentNodeHasChildren) {
        onOpenQuery?.(markQueryToOpenInResults(currentNode.query), createOntologyBrowserSnapshot(context));
        return true;
      }
      return false;
    },
    getInteractionActions: getOntologyBrowserInteractionActions,
    onAction: (action, keyContext) => {
      if (action.id === "commands") {
        const commandEntries = buildOntologyCommandEntries(keyContext, onOpenQuery, mode);
        if (commandEntries.length === 0) {
          return true;
        }
        void adapters
          .promptCommandPalette({
            title: "Ontology Commands",
            prompt: "Filter ontology commands",
            entries: commandEntries,
          })
          .then((selected) => {
            if (selected === "openResults" && keyContext.selectedQuery?.kind === "listRecords") {
              onOpenQuery?.(markQueryToOpenInResults(keyContext.selectedQuery), createOntologyBrowserSnapshot(keyContext));
              return;
            }
            if (selected === "openQuery" && keyContext.selectedQuery) {
              onOpenQuery?.(keyContext.selectedQuery, createOntologyBrowserSnapshot(keyContext));
            }
          });
        return true;
      }
      if (action.id !== "help") {
        return false;
      }
      void showTerminalReturnDialog(
        adapters,
        "Ontology Browser Help",
        buildOntologyBrowserHelpLines(keyContext, onOpenQuery, mode),
      );
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
