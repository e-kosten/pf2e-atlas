import type { RecordSummaryView } from "../../generated/atlas";

export function SearchNameLabel({ record }: { record: RecordSummaryView }) {
  const edition =
    record.edition === "legacy"
      ? "Legacy"
      : record.edition === "remaster"
        ? "Remaster"
        : null;
  const source = record.publication || record.pack;
  return (
    <span className="search-name-label">
      <strong>{record.title}</strong>
      <span className="search-name-label__source">
        {[record.kind_label, source, edition].filter(Boolean).join(" · ")}
      </span>
    </span>
  );
}
