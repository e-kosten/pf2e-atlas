import { Select } from "antd";
import { useId, useState } from "react";
import type { RecordSummaryView } from "../../generated/atlas";
import { useReferenceRecordNavigation } from "../../shared/records/ReferenceHelp";
import { ReferenceResult } from "../../shared/records/ReferenceResult";

type FamilyChoice = {
  record: RecordSummaryView;
  qualifier?: string;
  qualifiers?: string[];
};

type FamilyGrouping = {
  id: string;
  label: string;
  groups: {
    heading: string;
    members: string[];
    qualifiers?: Record<string, string[]>;
  }[];
};

const flat = "flat";
const groupingValue = (id: string) => `group:${id}`;

export function FamilyMembers({
  choices,
  groupings,
  allVariantsByDefault = false,
}: {
  choices: FamilyChoice[];
  groupings: FamilyGrouping[];
  allVariantsByDefault?: boolean;
}) {
  const [selection, setSelection] = useState<string>();
  const selector = useId();
  const grouped =
    selection === flat || (selection === undefined && allVariantsByDefault)
      ? undefined
      : (groupings.find((group) => groupingValue(group.id) === selection) ??
        groupings[0]);
  const members = new Map(choices.map((choice) => [choice.record.record_key, choice]));

  return (
    <>
      {groupings.length > 0 && (
        <div className="record-family__group-by">
          <label htmlFor={selector}>Group by</label>
          <Select
            id={selector}
            popupMatchSelectWidth={false}
            style={{ maxWidth: "100%" }}
            value={grouped ? groupingValue(grouped.id) : flat}
            onChange={setSelection}
            options={[
              ...groupings.map(({ id, label }) => ({
                value: groupingValue(id),
                label,
              })),
              { value: flat, label: "All variants" },
            ]}
          />
        </div>
      )}
      {grouped ? (
        grouped.groups.map((group) => (
          <section
            className="record-family__group"
            key={JSON.stringify([group.heading, group.members[0]])}
          >
            <h4>{group.heading}</h4>
            <FamilyRows
              choices={group.members.flatMap((key) => {
                const member = members.get(key);
                return member
                  ? [{ ...member, qualifiers: group.qualifiers?.[key] }]
                  : [];
              })}
            />
          </section>
        ))
      ) : (
        <FamilyRows choices={choices} />
      )}
    </>
  );
}

export function FamilyRows({ choices }: { choices: FamilyChoice[] }) {
  const onReference = useReferenceRecordNavigation();
  return (
    <div className="record-explore__results" role="list">
      {choices.map(({ record, qualifier, qualifiers }) => (
        <div key={record.record_key} role="listitem">
          <ReferenceResult
            row={{ record, matches: [], relationship_evidence: [] }}
            selected={false}
            disabled={false}
            context={qualifier}
            qualifiers={qualifiers}
            onPreview={(anchor) => onReference?.(record.record_key, anchor)}
          />
        </div>
      ))}
    </div>
  );
}
