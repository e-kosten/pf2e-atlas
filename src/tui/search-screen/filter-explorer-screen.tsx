import React from "react";

import { FilterExplorerScreen } from "../filter-explorer/index.js";
import { useDerivedTagTerminalApp } from "../framework/context.js";
import { useTerminalInteractionContextAdapters } from "../interaction-context-adapters.js";
import {
  applyFilterExplorerComposeDraft,
  createFilterExplorerComposeDraft,
} from "../search/service.js";
import type { SearchFilterExplorerSession } from "./query-field-builder-session.js";
import { promptNumericScalarClause } from "./scalar-editor.js";

export function SearchFilterExplorerScreen({
  session,
}: {
  session: SearchFilterExplorerSession;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const prompts = useTerminalInteractionContextAdapters();
  const [composeDraft, setComposeDraft] = React.useState(() => createFilterExplorerComposeDraft(session.draft));
  const draftRef = React.useRef(session.draft);

  React.useEffect(() => {
    draftRef.current = session.draft;
    setComposeDraft(createFilterExplorerComposeDraft(session.draft));
  }, [session.draft]);

  const applyDraft = React.useCallback(() => {
    session.onApply(draftRef.current);
  }, [session]);

  const updateComposeDraft = React.useCallback((nextComposeDraft: typeof composeDraft) => {
    const nextDraft = applyFilterExplorerComposeDraft(draftRef.current, nextComposeDraft);
    const normalizedComposeDraft = createFilterExplorerComposeDraft(nextDraft);
    draftRef.current = nextDraft;
    setComposeDraft(normalizedComposeDraft);
  }, []);

  return (
    <FilterExplorerScreen
      title={session.title}
      model={session.model}
      initialSnapshot={session.initialSnapshot}
      rootDepth={session.rootDepth}
      exitAtRootDepth={session.exitAtRootDepth}
      onExit={applyDraft}
      mode={{
        kind: "compose",
        draft: composeDraft,
        onDraftChange: (nextComposeDraft) => {
          updateComposeDraft(nextComposeDraft);
        },
        resolveSelectionTarget: session.resolveSelectionTarget,
        onEditScalarTarget: async ({ target, currentClause }) => {
          if (target.valueType !== "number") {
            return undefined;
          }

          const nextClause = await promptNumericScalarClause(prompts, terminal, {
            title: target.editorLabel ?? `${target.fieldLabel} / ${target.subjectLabel}`,
            currentClause:
              currentClause?.operator === "between"
                ? { op: "between", min: currentClause.min, max: currentClause.max }
                : typeof currentClause?.value === "number"
                  ? { op: currentClause.operator, value: currentClause.value }
                  : null,
          });

          if (nextClause === undefined) {
            return undefined;
          }
          if (nextClause === null) {
            return null;
          }

          return nextClause.op === "between"
            ? { operator: "between", min: nextClause.min, max: nextClause.max }
            : { operator: nextClause.op, value: nextClause.value };
        },
      }}
    />
  );
}
