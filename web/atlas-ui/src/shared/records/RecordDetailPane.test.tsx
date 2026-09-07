import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type React from "react";
import type {
  RecordDetailView,
  SpellResolvedDefinitionView,
} from "../../generated/atlas";

type AvailableValue<T> = T extends { state: "available"; value: infer V } ? V : never;
type ResolvedSpellField = Exclude<
  keyof SpellResolvedDefinitionView,
  "applied_fixed_ranks" | "ritual"
>;
type SpellDefinitionFixture = {
  [K in ResolvedSpellField]: AvailableValue<SpellResolvedDefinitionView[K]>;
} & Pick<SpellResolvedDefinitionView, "ritual">;
import { getRecordDetail } from "../../api/atlasApi";
import { recordDetailFixture } from "../../test/recordFixtures";
import { RecordDetailPane } from "./RecordDetailPane";

vi.mock("../../api/atlasApi", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../api/atlasApi")>()),
  getRecordDetail: vi.fn(),
}));

describe("RecordDetailPane", () => {
  beforeEach(() => vi.resetAllMocks());

  it("renders the typed creature surface inside the detail panel", () => {
    const { container } = renderWithClient(
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
    renderWithClient(
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
    renderWithClient(
      <RecordDetailPane
        detail={undefined}
        emptyMessage="This saved record is unresolved."
        errors={[new Error("Unable to load detail")]}
        loading={false}
        onReference={vi.fn()}
      />,
    );

    expect(
      screen.queryByText("This saved record is unresolved."),
    ).not.toBeInTheDocument();
    expect(screen.getByText("Unable to load detail")).toBeInTheDocument();
  });

  it("keeps stale content visible while announcing refresh", () => {
    renderWithClient(
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

  it("uses a refreshed same-key parent spell when no form selection is active", () => {
    const client = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const initial = rimeDetail();
    const refreshed = rimeDetail(3, "3d4", [], 20);
    if (refreshed.surface.presentation.presentation_type !== "spell") {
      throw new Error("spell fixture");
    }
    refreshed.surface.presentation.body.forms[0]!.label = "Refreshed base spell";
    refreshed.surface.presentation.body.forms[0]!.minimum_cast_rank = 3;
    const { rerender } = render(
      <QueryClientProvider client={client}>
        <RecordDetailPane detail={initial} loading={false} onReference={vi.fn()} />
      </QueryClientProvider>,
    );

    expect(screen.getByText("2d4 Cold")).toBeInTheDocument();
    expect(screen.getByText("15-foot burst")).toBeInTheDocument();
    rerender(
      <QueryClientProvider client={client}>
        <RecordDetailPane detail={refreshed} loading={false} onReference={vi.fn()} />
      </QueryClientProvider>,
    );

    expect(screen.getByText("3d4 Cold")).toBeInTheDocument();
    expect(screen.getByText("20-foot burst")).toBeInTheDocument();
    expect(screen.getByText("Applied rank 3")).toBeInTheDocument();
    expect(screen.getByRole("spinbutton", { name: "Cast rank" })).toHaveValue("3");
    expect(screen.queryByText("2d4 Cold")).not.toBeInTheDocument();
    expect(screen.queryByText("15-foot burst")).not.toBeInTheDocument();
  });

  it("requests explicit backlinks only when the common record disclosure opens", async () => {
    vi.mocked(getRecordDetail).mockResolvedValue(hazardReferenceDetail(true));
    renderWithClient(
      <RecordDetailPane
        detail={hazardReferenceDetail(false)}
        loading={false}
        onReference={vi.fn()}
      />,
    );

    expect(getRecordDetail).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "References" }));

    await waitFor(() =>
      expect(getRecordDetail).toHaveBeenCalledWith(
        "hazards:hidden-pit",
        {
          reference_outgoing_limit: 8,
          reference_backlink_limit: 8,
        },
        expect.any(AbortSignal),
      ),
    );
    expect(
      await screen.findByRole("button", { name: "Backlink Hazard" }),
    ).toBeInTheDocument();
  });

  it.each(["outgoing", "backlinks"] as const)(
    "rejects a reference response with a mismatched %s limit",
    async (direction) => {
      const response = hazardReferenceDetail(true);
      const section = response.surface.references![direction];
      if (section.state !== "available") throw new Error("available references");
      section.requested_limit = 16;
      vi.mocked(getRecordDetail).mockResolvedValue(response);
      const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
      render(
        <QueryClientProvider client={client}>
          <RecordDetailPane
            detail={hazardReferenceDetail(false)}
            loading={false}
            onReference={vi.fn()}
          />
        </QueryClientProvider>,
      );
      fireEvent.click(screen.getByRole("button", { name: "References" }));
      await act(async () => {
        await client.refetchQueries({
          queryKey: ["record-detail", "hazards:hidden-pit", null, null, 8, 8],
          exact: true,
        });
      });
      await waitFor(() => expect(client.isFetching()).toBe(0));
      expect(
        screen.queryByRole("button", { name: "Backlink Hazard" }),
      ).not.toBeInTheDocument();
    },
  );

  it("keeps the selected spell form and rank while loading common references", async () => {
    vi.mocked(getRecordDetail).mockImplementation((_recordKey, request) => {
      if (request?.reference_backlink_limit === 8) {
        return Promise.resolve(spellReferenceDetail());
      }
      if (request?.spell_cast_rank === 5) {
        return Promise.resolve(rimeDetail(5, "8d4", [5], 30));
      }
      throw new Error("unexpected record detail request");
    });
    renderWithClient(
      <RecordDetailPane detail={rimeDetail()} loading={false} onReference={vi.fn()} />,
    );

    expect(
      screen.queryByRole("combobox", { name: "Spell form" }),
    ).not.toBeInTheDocument();
    fireEvent.change(screen.getByRole("spinbutton", { name: "Cast rank" }), {
      target: { value: "5" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Apply" }));
    expect(await screen.findByText("8d4 Cold")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "References" }));
    expect(
      await screen.findByRole("button", { name: "Backlink Caster" }),
    ).toBeInTheDocument();
    expect(screen.getByText("8d4 Cold")).toBeInTheDocument();
    expect(screen.getByText("Applied rank 5")).toBeInTheDocument();
    expect(getRecordDetail).toHaveBeenCalledWith(
      "spells-srd:rime-slick",
      {
        spell_form_id: "opaque:rime:base",
        spell_cast_rank: 5,
      },
      expect.any(AbortSignal),
    );
    expect(getRecordDetail).toHaveBeenCalledWith(
      "spells-srd:rime-slick",
      {
        reference_outgoing_limit: 8,
        reference_backlink_limit: 8,
      },
      expect.any(AbortSignal),
    );
  });

  it("does not attach a late reference supplement after switching records", async () => {
    const pending = new Map<string, (detail: RecordDetailView) => void>();
    let referenceSignal: AbortSignal | undefined;
    vi.mocked(getRecordDetail).mockImplementation((recordKey, request, signal) => {
      referenceSignal = signal;
      if (request?.reference_backlink_limit !== 8) {
        throw new Error("expected a reference request");
      }
      return new Promise((resolve) => pending.set(recordKey, resolve));
    });
    const first = hazardReferenceDetail(false);
    const second = hazardReferenceDetail(false);
    second.surface.metadata.record_key = "hazards:dragon-pillar";
    second.surface.metadata.title = "Dragon Pillar";
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    const { rerender } = render(
      <QueryClientProvider client={client}>
        <RecordDetailPane detail={first} loading={false} onReference={vi.fn()} />
      </QueryClientProvider>,
    );

    fireEvent.click(screen.getByRole("button", { name: "References" }));
    await waitFor(() => expect(pending.has("hazards:hidden-pit")).toBe(true));
    rerender(
      <QueryClientProvider client={client}>
        <RecordDetailPane detail={second} loading={false} onReference={vi.fn()} />
      </QueryClientProvider>,
    );
    expect(referenceSignal?.aborted).toBe(true);
    pending.get("hazards:hidden-pit")!(hazardReferenceDetail(true));

    await waitFor(() =>
      expect(
        screen.getByRole("heading", { name: "Dragon Pillar" }),
      ).toBeInTheDocument(),
    );
    expect(screen.queryByRole("button", { name: "Backlink Hazard" })).toBeNull();
    const disclosure = screen.getByRole("button", { name: "References" });
    fireEvent.click(disclosure);
    fireEvent.click(disclosure);
    await waitFor(() => expect(pending.has("hazards:dragon-pillar")).toBe(true));
    expect(getRecordDetail).toHaveBeenCalledTimes(2);
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

    expect(
      screen.queryByRole("combobox", { name: "Spell form" }),
    ).not.toBeInTheDocument();
    const rankInput = screen.getByRole("spinbutton", { name: "Cast rank" });
    fireEvent.change(rankInput, { target: { value: "5" } });
    expect(rankInput).toHaveValue("5");
    const resolveButton = screen.getByRole("button", { name: "Apply" });
    expect(resolveButton).toBeEnabled();
    fireEvent.click(resolveButton);
    await waitFor(() => expect(getRecordDetail).toHaveBeenCalled());
    expect(pending.has(5)).toBe(true);
    expect(screen.getByLabelText("Resolve spell form")).toHaveAttribute(
      "aria-busy",
      "true",
    );
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    expect(screen.getByRole("spinbutton", { name: "Cast rank" })).toBe(rankInput);
    expect(screen.getByText("2d4 Cold")).toBeInTheDocument();
    expect(screen.getByText(/Resolving Base spell at rank 5/)).toBeInTheDocument();

    fireEvent.change(rankInput, { target: { value: "8" } });
    fireEvent.click(screen.getByRole("button", { name: "Apply" }));
    await waitFor(() => expect(pending.has(8)).toBe(true));
    expect(pending.get(5)?.signal.aborted).toBe(true);

    pending.get(8)!.resolve(rimeDetail(8, "14d4", [5, 8], 60));
    expect(await screen.findByText("14d4 Cold")).toBeInTheDocument();
    expect(screen.getByRole("spinbutton", { name: "Cast rank" })).toBe(rankInput);
    expect(screen.getByRole("button", { name: "Apply" })).toBe(resolveButton);
    expect(screen.getByLabelText("Resolve spell form")).toHaveAttribute(
      "aria-busy",
      "false",
    );
    expect(screen.getByText("60-foot burst")).toBeInTheDocument();
    expect(screen.queryByText("8d4 Cold")).not.toBeInTheDocument();

    pending.get(5)!.resolve(rimeDetail(5, "8d4", [5], 30));
    await waitFor(() => expect(screen.getByText("14d4 Cold")).toBeInTheDocument());
    expect(screen.queryByText("8d4 Cold")).not.toBeInTheDocument();

    fireEvent.change(rankInput, { target: { value: "10" } });
    fireEvent.click(resolveButton);
    await waitFor(() => expect(pending.has(10)).toBe(true));
    fireEvent.click(screen.getByRole("button", { name: "Reset" }));
    await waitFor(() => expect(pending.has(2)).toBe(true));
    expect(pending.get(10)?.signal.aborted).toBe(true);
    expect(screen.getByText("Applied rank 8")).toBeInTheDocument();
    pending.get(2)!.resolve(rimeDetail(2));
    expect(await screen.findByText("Applied rank 2")).toBeInTheDocument();
    expect(screen.getByText("2d4 Cold")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Reset" })).not.toBeInTheDocument();
    pending.get(10)!.resolve(rimeDetail(10, "20d4", [5, 8], 90));
    await waitFor(() => expect(screen.getByText("Applied rank 2")).toBeInTheDocument());
    expect(screen.queryByText("20d4 Cold")).not.toBeInTheDocument();
  });

  it("updates Heal forms in the same main regions without exposing form identities", async () => {
    vi.mocked(getRecordDetail).mockImplementation((_recordKey, request) => {
      if (!request?.spell_form_id) throw new Error("expected a selected form");
      return Promise.resolve(healDetail(request.spell_form_id));
    });
    renderWithClient(
      <RecordDetailPane
        detail={healDetail("opaque:heal:touch")}
        loading={false}
        onReference={vi.fn()}
      />,
    );

    expect(screen.getByText("1 action")).toBeInTheDocument();
    expect(screen.getAllByText("touch").length).toBeGreaterThanOrEqual(1);
    await chooseSpellForm("Living creature");
    fireEvent.click(screen.getByRole("button", { name: "Apply" }));
    expect(await screen.findByText("2 actions")).toBeInTheDocument();
    expect(screen.getAllByText("Living creature").length).toBeGreaterThanOrEqual(1);
    for (const label of ["Spell casting", "Spell range and targets", "Spell damage"]) {
      expect(screen.getAllByLabelText(label)).toHaveLength(1);
    }

    await chooseSpellForm("3 actions — 30-foot emanation");
    fireEvent.click(screen.getByRole("button", { name: "Apply" }));
    expect(await screen.findByText("3 actions")).toBeInTheDocument();
    expect(screen.getByText("30-foot emanation")).toBeInTheDocument();
    expect(screen.queryByText("Duration", { exact: true })).not.toBeInTheDocument();
    expect(screen.queryByText(/opaque:heal/)).not.toBeInTheDocument();
    expect(screen.queryByText(/Overlay \d/)).not.toBeInTheDocument();
  });

  it("retains the last valid effective definition when a later selection fails", async () => {
    vi.mocked(getRecordDetail).mockImplementation((_recordKey, request) => {
      if (request?.spell_cast_rank === 5) {
        return Promise.resolve(rimeDetail(5, "8d4", [5], 30));
      }
      return Promise.reject(new Error("Selected rank could not be resolved"));
    });
    renderWithClient(
      <RecordDetailPane detail={rimeDetail()} loading={false} onReference={vi.fn()} />,
    );

    let rankInput = await selectBaseForm();
    fireEvent.change(rankInput, { target: { value: "5" } });
    fireEvent.click(screen.getByRole("button", { name: "Apply" }));
    expect(await screen.findByText("8d4 Cold")).toBeInTheDocument();

    expect(screen.getByRole("spinbutton", { name: "Cast rank" })).toBe(rankInput);
    rankInput = screen.getByRole("spinbutton", { name: "Cast rank" });
    fireEvent.change(rankInput, { target: { value: "8" } });
    fireEvent.click(screen.getByRole("button", { name: "Apply" }));
    expect(
      await screen.findByText(/Unable to resolve the selected form/),
    ).toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveTextContent(
      "Selected rank could not be resolved",
    );
    expect(screen.getByRole("status")).toHaveTextContent("Applied rank 5");
    expect(screen.getByRole("spinbutton", { name: "Cast rank" })).toBe(rankInput);
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    expect(screen.getByText("8d4 Cold")).toBeInTheDocument();
    expect(screen.queryByText("14d4 Cold")).not.toBeInTheDocument();
  });

  it("retains the last valid definition when the selected form is wholly unavailable", async () => {
    vi.mocked(getRecordDetail).mockImplementation((_recordKey, request) => {
      if (request?.spell_cast_rank === 5) {
        return Promise.resolve(rimeDetail(5, "8d4", [5], 30));
      }
      const unavailable = rimeDetail(8, "14d4", [5, 8], 60);
      if (unavailable.surface.presentation.presentation_type !== "spell") {
        throw new Error("spell fixture");
      }
      unavailable.surface.presentation.body.effective_form.result = {
        state: "unavailable",
        reason: {
          reason: "cast_rank_below_base",
          base_rank: 9,
          cast_rank: 8,
        },
      };
      unavailable.surface.issues = [
        {
          code: "unavailable",
          placement: "forms",
          message: "Spell form is unavailable.",
        },
      ];
      return Promise.resolve(unavailable);
    });
    renderWithClient(
      <RecordDetailPane detail={rimeDetail()} loading={false} onReference={vi.fn()} />,
    );

    let rankInput = await selectBaseForm();
    fireEvent.change(rankInput, { target: { value: "5" } });
    fireEvent.click(screen.getByRole("button", { name: "Apply" }));
    expect(await screen.findByText("8d4 Cold")).toBeInTheDocument();

    rankInput = screen.getByRole("spinbutton", { name: "Cast rank" });
    fireEvent.change(rankInput, { target: { value: "8" } });
    fireEvent.click(screen.getByRole("button", { name: "Apply" }));
    expect(
      await screen.findByText("Base spell at rank 8 is unavailable."),
    ).toBeInTheDocument();
    expect(screen.getAllByText("Spell form is unavailable.")).toHaveLength(1);
    expect(screen.getByText("8d4 Cold")).toBeInTheDocument();
    expect(screen.queryByText("14d4 Cold")).not.toBeInTheDocument();
    expect(screen.queryByText("Cast Rank Below Base")).not.toBeInTheDocument();
  });
});

async function selectBaseForm() {
  expect(
    screen.queryByRole("combobox", { name: "Spell form" }),
  ).not.toBeInTheDocument();
  return screen.getByRole("spinbutton", { name: "Cast rank" });
}

async function chooseSpellForm(name: string) {
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Spell form" }));
  await screen.findByRole("listbox");
  fireEvent.click(screen.getByRole("option", { name }));
}

function renderWithClient(ui: React.ReactElement) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(<QueryClientProvider client={client}>{ui}</QueryClientProvider>);
}

function hazardReferenceDetail(withBacklink: boolean): RecordDetailView {
  return {
    surface: {
      metadata: {
        record_key: "hazards:hidden-pit",
        title: "Hidden Pit",
        kind: "hazard",
        kind_label: "Hazard",
        source: {
          pack_label: "Hazards",
          document_type: "Actor",
          record_type: "hazard",
        },
      },
      profile: "record_detail",
      presentation: {
        presentation_type: "hazard",
        body: {
          provenance: {
            source_path: "packs/hazards/hidden-pit.json",
            source_contract_version: "v1",
            source_system_version: "7",
            source_upstream_commit: "fixture",
            convenience_rule_id: "pf2e-hazard-conveniences",
            convenience_rule_version: 1,
            image: { state: "missing" },
            publication_license: { state: "missing" },
            source_metadata: [],
          },
        },
      },
      references: {
        outgoing: {
          state: "available",
          requested_limit: 8,
          records: [],
          edges: [],
          total_records: 0,
          total_edges: 0,
          truncated: false,
        },
        backlinks: withBacklink
          ? {
              state: "available",
              requested_limit: 8,
              records: [
                {
                  record_key: "hazards:backlink",
                  title: "Backlink Hazard",
                  kind: "hazard",
                },
              ],
              edges: [
                {
                  from_record_key: "hazards:backlink",
                  to_record_key: "hazards:hidden-pit",
                  reference_text: "Hidden Pit",
                  source: {
                    kind: "rich_content",
                    visibility: "public",
                    relation_kind: "references",
                  },
                },
              ],
              total_records: 1,
              total_edges: 1,
              truncated: false,
            }
          : { state: "not_requested" },
      },
    },
  };
}

function spellReferenceDetail(): RecordDetailView {
  const detail = rimeDetail(5, "8d4", [5], 30);
  detail.surface.references = {
    outgoing: {
      state: "available",
      requested_limit: 8,
      records: [],
      edges: [],
      total_records: 0,
      total_edges: 0,
      truncated: false,
    },
    backlinks: {
      state: "available",
      requested_limit: 8,
      records: [
        {
          record_key: "creatures:backlink-caster",
          title: "Backlink Caster",
          kind: "creature",
        },
      ],
      edges: [
        {
          from_record_key: "creatures:backlink-caster",
          to_record_key: "spells-srd:rime-slick",
          reference_text: "Rime Slick",
          source: {
            kind: "rich_content",
            visibility: "public",
            relation_kind: "references",
          },
        },
      ],
      total_records: 1,
      total_edges: 1,
      truncated: false,
    },
  };
  return detail;
}

function healDetail(selectedFormId: string): RecordDetailView {
  const labels = new Map([
    ["opaque:heal:touch", "1 action — touch"],
    ["opaque:heal:living", "Living creature"],
    ["opaque:heal:undead", "Undead creature"],
    ["opaque:heal:emanation", "3 actions — 30-foot emanation"],
  ]);
  return {
    surface: {
      metadata: {
        record_key: "spells-srd:heal",
        title: "Heal",
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
          family: "spell",
          forms: Array.from(labels, ([id, label], order) => ({
            id,
            label,
            order,
            minimum_cast_rank: 1,
            kind: "overlay" as const,
          })),
          effective_form: {
            id: selectedFormId,
            cast_rank: 1,
            result: {
              state: "available",
              definition: resolvedHealDefinition(selectedFormId),
            },
          },
        },
      },
    },
  };
}

function resolvedHealDefinition(selectedFormId: string): SpellResolvedDefinitionView {
  const emanation = selectedFormId === "opaque:heal:emanation";
  const touch = selectedFormId === "opaque:heal:touch";
  const living = selectedFormId === "opaque:heal:living";
  const undead = selectedFormId === "opaque:heal:undead";
  const definition = rimeDefinition();
  definition.classification = known({
    rank: known(1),
    traits: known(["concentrate", "healing", "manipulate", "vitality"]),
    traditions: known(["divine", "primal"]),
  });
  definition.casting = known({
    time: known(touch ? "1 action" : emanation ? "3 actions" : "2 actions"),
    cost: missing,
    requirements: missing,
    counteraction: known(false),
  });
  definition.targeting = known({
    target: known(
      touch
        ? "touch"
        : living
          ? "Living creature"
          : undead
            ? "Undead creature"
            : "Living and undead creatures",
    ),
    range: touch
      ? known({ authored_text: "touch" })
      : emanation
        ? missing
        : known({ authored_text: "30 feet" }),
    area: emanation
      ? known({
          value: known(30),
          area_type: known("emanation"),
          legacy_area_type: missing,
          details: missing,
        })
      : missing,
  });
  definition.defense = undead
    ? known({
        passive: missing,
        save: known({ statistic: known("fortitude"), basic: known(true) }),
      })
    : missing;
  definition.damage = known([
    {
      label: living || touch || emanation ? "Healing" : "Vitality damage",
      formula: known("1d8+8"),
      damage_type: known("vitality"),
      category: missing,
      kinds: known(living || touch || emanation ? ["healing"] : ["damage"]),
      materials: known([]),
      apply_modifier: known(false),
    },
  ]);
  definition.duration = missing;
  definition.heightening = known({
    kind: "interval",
    interval: known(1),
    area: missing,
    damage: known([{ label: "Healing", value: "1d8" }]),
  });
  return resolvedFixtureDefinition(definition);
}

const missing = { state: "missing" as const };
const known = <T,>(value: T) => ({ state: "known" as const, value });

function rimeDetail(
  selectedRank?: number,
  formula = "2d4",
  appliedRanks: number[] = [],
  area = 15,
): RecordDetailView {
  const definition = rimeDefinition();
  const result = {
    state: "available" as const,
    definition: resolvedRimeDefinition(definition, formula, appliedRanks, area),
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
          family: "spell",
          forms: [
            {
              id: "opaque:rime:base",
              label: "Base spell",
              order: 0,
              minimum_cast_rank: 2,
              kind: "base",
            },
          ],
          effective_form: {
            id: "opaque:rime:base",
            cast_rank: selectedRank ?? 2,
            result,
          },
        },
      },
    },
  };
}

