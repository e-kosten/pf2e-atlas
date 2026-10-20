// Capability reasons are projected sentences, displayed here as short fact labels.
export function statusReasonLabel(status: string, reason?: string | null) {
  const label = reason
    ?.trim()
    .replace(/[.!?]+$/, "")
    .trimEnd();
  return label ? `${status} · ${label}` : status;
}
