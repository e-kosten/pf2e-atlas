import React from "react";

import type { OntologyNodeQuery } from "../../domain/ontology-types.js";
import type { Pf2eTerminalSearchSession } from "../search/service.js";
import {
  appendRouteTransitionFooterLine,
  type RouteTransitionStatus,
} from "../route-transition-status.js";
import {
  buildSearchStructuredEditorDetailLines,
  buildSearchStructuredEditorFooterText,
  buildSearchStructuredEditorHelpLines,
  getSearchStructuredEditorInteractionActions,
  buildSearchStructuredEditorMenuItems,
  buildSearchStructuredEditorStatusLine,
} from "./query-field-builder-session.js";
import { useSearchScreenController } from "./controller.js";
import { SearchFilterExplorerScreen } from "./filter-explorer-screen.js";
import type { SearchScreenOrigin } from "./workflow-types.js";
import { TerminalTwoPaneScreen } from "../framework/rendering.js";
import { TerminalMenuScreen } from "../shared-screens.js";

export { parseJumpToResultInput } from "./model.js";

export function SearchScreen({
  initialQuery,
  initialSession,
  transitionStatus,
  origin = "app",
  onBack,
}: {
  initialQuery?: OntologyNodeQuery;
  initialSession?: Pf2eTerminalSearchSession;
  transitionStatus?: RouteTransitionStatus | null;
  origin?: SearchScreenOrigin;
  onBack: () => void;
}): React.JSX.Element {
  const controller = useSearchScreenController({
    initialQuery,
    initialSession,
    transitionStatus,
    origin,
    onBack,
  });

  if (controller.filterExplorerSession) {
    return <SearchFilterExplorerScreen session={controller.filterExplorerSession} />;
  }

  if (controller.structuredEditorSession) {
    const session = controller.structuredEditorSession;
    return (
      <TerminalMenuScreen
        title={session.title ?? "Structured Query Editor"}
        subtitle={session.subtitle ?? "Stage structured search changes before applying them to the live query"}
        leftTitle={session.leftTitle ?? "[STAGED QUERY]"}
        rightTitle={session.rightTitle ?? "Staged Summary & Detail"}
        items={buildSearchStructuredEditorMenuItems(session)}
        selectedIndex={session.selectedIndex}
        interactionActions={getSearchStructuredEditorInteractionActions(session)}
        footer={[
          {
            text: buildSearchStructuredEditorFooterText(session),
            tone: "dim",
          },
        ]}
        status={buildSearchStructuredEditorStatusLine(session)}
        transitionStatus={transitionStatus}
        helpTitle={session.helpTitle ?? "Structured Query Editor Help"}
        helpBody={buildSearchStructuredEditorHelpLines(session)}
        buildDetailLines={() => buildSearchStructuredEditorDetailLines(session)}
        onMove={session.moveSelection}
        onSelect={session.selectCurrent}
        onBack={session.cancel}
      />
    );
  }

  return (
    <TerminalTwoPaneScreen
      {...controller.screen}
      footer={appendRouteTransitionFooterLine(controller.screen.footer ?? [], transitionStatus)}
    />
  );
}
