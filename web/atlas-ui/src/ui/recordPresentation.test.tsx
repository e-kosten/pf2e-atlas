import { fireEvent, render, screen } from "@testing-library/react";
import type { RecordDetailView } from "../generated/atlas";
import { RecordPresentation } from "./recordPresentation";

describe("RecordPresentation", () => {
  it("renders loading and empty states", () => {
    const { rerender } = render(
      <RecordPresentation detail={undefined} loading onReference={vi.fn()} />,
    );

    expect(screen.getByText("Loading record...")).toBeInTheDocument();

    rerender(
      <RecordPresentation detail={undefined} loading={false} onReference={vi.fn()} />,
    );

    expect(screen.getByText("Select a result to inspect it.")).toBeInTheDocument();
  });

  it("renders structured presentation content", () => {
    render(
      <RecordPresentation
        detail={recordDetailFixture()}
        loading={false}
        onReference={vi.fn()}
      />,
    );

    expect(screen.getByRole("heading", { name: "Dirge of Doom" })).toBeInTheDocument();
    expect(screen.getByText("spell:dirge-of-doom")).toBeInTheDocument();
    expect(screen.getByText("auditory")).toBeInTheDocument();
    expect(screen.getByText("Level")).toBeInTheDocument();
    expect(screen.getByText("3")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Description" })).toBeInTheDocument();
    expect(screen.getByText("Primary effect.")).toBeInTheDocument();
    expect(screen.getByText("Nested heading")).toBeInTheDocument();
    expect(screen.getByText("first item")).toBeInTheDocument();
    expect(screen.getByText("Sample table")).toBeInTheDocument();
    expect(screen.getByText("cell text")).toBeInTheDocument();
    expect(screen.getByText("Unlinked Reference")).toBeInTheDocument();
  });

  it("calls onReference for linked relationships and inline references", () => {
    const onReference = vi.fn();
    render(
      <RecordPresentation
        detail={recordDetailFixture()}
        loading={false}
        onReference={onReference}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Linked Spell" }));
    fireEvent.click(screen.getByRole("button", { name: "Fear" }));

    expect(onReference).toHaveBeenCalledWith("spell:linked");
    expect(onReference).toHaveBeenCalledWith("spell:fear");
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
      identity: [{ key: "level", label: "Level", value: "3" }],
      badges: [{ kind: "trait", label: "Trait", value: "auditory" }],
      sections: [
        {
          kind: "description",
          title: "Description",
          blocks: [
            { kind: "prose", content: { text: "Primary effect." } },
            {
              kind: "content",
              content: {
                blocks: [
                  { kind: "heading", level: 1, text: "Nested heading" },
                  {
                    kind: "paragraph",
                    spans: [
                      { kind: "text", text: "See " },
                      {
                        kind: "reference",
                        label: "Fear",
                        record_key: "spell:fear",
                        embedded: false,
                      },
                      { kind: "text", text: "." },
                    ],
                  },
                  {
                    kind: "list",
                    ordered: false,
                    items: [
                      {
                        blocks: [
                          {
                            kind: "paragraph",
                            spans: [{ kind: "text", text: "first item" }],
                          },
                        ],
                      },
                    ],
                  },
                  {
                    kind: "table",
                    caption: "Sample table",
                    rows: [
                      {
                        cells: [
                          {
                            blocks: [
                              {
                                kind: "paragraph",
                                spans: [{ kind: "text", text: "cell text" }],
                              },
                            ],
                          },
                        ],
                      },
                    ],
                  },
                  { kind: "rule" },
                ],
              },
            },
            {
              kind: "relationships",
              content: [
                {
                  kind: "reference",
                  label: "Linked Spell",
                  record_key: "spell:linked",
                },
                {
                  kind: "reference",
                  label: "Unlinked Reference",
                  record_key: null,
                },
              ],
            },
          ],
        },
      ],
    },
  };
}
