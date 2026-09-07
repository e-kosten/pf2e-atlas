import { AtlasApiError } from "../../api/atlasApi";
import { navigateToAtlasRoute } from "../../app/routes";
import { Alert, Button, Empty, Skeleton, Space } from "antd";
import { useState } from "react";
import type { RecordDetailView } from "../../generated/atlas";
import { RecordSurface } from "./RecordSurface";
import type { SpellFormSelection } from "./SpellRecordSurface";
import { useRecordDetail } from "./useRecordDetail";

type RecordDetailPaneError = Error | { message: string } | null | undefined;

export function RecordDetailPane({
  detail,
  emptyMessage,
  errors = [],
  loading,
  loadingMessage,
  onReference,
  showTitle = true,
  stale = false,
}: {
  detail: RecordDetailView | undefined;
  emptyMessage?: string;
  errors?: RecordDetailPaneError[];
  loading: boolean;
  loadingMessage?: string;
  onReference: (recordKey: string) => void;
  showTitle?: boolean;
  stale?: boolean;
}) {
  const recordKey = detail?.surface.metadata.record_key;
  const [referenceRequest, setReferenceRequest] = useState<{
    recordKey: string;
    outgoingLimit: number;
    backlinkLimit: number;
  }>();
  const referencesRequested =
    recordKey !== undefined && referenceRequest?.recordKey === recordKey;
  const referenceDetail = useRecordDetail(
    referencesRequested ? recordKey : null,
    undefined,
    {
      reference_outgoing_limit: referenceRequest?.outgoingLimit ?? 8,
      reference_backlink_limit: referenceRequest?.backlinkLimit ?? 8,
    },
  );
  function openReferences() {
    if (!recordKey) return;
    if (referencesRequested) {
      void referenceDetail.refetch();
      return;
    }
    setReferenceRequest({ recordKey, outgoingLimit: 8, backlinkLimit: 8 });
  }
  function expandReferences(direction: "outgoing" | "backlinks", limit: number) {
    if (!recordKey) return;
    if (
      referencesRequested &&
      (direction === "outgoing"
        ? referenceRequest.outgoingLimit
        : referenceRequest.backlinkLimit) === limit
    ) {
      void referenceDetail.refetch();
      return;
    }
    setReferenceRequest((current) => {
      const matching = current?.recordKey === recordKey ? current : undefined;
      return {
        recordKey,
        outgoingLimit:
          direction === "outgoing" ? limit : (matching?.outgoingLimit ?? 8),
        backlinkLimit:
          direction === "backlinks" ? limit : (matching?.backlinkLimit ?? 8),
      };
    });
  }
  const referenceSurface = referenceDetail.data?.surface;
  const loadedReferences =
    referenceSurface &&
    referenceSurface.metadata.record_key === recordKey &&
    referenceSectionMatches(
      referenceSurface.references?.outgoing,
      referenceRequest?.outgoingLimit,
    ) &&
    referenceSectionMatches(
      referenceSurface.references?.backlinks,
      referenceRequest?.backlinkLimit,
    )
      ? referenceSurface.references
      : undefined;
  const visibleErrors = errors.filter((error): error is Error | { message: string } =>
    Boolean(error),
  );
  const notFound = visibleErrors.some(
    (error) =>
      error instanceof AtlasApiError && error.appError?.code === "record_not_found",
  );
  if (notFound && !detail)
    return (
      <section className="detail-state" aria-label="Record not found">
        <Alert
          type="info"
          showIcon
          message="This record could not be found"
          description="Try searching for its name or return to the previous page."
        />
        <Space>
          <Button
            onClick={() =>
              navigateToAtlasRoute({ kind: "search", selectedRecordKey: null })
            }
          >
            Search
          </Button>
          <Button onClick={() => history.back()}>Back</Button>
        </Space>
      </section>
    );
  return (
    <section aria-busy={loading || stale} className="detail-panel">
      {loading ? (
        <div aria-label={loadingMessage ?? "Loading record"} className="detail-state">
          <Skeleton active paragraph={{ rows: 8 }} title />
        </div>
      ) : detail ? (
        detail.surface.presentation.presentation_type === "spell" ? (
          <SelectableSpellDetail
            detail={detail}
            onReference={onReference}
            onReferencesOpen={openReferences}
            onReferenceLimit={expandReferences}
            references={loadedReferences ?? detail.surface.references}
            referencesLoading={
              referencesRequested &&
              (referenceDetail.isLoading || referenceDetail.isFetching)
            }
            showTitle={showTitle}
          />
        ) : (
          <RecordSurface
            onReferencesOpen={openReferences}
            onReferenceLimit={expandReferences}
            referenceLoading={
              referencesRequested &&
              (referenceDetail.isLoading || referenceDetail.isFetching)
            }
            references={loadedReferences ?? detail.surface.references}
            surface={detail.surface}
            onReference={onReference}
            showTitle={showTitle}
          />
        )
      ) : visibleErrors.length ? null : (
        <Empty
          className="detail-state"
          description={emptyMessage ?? "Select a result to inspect it."}
          image={Empty.PRESENTED_IMAGE_SIMPLE}
        />
      )}
      {stale && detail && (
        <Alert
          className="detail-state__stale"
          message="Refreshing this record…"
          description="Keeping the last valid selection visible."
          showIcon
          type="info"
        />
      )}
      {visibleErrors.map((error, index) => (
        <Alert
          className="detail-state__error"
          description={error.message}
          key={index}
          message="Unable to load this record"
          showIcon
          type="error"
        />
      ))}
      {referencesRequested && referenceDetail.error ? (
        <Alert
          className="detail-state__error"
          description="Try loading the linked records again."
          action={<Button onClick={() => void referenceDetail.refetch()}>Retry</Button>}
          message="Unable to load linked records"
          showIcon
          type="error"
        />
      ) : null}
    </section>
  );
}

