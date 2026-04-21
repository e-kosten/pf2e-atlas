import React from "react";

import { FilterExplorerScreen } from "../filter-explorer/index.js";
import { cloneFilterExplorerDraft, withFilterExplorerComposeDraft } from "../filter-explorer/search-draft.js";
import { useDerivedTagTerminalApp } from "../framework/context.js";
import { useTerminalInteractionContextAdapters } from "../interaction-context-adapters.js";
import type { SearchFilterExplorerSession } from "./query-field-builder/query-field-builder-session.js";
import { promptNumericScalarClause } from "../filter-explorer/scalar-editor.js";

export function SearchFilterExplorerScreen({
  session,
}: {
  session: SearchFilterExplorerSession;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const prompts = useTerminalInteractionContextAdapters();
  const [draft, setDraft] = React.useState(() => cloneFilterExplorerDraft(session.draft));
  const draftRef = React.useRef(session.draft);

  React.useEffect(() => {
    const nextDraft = cloneFilterExplorerDraft(session.draft);
    draftRef.current = nextDraft;
    setDraft(nextDraft);
  }, [session.draft]);

  const applyDraft = React.useCallback(() => {
    session.onApply(draftRef.current);
  }, [session]);

  const updateDraft = React.useCallback((nextComposeDraft: Pick<typeof draft, "selection" | "scalarClauses">) => {
    const nextDraft = withFilterExplorerComposeDraft(draftRef.current, nextComposeDraft);
    draftRef.current = nextDraft;
    setDraft(nextDraft);
  }, []);

  return (
    <FilterExplorerScreen
      title={session.title}
      model={session.model}
      rootDepth={0}
      exitAtRootDepth
      onExit={applyDraft}
      mode={{
        kind: "compose",
        draft,
        onDraftChange: (nextComposeDraft) => {
          updateDraft(nextComposeDraft);
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
