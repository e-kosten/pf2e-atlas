import { Button, Dropdown } from "antd";
import { ChevronDown } from "lucide-react";
import { useRef, useState, type KeyboardEvent } from "react";
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
  const trigger = useRef<HTMLButtonElement>(null);
  const [open, setOpen] = useState(false);
  const chooseRank = (nextRank: number) => {
    setOpen(false);
    if (nextRank !== rank) onChange(nextRank);
    trigger.current?.focus({ preventScroll: true });
  };
  const closeOnEscape = (event: KeyboardEvent<HTMLElement>) => {
    if (event.key !== "Escape" || !open) return;
    event.preventDefault();
    event.stopPropagation();
    setOpen(false);
    trigger.current?.focus({ preventScroll: true });
  };
  return (
    <div className="spell-rank-control">
      <Dropdown
        trigger={["click"]}
        // Focus the menu only after explicit activation, never on document load.
        // eslint-disable-next-line jsx-a11y/no-autofocus
        autoFocus
        open={open}
        onOpenChange={setOpen}
        menu={{
          "aria-label":
            spell.ritual.state === "value" && spell.ritual.value
              ? "Ritual rank"
              : "Spell rank",
          selectable: true,
          selectedKeys: [String(rank)],
          onKeyDown: closeOnEscape,
          items: [
            ...Array.from({ length: 11 - base }, (_, i) => ({
              key: String(base + i),
              label: `Rank ${base + i}${i === 0 ? " (base)" : ""}`,
            })),
            ...(embedded && openingRank !== base && rank !== openingRank
              ? [
                  { type: "divider" as const },
                  { key: "reset", label: `Return to opening rank ${openingRank}` },
                ]
              : []),
          ],
          onClick: ({ key }) => chooseRank(key === "reset" ? openingRank : Number(key)),
        }}
      >
        <Button
          ref={trigger}
          type="text"
          size="small"
          aria-haspopup="menu"
          aria-expanded={open}
          icon={<ChevronDown size={14} aria-hidden="true" />}
          iconPlacement="end"
          onKeyDown={(event) => {
            closeOnEscape(event);
            if (event.key === "ArrowDown") {
              event.preventDefault();
              setOpen(true);
            }
          }}
        >
          Change rank
        </Button>
      </Dropdown>
    </div>
  );
}
