import {
  useDefinitionDetail,
  definitionHelpMatchesRecord,
} from "../../shared/records/useDefinitionDetail";
import { DefinitionAssociations } from "../../shared/records/DefinitionAssociations";
import { useQueryClient } from "@tanstack/react-query";
import {
  isArtifactGenerationChanged,
  retireReferenceQueries,
} from "../../shared/queries/referenceQueries";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { Button } from "antd";
import { rootNavigation } from "../../shared/records/useReferenceReading";
import { atlasRouter } from "../../app/router";
import type { RecordPageInput } from "../../app/destinations";
import { navigateDestination } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { recordLayout } from "../../shared/records/recordLayout";
import { DocumentPagePane } from "../../shared/records/DocumentPagePane";
import { AddToRecordButton } from "../record-actions/AddToRecordButton";
import { RecordDetailPane } from "../../shared/records/RecordDetailPane";
import { RecordReferenceDrawer } from "../../shared/records/RecordReferenceDrawer";
import { useRecordPreview } from "../../shared/records/useRecordPreview";
import { useRecordDetail } from "../../shared/records/useRecordDetail";
import { recordReadingIdentity } from "../../shared/records/useRetainedRecord";
type RecordViewProps = {
  route: RecordPageInput;
};
export function RecordView({ route }: RecordViewProps) {
  const detail = useRecordDetail(
    route.recordKey,
    route.selection,
    route.readingContext,
  );
  usePageTitle(detail.data?.presentation.identity.title ?? "Record");
  const preview = useRecordPreview();
  const client = useQueryClient();
  const property = useDefinitionDetail(
    route.definitionKey ?? null,
    detail.data?.artifact_generation,
  );
  const propertyChanged = isArtifactGenerationChanged(property.error);
  const propertyHelp =
    property.data &&
    definitionHelpMatchesRecord(
      property.data.help,
      route.selection ?? rootNavigation(route.recordKey),
      route.readingContext,
    )
      ? property.data.help
      : undefined;
  const reload = async () => {
    await retireReferenceQueries(client);
    await detail.refetch();
  };
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
            onHelp={preview.openHelp}
            sectionLanding={route.section}
            identityActions={
              <AddToRecordButton
                key={route.recordKey}
                recordKey={route.recordKey}
                record={detail.data?.record}
              />
            }
            readingKey={JSON.stringify([
              recordReadingIdentity(route.recordKey, route.selection),
              route.readingContext,
              route.definitionKey,
            ])}
            afterContent={
              route.definitionKey && (
                <>
                  {property.error && (
                    <ErrorNotice
                      error={property.error}
                      title={
                        propertyChanged
                          ? "Reference collection changed"
                          : "Unable to load related searches"
                      }
                      type={propertyChanged ? "warning" : "error"}
                      action={
                        <Button
                          onClick={() =>
                            void (propertyChanged ? reload() : property.refetch())
                          }
                        >
                          {propertyChanged ? "Refresh" : "Retry"}
                        </Button>
                      }
                    />
                  )}
                  {propertyHelp && property.data && (
                    <DefinitionAssociations
                      actions={propertyHelp.related_filters}
                      generation={property.data.artifact_generation}
                      onReference={preview.open}
                      onReload={() => void reload()}
                    />
                  )}
                  {property.data && !propertyHelp && (
                    <p className="muted">
                      This definition no longer points to this entry.
                    </p>
                  )}
                </>
              )
            }
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
      {preview.destination && (
        <RecordReferenceDrawer
          destination={preview.destination}
          definition={preview.definition}
          help={preview.help}
          onHelp={preview.openHelp}
          actions={
            preview.recordKey ? (
              <AddToRecordButton
                recordKey={preview.recordKey!}
                record={preview.detail?.record}
              />
            ) : undefined
          }
          visibleOwnerRecordKey={
            detail.data?.selected.embedded_path.length === 0
              ? detail.data.record.record_key
              : undefined
          }
          anchor={preview.anchor}
          readingHistory={preview.readingHistory}
          requestedNavigation={preview.selection}
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
