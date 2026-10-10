import { render, screen } from "@testing-library/react";
import { RecordSurface } from "./RecordSurface";
import { detailFixture } from "../../test/fixtures";
import type { RecordSurfaceSectionView } from "../../generated/atlas";

describe("source-backed surfaces", () => {
  it("uses the authored level or rank label without adding another prefix", () => {
    const surface = detailFixture("spells:dragon", "Dragon Form").surface;
    surface.header.level_label = "Rank 6";
    render(<RecordSurface surface={surface} onReference={vi.fn()} />);
    expect(screen.getByText("Rank 6")).toBeVisible();
    expect(screen.queryByText("Level Rank 6")).toBeNull();
  });

  it("keeps independent authored stat groups and context alongside encounter HP controls", () => {
    const surface = detailFixture("actors:ghoul", "Ghoul").surface;
    surface.profile = "encounter_participant";
    const fact = (
      kind: RecordSurfaceSectionView["kind"],
      label: string,
      value: number,
    ): RecordSurfaceSectionView => ({
      kind,
      title: label,
      collapsed_by_default: false,
      values: [
        {
          key: label,
          label,
          value: { kind: "number", value },
          display: "static_number",
          adjusted: false,
        },
      ],
    });
    surface.sections = [
      fact("vitals", "Authored HP", 20),
      fact("defenses", "AC", 16),
      fact("saves", "Fortitude", 7),
      fact("abilities", "Strength", 3),
      fact("skills", "Stealth", 7),
      fact("movement", "Land Speed", 25),
      {
        kind: "notes",
        title: "Authored context",
        collapsed_by_default: false,
        notes: [{ label: "Conditional", text: "Bonus against poison" }],
      },
    ];
    render(
      <RecordSurface
        surface={surface}
        onReference={vi.fn()}
        slots={{ vitals: <button>Edit HP</button> }}
      />,
    );
    expect(screen.getByRole("button", { name: "Edit HP" })).toBeVisible();
    expect(screen.queryByText("Authored HP")).toBeNull();
    for (const label of [
      "AC",
      "Fortitude",
      "Strength",
      "Stealth",
      "Land Speed",
      "Bonus against poison",
    ])
      expect(screen.getAllByText(label, { exact: false })[0]).toBeVisible();
  });
});
