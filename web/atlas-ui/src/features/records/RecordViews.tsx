import {
  useReferenceHelpHandler,
  useReferenceRecordNavigation,
} from "../../shared/records/ReferenceHelp";
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
import { recordDestination } from "../../app/destinations";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { recordLayout } from "../../shared/records/recordLayout";
import { DocumentPagePane } from "../../shared/records/DocumentPagePane";
import { AddToRecordButton } from "../record-actions/AddToRecordButton";
import { RecordDetailPane } from "../../shared/records/RecordDetailPane";
import { ReferenceHelpHost } from "../../shared/records/ReferenceHelpHost";
import { useRecordDetail } from "../../shared/records/useRecordDetail";
import { recordReadingIdentity } from "../../shared/records/useRetainedRecord";
type RecordViewProps = {
  route: RecordPageInput;
};
export function RecordView({ route }: RecordViewProps) {
  const detail = useRecordDetail(route.recordKey, route.selection);
  return (
    <ReferenceHelpHost
      visibleOwnerRecordKey={
        detail.data?.selected.embedded_path.length === 0
          ? detail.data.record.record_key
          : undefined
      }
      recordActions={(record) => (
        <AddToRecordButton recordKey={record.record_key} record={record} />
      )}
    >
      <RecordViewContent route={route} detail={detail} />
    </ReferenceHelpHost>
  );
}

function RecordViewContent({
  route,
  detail,
}: RecordViewProps & { detail: ReturnType<typeof useRecordDetail> }) {
  usePageTitle(detail.data?.presentation.identity.title ?? "Record");
  const onHelp = useReferenceHelpHandler();
  const onReference = useReferenceRecordNavigation()!;
  const client = useQueryClient();
  const property = useDefinitionDetail(
    route.definitionKey ?? null,
    detail.data?.artifact_generation,
  );
  const propertyHelp =
    property.data &&
    definitionHelpMatchesRecord(
      property.data.help,
      route.selection ?? rootNavigation(route.recordKey),
    )
      ? property.data.help
      : undefined;
  const relatedError = property.error;
  const relatedChanged = isArtifactGenerationChanged(relatedError);
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
            onHelp={onHelp}
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
              route.definitionKey,
            ])}
            afterContent={
              route.definitionKey && (
                <>
                  {relatedError && (
                    <ErrorNotice
                      error={relatedError}
                      title={
                        relatedChanged
                          ? "Reference collection changed"
                          : "Unable to load related searches"
                      }
                      type={relatedChanged ? "warning" : "error"}
                      action={
                        <Button
                          onClick={() =>
                            void (relatedChanged ? reload() : property.refetch())
                          }
                        >
                          {relatedChanged ? "Refresh" : "Retry"}
                        </Button>
                      }
                    />
                  )}
                  {propertyHelp && property.data && (
                    <DefinitionAssociations
                      actions={property.data.related_filters}
                      generation={property.data.artifact_generation}
                      onReference={onReference}
                      onReload={() => void reload()}
                    />
                  )}
                  {route.definitionKey && property.data && !propertyHelp && (
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
            onReference={onReference}
            onRankChange={changeRank}
          />
        </DocumentPagePane>
      </main>
    </>
  );
}
