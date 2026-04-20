import React from "react";

import type { OntologyNodeQuery } from "../../domain/index.js";
import { OntologyPickerScreen } from "../ontology-explorer/picker-screen.js";
import {
  buildSearchStructuredEditorDetailLines,
  buildSearchStructuredEditorMenuItems,
  buildSearchStructuredEditorStatusLine,
} from "./query-field-builder-session.js";
import { useSearchScreenController } from "./controller.js";
import type { SearchScreenOrigin } from "./workflow-types.js";
import { TerminalTwoPaneScreen } from "../framework/rendering.js";
import { TerminalMenuScreen } from "../shared-screens.js";
import { formatTerminalInteractionFooter } from "../interaction-bindings.js";

export { parseJumpToResultInput } from "./model.js";

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

  if (controller.structuredEditorSession) {
    const session = controller.structuredEditorSession;
    const backLabel = session.kind === "structuredEditor" ? "return" : "cancel";
    return (
      <TerminalMenuScreen
        title={session.title ?? "Structured Query Editor"}
        subtitle={session.subtitle ?? "Stage structured search changes before applying them to the live query"}
        leftTitle={session.leftTitle ?? "[STAGED QUERY]"}
        rightTitle={session.rightTitle ?? "Staged Summary & Detail"}
        items={buildSearchStructuredEditorMenuItems(session)}
        selectedIndex={session.selectedIndex}
        interactionActions={[
          { id: "move", label: "select" },
          { id: "jump" },
          { id: "page" },
          { id: "edge" },
          { id: "select", label: "open" },
          { id: "help" },
          { id: "back", label: backLabel },
          { id: "quit", label: backLabel },
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
              { id: "back", label: backLabel },
            ]),
            tone: "dim",
          },
        ]}
        status={buildSearchStructuredEditorStatusLine(session)}
        helpTitle={session.helpTitle ?? "Structured Query Editor Help"}
        helpBody={
          session.helpBody ?? [
            { text: "Use this editor to stage structured search changes before committing them.", tone: "section" },
            { text: "The right pane keeps the full staged query summary visible while you move focus on the left." },
            {
              text: "Open a row to edit it, then continue staging more changes or finish when the draft looks correct.",
            },
          ]
        }
        buildDetailLines={() => buildSearchStructuredEditorDetailLines(session)}
        onMove={session.moveSelection}
        onSelect={session.selectCurrent}
        onBack={session.cancel}
      />
    );
  }

  return <TerminalTwoPaneScreen {...controller.screen} />;
}
