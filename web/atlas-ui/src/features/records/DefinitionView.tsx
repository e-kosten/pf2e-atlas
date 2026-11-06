import { useEffect } from "react";
import { atlasRouter } from "../../app/router";
import { definitionDestination } from "../../app/destinations";
import { useDefinitionDetail } from "../../shared/records/useDefinitionDetail";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { useRecordPreview } from "../../shared/records/useRecordPreview";
import {
  referenceDestination,
  ReferenceHelpProvider,
} from "../../shared/records/ReferenceHelp";
import { DefinitionDetailPane } from "../../shared/records/DefinitionDetailPane";
import { DocumentPagePane } from "../../shared/records/DocumentPagePane";
import { RecordReferenceDrawer } from "../../shared/records/RecordReferenceDrawer";

export function DefinitionView({ definitionKey }: { definitionKey: string }) {
  const query = useDefinitionDetail(definitionKey);
  const preview = useRecordPreview();
  usePageTitle(query.data?.label ?? "Definition");
  const recordTarget =
    query.data?.help.destination?.kind === "record"
      ? query.data.help.destination
      : undefined;
  useEffect(() => {
    if (definitionKey.startsWith("trait:"))
      void atlasRouter.navigate({
        ...definitionDestination({ definitionKey }),
        replace: true,
      });
    else if (recordTarget)
      void atlasRouter.navigate({
        ...referenceDestination(recordTarget, definitionKey),
        replace: true,
      });
  }, [definitionKey, recordTarget]);
  return (
    <ReferenceHelpProvider
      onHelp={preview.openHelp}
      generation={query.data?.artifact_generation}
    >
      <main className="record-view">
        <DocumentPagePane>
          <DefinitionDetailPane
            detail={recordTarget ? undefined : query.data}
            loading={query.isFetching || Boolean(recordTarget)}
            error={query.error}
            onRetry={() => void query.refetch()}
            onReference={preview.open}
          />
        </DocumentPagePane>
      </main>
      {preview.destination && (
        <RecordReferenceDrawer
          destination={preview.destination}
          definition={preview.definition}
          help={preview.help}
          onHelp={preview.openHelp}
          anchor={preview.anchor}
          detail={preview.detail}
          loading={preview.loading}
          error={preview.error}
          onRetry={preview.retry}
          onClose={preview.close}
          onReference={preview.open}
          onRankChange={preview.changeRank}
          readingHistory={preview.readingHistory}
          requestedNavigation={preview.selection}
          landing={preview.landing}
          onOpenFullPage={preview.close}
        />
      )}
    </ReferenceHelpProvider>
  );
}