function rimeDefinition(): SpellDefinitionFixture {
  return {
    classification: known({
      rank: known(2),
      traits: known(["cold", "concentrate", "manipulate"]),
      traditions: known(["arcane", "primal"]),
    }),
    casting: missing,
    targeting: known({
      target: missing,
      range: known({ authored_text: "60 feet" }),
      area: known({
        value: known(15),
        area_type: known("burst"),
        legacy_area_type: missing,
        details: missing,
      }),
    }),
    defense: missing,
    damage: known([
      {
        label: "Damage",
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
}

function resolvedRimeDefinition(
  definition: ReturnType<typeof rimeDefinition>,
  formula: string,
  appliedRanks: number[],
  area: number,
): SpellResolvedDefinitionView {
  if (
    definition.targeting.state === "known" &&
    definition.targeting.value.area.state === "known"
  ) {
    const areaValue = definition.targeting.value.area.value.value;
    if (areaValue.state === "known") areaValue.value = area;
  }
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
          label: "Damage",
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
    ritual: definition.ritual,
    rules: { state: "available", value: definition.rules },
  };
}

function resolvedFixtureDefinition(
  definition: SpellDefinitionFixture,
): SpellResolvedDefinitionView {
  return {
    applied_fixed_ranks: [],
    classification: { state: "available", value: definition.classification },
    casting: { state: "available", value: definition.casting },
    targeting: { state: "available", value: definition.targeting },
    defense: { state: "available", value: definition.defense },
    damage: { state: "available", value: definition.damage },
    duration: { state: "available", value: definition.duration },
    heightening: { state: "available", value: definition.heightening },
    ritual: definition.ritual,
    rules: { state: "available", value: definition.rules },
  };
}
