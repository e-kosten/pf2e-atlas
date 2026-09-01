import { fireEvent, render, screen, within } from "@testing-library/react";
import type {
  CreatureSurfaceContentView,
  RecordSurfaceView,
} from "../../generated/atlas";
import {
  creatureSurfaceFixture,
  encounterRuntimeFixture,
} from "../../test/recordFixtures";
import { RecordSurface } from "./RecordSurface";

const onReference = vi.fn();

describe("RecordSurface", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders description before mechanics without a redundant creature eyebrow", () => {
    const { container } = renderSurface();

    const description = screen.getByRole("heading", { name: "Description & Lore" });
    const mechanics = screen.getByRole("heading", { name: "Actions & Abilities" });
    expect(
      description.compareDocumentPosition(mechanics) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    expect(
      screen.getByRole("heading", { name: "Dream-Coven Envoy" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Level 9")).toBeInTheDocument();
    expect(screen.queryByText("Creature")).not.toBeInTheDocument();
    expect(container.querySelector(".creature-sheet__snapshot")).not.toHaveTextContent(
      "Speed",
    );
  });

  it("keeps all movement modes together outside defenses", () => {
    renderSurface();

    const movement = screen.getByRole("heading", { name: "Movement" }).parentElement;
    expect(movement).not.toBeNull();
    expect(within(movement!).getByText("Speed")).toBeInTheDocument();
    expect(within(movement!).getByText("Fly")).toBeInTheDocument();
    expect(within(movement!).getByText("25 ft")).toBeInTheDocument();
    expect(within(movement!).getByText("40 ft")).toBeInTheDocument();
  });

  it("expands typed activity content inline with check DC and divider structure", () => {
    const { container } = renderSurface();

    expect(screen.queryByText("View rules")).not.toBeInTheDocument();
    fireEvent.click(screen.getByText("Dream Bargain"));
    expect(screen.getByText("Will DC 28")).toBeInTheDocument();
    expect(container.querySelector(".creature-sheet__activity hr")).toBeInTheDocument();
  });

  it("uses collapsible grouped spellcasting and standalone content", () => {
    renderSurface();

    fireEvent.click(screen.getByText("Occult Innate Spells"));
    expect(screen.getByText("5th")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Dream Message" })).toBeInTheDocument();

    fireEvent.click(screen.getByText("Standalone Spells & Rituals"));
    expect(screen.getByText("Control Weather")).toBeInTheDocument();
  });

  it("places record identity only in the inline provenance disclosure", () => {
    renderSurface();

    expect(screen.queryByText("concept:f1-record")).not.toBeInTheDocument();
    fireEvent.click(screen.getByText("References & Source"));
    expect(screen.getByText("Record ID")).toBeInTheDocument();
    expect(screen.getByText("concept:f1-record")).toBeInTheDocument();
    expect(screen.queryByText("source contract")).not.toBeInTheDocument();
    expect(screen.queryByText("upstream commit")).not.toBeInTheDocument();
  });

  it("keeps description secondary in the encounter profile", () => {
    const surface = detailedSurfaceFixture();
    surface.profile = "encounter_participant";
    surface.encounter = encounterRuntimeFixture();

    render(<RecordSurface onReference={onReference} surface={surface} />);

    expect(screen.getByRole("button", { name: /Description & Lore/ })).toHaveAttribute(
      "aria-expanded",
      "false",
    );
    expect(
      screen.getByRole("heading", { name: "Combat Snapshot" }),
    ).toBeInTheDocument();
  });
});

function renderSurface() {
  return render(
    <RecordSurface onReference={onReference} surface={detailedSurfaceFixture()} />,
  );
}

function detailedSurfaceFixture(): RecordSurfaceView {
  const surface = creatureSurfaceFixture({
    level: 9,
    recordKey: "concept:f1-record",
    title: "Dream-Coven Envoy",
    traits: ["fiend", "hag", "unholy"],
  });
  if (surface.presentation.presentation_type !== "creature") {
    throw new Error("Fixture must be a creature surface");
  }
  surface.presentation.body = {
    ...surface.presentation.body,
    movement: [
      {
        component_id: "land",
        authored_order: 0,
        mode: "land",
        label: "Speed",
        speed_feet: 25,
      },
      {
        component_id: "fly",
        authored_order: 1,
        mode: "fly",
        speed_feet: 40,
      },
    ],
    activities: [
      ...(surface.presentation.body.activities ?? []),
      {
        occurrence_id: "dream-bargain",
        authored_order: 1,
        activity_type: "action",
        label: "Dream Bargain",
        action_cost: { cost_type: "actions", count: 2 },
        content: [activityContent()],
      },
    ],
    spellcasting: [
      {
        occurrence_id: "innate",
        authored_order: 0,
        label: "Occult Innate Spells",
        tradition: "occult",
        spells: [
          {
            occurrence_id: "dream-message",
            authored_order: 0,
            label: "Dream Message",
            target_record_key: "spells:dream-message",
            rank: 5,
          },
        ],
      },
    ],
    content: [
      ...(surface.presentation.body.content ?? []),
      {
        content_key: "control-weather",
        role: "embedded_capability",
        authored_order: 1,
        label: "Control Weather",
        blocks: [
          {
            block_type: "paragraph",
            spans: [{ span_type: "text", text: "You alter the weather." }],
          },
        ],
        content_hash: "control-weather",
        visibility: "public",
        provenance: {
          source_record_key: "concept:f1-record",
          relative_source_path: "fixture.json",
          field_family: "fixture.spell",
        },
      },
    ],
    provenance: {
      source_path: "fixture.json",
      source_contract_version: "hidden-contract",
      source_system_version: "hidden-system",
      source_upstream_commit: "hidden-commit",
    },
  };
  return surface;
}

function activityContent(): CreatureSurfaceContentView {
  return {
    content_key: "dream-bargain-content",
    role: "embedded_capability",
    authored_order: 0,
    label: "Dream Bargain",
    blocks: [
      {
        block_type: "paragraph",
        spans: [{ span_type: "text", text: "The envoy offers a bargain." }],
      },
      {
        block_type: "paragraph",
        spans: [
          {
            span_type: "check",
            display: "Will DC 28",
            statistic: "will",
            difficulty_class: 28,
          },
        ],
      },
      { block_type: "divider" },
    ],
    content_hash: "dream-bargain",
    visibility: "public",
    provenance: {
      source_record_key: "concept:f1-record",
      relative_source_path: "fixture.json",
      field_family: "fixture.activity",
    },
  };
}
