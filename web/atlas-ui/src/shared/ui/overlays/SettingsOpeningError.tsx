import { Button } from "antd";
import { ErrorNotice } from "../../feedback/ErrorNotice";

export function SettingsOpeningError({
  error,
  title,
  onRetry,
}: {
  error: Error | null;
  title: string;
  onRetry: () => void;
}) {
  return error ? (
    <ErrorNotice
      error={error}
      title={title}
      action={<Button onClick={onRetry}>Retry</Button>}
    />
  ) : null;
}
