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

  it("prefers the composed record surface when present", () => {
    render(
      <RecordDetailPane
        detail={{
          ...recordDetailFixture(),
          surface: {
            record_key: "actors:testCreature",
            title: "Test Creature",
            kind: "creature",
            profile: "record_detail",
            header: {
              level_label: "3",
              kind_label: "Creature",
              traits: [{ kind: "trait", label: "hag", value: "hag" }],
            },
            sections: [
              {
                kind: "defenses",
                title: "Defenses",
                values: [
                  {
                    key: "ac",
                    label: "AC",
                    value: { kind: "number", value: 25n },
                    base_value: { kind: "number", value: 25n },
                    adjusted: false,
                    display: "static_number",
                  },
                ],
                collapsed_by_default: false,
              },
            ],
            fallback_presentation: recordDetailFixture().presentation,
          },
        }}
        loading={false}
        onReference={vi.fn()}
      />,
    );

    expect(screen.getAllByRole("heading", { name: "Test Creature" })).toHaveLength(2);
    expect(screen.getByText("AC")).toBeInTheDocument();
    expect(screen.getByText("25")).toBeInTheDocument();
    expect(screen.getByText("Source presentation")).toBeInTheDocument();
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
