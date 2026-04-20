import React from "react";

import { FilterExplorerScreen } from "../filter-explorer/index.js";
import type { SearchFilterExplorerSession } from "./query-field-builder-session.js";

export function SearchFilterExplorerScreen({
  session,
}: {
  session: SearchFilterExplorerSession;
}): React.JSX.Element {
  const [draft, setDraft] = React.useState(session.draft);
  const draftRef = React.useRef(draft);

  React.useEffect(() => {
    draftRef.current = session.draft;
    setDraft(session.draft);
  }, [session.draft]);

  const applyDraft = React.useCallback(() => {
    session.onApply(draftRef.current);
  }, [session]);

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
        selection: draft.fieldSelections,
        onSelectionChange: (selection) => {
          const nextDraft = {
            ...draftRef.current,
            fieldSelections: selection,
          };
          draftRef.current = nextDraft;
          setDraft(nextDraft);
        },
        resolveSelectionTarget: session.resolveSelectionTarget,
      }}
    />
  );
}
