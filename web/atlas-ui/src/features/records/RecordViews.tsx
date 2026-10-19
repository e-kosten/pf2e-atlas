import { ExternalLink, X } from "lucide-react";
import type React from "react";
import { useEffect } from "react";
import {
  useReferenceReading,
  rootNavigation,
} from "../../shared/records/useReferenceReading";
import { ReferenceReader } from "../../shared/records/ReferenceReader";
import { Typography } from "antd";
import { AddToListButton } from "../lists/AddToListButton";
import { PaneFrame, ResizablePaneGroup } from "../../shared/layout/PaneLayout";
import { RecordDetailPane } from "../../shared/records/RecordDetailPane";
import { RecordPreviewPopover } from "../../shared/records/RecordPreviewPopover";
import { useRecordPreview } from "../../shared/records/useRecordPreview";
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
  const detail = useRecordDetail(route.recordKey, route.selection);
  const preview = useRecordPreview();
  return (
    <>
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
                    selection: route.selection,
                    landing: route.landing,
                  }}
                >
                  Reader view
                </RouteLink>
              </>
            }
            title="Record Detail"
          >
            <RecordDetailPane
              detail={detail.data}
              error={detail.error}
              onRetry={() => {
                void detail.refetch();
              }}
              loading={detail.isLoading || detail.isFetching}
              onReference={preview.open}
              onRankChange={(spell_rank) => {
                if (detail.data)
                  navigateToAtlasRoute({
                    ...route,
                    selection: { ...detail.data.selected, spell_rank },
                  });
              }}
            />
          </RecordPane>
        }
      />
      {preview.recordKey && (
        <RecordPreviewPopover
          visibleOwnerRecordKey={
            detail.data?.selected.embedded_path.length === 0
              ? detail.data.record.record_key
              : undefined
          }
          anchor={preview.anchor}
          readingHistory={preview.readingHistory}
          landing={preview.landing}
          detail={preview.detail}
          loading={preview.loading}
          error={preview.error}
          onRetry={preview.retry}
          onClose={preview.close}
          onReference={preview.open}
          onRankChange={preview.changeRank}
          onOpenFullPage={() => {
            navigateToAtlasRoute({
              kind: "record",
              recordKey: preview.recordKey!,
              selection: preview.selection,
            });
            preview.close();
          }}
        />
      )}
    </>
  );
}

export function ReaderView({ route }: ReaderViewProps) {
  const detail = useRecordDetail(route.recordKey, route.selection);
  const reading = useReferenceReading(
    route.previewRecordKey
      ? {
          navigation: route.previewSelection ?? rootNavigation(route.previewRecordKey),
          landing: route.landing,
        }
      : null,
  );
  const selectReading = reading.select;
  useEffect(() => {
    selectReading(
      route.previewRecordKey
        ? {
            navigation:
              route.previewSelection ?? rootNavigation(route.previewRecordKey),
            landing: route.landing,
          }
        : null,
    );
  }, [
    route.recordKey,
    route.previewRecordKey,
    route.previewSelection,
    route.landing,
    selectReading,
  ]);
  const navigation = reading.detail?.selected ?? reading.request?.navigation;
  return (
    <RecordViewLayout
      primary={
        <RecordPane
          title="Reader"
          actions={
            <>
              <AddToListButton recordKey={route.recordKey} />
              <RouteLink
                route={{
                  kind: "record",
                  recordKey: route.recordKey,
                  selection: route.selection,
                  landing: route.landing,
                }}
              >
                Detail page
              </RouteLink>
            </>
          }
        >
          <RecordDetailPane
            detail={detail.data}
            error={detail.error}
            onRetry={() => void detail.refetch()}
            loading={detail.isLoading || detail.isFetching}
            onRankChange={(spell_rank) => {
              if (detail.data)
                navigateToAtlasRoute({
                  ...route,
                  selection: { ...detail.data.selected, spell_rank },
                });
            }}
            onOwnerReturn={(parent, landing) =>
              navigateToAtlasRoute({
                ...route,
                previewRecordKey: parent,
                previewSelection: undefined,
                landing,
              })
            }
            onReference={(previewRecordKey, _anchor, previewSelection) =>
              navigateToAtlasRoute({ ...route, previewRecordKey, previewSelection })
            }
          />
        </RecordPane>
      }
      auxiliary={
        <ReferenceReader
          reading={reading}
          readerActions={
            navigation && (
              <>
                <AddToListButton recordKey={navigation.record_key} />
                <RouteIconLink
                  label="Open preview as reader"
                  route={{
                    kind: "reader",
                    recordKey: navigation.record_key,
                    selection:
                      navigation.embedded_path.length ||
                      navigation.field ||
                      navigation.passage ||
                      navigation.source_fingerprint
                        ? navigation
                        : undefined,
                    previewRecordKey: null,
                    landing: reading.landing,
                  }}
                />
                <PaneIconButton
                  icon={<X size={16} />}
                  label="Close preview"
                  onClick={() => {
                    reading.select(null);
                    navigateToAtlasRoute({
                      ...route,
                      previewRecordKey: null,
                      previewSelection: undefined,
                    });
                  }}
                />
              </>
            )
          }
        />
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
    <Typography.Link
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
    </Typography.Link>
  );
}
