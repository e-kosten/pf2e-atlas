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

  if (controller.queryFieldPickerSession) {
    return (
      <OntologyPickerScreen
        model={controller.queryFieldPickerSession.model}
        initialSelections={controller.queryFieldPickerSession.initialSelections}
        onApply={controller.queryFieldPickerSession.applySelection}
      />
    );
  }

  return <TerminalTwoPaneScreen {...controller.screen} />;
}
