import { Button, Select } from "antd";
import { useState } from "react";
import type { SpellPresentationView } from "../../generated/atlas";

export function SpellRankControl({
  spell,
  embedded,
  onChange,
}: {
  spell: SpellPresentationView;
  embedded: boolean;
  onChange: (rank: number | undefined) => void;
}) {
  const base = Number(spell.base_rank.value);
  const rank = Number(spell.rank.value);
  const [openingRank] = useState(() => (embedded ? rank : base));
  return (
    <div className="spell-rank-control">
      <div className="spell-rank-control__row">
        <Select
          aria-label="Spell rank"
          className="record-reference__rank"
          size="small"
          virtual={false}
          prefix="Rank"
          value={rank}
          options={Array.from({ length: 11 - base }, (_, i) => ({
            value: base + i,
            label: String(base + i),
          }))}
          onChange={onChange}
        />
      </div>
      <div className="spell-rank-control__secondary">
        {rank !== base && (
          <span className="spell-rank-control__base">Base rank {base}</span>
        )}
        <Button
          type="link"
          size="small"
          className="spell-rank-control__reset"
          style={{ display: rank === openingRank ? "none" : undefined }}
          tabIndex={rank === openingRank ? -1 : undefined}
          onClick={() => onChange(openingRank)}
        >
          Reset rank
        </Button>
      </div>
    </div>
  );
}
