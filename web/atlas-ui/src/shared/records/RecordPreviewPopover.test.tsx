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
    );
  });
});
