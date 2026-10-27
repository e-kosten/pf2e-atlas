import { Table, Typography } from "antd";
import type {
  RecordNavigationView,
  SimilarityEvidenceView,
} from "../../generated/atlas";
import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";

const formatWeight = (weight: number) => `${(weight * 100).toFixed(1)}%`;

export function SimilarityExplanation({
  evidence,
  onNavigate,
  showSmallerWeights = false,
}: {
  evidence: SimilarityEvidenceView;
  onNavigate?: (navigation: RecordNavigationView, anchor: HTMLElement) => void;
  showSmallerWeights?: boolean;
}) {
  const includesIdentity =
    evidence.parts.some((part) => part.part.kind === "identity") ||
    evidence.roles.some((role) => role.key.kind === "identity");
  const contributions = evidence.roles.length
    ? evidence.roles.map((role) => ({
        key: JSON.stringify(role.key),
        label: role.role_label,
        score: role.score,
        weight: role.weight,
        comparable: role.comparable,
      }))
    : evidence.parts.map((part) => ({
        key: part.part.selector,
        label: `${part.part.owner_label}: ${part.part.label}`,
        score: part.score,
        weight: part.weight,
        comparable: part.comparable,
      }));
  const visibleContributions = showSmallerWeights
    ? contributions
    : contributions.filter((row) => formatWeight(row.weight) !== "0.0%");
  const hiddenCount = contributions.length - visibleContributions.length;
  return (
    <div className="similarity-explanation">
      {evidence.truncated && (
        <Typography.Paragraph type="secondary">
          Showing part of the comparison evidence. The score includes the full
          selection.
        </Typography.Paragraph>
      )}
      <Typography.Text>
        {evidence.comparable_parts} of {evidence.selected_parts} selections have
        comparable content.
      </Typography.Text>
      {visibleContributions.length > 0 && (
        <Table
          size="small"
          rowKey="key"
          pagination={false}
          dataSource={visibleContributions}
          columns={[
            {
              title: includesIdentity ? "Selected content" : "Section",
              dataIndex: "label",
              render: (label: string, row) => (
                <>
                  {label}
                  {!row.comparable && (
                    <div>
                      <Typography.Text type="secondary">
                        No corresponding content
                      </Typography.Text>
                    </div>
                  )}
                </>
              ),
            },
            {
              title: "Score",
              dataIndex: "score",
              width: 68,
              render: (score: number) => score.toFixed(3),
            },
            {
              title: "Weight",
              dataIndex: "weight",
              width: 68,
              render: formatWeight,
            },
          ]}
        />
      )}
      {hiddenCount > 0 && (
        <Typography.Paragraph type="secondary">
          {hiddenCount} smaller {hiddenCount === 1 ? "weight" : "weights"} hidden.
        </Typography.Paragraph>
      )}
      {evidence.representative && (
        <Typography.Paragraph type="secondary">
          These excerpts illustrate the content; the score compares averaged vectors.
        </Typography.Paragraph>
      )}
      {evidence.pairs.map((pair, index) => (
        <div
          key={`${pair.seed_part}-${index}`}
          className="similarity-explanation__pair"
        >
          <div>
            <strong>Seed</strong>{" "}
            {pair.seed ? (
              <PassageLink
                navigation={pair.seed.navigation}
                label={pair.seed.label || "Selected passage"}
                onNavigate={onNavigate}
              />
            ) : null}
            <Typography.Paragraph>{pair.seed_text}</Typography.Paragraph>
          </div>
          <div>
            <strong>Match</strong>{" "}
            <PassageLink
              navigation={pair.candidate.navigation}
              label={pair.candidate.label || "Matching passage"}
              onNavigate={onNavigate}
            />
            {pair.candidate.snippet && (
              <Typography.Paragraph>{pair.candidate.snippet}</Typography.Paragraph>
            )}
          </div>
        </div>
      ))}
    </div>
  );
}

function PassageLink({
  navigation,
  label,
  onNavigate,
}: {
  navigation: RecordNavigationView;
  label: string;
  onNavigate?: (navigation: RecordNavigationView, anchor: HTMLElement) => void;
}) {
  return (
    <Typography.Link
      href={destinationHref(
        recordDestination({ recordKey: navigation.record_key, selection: navigation }),
      )}
      onClick={(event) => {
        if (!onNavigate || !isPrimaryActivation(event)) return;
        event.preventDefault();
        onNavigate(navigation, event.currentTarget);
      }}
    >
      {label}
    </Typography.Link>
  );
}
