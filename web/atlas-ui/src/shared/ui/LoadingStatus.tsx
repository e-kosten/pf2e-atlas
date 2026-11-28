import { Spin } from "antd";
import { useEffect, useState } from "react";

export function useDelayedLoading(loading: boolean) {
  const [status, setStatus] = useState({ loading, visible: false });
  if (status.loading !== loading) setStatus({ loading, visible: false });
  useEffect(() => {
    if (!loading) return;
    const timer = setTimeout(() => setStatus({ loading, visible: true }), 500);
    return () => clearTimeout(timer);
  }, [loading]);
  return status.loading === loading && status.visible;
}

/** Mount only while loading. Reserve the slot, but delay the entire announcement. */
export function LoadingStatus({
  label,
  className,
  showLabel = true,
}: {
  label: string;
  className?: string;
  showLabel?: boolean;
}) {
  const visible = useDelayedLoading(true);
  return (
    <div
      className={className}
      role={visible ? "status" : undefined}
      aria-label={visible ? label : undefined}
      style={{ visibility: visible ? undefined : "hidden" }}
    >
      <Spin size="small" />
      {showLabel && " "}
      <span className={showLabel ? undefined : "sr-only"}>{label}</span>
    </div>
  );
}
