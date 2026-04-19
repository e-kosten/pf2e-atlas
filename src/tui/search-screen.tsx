import React from "react";

import type { OntologyNodeQuery } from "../types.js";
import { OntologyPickerScreen } from "./ontology-explorer/picker-screen.js";
import { useSearchScreenController } from "./search-screen-controller.js";
import type { SearchScreenOrigin } from "./search-workflow-types.js";
import { TerminalTwoPaneScreen } from "./terminal-ui.js";

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

  return <TerminalTwoPaneScreen {...controller.screen} />;
}
