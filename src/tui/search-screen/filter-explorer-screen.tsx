import React from "react";

import { FilterExplorerScreen } from "../filter-explorer/screen.js";
import { cloneFilterExplorerComposeDraft } from "../filter-explorer/compose-state.js";
import { useDerivedTagTerminalApp } from "../framework/context.js";
import { useTerminalInteractionContextAdapters } from "../interaction-context-adapters.js";
import type { SearchFilterExplorerSession } from "./query-field-builder/query-field-builder-session.js";
import { promptNumericScalarClause } from "../filter-explorer/scalar-editor.js";
import type { FilterExplorerDiscoveryMode, FilterExplorerDiscoveryState } from "../filter-explorer/types.js";

export function SearchFilterExplorerScreen({
  session,
}: {
  session: SearchFilterExplorerSession;
}): React.JSX.Element {
  const terminal = useDerivedTagTerminalApp();
  const prompts = useTerminalInteractionContextAdapters();
  const initialDiscoveryMode = session.initialDiscoveryMode ?? "matching";
  const [model, setModel] = React.useState(session.model);
  const [discoveryMode, setDiscoveryMode] = React.useState<FilterExplorerDiscoveryMode>(initialDiscoveryMode);
  const [draft, setDraft] = React.useState(() => cloneFilterExplorerComposeDraft(session.draft));
  const draftRef = React.useRef(cloneFilterExplorerComposeDraft(session.draft));
  const refreshRequestIdRef = React.useRef(0);

  React.useEffect(() => {
    setModel(session.model);
    setDiscoveryMode(initialDiscoveryMode);
    const nextDraft = cloneFilterExplorerComposeDraft(session.draft);
    draftRef.current = nextDraft;
    setDraft(nextDraft);
  }, [initialDiscoveryMode, session.draft, session.model]);

  const onDiscoveryModeChange = React.useCallback(
    (nextMode: FilterExplorerDiscoveryMode) => {
      if (nextMode === discoveryMode) {
        return;
      }

      if (!session.loadModelForDiscoveryMode) {
        setDiscoveryMode(nextMode);
        return;
      }

      const requestId = refreshRequestIdRef.current + 1;
      refreshRequestIdRef.current = requestId;
      void session
        .loadModelForDiscoveryMode(nextMode)
        .then((nextModel) => {
          if (refreshRequestIdRef.current !== requestId) {
            return;
          }
          setModel(nextModel);
          setDiscoveryMode(nextMode);
        })
        .catch((error) => {
          if (refreshRequestIdRef.current !== requestId) {
            return;
          }
          void terminal.pauseForAnyKey(`Could not refresh explorer data.\n\n${(error as Error).message}`);
        });
    },
    [discoveryMode, session, terminal],
  );

  const discovery = React.useMemo<FilterExplorerDiscoveryState>(
    () => ({
      mode: discoveryMode,
      onModeChange: onDiscoveryModeChange,
    }),
    [discoveryMode, onDiscoveryModeChange],
  );

  const applyDraft = React.useCallback(() => {
    session.onApply(draftRef.current);
  }, [session]);

  const updateDraft = React.useCallback((nextComposeDraft: typeof draft) => {
    const nextDraft = cloneFilterExplorerComposeDraft(nextComposeDraft);
    draftRef.current = nextDraft;
    setDraft(nextDraft);
  }, []);

  return (
    <FilterExplorerScreen
      title={session.title}
      model={model}
      rootDepth={0}
      exitAtRootDepth
      onExit={applyDraft}
      discovery={discovery}
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
