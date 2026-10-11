import { Alert, Button, List, Modal, Select, Space, Spin, Typography } from "antd";
import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import {
  getRecordGraph,
  getRecordRemaster,
  getRecordVariants,
  getSimilarRecords,
} from "../../api/atlasApi";
import { navigateToAtlasRoute } from "../../app/routes";
import type { RecordDetailView, RecordSummaryView } from "../../generated/atlas";
import { RecordDetailPane } from "./RecordDetailPane";
import { useRecordDetail } from "./useRecordDetail";

export function RelatedRecords({ detail }: { detail: RecordDetailView }) {
  const [active, setActive] = useState<string | null>(null);
  const [comparison, setComparison] = useState<string | null>(null);
  const key = detail.record.record_key;
  const graph = useQuery({
    queryKey: ["graph", detail.selected],
    enabled: active === "connections",
    queryFn: () =>
      getRecordGraph(key, detail.selected.owners, detail.selected.source_fingerprint),
  });
  const remaster = useQuery({
    queryKey: ["remaster", key],
    enabled: active === "remaster",
    queryFn: () => getRecordRemaster(key),
  });
  const variants = useQuery({
    queryKey: ["variants", key],
    enabled: active === "variants",
    queryFn: () => getRecordVariants(key),
  });
  const similar = useQuery({
    queryKey: ["similar", key],
    enabled: active === "similar",
    queryFn: () => getSimilarRecords(key),
  });
  const query =
    active === "connections"
      ? graph
      : active === "remaster"
        ? remaster
        : active === "variants"
          ? variants
          : similar;
  function records(rows: RecordSummaryView[]) {
    return (
      <List
        dataSource={rows}
        renderItem={(record) => (
          <List.Item>
            <Button
              type="link"
              onClick={() =>
                navigateToAtlasRoute({ kind: "record", recordKey: record.record_key })
              }
            >
              {record.title}
            </Button>
          </List.Item>
        )}
      />
    );
  }
  return (
    <section aria-label="Related records">
      <Space wrap>
        {["connections", "remaster", "variants", "similar"].map((name) => (
          <Button
            key={name}
            type={active === name ? "primary" : "default"}
            onClick={() => setActive(active === name ? null : name)}
          >
            {name === "variants"
              ? "Suggested variants"
              : name[0].toUpperCase() + name.slice(1)}
          </Button>
        ))}
      </Space>
      {active && query.isFetching && <Spin />}
      {active && query.error && <Alert type="info" message={query.error.message} />}
      {active === "connections" && graph.data && (
        <>
          <Typography.Title level={5}>Outgoing</Typography.Title>
          {records(graph.data.outgoing.records)}
          <Typography.Title level={5}>Backlinks</Typography.Title>
          {records(graph.data.backlinks.records)}
          <List
            dataSource={[
              ...graph.data.outgoing.occurrences,
              ...graph.data.backlinks.occurrences,
            ]}
            renderItem={(occurrence) => (
              <List.Item>
                {occurrence.target ? (
                  <Button
                    type="link"
                    onClick={() =>
                      navigateToAtlasRoute({
                        kind: "record",
                        recordKey: occurrence.target!.record_key,
                        selection: occurrence.target!,
                      })
                    }
                  >
                    {occurrence.authored_target || occurrence.kind}
                  </Button>
                ) : (
                  <Typography.Text>
                    {occurrence.authored_target || occurrence.kind}: {occurrence.status}
                  </Typography.Text>
                )}
              </List.Item>
            )}
          />
          {(graph.data.outgoing.truncated || graph.data.backlinks.truncated) && (
            <Alert type="info" message="Showing bounded relationship occurrences." />
          )}
        </>
      )}
      {active === "remaster" && remaster.data && (
        <List
          dataSource={remaster.data.links}
          renderItem={(link) => (
            <List.Item>
              {records([link.legacy_record, link.remaster_record])}
              <Typography.Text type="secondary">{link.evidence}</Typography.Text>
            </List.Item>
          )}
        />
      )}
      {active === "variants" && variants.data && (
        <>
          <Alert
            type="info"
            message={
              variants.data.ambiguous
                ? "Suggested variant evidence is ambiguous."
                : "Suggestions share naming and compatible source facts; they do not create aliases."
            }
          />
          {records(variants.data.variants)}
          <List
            dataSource={variants.data.evidence}
            renderItem={(e) => (
              <List.Item>
                {e.qualifier}:{" "}
                {e.naming_convention === "trailing_parenthetical"
                  ? "Names share a base with parenthesized variants"
                  : e.naming_convention === "leading_grade"
                    ? "Names share a base with different grades"
                    : "Shared naming pattern"}
                ; {e.compatibility}
              </List.Item>
            )}
          />
          <Select
            aria-label="Compare variant"
            placeholder="Compare variant"
            value={comparison}
            options={variants.data.variants
              .filter((v) => v.record_key !== key)
              .map((v) => ({ value: v.record_key, label: v.title }))}
            onChange={setComparison}
          />
          {variants.data.truncated && (
            <Alert type="info" message="Suggested variants are bounded." />
          )}
        </>
      )}
      {active === "similar" && similar.data && (
        <>
          <Typography.Paragraph type="secondary">
            {similar.data.results.coverage.count_basis};{" "}
            {similar.data.results.coverage.semantic_unit_window} candidate units
          </Typography.Paragraph>
          {records(similar.data.results.rows.map((row) => row.record))}
        </>
      )}
      {active && !query.isFetching && !query.error && query.data === null && (
        <Typography.Paragraph>No related results.</Typography.Paragraph>
      )}
      {comparison && (
        <Comparison
          source={key}
          target={comparison}
          onClose={() => setComparison(null)}
        />
      )}
    </section>
  );
}
function Comparison({
  source,
  target,
  onClose,
}: {
  source: string;
  target: string;
  onClose: () => void;
}) {
  const left = useRecordDetail(source);
  const right = useRecordDetail(target);
  const navigate: import("./PreparedContent").RecordReferenceHandler = (
    recordKey,
    _anchor,
    selection,
  ) => navigateToAtlasRoute({ kind: "record", recordKey, selection });
  return (
    <Modal
      title="Variant comparison"
      open
      footer={null}
      onCancel={onClose}
      width="90vw"
    >
      <div className="record-comparison">
        <RecordDetailPane
          detail={left.data}
          loading={left.isLoading}
          errors={[left.error]}
          onReference={navigate}
        />
        <RecordDetailPane
          detail={right.data}
          loading={right.isLoading}
          errors={[right.error]}
          onReference={navigate}
        />
      </div>
    </Modal>
  );
}
