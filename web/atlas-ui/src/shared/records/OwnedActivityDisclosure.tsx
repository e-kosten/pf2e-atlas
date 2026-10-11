import { useState } from "react";
import { Button, Collapse, Tag } from "antd";
import type { ActorActivityView } from "../../generated/atlas";
import { atlasRoutePath } from "../../app/routes";
import { ActionCost } from "./ActionCost";
import { RecordDetailPane } from "./RecordDetailPane";
import { useRecordDetail } from "./useRecordDetail";
import type { RecordReferenceHandler } from "./PreparedContent";

/** An exact owned detail is mounted only after disclosure, sharing preview cache identity. */
export function OwnedActivityDisclosure({
  activity,
  onReference,
}: {
  activity: ActorActivityView;
  onReference: RecordReferenceHandler;
}) {
  const [open, setOpen] = useState(false);
  return (
    <Collapse
      size="small"
      ghost
      activeKey={open ? ["ability"] : []}
      onChange={(keys) => setOpen(keys.includes("ability"))}
      items={[
        {
          key: "ability",
          label: (
            <span className="actor-ability__label">
              <strong>{activity.title}</strong>{" "}
              <ActionCost fact={activity.action_cost} />
              {activity.traits.map((trait) => (
                <Tag key={`${trait.kind}-${trait.value}`}>{trait.label}</Tag>
              ))}
            </span>
          ),
          children: open ? (
            <OwnedActivityDetail
              activity={activity}
              onReference={onReference}
              onClose={() => setOpen(false)}
            />
          ) : null,
        },
      ]}
    />
  );
}

function OwnedActivityDetail({
  activity,
  onReference,
  onClose,
}: {
  activity: ActorActivityView;
  onReference: RecordReferenceHandler;
  onClose: () => void;
}) {
  const query = useRecordDetail(activity.navigation.record_key, activity.navigation);
  return (
    <div className="actor-ability__detail">
      <Button
        type="link"
        href={atlasRoutePath({
          kind: "record",
          recordKey: activity.navigation.record_key,
          selection: activity.navigation,
        })}
      >
        Open {activity.title} full page
      </Button>
      <RecordDetailPane
        detail={query.data ?? undefined}
        error={query.error}
        loading={query.isFetching}
        onRetry={() => void query.refetch()}
        onBack={onClose}
        backLabel="Close ability"
        onReference={onReference}
        compact
        inline
        showRelated={false}
      />
    </div>
  );
}