function SelectableSpellDetail({
  detail,
  onReference,
  onReferencesOpen,
  onReferenceLimit,
  references,
  referencesLoading,
  showTitle,
}: {
  detail: RecordDetailView;
  onReference: (recordKey: string) => void;
  onReferencesOpen?: () => void;
  onReferenceLimit?: (direction: "backlinks" | "outgoing", limit: number) => void;
  references: RecordDetailView["surface"]["references"];
  referencesLoading?: boolean;
  showTitle: boolean;
}) {
  const recordKey = detail.surface.metadata.record_key;
  const [requestedSelection, setRequestedSelection] = useState<
    (SpellFormSelection & { recordKey: string }) | undefined
  >();
  const [retainedSurface, setRetainedSurface] = useState(detail.surface);
  const selection =
    recordKey && requestedSelection?.recordKey === recordKey
      ? requestedSelection
      : undefined;
  const selectedDetail = useRecordDetail(
    recordKey && selection ? recordKey : null,
    selection,
  );
  const selectedResponseSurface = matchingSelectedSpellSurface(
    selectedDetail.data,
    recordKey,
    selection,
  );
  const retainedForRecord =
    selection && retainedSurface.metadata.record_key === recordKey
      ? retainedSurface
      : detail.surface;
  const selectedSurface = spellSurfaceHasAvailableResult(selectedResponseSurface)
    ? selectedResponseSurface
    : undefined;
  if (selectedSurface && retainedSurface !== selectedSurface) {
    setRetainedSurface(selectedSurface);
  }
  const displayedSurface = selectedSurface ?? retainedForRecord;
  const selectedUnavailable =
    selectedResponseSurface?.presentation.presentation_type === "spell" &&
    selectedResponseSurface.presentation.body.effective_form.result.state ===
      "unavailable";
  const visibleSurface = selectedUnavailable
    ? { ...displayedSurface, issues: selectedResponseSurface?.issues }
    : displayedSurface;

  return (
    <RecordSurface
      key={recordKey}
      onReference={onReference}
      onReferencesOpen={onReferencesOpen}
      onReferenceLimit={onReferenceLimit}
      onSpellFormSelection={
        recordKey
          ? (nextSelection) => {
              setRetainedSurface(displayedSurface);
              setRequestedSelection({ ...nextSelection, recordKey });
            }
          : undefined
      }
      showTitle={showTitle}
      referenceLoading={referencesLoading}
      references={references}
      spellCatalog={detail.surface}
      spellFormSelection={selection}
      spellFormSelectionError={selectedDetail.error?.message}
      spellFormSelectionLoading={selectedDetail.isFetching}
      spellFormSelectionUnavailable={selectedUnavailable}
      surface={visibleSurface}
    />
  );
}

function spellSurfaceHasAvailableResult(
  surface: RecordDetailView["surface"] | undefined,
) {
  return (
    surface?.presentation.presentation_type === "spell" &&
    surface.presentation.body.effective_form.result.state === "available"
  );
}

function matchingSelectedSpellSurface(
  selectedDetail: RecordDetailView | undefined,
  recordKey: string | undefined,
  selection: SpellFormSelection | undefined,
) {
  if (
    !selectedDetail ||
    !recordKey ||
    !selection ||
    selectedDetail.surface.metadata.record_key !== recordKey
  ) {
    return undefined;
  }
  const presentation = selectedDetail.surface.presentation;
  if (presentation.presentation_type !== "spell") return undefined;
  const selected = presentation.body.effective_form;
  return selected.id === selection.formId && selected.cast_rank === selection.castRank
    ? selectedDetail.surface
    : undefined;
}

function referenceSectionMatches(
  section:
    | NonNullable<RecordDetailView["surface"]["references"]>["outgoing"]
    | undefined,
  limit: number | undefined,
) {
  return section?.state === "not_requested"
    ? limit === 0
    : section?.requested_limit === limit;
}
