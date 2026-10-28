import { Spin } from "antd";
import { useEffect, useState } from "react";

/** Mount only while loading. Reserve the slot, but delay the entire announcement. */
export function LoadingStatus({
  label,
  className,
}: {
  label: string;
  className?: string;
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
      <Spin size="small" /> <span>{label}</span>
    </div>
  );
}
