import { Button, Tooltip } from "antd";
import { Diamond } from "lucide-react";
import { useState } from "react";
import type { SpellRankChangeView } from "../../generated/atlas";

/** Explain a verified rank-derived difference without claiming complete coverage. */
export function SpellRankMarker({
  change,
  label,
}: {
  change?: SpellRankChangeView;
  label: string;
}) {
  const [open, setOpen] = useState(false);
  if (!change) return null;
  return (
    <span className="spell-rank-annotation">
      {/* Keep this inline annotation with the preceding fact's final word. */}
      {"\u2060"}
      <Tooltip
        open={open}
        onOpenChange={setOpen}
        trigger={["hover", "focus"]}
        fresh
        destroyOnHidden
        title={
          <span className="spell-rank-marker__tooltip">
            <span>
              Base rank {change.base_rank}: {change.base_value}
            </span>
            {change.explanation && <span>{change.explanation}</span>}
          </span>
        }
      >
        <Button
          type="text"
          size="small"
          shape="circle"
          className="spell-rank-marker"
          aria-label={`Heightening details for ${label}`}
          icon={<Diamond size={10} aria-hidden="true" />}
          onClick={() => setOpen(true)}
          onKeyDown={(event) => {
            if (event.key !== "Escape" || !open) return;
            event.preventDefault();
            event.stopPropagation();
            setOpen(false);
          }}
        />
      </Tooltip>
    </span>
  );
}
