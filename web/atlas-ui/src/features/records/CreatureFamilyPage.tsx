import { useQuery } from "@tanstack/react-query";
import { Button } from "antd";
import { getCreatureFamily } from "../../api/atlasApi";
import { LinkNotFound } from "../../app/InvalidUrl";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { referenceQueryMeta } from "../../shared/queries/referenceQueries";
import { DocumentIdentityHeader } from "../../shared/records/DocumentIdentityHeader";
import { DocumentPagePane } from "../../shared/records/DocumentPagePane";
import { ReferenceHelpHost } from "../../shared/records/ReferenceHelpHost";
import { LoadingStatus } from "../../shared/ui/LoadingStatus";
import { FamilyMembers, FamilyRows } from "./FamilyMembers";

export function CreatureFamilyPage({ familyId }: { familyId: string }) {
  const query = useQuery({
    queryKey: ["creature-family", familyId],
    queryFn: ({ signal }) => getCreatureFamily(familyId, signal),
    meta: referenceQueryMeta,
  });
  const family = query.data;
  usePageTitle(
    family?.name ?? (query.isSuccess ? "Link not found" : "Creature family"),
  );
  if (query.isSuccess && !family) return <LinkNotFound />;
  return (
    <ReferenceHelpHost>
      <main className="record-view">
        <DocumentPagePane>
          <section className="detail-panel" aria-busy={query.isFetching}>
            {query.isPending && <LoadingStatus label="Loading creature family…" />}
            {query.error && (
              <ErrorNotice
                error={query.error}
                title="Unable to load creature family"
                action={<Button onClick={() => void query.refetch()}>Retry</Button>}
              />
            )}
            {family && (
              <>
                <DocumentIdentityHeader
                  title={family.name}
                  classification="Creature family"
                />
                <div className="record-explore__task">
                  <h3>{family.versions.length === 1 ? "Creature" : "Variants"}</h3>
                  <FamilyMembers
                    choices={family.versions.map((record) => ({ record }))}
                    groupings={family.groupings ?? []}
                    allVariantsByDefault={family.all_variants_by_default}
                  />
                  {family.named_creatures.length > 0 && (
                    <>
                      <h3>Named creatures</h3>
                      <FamilyRows
                        choices={family.named_creatures.map((record) => ({ record }))}
                      />
                    </>
                  )}
                </div>
              </>
            )}
          </section>
        </DocumentPagePane>
      </main>
    </ReferenceHelpHost>
  );
}
