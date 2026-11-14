import { CopyEntityLink } from "../../app/CopyEntityLink";
import { useReferenceRecordNavigation } from "../../shared/records/ReferenceHelp";
import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { LinkNotFound } from "../../app/InvalidUrl";
import { atlasRouter } from "../../app/router";
import { definitionDestination } from "../../app/destinations";
import { useDefinitionDetail } from "../../shared/records/useDefinitionDetail";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { ReferenceHelpHost } from "../../shared/records/ReferenceHelpHost";
import {
  referenceDestination,
  ReferenceHelpProvider,
} from "../../shared/records/ReferenceHelp";
import { DefinitionDetailPane } from "../../shared/records/DefinitionDetailPane";
import { DocumentPagePane } from "../../shared/records/DocumentPagePane";
import { DefinitionAssociations } from "../../shared/records/DefinitionAssociations";
import { retireReferenceQueries } from "../../shared/queries/referenceQueries";

export function DefinitionView({ definitionKey }: { definitionKey: string }) {
  return (
    <ReferenceHelpHost>
      <DefinitionViewContent definitionKey={definitionKey} />
    </ReferenceHelpHost>
  );
}

function DefinitionViewContent({ definitionKey }: { definitionKey: string }) {
  const query = useDefinitionDetail(definitionKey);
  const onReference = useReferenceRecordNavigation()!;
  const client = useQueryClient();
  const destination = query.data?.help?.destination;
  const recordTarget = destination?.kind === "record" ? destination : undefined;
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
    destination?.kind === "definition" &&
    destination.key === definitionKey &&
    query.data?.content != null;
  const reload = async () => {
    await retireReferenceQueries(client);
    await query.refetch();
  };
  const unavailable = Boolean(query.data && !recordTarget && !traitTarget && !readable);
  usePageTitle(unavailable ? "Link not found" : (query.data?.label ?? "Definition"));
  if (unavailable) return <LinkNotFound />;
  return (
    <ReferenceHelpProvider generation={query.data?.artifact_generation}>
      <main className="record-view">
        <DocumentPagePane>
          <DefinitionDetailPane
            detail={recordTarget || traitTarget ? undefined : query.data}
            loading={query.isFetching || Boolean(recordTarget) || traitTarget}
            error={query.error}
            onRetry={() => void query.refetch()}
            onReference={onReference}
            identityActions={
              readable && query.data ? (
                <CopyEntityLink
                  primary
                  kind="definition"
                  identity={query.data.definition_key}
                  title={query.data.label}
                />
              ) : undefined
            }
          />
          {readable && query.data && (
            <DefinitionAssociations
              key={JSON.stringify([definitionKey, query.data.artifact_generation])}
              actions={query.data.related_filters}
              generation={query.data.artifact_generation}
              onReference={onReference}
              onReload={() => void reload()}
            />
          )}
        </DocumentPagePane>
      </main>
    </ReferenceHelpProvider>
  );
}
