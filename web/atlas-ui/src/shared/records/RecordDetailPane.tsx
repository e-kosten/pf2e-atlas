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
            showTitle={showTitle}
          />
        ) : (
          <RecordSurface
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
    </section>
  );
}

function SelectableSpellDetail({
  detail,
  onReference,
  showTitle,
}: {
  detail: RecordDetailView;
  onReference: (recordKey: string) => void;
  showTitle: boolean;
}) {
  const recordKey = detail.surface.metadata.record_key;
  const [requestedSelection, setRequestedSelection] = useState<
    (SpellFormSelection & { recordKey: string }) | undefined
  >();
  const selection =
    recordKey && requestedSelection?.recordKey === recordKey
      ? requestedSelection
      : undefined;
  const selectedDetail = useRecordDetail(
    recordKey && selection ? recordKey : null,
    selection,
  );
  const selectedSurface = matchingSelectedSpellSurface(
    selectedDetail.data,
    recordKey,
    selection,
  );

  return (
    <>
      <RecordSurface
        key={recordKey}
        onReference={onReference}
        onSpellFormSelection={
          recordKey
            ? (nextSelection) => {
                setRequestedSelection({ ...nextSelection, recordKey });
              }
            : undefined
        }
        showTitle={showTitle}
        spellCatalog={detail.surface}
        spellFormSelection={selection}
        spellFormSelectionLoading={selectedDetail.isFetching}
        surface={selectedSurface ?? detail.surface}
      />
      {selectedDetail.error ? (
        <Alert
          className="detail-state__error"
          description={selectedDetail.error.message}
          message="Unable to resolve this spell form"
          showIcon
          type="error"
        />
      ) : null}
    </>
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
  const selected = presentation.body.selected_form;
  return selected?.id === selection.formId && selected.cast_rank === selection.castRank
    ? selectedDetail.surface
    : undefined;
}
