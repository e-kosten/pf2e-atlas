import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { LinkNotFound } from "../../app/InvalidUrl";
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
import { DefinitionAssociations } from "../../shared/records/DefinitionAssociations";
import { retireReferenceQueries } from "../../shared/queries/referenceQueries";

export function DefinitionView({ definitionKey }: { definitionKey: string }) {
  const query = useDefinitionDetail(definitionKey);
  const preview = useRecordPreview();
  const client = useQueryClient();
  const recordTarget =
    query.data?.help.destination?.kind === "record"
      ? query.data.help.destination
      : undefined;
  const canonical = definitionDestination({ definitionKey });
  const traitTarget = canonical.to === "/traits/$traitKey";
  useEffect(() => {
    if (traitTarget)
      void atlasRouter.navigate({
        ...definitionDestination({ definitionKey }),
        replace: true,
      });
    else if (recordTarget)
      void atlasRouter.navigate({
        ...referenceDestination(recordTarget, definitionKey),
        replace: true,
      });
  }, [definitionKey, recordTarget, traitTarget]);
  const readable =
    !traitTarget &&
    query.data?.help.destination?.kind === "definition" &&
    query.data.help.destination.key === definitionKey &&
    query.data.content !== null;
  const reload = async () => {
    await retireReferenceQueries(client);
    await query.refetch();
  };
  const unavailable = Boolean(query.data && !recordTarget && !traitTarget && !readable);
  usePageTitle(unavailable ? "Link not found" : (query.data?.label ?? "Definition"));
  if (unavailable) return <LinkNotFound />;
  return (
    <ReferenceHelpProvider
      onHelp={preview.openHelp}
      generation={query.data?.artifact_generation}
    >
      <main className="record-view">
        <DocumentPagePane>
          <DefinitionDetailPane
            detail={recordTarget || traitTarget ? undefined : query.data}
            loading={query.isFetching || Boolean(recordTarget) || traitTarget}
            error={query.error}
            onRetry={() => void query.refetch()}
            onReference={preview.open}
          />
          {readable && query.data && (
            <DefinitionAssociations
              key={JSON.stringify([definitionKey, query.data.artifact_generation])}
              actions={query.data.help.related_filters}
              generation={query.data.artifact_generation}
              onReference={preview.open}
              onReload={() => void reload()}
            />
          )}
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
