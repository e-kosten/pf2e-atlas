import { render, screen } from "@testing-library/react";
import type { RecordDetailView } from "../../generated/atlas";
import { RecordDetailPane } from "./RecordDetailPane";

describe("RecordDetailPane", () => {
  it("wraps record presentation with a detail panel", () => {
    const { container } = render(
      <RecordDetailPane
        detail={recordDetailFixture()}
        loading={false}
        onReference={vi.fn()}
      />,
    );

    expect(container.querySelector(".detail-panel")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Dirge of Doom" })).toBeInTheDocument();
  });

  it("renders custom empty states and errors", () => {
    render(
      <RecordDetailPane
        detail={undefined}
        emptyMessage="This saved record is unresolved."
        errors={[new Error("Unable to load detail")]}
        loading={false}
        onReference={vi.fn()}
      />,
    );

    expect(screen.getByText("This saved record is unresolved.")).toBeInTheDocument();
    expect(screen.getByText("Unable to load detail")).toBeInTheDocument();
  });
});

function recordDetailFixture(): RecordDetailView {
  return {
    record_key: "spell:dirge-of-doom",
    title: "Dirge of Doom",
    kind: "spell",
    presentation: {
      record_key: "spell:dirge-of-doom",
      kind: "spell",
      title: "Dirge of Doom",
      identity: [],
      badges: [],
      sections: [],
    },
  };
}
