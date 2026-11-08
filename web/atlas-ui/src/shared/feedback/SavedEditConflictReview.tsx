import { Alert, Button, Flex, Typography } from "antd";

type ConflictField = {
  field: string;
  label: string;
  saved: string;
  draft: string;
  onUseSaved: () => void;
  onKeepMine: () => void;
};

export function SavedEditConflictReview({
  fields,
  loading,
  refreshFailed,
  onRetry,
}: {
  fields: readonly ConflictField[];
  loading?: boolean;
  refreshFailed?: boolean;
  onRetry?: () => void;
}) {
  return (
    <Alert
      type="warning"
      showIcon
      title="Saved values changed"
      description={
        loading ? (
          "Loading the latest saved values…"
        ) : refreshFailed ? (
          <Flex vertical gap="small" align="start">
            <span>Could not load the latest values. Your draft is still here.</span>
            {onRetry && (
              <Button size="small" onClick={onRetry}>
                Retry loading saved values
              </Button>
            )}
          </Flex>
        ) : (
          <Flex vertical gap="middle">
            {fields.map(({ field, label, saved, draft, onUseSaved, onKeepMine }) => (
              <Flex key={field} vertical gap="small" role="group" aria-label={label}>
                <Typography.Text strong>{label}</Typography.Text>
                <Typography.Text
                  style={{ whiteSpace: "pre-wrap", overflowWrap: "anywhere" }}
                >
                  <strong>Saved:</strong> {saved}
                </Typography.Text>
                <Typography.Text
                  style={{ whiteSpace: "pre-wrap", overflowWrap: "anywhere" }}
                >
                  <strong>Your draft:</strong> {draft}
                </Typography.Text>
                <Flex gap="small" wrap>
                  <Button
                    size="small"
                    aria-label={`Use saved ${label}`}
                    onClick={onUseSaved}
                  >
                    Use saved
                  </Button>
                  <Button
                    size="small"
                    aria-label={`Keep mine for ${label}`}
                    onClick={onKeepMine}
                  >
                    Keep mine
                  </Button>
                </Flex>
              </Flex>
            ))}
          </Flex>
        )
      }
    />
  );
}
