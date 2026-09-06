import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type {
  RecordDetailView,
  SpellDefinitionSurfaceView,
  SpellResolvedDefinitionView,
} from "../../generated/atlas";
import { getRecordDetail } from "../../api/atlasApi";
import { recordDetailFixture } from "../../test/recordFixtures";
import { RecordDetailPane } from "./RecordDetailPane";

vi.mock("../../api/atlasApi", () => ({ getRecordDetail: vi.fn() }));

describe("RecordDetailPane", () => {
  it("renders the typed creature surface inside the detail panel", () => {
    const { container } = render(
      <RecordDetailPane
        detail={recordDetailFixture({ title: "Dirge of Doom" })}
        loading={false}
        onReference={vi.fn()}
      />,
    );

    expect(container.querySelector(".detail-panel")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Dirge of Doom" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Overview" })).toBeInTheDocument();
  });

  it("renders an accessible loading state", () => {
    render(
      <RecordDetailPane
        detail={undefined}
        loading
        loadingMessage="Loading creature"
        onReference={vi.fn()}
      />,
    );

    expect(
      screen.getByLabelText("Loading creature").closest(".detail-panel"),
    ).toHaveAttribute("aria-busy", "true");
  });

  it("renders custom empty and error states", () => {
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

  it("keeps stale content visible while announcing refresh", () => {
    render(
      <RecordDetailPane
        detail={recordDetailFixture()}
        loading={false}
        onReference={vi.fn()}
        stale
      />,
    );

    expect(screen.getByRole("heading", { name: "Goblin Warrior" })).toBeInTheDocument();
    expect(screen.getByText("Refreshing this record…")).toBeInTheDocument();
  });

  it("renders only the latest opaque form and rank response", async () => {
    const pending = new Map<
      number,
      { resolve: (detail: RecordDetailView) => void; signal: AbortSignal }
    >();
    vi.mocked(getRecordDetail).mockImplementation((_recordKey, request, signal) => {
      if (request?.spell_cast_rank === undefined || !signal) {
        throw new Error("expected a complete selected spell request");
      }
      const rank = request.spell_cast_rank;
      return new Promise((resolve) => {
        pending.set(rank, { resolve, signal: signal! });
      });
    });

    const client = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    render(
      <QueryClientProvider client={client}>
        <RecordDetailPane detail={rimeDetail()} loading={false} onReference={vi.fn()} />
      </QueryClientProvider>,
    );

    fireEvent.mouseDown(screen.getByRole("combobox", { name: "Spell form" }));
    await screen.findByRole("listbox");
    const formOption = document.querySelector(".ant-select-item-option-content");
    expect(formOption).not.toBeNull();
    fireEvent.click(formOption!);
    const rankInput = screen.getByRole("spinbutton", { name: "Cast rank" });
    fireEvent.change(rankInput, { target: { value: "5" } });
    expect(rankInput).toHaveValue("5");
    const resolveButton = screen.getByRole("button", { name: "Resolve form" });
    expect(resolveButton).toBeEnabled();
    fireEvent.click(resolveButton);
    await waitFor(() => expect(getRecordDetail).toHaveBeenCalled());
    expect(pending.has(5)).toBe(true);

    fireEvent.change(rankInput, { target: { value: "8" } });
    fireEvent.click(screen.getByRole("button", { name: "Resolve form" }));
    await waitFor(() => expect(pending.has(8)).toBe(true));
    expect(pending.get(5)?.signal.aborted).toBe(true);

    pending.get(8)!.resolve(rimeDetail(8, "6d4", [5, 8]));
    expect(await screen.findByText("Selected form at 8th rank")).toBeInTheDocument();
    expect(screen.getByText("6d4")).toBeInTheDocument();

    pending.get(5)!.resolve(rimeDetail(5, "4d4", [5]));
    await waitFor(() => expect(screen.getByText("6d4")).toBeInTheDocument());
    expect(screen.queryByText("Selected form at 5th rank")).not.toBeInTheDocument();
  });
});

const missing = { state: "missing" as const };
const known = <T,>(value: T) => ({ state: "known" as const, value });

function rimeDetail(
  selectedRank?: number,
  formula = "2d4",
  appliedRanks: number[] = [],
): RecordDetailView {
  const definition: SpellDefinitionSurfaceView = {
    classification: known({
      rank: known(2),
      traits: known(["cold"]),
      traditions: known(["arcane"]),
    }),
    casting: missing,
    targeting: known({
      target: missing,
      range: known({ authored_text: "120 feet" }),
      area: missing,
    }),
    defense: missing,
    damage: known([
      {
        key: "cold",
        order: 0,
        formula: known("2d4"),
        damage_type: known("cold"),
        category: missing,
        kinds: known(["damage"]),
        materials: known([]),
        apply_modifier: known(false),
      },
    ]),
    duration: missing,
    heightening: missing,
    ritual: missing,
    rules: missing,
  };
  const result = {
    state: "available" as const,
    definition: resolvedRimeDefinition(definition, formula, appliedRanks),
  };
  return {
    surface: {
      metadata: {
        record_key: "spells-srd:rime-slick",
        title: "Rime Slick",
        kind: "spell",
        kind_label: "Spell",
        source: {
          pack_label: "Spells",
          document_type: "Item",
          record_type: "spell",
        },
      },
      profile: "record_detail",
      presentation: {
        presentation_type: "spell",
        body: {
          definition,
          forms: [
            {
              id: "opaque:rime:base",
              label: "Base spell",
              order: 0,
              cast_rank: 2,
              kind: "base",
              result,
            },
          ],
          ...(selectedRank === undefined
            ? {}
            : {
                selected_form: {
                  id: "opaque:rime:base",
                  cast_rank: selectedRank,
                  result,
                },
              }),
        },
      },
    },
  };
}

function resolvedRimeDefinition(
  definition: SpellDefinitionSurfaceView,
  formula: string,
  appliedRanks: number[],
): SpellResolvedDefinitionView {
  return {
    applied_fixed_ranks: appliedRanks,
    classification: { state: "available", value: definition.classification },
    casting: { state: "available", value: definition.casting },
    targeting: { state: "available", value: definition.targeting },
    defense: { state: "available", value: definition.defense },
    damage: {
      state: "available",
      value: known([
        {
          key: "cold",
          order: 0,
          formula: known(formula),
          damage_type: known("cold"),
          category: missing,
          kinds: known(["damage"]),
          materials: known([]),
          apply_modifier: known(false),
        },
      ]),
    },
    duration: { state: "available", value: definition.duration },
    heightening: { state: "available", value: definition.heightening },
    rules: { state: "available", value: definition.rules },
  };
}
