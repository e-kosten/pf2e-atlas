import React from "react";

import { TerminalPaneScreen, TerminalTwoPaneScreen } from "../framework/screen-components.js";
import { useFilterExplorerController } from "./controller.js";
import { buildFilterExplorerScreenModel } from "./screen-models.js";
import type { FilterExplorerOptions } from "./types.js";

export function FilterExplorerScreen(props: FilterExplorerOptions): React.JSX.Element {
  const controller = useFilterExplorerController(props);
  const screen = buildFilterExplorerScreenModel(controller);

  if (screen.kind === "detail-only") {
    return <TerminalPaneScreen {...screen.props} />;
  }

  return <TerminalTwoPaneScreen {...screen.props} />;
}
