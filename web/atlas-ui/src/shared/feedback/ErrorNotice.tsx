import { Alert, Typography } from "antd";
import type { ReactNode } from "react";
import { errorExplanation, isSharedFailure } from "./errorMessages";
import { AtlasConnectionError } from "../../api/availability";

/** One local owner per failed task; the shell owns shared transport failures. */
export function ErrorNotice({
  error,
  title,
  description,
  action,
  compact = false,
  type = "error",
  className,
  outcome,
}: {
  error: unknown;
  title: ReactNode;
  description?: ReactNode;
  action?: ReactNode;
  compact?: boolean;
  type?: "error" | "warning" | "info";
  className?: string;
  outcome?: "saved" | "changed";
}) {
  const unconfirmedWrite = error instanceof AtlasConnectionError && error.write;
  const shared = isSharedFailure(error);
  if (shared && !unconfirmedWrite && !outcome) return null;
  // The shell explains the outage. An attempted write still needs its own
  // outcome beside the action, including inside an open editor/confirmation.
  const heading =
    unconfirmedWrite && !outcome ? "Change could not be confirmed" : title;
  const explanation =
    shared && outcome
      ? undefined
      : unconfirmedWrite
        ? errorExplanation(error)
        : (description ?? errorExplanation(error));
  if (compact || (shared && outcome && !unconfirmedWrite))
    return (
      <div className={`local-feedback${className ? ` ${className}` : ""}`}>
        <Typography.Text role="status" type="secondary">
          {heading}
          {explanation && <> {explanation}</>}
        </Typography.Text>
        {action}
      </div>
    );
  return (
    <Alert
      className={className}
      type={type}
      showIcon
      role={type === "error" ? "alert" : "status"}
      title={heading}
      description={explanation}
      action={action}
    />
  );
}
