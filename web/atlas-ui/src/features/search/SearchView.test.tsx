import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { RecordDetailRequest, RecordDetailView } from "../../generated/atlas";
import { AtlasApiError, getRecordDetail } from "../../api/atlasApi";
import { SearchView } from "./SearchView";
import { useSearchWorkspace } from "./useSearchWorkspace";

vi.mock("../../api/atlasApi", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../api/atlasApi")>()),
  getRecordDetail: vi.fn(),
  openResultWindow: vi.fn().mockResolvedValue({
    window_id: 1,
    rows: [],
    page: { number: 1, size: 25, total: 0n, has_more: false },
  }),
  getReadiness: vi.fn().mockResolvedValue({ status: "ready", message: "Ready" }),
  discoverFilterEditor: vi
    .fn()
    .mockResolvedValue({ matching_record_count: 0, groups: [] }),
  discoverFilterValues: vi
    .fn()
    .mockResolvedValue({ matching_record_count: 0, options: [] }),
}));

function Harness() {
  const workspace = useSearchWorkspace();
  return (
    <>
      <button onClick={() => workspace.selectRecord("spells:other")}>
        Other record
      </button>
      <SearchView workspace={workspace} />
    </>
  );
}

function fixture(key = "spells:heal", id = "base", rank = 1): RecordDetailView {
  const missing = { state: "missing" as const };
  const absent = { state: "available" as const, value: missing };
  return {
    surface: {
      profile: "record_detail",
      metadata: {
        record_key: key,
        title: key === "spells:heal" ? "Heal" : "Other spell",
        kind: "spell",
        kind_label: "Spell",
      },
      presentation: {
        presentation_type: "spell",
        body: {
          family: "spell",
          forms: [
            { id: "base", label: "Base", kind: "base", order: 0, minimum_cast_rank: 1 },
            {
              id: "living",
              label: "Living",
              kind: "overlay",
              order: 1,
              minimum_cast_rank: 1,
            },
          ],
          effective_form: {
            id,
            cast_rank: rank,
            result: {
              state: "available",
              definition: {
                applied_fixed_ranks: [],
                classification: absent,
                casting: absent,
                targeting: absent,
                defense: absent,
                damage: {
                  state: "available",
                  value: {
                    state: "known",
                    value: [
                      {
                        label: "Healing",
                        formula: { state: "known", value: `${rank}d8` },
                        damage_type: missing,
                        category: missing,
                        kinds: missing,
                        materials: missing,
                        apply_modifier: missing,
                      },
                    ],
                  },
                },
                duration: absent,
                heightening: absent,
                rules: absent,
                ritual: missing,
              },
            },
          },
        },
      },
    },
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

function mount() {
  history.replaceState(null, "", "/search/records/spells%3Aheal");
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const view = render(
    <QueryClientProvider client={client}>
      <Harness />
    </QueryClientProvider>,
  );
  return { client, ...view };
}

beforeEach(() => {
  vi.mocked(getRecordDetail).mockReset();
  vi.mocked(getRecordDetail).mockImplementation(async (key, request) =>
    fixture(key, request?.spell_form_id, request?.spell_cast_rank),
  );
});

it("keeps the actual SearchView selection mounted through unresolved and completed parent refetch", async () => {
  const { client } = mount();
  await screen.findByRole("heading", { name: "Heal" });
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Spell form" }));
  fireEvent.click(await screen.findByRole("option", { name: "Living" }));
  fireEvent.change(screen.getByRole("spinbutton", { name: "Cast rank" }), {
    target: { value: "3" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  await screen.findByText("3d8");
  const pending = deferred<RecordDetailView>();
  let signal: AbortSignal | undefined;
  vi.mocked(getRecordDetail).mockImplementationOnce((_key, _request, requestSignal) => {
    signal = requestSignal;
    return pending.promise;
  });
  let refresh!: Promise<void>;
  act(() => {
    refresh = client.refetchQueries({
      queryKey: ["record-detail", "spells:heal"],
      exact: true,
    });
  });
  await screen.findByText("Refreshing this record…");
  expect(signal).toBeInstanceOf(AbortSignal);
  expect(signal?.aborted).toBe(false);
  expect(screen.getByText("3d8")).toBeInTheDocument();
  expect(screen.queryByLabelText("Loading record")).not.toBeInTheDocument();
  await act(async () => {
    pending.resolve(fixture());
    await refresh;
  });
  expect(screen.getByText("3d8")).toBeInTheDocument();
  const calls = vi.mocked(getRecordDetail).mock.calls;
  expect(
    calls.some(
      ([key, request, sentSignal]) =>
        key === "spells:heal" &&
        request?.spell_form_id === "living" &&
        request.spell_cast_rank === 3 &&
        sentSignal instanceof AbortSignal,
    ),
  ).toBe(true);
});

it("aborts an unresolved prior record and ignores its late response after a record switch", async () => {
  const pending = deferred<RecordDetailView>();
  let oldSignal: AbortSignal | undefined;
  vi.mocked(getRecordDetail).mockImplementationOnce((_key, _request, signal) => {
    oldSignal = signal;
    return pending.promise;
  });
  mount();
  await screen.findByLabelText("Loading record");
  await waitFor(() => expect(oldSignal).toBeInstanceOf(AbortSignal));
  fireEvent.click(screen.getByRole("button", { name: "Other record" }));
  await screen.findByRole("heading", { name: "Other spell" });
  expect(oldSignal?.aborted).toBe(true);
  await act(async () => {
    pending.resolve(fixture());
  });
  expect(screen.queryByRole("heading", { name: "Heal" })).not.toBeInTheDocument();
  expect(screen.getByText("1d8")).toBeInTheDocument();
});

it("isolates selected request keys and keeps the latest matching response", async () => {
  const prior = deferred<RecordDetailView>();
  let oldSignal: AbortSignal | undefined;
  vi.mocked(getRecordDetail).mockImplementation(
    (key, request?: RecordDetailRequest, signal?: AbortSignal) => {
      if (request?.spell_cast_rank === 3) {
        oldSignal = signal;
        return prior.promise;
      }
      return Promise.resolve(
        fixture(key, request?.spell_form_id, request?.spell_cast_rank),
      );
    },
  );
  mount();
  await screen.findByRole("heading", { name: "Heal" });
  fireEvent.change(screen.getByRole("spinbutton", { name: "Cast rank" }), {
    target: { value: "3" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  await waitFor(() => expect(oldSignal).toBeInstanceOf(AbortSignal));
  fireEvent.change(screen.getByRole("spinbutton", { name: "Cast rank" }), {
    target: { value: "5" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  await screen.findByText("5d8");
  expect(oldSignal?.aborted).toBe(true);
  await act(async () => {
    prior.resolve(fixture("spells:heal", "base", 3));
  });
  expect(screen.getByText("5d8")).toBeInTheDocument();
  expect(screen.queryByText("3d8")).not.toBeInTheDocument();
});

it("retains the last valid selected response after same-key refresh becomes unavailable", async () => {
  const { client } = mount();
  await screen.findByRole("heading", { name: "Heal" });
  fireEvent.change(screen.getByRole("spinbutton", { name: "Cast rank" }), {
    target: { value: "3" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  await screen.findByText("3d8");
  const queryKey = ["record-detail", "spells:heal", "base", 3, null, null];
  const updated = fixture("spells:heal", "base", 3);
  if (
    updated.surface.presentation.presentation_type !== "spell" ||
    updated.surface.presentation.body.effective_form.result.state !== "available"
  )
    throw new Error("spell fixture");
  const missing = { state: "missing" as const };
  updated.surface.presentation.body.effective_form.result.definition.damage = {
    state: "available",
    value: {
      state: "known",
      value: [
        {
          label: "Healing",
          formula: { state: "known", value: "3d8+24" },
          damage_type: missing,
          category: missing,
          kinds: missing,
          materials: missing,
          apply_modifier: missing,
        },
      ],
    },
  };
  expect(client.getQueryState(queryKey)?.status).toBe("success");
  vi.mocked(getRecordDetail).mockResolvedValueOnce(updated);
  await act(async () => {
    await client.refetchQueries({ queryKey, exact: true });
  });
  await waitFor(() => expect(client.getQueryData(queryKey)).toEqual(updated));
  expect(await screen.findByText("3d8+24")).toBeInTheDocument();
  const unavailable = fixture("spells:heal", "base", 3);
  if (unavailable.surface.presentation.presentation_type !== "spell")
    throw new Error("spell fixture");
  unavailable.surface.presentation.body.effective_form.result = {
    state: "unavailable",
    reason: { reason: "unknown_overlay" },
  };
  vi.mocked(getRecordDetail).mockResolvedValueOnce(unavailable);
  await act(async () => {
    await client.refetchQueries({ queryKey, exact: true });
  });
  expect(await screen.findByText("3d8+24")).toBeInTheDocument();
  expect(screen.queryByText("3d8")).not.toBeInTheDocument();
  vi.mocked(getRecordDetail).mockRejectedValueOnce(new Error("Refresh failed"));
  await act(async () => {
    await client.refetchQueries({ queryKey, exact: true });
  });
  expect(await screen.findByText("3d8+24")).toBeInTheDocument();
});

it("shows typed missing-record recovery once without record actions or raw identity", async () => {
  vi.mocked(getRecordDetail).mockRejectedValue(
    new AtlasApiError(404, "missing spells:heal", {
      code: "record_not_found",
      message: "missing spells:heal",
    }),
  );
  mount();
  await screen.findByText("This record could not be found");
  expect(screen.getAllByRole("button", { name: "Search" })).toHaveLength(1);
  expect(screen.getAllByRole("button", { name: "Back" })).toHaveLength(1);
  expect(
    screen.queryByRole("link", { name: "Open full page" }),
  ).not.toBeInTheDocument();
  expect(
    screen.queryByRole("button", { name: "Add to saved list" }),
  ).not.toBeInTheDocument();
  expect(screen.queryByText(/missing spells:heal/)).not.toBeInTheDocument();
});

it.each([1440, 1024, 390])(
  "owns the exact relationship in the filter pane or drawer at %ipx",
  async (width) => {
    Object.defineProperty(window, "innerWidth", { configurable: true, value: width });
    history.replaceState(
      null,
      "",
      "/search?reference-direction=incoming&reference-record=spells%3Aheal",
    );
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    render(
      <QueryClientProvider client={client}>
        <Harness />
      </QueryClientProvider>,
    );
    expect(document.querySelector(".workspace-pane--results")).not.toBeNull();
    expect(document.querySelector(".workspace-pane--detail")).toBeNull();
    if (width <= 1100) {
      expect(
        screen.queryByText("Records that reference: Heal"),
      ).not.toBeInTheDocument();
      fireEvent.click(screen.getByRole("button", { name: "Filters (1)" }));
    }
    const chip = await screen.findByText("Records that reference: Heal");
    expect(chip).toBeVisible();
    expect(chip.closest(".filter-panel")).not.toBeNull();
    expect(chip.closest(".search-workspace__toolbar")).toBeNull();
    const remove = screen.getByRole("button", {
      name: "Remove relationship filter for Heal",
    });
    expect(remove).toBeVisible();
    fireEvent.click(remove);
    await waitFor(() =>
      expect(
        screen.queryByText("Records that reference: Heal"),
      ).not.toBeInTheDocument(),
    );
    expect(location.search).not.toContain("reference-record");
  },
);

it("retains the relationship and returns focus after dismissing the filter drawer", async () => {
  Object.defineProperty(window, "innerWidth", { configurable: true, value: 1024 });
  history.replaceState(
    null,
    "",
    "/search?reference-direction=incoming&reference-record=spells%3Aheal",
  );
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={client}>
      <Harness />
    </QueryClientProvider>,
  );
  const launcher = screen.getByRole("button", { name: "Filters (1)" });
  fireEvent.click(launcher);
  await screen.findByText("Records that reference: Heal");
  const drawer = screen.getByRole("dialog").closest(".ant-drawer")!;
  fireEvent.keyDown(drawer, { key: "Escape", keyCode: 27 });
  await waitFor(() => expect(launcher).toHaveFocus());
  expect(location.search).toContain("reference-record");
});
