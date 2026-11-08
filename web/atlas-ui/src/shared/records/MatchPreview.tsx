import { Typography } from "antd";
import type { MatchPreviewView } from "../../generated/atlas";

export function MatchPreview({ preview }: { preview?: MatchPreviewView | null }) {
  if (!preview) return null;
  if (preview.kind === "table") {
    return (
      <div className="match-preview">
        <Typography.Text type="secondary">
          {preview.columns.length ? `Table · ${preview.columns.join(", ")}` : "Table"}
        </Typography.Text>
      </div>
    );
  }
  return (
    <div className="match-preview">
      {preview.blocks.map((block, index) => (
        <div key={index}>
          <Typography.Text type="secondary">
            {block}
            {preview.truncated && index === preview.blocks.length - 1 ? "…" : ""}
          </Typography.Text>
        </div>
      ))}
    </div>
  );
}
