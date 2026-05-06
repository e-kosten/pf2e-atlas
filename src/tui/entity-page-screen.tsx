import React from "react";

import type { EntityPageDocument, EntityPageTarget } from "../app/ontology/entity-page.js";
import type { RecordKey } from "../domain/record-types.js";
import type { FilterExplorerNode, FilterExplorerOptions } from "./filter-explorer/types.js";
import { FilterExplorerScreen } from "./filter-explorer/screen.js";
import { createInspectFilterExplorerHostAdapter } from "./filter-explorer/host-adapter.js";
import { usePf2eTerminalAppServices } from "./app-service-context.js";
import { buildPageDocumentModel } from "./page-document/model.js";
import type { RouteTransitionStatus } from "./route-transition-status.js";

function createEntityPageNode(recordKey: RecordKey, title: string): FilterExplorerNode {
  return {
    id: recordKey,
    kind: "record",
    label: title,
    listLabel: title,
    filterText: title.toLowerCase(),
    detailTitle: title,
    detailLines: [{ text: title, tone: "section" }],
  };
}

export function EntityPageScreen({
  document: initialDocument,
  onBack,
  onActivatePageTarget,
  transitionStatus,
}: {
  document: EntityPageDocument;
  onBack: () => void;
  onActivatePageTarget?: (target: EntityPageTarget) => boolean | void;
  transitionStatus?: RouteTransitionStatus | null;
}): React.JSX.Element {
  const { user } = usePf2eTerminalAppServices();
  const [activeDocument, setActiveDocument] = React.useState(initialDocument);
  const activeRecordKey = activeDocument.recordKey;

  React.useEffect(() => {
    setActiveDocument(initialDocument);
  }, [initialDocument]);

  const model = React.useMemo(
    () => ({
      id: "searchSemantics" as const,
      label: activeDocument.title,
      description: `Entity page for ${activeDocument.title}`,
      rootNodes: [createEntityPageNode(activeRecordKey, activeDocument.title)],
    }),
    [activeDocument.title, activeRecordKey],
  );
  const host = React.useMemo(
    () =>
      createInspectFilterExplorerHostAdapter({
        resolvePageDocument: (node) =>
          node?.id === activeRecordKey ? buildPageDocumentModel(activeDocument) : null,
        activatePageTarget: ({ target }) => {
          if (target.kind === "record" && target.action === "preview") {
            const previewDocument = user.entityPages.buildDocumentByRecordKey(target.recordKey);
            if (!previewDocument) {
              return false;
            }
            setActiveDocument(previewDocument);
            return true;
          }
          return onActivatePageTarget?.(target);
        },
      }),
    [activeDocument, activeRecordKey, onActivatePageTarget, user.entityPages],
  );
  const initialSnapshot = React.useMemo<FilterExplorerOptions["initialSnapshot"]>(
    () => ({
      activePane: "detail",
      layoutMode: "detail-only",
      browserState: {
        depth: 0,
        selectedNodeIds: [activeRecordKey],
        filter: "",
        detailScroll: 0,
      },
      materializedChildrenByNodeId: new Map(),
      searchInput: "",
      searchMode: false,
    }),
    [activeRecordKey],
  );

  return (
    <FilterExplorerScreen
      title={activeDocument.title}
      model={model}
      host={host}
      initialSnapshot={initialSnapshot}
      onOutcome={(outcome) => {
        if (outcome.kind === "back" || outcome.kind === "cancel" || outcome.kind === "exitRoot") {
          onBack();
        }
      }}
      transitionStatus={transitionStatus}
      mode={{ kind: "inspect-and-open" }}
    />
  );
}
