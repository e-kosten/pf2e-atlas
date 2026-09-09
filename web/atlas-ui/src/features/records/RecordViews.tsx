import { ExternalLink, X } from "lucide-react";
import type React from "react";
import { AddToListButton } from "../lists/AddToListButton";
import { PaneFrame, ResizablePaneGroup } from "../../shared/layout/PaneLayout";
import { RecordDetailPane } from "../../shared/records/RecordDetailPane";
import { useRecordDetail } from "../../shared/records/useRecordDetail";
import { PaneIconButton, PaneIconLink } from "../../shared/ui/actions/PaneAction";
import {
  atlasRoutePath,
  navigateToAtlasRoute,
  shouldHandleAtlasRouteClick,
  type AtlasRoute,
} from "../../app/routes";

type RecordViewProps = {
  route: Extract<AtlasRoute, { kind: "record" }>;
};

type ReaderViewProps = {
  route: Extract<AtlasRoute, { kind: "reader" }>;
};

const RECORD_VIEW_WIDTH_SPECS = {
  preview: { defaultWidth: 420, minWidth: 320 },
};

export function RecordView({ route }: RecordViewProps) {
  const detail = useRecordDetail(
    route.recordKey,
    undefined,
    undefined,
    route.childLocator ? { child_locator: route.childLocator } : undefined,
  );
  return (
    <RecordViewLayout
      primary={
        <RecordPane
          actions={
            detail.data ? (
              <>
                <AddToListButton recordKey={route.recordKey} />
                <RouteLink
                  route={{
                    kind: "reader",
                    recordKey: route.recordKey,
                    previewRecordKey: null,
                  }}
                >
                  Reader view
                </RouteLink>
              </>
            ) : null
          }
          title="Record Detail"
        >
          <RecordDetailPane
            detail={detail.data}
            errors={[detail.error]}
            loading={detail.isLoading}
            onReference={(recordKey, childLocator) =>
              navigateToAtlasRoute({ kind: "record", recordKey, childLocator })
            }
            stale={detail.isFetching && Boolean(detail.data)}
          />
        </RecordPane>
      }
    />
  );
}

export function ReaderView({ route }: ReaderViewProps) {
  const detail = useRecordDetail(route.recordKey);
  const preview = useRecordDetail(route.previewRecordKey);
  return (
    <RecordViewLayout
      primary={
        <RecordPane
          actions={
            detail.data ? (
              <>
                <AddToListButton recordKey={route.recordKey} />
                <RouteLink route={{ kind: "record", recordKey: route.recordKey }}>
                  Detail page
                </RouteLink>
              </>
            ) : null
          }
          title="Reader"
        >
          <RecordDetailPane
            detail={detail.data}
            errors={[detail.error]}
            loading={detail.isLoading}
            onReference={(previewRecordKey) =>
              navigateToAtlasRoute({
                kind: "reader",
                recordKey: route.recordKey,
                previewRecordKey,
              })
            }
            stale={detail.isFetching && Boolean(detail.data)}
          />
        </RecordPane>
      }
      auxiliary={
        <RecordPane
          actions={
            route.previewRecordKey &&
            preview.data && (
              <>
                <AddToListButton recordKey={route.previewRecordKey} />
                <RouteIconLink
                  label="Open preview as reader"
                  route={{
                    kind: "reader",
                    recordKey: route.previewRecordKey,
                    previewRecordKey: null,
                  }}
                />
                <PaneIconButton
                  icon={<X size={16} />}
                  label="Close preview"
                  onClick={() =>
                    navigateToAtlasRoute({
                      kind: "reader",
                      recordKey: route.recordKey,
                      previewRecordKey: null,
                    })
                  }
                />
              </>
            )
          }
          title="Preview"
        >
          <RecordDetailPane
            detail={route.previewRecordKey ? preview.data : undefined}
            emptyMessage="Select a linked record to preview it."
            errors={[preview.error]}
            loading={route.previewRecordKey ? preview.isLoading : false}
            onReference={(previewRecordKey) =>
              navigateToAtlasRoute({
                kind: "reader",
                recordKey: route.recordKey,
                previewRecordKey,
              })
            }
            stale={
              Boolean(route.previewRecordKey) &&
              preview.isFetching &&
              Boolean(preview.data)
            }
          />
        </RecordPane>
      }
    />
  );
}

function RecordViewLayout({
  auxiliary,
  primary,
}: {
  auxiliary?: React.ReactNode;
  primary: React.ReactNode;
}) {
  return auxiliary ? (
    <ResizablePaneGroup
      className="record-view record-view--with-aux"
      widthSpecs={RECORD_VIEW_WIDTH_SPECS}
      items={(widths) => [
        {
          kind: "pane",
          key: "primary",
          column: "minmax(0, 1fr)",
          content: primary,
        },
        {
          kind: "handle",
          key: "primary-preview",
          label: "Resize preview",
          resizePane: "preview",
          deltaMultiplier: -1,
        },
        {
          kind: "pane",
          key: "preview",
          column: `minmax(0, ${widths.preview}px)`,
          content: auxiliary,
        },
      ]}
    />
  ) : (
    <main className="record-view">{primary}</main>
  );
}

function RecordPane({
  actions,
  children,
  title,
}: {
  actions?: React.ReactNode;
  children: React.ReactNode;
  title: string;
}) {
  return (
    <PaneFrame className="record-view__pane" headerActions={actions} label={title}>
      {children}
    </PaneFrame>
  );
}

function RouteIconLink({ label, route }: { label: string; route: AtlasRoute }) {
  return (
    <PaneIconLink
      href={atlasRoutePath(route)}
      icon={<ExternalLink size={16} />}
      label={label}
      onClick={(event) => {
        if (!shouldHandleAtlasRouteClick(event)) {
          return;
        }
        event.preventDefault();
        navigateToAtlasRoute(route);
      }}
    />
  );
}

function RouteLink({
  children,
  route,
}: {
  children: React.ReactNode;
  route: AtlasRoute;
}) {
  return (
    <a
      className="record-view__link"
      href={atlasRoutePath(route)}
      onClick={(event) => {
        if (!shouldHandleAtlasRouteClick(event)) {
          return;
        }
        event.preventDefault();
        navigateToAtlasRoute(route);
      }}
    >
      {children}
    </a>
  );
}
