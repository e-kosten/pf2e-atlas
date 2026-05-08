import React from "react";

import { TerminalPaneScreen, TerminalTwoPaneScreen } from "../framework/screen-components.js";
import { useTerminalTwoPaneResize } from "../framework/two-pane-resize.js";
import { useFilterExplorerController } from "./controller.js";
import { buildFilterExplorerScreenModel } from "./screen-models.js";
import type { FilterExplorerOptions } from "./types.js";

const FILTER_EXPLORER_LEFT_WIDTH = 46;

export function FilterExplorerScreen(props: FilterExplorerOptions): React.JSX.Element {
  const twoPaneResize = useTerminalTwoPaneResize({ defaultLeftWidth: FILTER_EXPLORER_LEFT_WIDTH });
  const controller = useFilterExplorerController({
    ...props,
    layout: twoPaneResize,
  });
  const span = props.debugTrace?.startSpan("filterExplorer.buildScreenModel", {
    model: props.model.id,
    roots: props.model.rootNodes.length,
  });
  const screen = buildFilterExplorerScreenModel(controller);
  span?.end({ footerLines: screen.props.footer?.length ?? 0 });

  if (screen.kind === "detail-only") {
    return <TerminalPaneScreen {...screen.props} />;
  }

  return <TerminalTwoPaneScreen {...screen.props} />;
}
