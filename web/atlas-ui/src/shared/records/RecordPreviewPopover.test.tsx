import { fireEvent, render, screen } from "@testing-library/react";
import { recordDetailFixture } from "../../test/recordFixtures";
import { RecordPreviewPopover } from "./RecordPreviewPopover";

describe("RecordPreviewPopover", () => {
  it("opens a nested generated-DT0 reference without re-anchoring the overlay", () => {
    const onReference = vi.fn();
    render(
      <RecordPreviewPopover
        anchor={{ top: 10, right: 20, bottom: 30, left: 5, width: 15, height: 20 }}
        detail={recordDetailFixture({
          referenceLabel: "Nested Rule",
          referenceRecordKey: "rules:nested",
        })}
        loading={false}
        onClose={vi.fn()}
        onOpenFullPage={vi.fn()}
        onReference={onReference}
      />,
    );

    fireEvent.click(screen.getByRole("link", { name: "Nested Rule" }));

    expect(onReference).toHaveBeenCalledWith(
      "rules:nested",
      expect.objectContaining({ width: expect.any(Number) }),
      expect.any(HTMLElement),
    );
  });

  it("renders immediate content and restores its trigger after Escape", () => {
    const onClose = vi.fn();
    const trigger = document.createElement("a");
    trigger.href = "/records/spells%3Adream-message";
    document.body.appendChild(trigger);
    render(
      <RecordPreviewPopover
        anchor={{ top: 10, right: 20, bottom: 30, left: 5, width: 15, height: 20 }}
        detail={undefined}
        label="Dream Message spell details"
        loading={false}
        onClose={onClose}
        onOpenFullPage={vi.fn()}
        onReference={vi.fn()}
        title="Spell"
        triggerElement={trigger}
      >
        <p>The immediate occurrence content.</p>
      </RecordPreviewPopover>,
    );

    expect(
      screen.getByRole("dialog", { name: "Dream Message spell details" }),
    ).toHaveTextContent("The immediate occurrence content.");
    fireEvent.keyDown(document, { key: "Escape" });
    expect(onClose).toHaveBeenCalledOnce();
    expect(document.activeElement).toBe(trigger);

    fireEvent.click(screen.getByLabelText("Reference preview overlay"));
    expect(onClose).toHaveBeenCalledTimes(2);
    trigger.remove();
  });
});
