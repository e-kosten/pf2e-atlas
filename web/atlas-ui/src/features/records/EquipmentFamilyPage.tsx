import { useQuery } from "@tanstack/react-query";
import { Alert, Button } from "antd";
import { AtlasApiError, getEquipmentFamily } from "../../api/atlasApi";
import { LinkNotFound } from "../../app/InvalidUrl";
import type { EquipmentFamilyView } from "../../generated/atlas";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { referenceQueryMeta } from "../../shared/queries/referenceQueries";
import { DocumentIdentityHeader } from "../../shared/records/DocumentIdentityHeader";
import { DocumentPagePane } from "../../shared/records/DocumentPagePane";
import { ReferenceHelpHost } from "../../shared/records/ReferenceHelpHost";
import { LoadingStatus } from "../../shared/ui/LoadingStatus";
import { FamilyMembers, FamilyRows } from "./FamilyMembers";

export function EquipmentFamilyPage({ familyId }: { familyId: string }) {
  const query = useQuery({
    queryKey: ["equipment-family", familyId],
    queryFn: ({ signal }) => getEquipmentFamily(familyId, signal),
    meta: referenceQueryMeta,
  });
  const family = query.data;
  const missing =
    query.error instanceof AtlasApiError &&
    query.error.appError?.code === "equipment_family_not_found";
  usePageTitle(missing ? "Link not found" : (family?.name ?? "Equipment family"));
  if (missing) return <LinkNotFound />;
  return (
    <ReferenceHelpHost>
      <main className="record-view">
        <DocumentPagePane>
          <section className="detail-panel" aria-busy={query.isFetching}>
            {query.isPending && <LoadingStatus label="Loading equipment family…" />}
            {query.error && (
              <ErrorNotice
                error={query.error}
                title="Unable to load equipment family"
                action={<Button onClick={() => void query.refetch()}>Retry</Button>}
              />
            )}
            {family && (
              <>
                <DocumentIdentityHeader
                  title={family.name}
                  classification="Equipment family"
                />
                {family.kind === "content" ? (
                  <EquipmentMembers family={family} />
                ) : (
                  <Alert
                    type="warning"
                    title="Family unavailable"
                    description="The reviewed members could not all be verified in this collection."
                  />
                )}
              </>
            )}
          </section>
        </DocumentPagePane>
      </main>
    </ReferenceHelpHost>
  );
}

function EquipmentMembers({
  family,
}: {
  family: Extract<EquipmentFamilyView, { kind: "content" }>;
}) {
  const overviewHeading = family.overviews.some((overview) => overview.qualifier)
    ? "Variant overviews"
    : "Overview";
  return (
    <div className="record-explore__task">
      {family.overviews.length > 0 && (
        <section>
          <h3>{overviewHeading}</h3>
          <FamilyRows choices={family.overviews} />
        </section>
      )}
      <h3>Variants</h3>
      <FamilyMembers
        choices={family.members}
        groupings={family.groupings}
        allVariantsByDefault={family.all_variants_by_default}
      />
    </div>
  );
}
