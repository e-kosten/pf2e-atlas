import { ExternalLink, X } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import type React from "react";
import { getRecordDetail } from "../../api/atlasApi";
import { AddToListButton } from "../lists/AddToListButton";
import { PaneFrame, ResizablePaneGroup } from "../../shared/layout/PaneLayout";
import { RecordPresentation } from "../../shared/records/RecordPresentation";
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
  const detail = useRecordDetail(route.recordKey);
  return (
    <RecordViewLayout
      primary={
        <RecordPane
          actions={
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
          }
          title="Record Detail"
        >
          <RecordPresentation
            detail={detail.data}
            loading={detail.isLoading || detail.isFetching}
            onReference={(recordKey) =>
              navigateToAtlasRoute({ kind: "record", recordKey })
            }
          />
          {detail.error && <InlineError message={detail.error.message} />}
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
            <>
              <AddToListButton recordKey={route.recordKey} />
              <RouteLink route={{ kind: "record", recordKey: route.recordKey }}>
                Detail page
              </RouteLink>
            </>
          }
          title="Reader"
        >
          <RecordPresentation
            detail={detail.data}
            loading={detail.isLoading || detail.isFetching}
            onReference={(previewRecordKey) =>
              navigateToAtlasRoute({
                kind: "reader",
                recordKey: route.recordKey,
                previewRecordKey,
              })
            }
          />
          {detail.error && <InlineError message={detail.error.message} />}
        </RecordPane>
      }
      auxiliary={
        <RecordPane
          actions={
            route.previewRecordKey && (
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
                <button
                  aria-label="Close preview"
                  className="pane-toggle"
                  onClick={() =>
                    navigateToAtlasRoute({
                      kind: "reader",
                      recordKey: route.recordKey,
                      previewRecordKey: null,
                    })
                  }
                  title="Close preview"
                  type="button"
                >
                  <X size={16} />
                </button>
              </>
            )
          }
          title="Preview"
        >
          {route.previewRecordKey ? (
            <RecordPresentation
              detail={preview.data}
              loading={preview.isLoading || preview.isFetching}
              onReference={(previewRecordKey) =>
                navigateToAtlasRoute({
                  kind: "reader",
                  recordKey: route.recordKey,
                  previewRecordKey,
                })
              }
            />
          ) : (
            <div className="detail-empty">Select a linked record to preview it.</div>
          )}
          {preview.error && <InlineError message={preview.error.message} />}
        </RecordPane>
      }
    />
  );
}

function useRecordDetail(recordKey: string | null) {
  return useQuery({
    queryKey: ["record-detail", recordKey],
    queryFn: () => getRecordDetail(recordKey!),
    enabled: recordKey !== null,
  });
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
      <div className="detail-panel">{children}</div>
    </PaneFrame>
  );
}

function InlineError({ message }: { message: string }) {
  return <div className="error-banner">{message}</div>;
}

function RouteIconLink({ label, route }: { label: string; route: AtlasRoute }) {
  return (
    <a
      aria-label={label}
      className="pane-toggle"
      href={atlasRoutePath(route)}
      onClick={(event) => {
        if (!shouldHandleAtlasRouteClick(event)) {
          return;
        }
        event.preventDefault();
        navigateToAtlasRoute(route);
      }}
      title={label}
    >
      <ExternalLink size={16} />
    </a>
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
