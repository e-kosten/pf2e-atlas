import React from "react";

import {
  buildSearchStructuredEditorDetailLines,
  createSearchStructuredEditorInteractions,
  buildSearchStructuredEditorMenuItems,
  buildSearchStructuredEditorStatusLine,
} from "./query-field-builder/query-field-builder-session.js";
import { useSearchScreenController } from "./controller.js";
import type { SearchScreenProps } from "./entry-props.js";
import { SearchFilterExplorerScreen } from "./filter-explorer-screen.js";
import { TerminalTwoPaneScreen } from "../framework/screen-components.js";
import { TerminalActionMenuScreen } from "../shared-screens.js";

export { parseJumpToResultInput } from "./model.js";

export function SearchScreen(props: SearchScreenProps): React.JSX.Element {
  const controller = useSearchScreenController(props);

  if (controller.filterExplorerSession) {
    return <SearchFilterExplorerScreen session={controller.filterExplorerSession} />;
  }

  if (controller.structuredEditorSession) {
    const session = controller.structuredEditorSession;
    return (
      <TerminalActionMenuScreen
        title={session.title ?? "Structured Query Editor"}
        subtitle={session.subtitle ?? "Edit the live structured query tree"}
        leftTitle={session.leftTitle ?? "[QUERY TREE]"}
        rightTitle={session.rightTitle ?? "Live Summary & Detail"}
        leftWidth={32}
        items={buildSearchStructuredEditorMenuItems(session)}
        selectedIndex={session.selectedIndex}
        interactions={createSearchStructuredEditorInteractions(session)}
        actionEntries={session.actionEntries}
        buildRightLines={() => buildSearchStructuredEditorDetailLines(session)}
        buildStatusLine={() => buildSearchStructuredEditorStatusLine(session)}
        onMove={session.moveSelection}
        onSelect={session.selectCurrent}
        onBack={session.cancel}
        onAction={session.runAction}
        transitionStatus={props.transitionStatus}
      />
    );
  }

  return <TerminalTwoPaneScreen {...controller.screen} />;
}
