import { detailFixture } from "../../test/fixtures";
import { fireEvent, render, screen } from "@testing-library/react";
import type { RecordDetailView } from "../../generated/atlas";
import { RecordPreviewPopover } from "./RecordPreviewPopover";

describe("RecordPreviewPopover", () => {
  it("opens nested references without re-anchoring the active popover", () => {
    const onReference = vi.fn();
    render(
      <RecordPreviewPopover
        anchor={{ top: 10, right: 20, bottom: 30, left: 5, width: 15, height: 20 }}
        detail={recordDetailFixture()}
        loading={false}
        onClose={vi.fn()}
        onOpenFullPage={vi.fn()}
        onReference={onReference}
      />,
    );

    fireEvent.click(screen.getByText("Nested Rule"));

    expect(onReference).toHaveBeenCalledWith(
      "rules:nested",
      expect.objectContaining({ width: expect.any(Number) }),
      expect.objectContaining({ record_key: "rules:nested" }),
    );
  });
});

function recordDetailFixture(): RecordDetailView {
  return detailFixture("spell:dirge", "Dirge of Doom", "rules:nested");
}
