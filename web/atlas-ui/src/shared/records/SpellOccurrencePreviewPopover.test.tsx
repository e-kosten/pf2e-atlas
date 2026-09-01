import { fireEvent, render, screen, within } from "@testing-library/react";
import type { CreatureSurfaceSpellView } from "../../generated/atlas";
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
    expect(within(dialog).getByText("8th")).toBeInTheDocument();
    expect(within(dialog).getByText("You alter the weather.")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Open spell record" }));
    expect(onOpenSpellRecord).toHaveBeenCalledWith("spells:control-weather");
  });
});

function spellFixture(): CreatureSurfaceSpellView {
  return {
    occurrence_id: "control-weather",
    authored_order: 0,
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
