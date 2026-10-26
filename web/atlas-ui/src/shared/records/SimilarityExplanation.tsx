import { Button, Table, Typography } from "antd";
import type {
  RecordNavigationView,
  SimilarityEvidenceView,
} from "../../generated/atlas";
import { destinationHref, isPrimaryActivation } from "../../app/navigation";
import { recordDestination } from "../../app/destinations";

export function SimilarityExplanation({
  evidence,
  onNavigate,
}: {
  evidence: SimilarityEvidenceView;
  onNavigate: (navigation: RecordNavigationView, anchor: HTMLElement) => void;
}) {
  const contributions = evidence.roles.length
    ? evidence.roles.map((role) => ({
        key: role.role,
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
  return (
    <div className="similarity-explanation">
      {evidence.truncated && (
        <Typography.Paragraph type="secondary">
          Showing part of the comparison evidence. The score includes all selected
          parts.
        </Typography.Paragraph>
      )}
      <Typography.Text>
        {evidence.comparable_parts} of {evidence.selected_parts} selected parts have
        comparable content.
      </Typography.Text>
      {contributions.length > 0 && (
        <Table
          size="small"
          rowKey="key"
          pagination={false}
          dataSource={contributions}
          columns={[
            {
              title: "Part",
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
              render: (weight: number) => weight.toFixed(3),
            },
          ]}
        />
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
            <strong>Seed</strong>
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
            <strong>Match</strong>
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
  onNavigate: (navigation: RecordNavigationView, anchor: HTMLElement) => void;
}) {
  return (
    <Button
      type="link"
      href={destinationHref(
        recordDestination({ recordKey: navigation.record_key, selection: navigation }),
      )}
      onClick={(event) => {
        if (!isPrimaryActivation(event)) return;
        event.preventDefault();
        onNavigate(navigation, event.currentTarget);
      }}
    >
      {label}
    </Button>
  );
}
