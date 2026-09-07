import { Alert, Empty, Skeleton } from "antd";
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
  const [referenceRecordKey, setReferenceRecordKey] = useState<string>();
  const referencesRequested =
    recordKey !== undefined && referenceRecordKey === recordKey;
  const referenceDetail = useRecordDetail(
    referencesRequested ? recordKey : null,
    undefined,
    {
      reference_outgoing_limit: 8,
      reference_backlink_limit: 8,
    },
  );
  const referenceSurface = referenceDetail.data?.surface;
  const loadedReferences =
    referenceSurface && referenceSurface.metadata.record_key === recordKey
      ? referenceSurface.references
      : undefined;
  const visibleErrors = errors.filter((error): error is Error | { message: string } =>
    Boolean(error),
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
            onReferencesOpen={() => setReferenceRecordKey(recordKey)}
            references={loadedReferences ?? detail.surface.references}
            referencesLoading={
              referencesRequested &&
              (referenceDetail.isLoading || referenceDetail.isFetching)
            }
            showTitle={showTitle}
          />
        ) : (
          <RecordSurface
            onReferencesOpen={() => setReferenceRecordKey(recordKey)}
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
      ) : (
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
          description={referenceDetail.error.message}
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
  references,
  referencesLoading,
  showTitle,
}: {
  detail: RecordDetailView;
  onReference: (recordKey: string) => void;
  onReferencesOpen?: () => void;
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
