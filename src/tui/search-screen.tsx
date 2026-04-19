import React from "react";

import type { OntologyNodeQuery } from "../types.js";
import { OntologyPickerScreen } from "./ontology-explorer/picker-screen.js";
import {
  buildSearchQueryFieldBuilderDetailLines,
  buildSearchQueryFieldBuilderStatusLine,
} from "./search-query-field-builder-session.js";
import { useSearchScreenController } from "./search-screen-controller.js";
import type { SearchScreenOrigin } from "./search-workflow-types.js";
import { TerminalTwoPaneScreen } from "./terminal-ui.js";
import { TerminalMenuScreen } from "./shared-screens.js";
import { formatTerminalInteractionFooter } from "./interaction-bindings.js";

export { parseJumpToResultInput } from "./search-screen-model.js";

export function SearchScreen({
  initialQuery,
  origin = "app",
  onBack,
}: {
  initialQuery?: OntologyNodeQuery;
  origin?: SearchScreenOrigin;
  onBack: () => void;
}): React.JSX.Element {
  const controller = useSearchScreenController({
    initialQuery,
    origin,
    onBack,
  });

  if (controller.selectionPickerSession) {
    return (
      <OntologyPickerScreen
        model={controller.selectionPickerSession.model}
        initialSelections={controller.selectionPickerSession.initialSelections}
        onApply={controller.selectionPickerSession.applySelection}
      />
    );
  }

  if (controller.builderSession) {
    return (
      <TerminalMenuScreen
        title="Query Field Builder"
        subtitle="Build a staged multi-field clause before applying it to the live query"
        leftTitle="[QUERY FIELDS]"
        rightTitle="Builder Detail"
        items={controller.builderSession.items}
        selectedIndex={controller.builderSession.selectedIndex}
        interactionActions={[
          { id: "move", label: "select" },
          { id: "jump" },
          { id: "page" },
          { id: "edge" },
          { id: "select", label: "open" },
          { id: "help" },
          { id: "back", label: "cancel" },
          { id: "quit", label: "cancel" },
        ]}
        footer={[
          {
            text: formatTerminalInteractionFooter([
              { id: "move", label: "select" },
              { id: "jump" },
              { id: "page" },
              { id: "edge" },
              { id: "select", label: "open" },
              { id: "help" },
              { id: "back", label: "cancel" },
            ]),
            tone: "dim",
          },
        ]}
        status={buildSearchQueryFieldBuilderStatusLine(controller.builderSession)}
        helpTitle="Query Field Builder Help"
        helpBody={[
          { text: "Use the builder to stage edits across multiple query fields.", tone: "section" },
          { text: "Open a field to edit it, then return here to continue with another field." },
          { text: "Finish applies the staged clause to the live query. Cancel discards the staged clause." },
        ]}
        buildDetailLines={() => buildSearchQueryFieldBuilderDetailLines(controller.builderSession!)}
        onMove={controller.builderSession.moveSelection}
        onSelect={controller.builderSession.selectCurrent}
        onBack={controller.builderSession.cancel}
      />
    );
  }

  return <TerminalTwoPaneScreen {...controller.screen} />;
}
