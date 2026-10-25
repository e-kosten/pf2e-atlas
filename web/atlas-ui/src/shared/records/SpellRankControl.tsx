import { Button, Dropdown } from "antd";
import { useEffect, useRef, useState, type KeyboardEvent } from "react";
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
  const popup = useRef<HTMLDivElement>(null);
  const [open, setOpen] = useState(false);
  useEffect(() => {
    if (!open) return;
    const focusMenu = () => {
      const document = trigger.current?.ownerDocument;
      if (!document) return;
      const active = document.activeElement;
      if (active !== trigger.current && active !== document.body) return;
      const item = popup.current?.querySelector<HTMLElement>(
        '[role="menuitem"]:not([aria-disabled="true"])',
      );
      item?.focus({ preventScroll: true });
      if (!item || document.activeElement !== item)
        frame = requestAnimationFrame(focusMenu);
    };
    let frame = requestAnimationFrame(focusMenu);
    return () => cancelAnimationFrame(frame);
  }, [open]);
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
        popupRender={(menu) => <div ref={popup}>{menu}</div>}
        open={open}
        onOpenChange={setOpen}
        menu={{
          "aria-label":
            spell.ritual.state === "value" && spell.ritual.value
              ? "Ritual rank"
              : "Spell rank",
          selectable: true,
          selectedKeys: [String(rank)],
          onKeyDown: (event) => {
            closeOnEscape(event);
            if (event.key === "Enter") event.preventDefault();
            if (event.key === "Tab") {
              event.stopPropagation();
              setOpen(false);
              trigger.current?.focus({ preventScroll: true });
            }
          },
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
          className="spell-rank-control__trigger"
          aria-haspopup="menu"
          aria-expanded={open}
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
