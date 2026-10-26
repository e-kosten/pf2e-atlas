import { atlasRouter } from "../../app/router";
import type { RecordPageInput } from "../../app/destinations";
import { navigateDestination } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { recordLayout } from "../../shared/records/recordLayout";
import { DocumentPagePane } from "../../shared/records/DocumentPagePane";
import { AddToListButton } from "../lists/AddToListButton";
import { RecordDetailPane } from "../../shared/records/RecordDetailPane";
import { RecordReferenceDrawer } from "../../shared/records/RecordReferenceDrawer";
import { useRecordPreview } from "../../shared/records/useRecordPreview";
import { useRecordDetail } from "../../shared/records/useRecordDetail";
import { recordReadingIdentity } from "../../shared/records/useRetainedRecord";
type RecordViewProps = {
  route: RecordPageInput;
};
export function RecordView({ route }: RecordViewProps) {
  const detail = useRecordDetail(route.recordKey, route.selection);
  usePageTitle(detail.data?.presentation.identity.title ?? "Record");
  const preview = useRecordPreview();
  const changeRank = (spell_rank: number | undefined) => {
    if (detail.data)
      void atlasRouter.navigate({
        ...recordDestination({
          ...route,
          selection: { ...detail.data.selected, spell_rank },
        }),
        replace: true,
        resetScroll: false,
        state: true,
      });
  };
  return (
    <>
      <main className="record-view">
        <DocumentPagePane layout={recordLayout(detail.data?.presentation)}>
          <RecordDetailPane
            sectionLanding={route.section}
            identityActions={<AddToListButton recordKey={route.recordKey} />}
            readingKey={recordReadingIdentity(route.recordKey, route.selection)}
            landing={
              route.landing?.record_key === route.recordKey ? route.landing : undefined
            }
            detail={detail.data}
            error={detail.error}
            onRetry={() => void detail.refetch()}
            loading={detail.isLoading || detail.isFetching}
            onReference={preview.open}
            onRankChange={changeRank}
          />
        </DocumentPagePane>
      </main>
      {preview.recordKey && (
        <RecordReferenceDrawer
          actions={<AddToListButton recordKey={preview.recordKey} />}
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
            navigateDestination(
              recordDestination({
                recordKey: preview.recordKey!,
                selection: preview.selection,
                landing: preview.landing,
              }),
            );
            preview.close();
          }}
        />
      )}
    </>
  );
}
