import { Button, ConfigProvider, Dropdown } from "antd";
import { useRef, useState, type KeyboardEvent } from "react";
import { ChevronDown } from "lucide-react";
import type { SpellPresentationView } from "../../generated/atlas";
import { useMenuFocus } from "../ui/overlays/useMenuFocus";

const tabbableSelector = [
  "a[href]",
  "button:not([disabled])",
  "input:not([disabled])",
  "select:not([disabled])",
  "textarea:not([disabled])",
  '[tabindex]:not([tabindex="-1"])',
].join(",");

function moveFocusFromTrigger(trigger: HTMLButtonElement, reverse: boolean) {
  const root =
    trigger.closest<HTMLElement>('[role="dialog"]') ?? trigger.ownerDocument.body;
  const candidates = [...root.querySelectorAll<HTMLElement>(tabbableSelector)].filter(
    (candidate) => !candidate.closest('[hidden],[aria-hidden="true"]'),
  );
  const index = candidates.indexOf(trigger);
  if (index < 0 || candidates.length < 2) return;
  const destination =
    candidates[(index + (reverse ? -1 : 1) + candidates.length) % candidates.length];
  destination?.focus({ preventScroll: true });
}

export function SpellRankControl({
  spell,
  classification,
  embedded,
  onChange,
}: {
  spell: SpellPresentationView;
  classification: string;
  embedded: boolean;
  onChange: (rank: number | undefined) => void;
}) {
  const base = Number(spell.base_rank.value);
  const rank = Number(spell.rank.value);
  const [openingRank] = useState(() => (embedded ? rank : base));
  const trigger = useRef<HTMLButtonElement>(null);
  const popup = useRef<HTMLDivElement>(null);
  const [open, setOpen] = useState(false);
  useMenuFocus(open, trigger, popup);
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
    <ConfigProvider
      theme={{
        components: {
          Button: {
            defaultBg: "var(--record-kind-bg)",
            defaultColor: "var(--record-kind-text)",
            defaultBorderColor: "var(--record-kind-border)",
          },
        },
      }}
    >
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
              event.preventDefault();
              event.stopPropagation();
              setOpen(false);
              const button = trigger.current;
              if (button) {
                requestAnimationFrame(() =>
                  moveFocusFromTrigger(button, event.shiftKey),
                );
              }
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
          className="record-reference__kind spell-rank-control"
          icon={<ChevronDown size={14} aria-hidden="true" />}
          iconPlacement="end"
          aria-label={`${classification}: Change rank`}
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
          {classification}
        </Button>
      </Dropdown>
    </ConfigProvider>
  );
}
