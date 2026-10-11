import {
  detailFixture,
  actorFixture,
  numberFact,
  summaryFixture,
} from "../../test/fixtures";
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

  it("renders the semantic family facts once", () => {
    render(
      <RecordDetailPane
        detail={{
          ...recordDetailFixture(),
          presentation: {
            identity: {
              ...summaryFixture("actors:testCreature", "Test Creature"),
              kind: "creature",
              kind_label: "Creature",
              level_label: "3",
              traits: [{ kind: "trait", label: "hag", value: "hag" }],
            },
            body: {
              kind: "creature",
              value: { ...actorFixture(), armor_class: numberFact(25) },
            },
            content: [],
            owned: [],
          },
        }}
        loading={false}
        onReference={vi.fn()}
      />,
    );

    expect(screen.getAllByRole("heading", { name: "Test Creature" })).toHaveLength(1);
    expect(screen.getByText("AC")).toBeInTheDocument();
    expect(screen.getByText("25")).toBeInTheDocument();
    expect(screen.queryByText("Source presentation")).toBeNull();
  });
});

function recordDetailFixture(): RecordDetailView {
  return detailFixture("spell:dirge", "Dirge of Doom");
}
