import { useQuery } from "@tanstack/react-query";
import { Alert, Button, Select } from "antd";
import { useId, useState } from "react";
import { AtlasApiError, getEquipmentFamily } from "../../api/atlasApi";
import { LinkNotFound } from "../../app/InvalidUrl";
import type {
  EquipmentFamilyView,
  VariantFamilyChoiceView,
} from "../../generated/atlas";
import { usePageTitle } from "../../shared/browser/usePageTitle";
import { ErrorNotice } from "../../shared/feedback/ErrorNotice";
import { referenceQueryMeta } from "../../shared/queries/referenceQueries";
import { DocumentIdentityHeader } from "../../shared/records/DocumentIdentityHeader";
import { DocumentPagePane } from "../../shared/records/DocumentPagePane";
import { useReferenceRecordNavigation } from "../../shared/records/ReferenceHelp";
import { ReferenceHelpHost } from "../../shared/records/ReferenceHelpHost";
import { ReferenceResult } from "../../shared/records/ReferenceResult";
import { LoadingStatus } from "../../shared/ui/LoadingStatus";

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
                  <FamilyMembers family={family} />
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

function FamilyMembers({
  family,
}: {
  family: Extract<EquipmentFamilyView, { kind: "content" }>;
}) {
  const [axis, setAxis] = useState(family.groupings[0]?.id);
  const selector = useId();
  const grouping =
    family.groupings.find((group) => group.id === axis) ?? family.groupings[0];
  const members = new Map(
    family.members.map((choice) => [choice.record.record_key, choice]),
  );
  return (
    <div className="record-explore__task">
      <h3>Variants</h3>
      {grouping ? (
        <>
          <div className="equipment-family__group-by">
            <label htmlFor={selector}>Group by</label>
            <Select
              id={selector}
              popupMatchSelectWidth={false}
              value={grouping.id}
              onChange={setAxis}
              options={family.groupings.map(({ id, label }) => ({ value: id, label }))}
            />
          </div>
          {grouping.groups.map((group) => (
            <section className="equipment-family__group" key={group.members[0]}>
              <h4>{group.heading}</h4>
              <FamilyRows
                choices={group.members.flatMap((key) => members.get(key) ?? [])}
              />
            </section>
          ))}
        </>
      ) : (
        <FamilyRows choices={family.members} />
      )}
      {family.overviews.length > 0 && (
        <section>
          <h3>Variant overviews</h3>
          <FamilyRows choices={family.overviews} />
        </section>
      )}
    </div>
  );
}

function FamilyRows({ choices }: { choices: VariantFamilyChoiceView[] }) {
  const onReference = useReferenceRecordNavigation();
  return (
    <div className="record-explore__results" role="list">
      {choices.map(({ record, qualifier }) => (
        <div key={record.record_key} role="listitem">
          <ReferenceResult
            row={{ record, matches: [], relationship_evidence: [] }}
            selected={false}
            disabled={false}
            context={qualifier}
            onPreview={(anchor) => onReference?.(record.record_key, anchor)}
          />
        </div>
      ))}
    </div>
  );
}
