import { useRouter, useRouterState } from "@tanstack/react-router";
import { Select, Space } from "antd";
import { atlasRouter } from "../../app/router";
import { recordDestination } from "../../app/destinations";
import { AtlasLink } from "../../app/AtlasLink";
import type { RecordDetailView } from "../../generated/atlas";

export function useCreatureScenario(recordKey: string | undefined) {
  const contextRouter = useRouter({ warn: false });
  const router = contextRouter ?? atlasRouter;
  const useId = useRouterState({
    router,
    select: ({ location }) => {
      const selection = location.search.scenario_use;
      if (!recordKey || typeof selection !== "string") return undefined;
      const prefix = recordKey + "~";
      return selection.startsWith(prefix) ? selection.slice(prefix.length) : undefined;
    },
  });
  function selectUse(id: string | undefined) {
    const search = { ...router.latestLocation.search };
    if (id && recordKey) search.scenario_use = recordKey + "~" + id;
    else delete search.scenario_use;
    void router.navigate({
      to: ".",
      search,
      hash: true,
      resetScroll: false,
      state: true,
    });
  }
  return { useId, selectUse };
}

export function CreatureScenarioChoices({ detail }: { detail: RecordDetailView }) {
  const uses = detail.record.creature_display?.uses ?? [];
  const { useId, selectUse } = useCreatureScenario(detail.record.record_key);
  if (!uses.length) return null;
  const selected = useId ?? (uses.length === 1 ? uses[0].id : undefined);
  const overrides = detail.creature_overrides?.filter(
    (entry) => entry.use_id === selected,
  );
  return (
    <Space direction="vertical" size="small" style={{ maxWidth: "100%" }}>
      {uses.length > 1 ? (
        <Select
          aria-label="Scenario use"
          placeholder="Choose scenario use"
          popupMatchSelectWidth={false}
          style={{ maxWidth: "100%" }}
          value={uses.some((entry) => entry.id === selected) ? selected : undefined}
          onChange={selectUse}
          allowClear
          options={uses.map(({ id, label }) => ({ value: id, label }))}
        />
      ) : (
        <span>{uses[0].label}</span>
      )}
      {overrides?.map(({ adjustment, record_key }) => (
        <AtlasLink key={adjustment} {...recordDestination({ recordKey: record_key })}>
          {adjustment === "elite" ? "Elite" : "Weak"} (published)
        </AtlasLink>
      ))}
    </Space>
  );
}
