import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import type { ReactNode } from "react";
import type { CreatureSurfaceSpellView } from "../../generated/atlas";
import { RecordPreviewScope } from "./RecordPreviewScope";
import { SpellOccurrencePreviewPopover } from "./SpellOccurrencePreviewPopover";

describe("SpellOccurrencePreviewPopover", () => {
  it("shows immediate typed occurrence content and opens the spell record", () => {
    const onOpenSpellRecord = vi.fn();
    render(
      <SpellOccurrencePreviewPopover
        onOpenSpellRecord={onOpenSpellRecord}
        onReference={vi.fn()}
        spell={spellFixture()}
      />,
    );

    fireEvent.click(screen.getByRole("link", { name: "Control Weather" }));
    const dialog = screen.getByRole("dialog", {
      name: "Control Weather spell details",
    });
    const header = document.querySelector<HTMLElement>(".preview-popover__header");
    expect(header).not.toBeNull();
    expect(within(header!).getByText("Control Weather")).toBeInTheDocument();
    expect(within(dialog).queryByText("Control Weather")).not.toBeInTheDocument();
    expect(within(dialog).getByText("8th")).toBeInTheDocument();
    expect(within(dialog).getByText("You alter the weather.")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Open spell record" }));
    expect(onOpenSpellRecord).toHaveBeenCalledWith("spells:control-weather");
  });

  it("shows actor-owned targetless spell content without inventing navigation", () => {
    const spell = spellFixture();
    spell.occurrence_id = "bind-soul-heartstone";
    spell.label = "Bind Soul (At Will) (Heartstone)";
    spell.target_record_key = undefined;
    spell.context = {
      contextual_label: "At will",
      group: "heartstone",
      uses: { maximum: 1 },
    };
    spell.content![0]!.blocks = [
      {
        block_type: "paragraph",
        spans: [
          {
            span_type: "text",
            text: "The heartstone binds a soul without a canonical spell target.",
          },
        ],
      },
    ];
    const onOpenSpellRecord = vi.fn();
    render(
      <SpellOccurrencePreviewPopover
        onOpenSpellRecord={onOpenSpellRecord}
        onReference={vi.fn()}
        spell={spell}
      />,
    );

    const trigger = screen.getByRole("button", {
      name: "Bind Soul (At Will) (Heartstone)",
    });
    expect(trigger).toHaveAttribute("aria-haspopup", "dialog");
    fireEvent.click(trigger);
    const dialog = screen.getByRole("dialog", {
      name: "Bind Soul (At Will) (Heartstone) spell details",
    });
    expect(
      within(dialog).getByText(
        "The heartstone binds a soul without a canonical spell target.",
      ),
    ).toBeInTheDocument();
    expect(
      within(dialog).getByText(/At will · Group heartstone · 1 use/),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Open spell record" }),
    ).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Close spell preview" }));
    expect(onOpenSpellRecord).not.toHaveBeenCalled();
  });

  it.each([
    ["targeted", "link"],
    ["targetless", "button"],
  ] as const)(
    "keeps typed references inside an open %s spell preview in the existing dialog",
    async (kind, triggerRole) => {
      const spell = spellFixture();
      if (kind === "targetless") {
        spell.target_record_key = undefined;
      }
      spell.content![0]!.blocks = [
        {
          block_type: "paragraph",
          spans: [
            { span_type: "text", text: "See " },
            {
              span_type: "reference",
              label: "Weather domain",
              record_key: "rules:weather-domain",
              embedded: false,
            },
            { span_type: "text", text: " for details." },
          ],
        },
      ];
      const onOpenFullPage = vi.fn();
      const onReference = vi.fn();
      render(
        <RecordPreviewScope onOpenFullPage={onOpenFullPage}>
          <SpellOccurrencePreviewPopover
            onOpenSpellRecord={vi.fn()}
            onReference={onReference}
            spell={spell}
          />
        </RecordPreviewScope>,
        { wrapper: queryClientWrapper() },
      );

      const trigger = screen.getByRole(triggerRole, { name: "Control Weather" });
      fireEvent.click(trigger);
      const dialog = screen.getByRole("dialog", {
        name: "Control Weather spell details",
      });
      const reference = within(dialog).getByRole("link", {
        name: "Weather domain",
      });

      reference.focus();
      expect(reference).toHaveFocus();
      expect(screen.getAllByRole("dialog")).toHaveLength(1);
      expect(onReference).not.toHaveBeenCalled();

      fireEvent.click(reference);
      expect(onReference).toHaveBeenCalledTimes(1);
      expect(onReference).toHaveBeenCalledWith("rules:weather-domain");
      expect(onOpenFullPage).not.toHaveBeenCalled();
      expect(screen.getAllByRole("dialog")).toHaveLength(1);

      fireEvent.keyDown(document, { key: "Escape" });
      await waitFor(() => expect(screen.queryByRole("dialog")).not.toBeInTheDocument());
      await waitFor(() => expect(trigger).toHaveFocus());
    },
  );
});

function queryClientWrapper() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return function QueryClientWrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
  };
}

function spellFixture(): CreatureSurfaceSpellView {
  return {
    occurrence_id: "control-weather",
    authored_order: 0,
    provenance: { identity_stability: "stable_nested_source_id" },
    label: "Control Weather",
    target_record_key: "spells:control-weather",
    rank: 8,
    content: [
      {
        content_key: "control-weather-content",
        role: "embedded_capability",
        authored_order: 0,
        blocks: [
          {
            block_type: "paragraph",
            spans: [{ span_type: "text", text: "You alter the weather." }],
          },
        ],
        content_hash: "control-weather-hash",
        visibility: "public",
        provenance: {
          source_record_key: "spells:control-weather",
          relative_source_path: "fixture.json",
          field_family: "fixture.spell",
        },
      },
    ],
  };
}
