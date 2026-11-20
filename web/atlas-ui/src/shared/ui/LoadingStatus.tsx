import { Spin } from "antd";
import { useEffect, useState } from "react";

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
  const [visible, setVisible] = useState(false);
  useEffect(() => {
    const timer = setTimeout(() => setVisible(true), 500);
    return () => clearTimeout(timer);
  }, []);
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
