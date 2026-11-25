import { useState } from "react";
import { recordNavigationIdentity } from "../../shared/records/recordNavigationIdentity";
import { useAtlasRouter } from "../../app/router";
import { CopyEntityLink } from "../../app/CopyEntityLink";
import { useReferenceRecordNavigation } from "../../shared/records/ReferenceHelp";
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
  const [rankGuard, setRankGuard] = useState<{
    address: string;
    generation: string;
  } | null>(null);
  const address = recordNavigationIdentity(route.recordKey, route.selection);
  const generation = rankGuard?.address === address ? rankGuard.generation : undefined;
  const detail = useRecordDetail(route.recordKey, route.selection, generation);
  const client = useQueryClient();
  const reload = async () => {
    await retireReferenceQueries(client);
    if (generation) setRankGuard(null);
    else await detail.refetch();
  };
  const atlasRouter = useAtlasRouter();
  const changeRank = (spell_rank: number | undefined) => {
    if (!detail.data) return;
    const selection = { ...detail.data.selected, spell_rank };
    setRankGuard({
      address: recordNavigationIdentity(route.recordKey, selection),
      generation: detail.data.artifact_generation,
    });
    void atlasRouter.navigate({
      ...recordDestination({
        ...route,
        selection,
      }),
      replace: true,
      resetScroll: false,
      state: true,
    });
  };
  return (
    <ReferenceHelpHost
      generation={detail.data?.artifact_generation}
      visibleOwnerRecordKey={
        detail.data?.selected.embedded_path.length === 0
          ? detail.data.record.record_key
          : undefined
      }
      recordActions={(record) => (
        <AddToRecordButton recordKey={record.record_key} record={record} />
      )}
    >
      <RecordViewContent
        route={route}
        detail={detail}
        reload={reload}
        changeRank={changeRank}
      />
    </ReferenceHelpHost>
  );
}

function RecordViewContent({
  route,
  detail,
  reload,
  changeRank,
}: RecordViewProps & {
  detail: ReturnType<typeof useRecordDetail>;
  reload: () => Promise<void>;
  changeRank: (rank: number | undefined) => void;
}) {
  usePageTitle(detail.data?.presentation.identity.title ?? "Record");
  const onReference = useReferenceRecordNavigation()!;
  const generation = detail.data?.artifact_generation;
  const property = useDefinitionDetail(
    generation === undefined ? null : (route.definitionKey ?? null),
    generation,
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
  return (
    <>
      <main className="record-view">
        <DocumentPagePane layout={recordLayout(detail.data?.presentation)}>
          <RecordDetailPane
            sectionLanding={route.section}
            identityActions={
              <>
                <AddToRecordButton
                  key={route.recordKey}
                  recordKey={route.recordKey}
                  record={detail.data?.record}
                />
                {detail.data?.record.record_key === route.recordKey && (
                  <CopyEntityLink
                    kind="record"
                    primary
                    identity={route.recordKey}
                    title={detail.data.record.title}
                  />
                )}
              </>
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
            onRetry={
              isArtifactGenerationChanged(detail.error)
                ? reload
                : () => void detail.refetch()
            }
            retryLabel={
              isArtifactGenerationChanged(detail.error) ? "Reload record" : "Retry"
            }
            onReload={reload}
            loading={detail.isLoading || detail.isFetching}
            onReference={onReference}
            onRankChange={changeRank}
          />
        </DocumentPagePane>
      </main>
    </>
  );
}
