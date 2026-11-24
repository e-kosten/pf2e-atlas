import { atlasRouter } from "../../../app/router";
import { recordDetailQueryOptions } from "../../records/recordDetailQuery";
import { chooseAttributeSort } from "../../../test/sortControls";
import type {
  QueryPredicate,
  ResultSort,
  SearchConnectionScope,
} from "../../../generated/atlas";
import { SearchProfileProvider } from "../../../test/SearchProfileProvider";
import type { WorkspaceSearchDefault } from "../../search/SearchProfileContext";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { App, ConfigProvider, Form, Grid, Input } from "antd";
import { useState } from "react";
import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import {
  AtlasApiError,
  searchPage,
  getRecordDetail,
  discoverFilterEditor,
  discoverFilterCounts,
} from "../../../api/atlasApi";
import {
  activityFixture,
  detailFixture,
  editorFixture,
  fieldFixture,
  summaryFixture,
} from "../../../test/fixtures";
import type { SearchResultsPage } from "../../../generated/atlas";
import { SearchPickerModal } from "./SearchPickerModal";
import { retireReferenceQueries } from "../../queries/referenceQueries";

vi.mock("../../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../../api/atlasApi")>()),
  searchPage: vi.fn(),
  getRecordDetail: vi.fn(),
  discoverFilterEditor: vi.fn(),
  discoverFilterCounts: vi.fn(),
}));
const search = vi.mocked(searchPage);
const addSelected = vi.fn();
const cancelPicker = vi.fn();
function result(
  title = "Ghoul",
  generation = "generation-a",
  profile = "bge-base",
): SearchResultsPage {
  return {
    search_content: "all",
    sort: {
      kind: "attributes",
      rules: [{ field: "name", direction: "asc" }],
    } satisfies ResultSort,
    relationship_clauses: [],
    ranking: "score" as const,
    search_profile: profile,
    artifact_generation: generation,
    mode: { kind: "list_records" as const },
    coverage: null,
    page: {
      number: 1,
      size: 25,
      count: 1,
      total: 1,
      has_more: false,
      next_page: null,
    },
    rows: [
      {
        record: summaryFixture("actors:" + title.toLowerCase(), title),
        relationship_evidence: [],
        matches: [],
      },
    ],
  };
}
function Picker({
  disabled = false,
  open = true,
  initialQuery,
  connectionScope,
  fixedFilter,
}: {
  disabled?: boolean;
  open?: boolean;
  initialQuery?: string;
  connectionScope?: SearchConnectionScope;
  fixedFilter?: QueryPredicate;
}) {
  const [form] = Form.useForm();
  const [selected, setSelected] = useState<string | null>(null);
  const [preparedAdjustment, setPreparedAdjustment] = useState<string>();
  return (
    <SearchPickerModal
      title="Choose record"
      focusSearchOnOpen={initialQuery !== undefined}
      initialQuery={initialQuery}
      disabled={disabled}
      open={open}
      form={form}
      searchHeader={
        connectionScope
          ? (scope) => (
              <output aria-label="Discovery scope">
                {scope?.kind ?? "unresolved"}
              </output>
            )
          : undefined
      }
      connectionScope={connectionScope}
      fixedFilter={fixedFilter}
      emptyPrompt="Search for a record"
      emptyResults="No matching records"
      selectedRecordKey={selected}
      onSelectionChange={(selection) => {
        setSelected(selection?.record.record_key ?? null);
        setPreparedAdjustment(selection?.detail.encounter_adjustment ?? undefined);
      }}
      onCancel={cancelPicker}
      onFinish={() => addSelected(selected)}
      okButtonProps={{ disabled: !selected }}
    >
      {(detail) => (
        <>
          <output aria-label="Prepared adjustment">{preparedAdjustment}</output>
          <output aria-label="Selected adjustment">
            {detail?.encounter_adjustment}
          </output>
          <Form.Item name="note" label="Note">
            <Input />
          </Form.Item>
        </>
      )}
    </SearchPickerModal>
  );
}
function renderPicker({
  initialQuery,
  usable = true,
  connectionScope,
  fixedFilter,
  resolveWorkspaceDefault,
}: {
  initialQuery?: string;
  usable?: boolean;
  connectionScope?: SearchConnectionScope;
  fixedFilter?: QueryPredicate;
  resolveWorkspaceDefault?: () => Promise<WorkspaceSearchDefault | undefined>;
} = {}) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: 20_000 } },
  });
  let profile = "bge-base";
  let disabled = false;
  let open = true;
  const content = () => (
    <ConfigProvider theme={{ token: { motion: false } }}>
      <QueryClientProvider client={client}>
        <SearchProfileProvider
          id={profile}
          usable={usable}
          resolveWorkspaceDefault={resolveWorkspaceDefault}
        >
          <App>
            <Picker
              initialQuery={initialQuery}
              disabled={disabled}
              open={open}
              connectionScope={connectionScope}
              fixedFilter={fixedFilter}
            />
          </App>
        </SearchProfileProvider>
      </QueryClientProvider>
    </ConfigProvider>
  );
  const view = render(content());
  return {
    client,
    setConnectionScope: (next: SearchConnectionScope) => {
      connectionScope = next;
      view.rerender(content());
    },
    setDisabled: (next: boolean) => {
      disabled = next;
      view.rerender(content());
    },
    setOpen: (next: boolean) => {
      open = next;
      view.rerender(content());
    },
    setProfile: (next: string) => {
      profile = next;
      view.rerender(content());
    },
  };
}
async function submit(query: string) {
  const back = screen.queryByRole("button", { name: "Results" });
  if (back) fireEvent.click(back);
  const input = screen.getByRole("searchbox", { name: "Search records" });
  fireEvent.change(input, { target: { value: query } });
  fireEvent.keyDown(input, { key: "Enter", code: "Enter", keyCode: 13 });
}
async function expectRetainedCandidate() {
  const calls = addSelected.mock.calls.length;
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Add" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Add" }));
  await waitFor(() => expect(addSelected).toHaveBeenCalledTimes(calls + 1));
  expect(addSelected).toHaveBeenLastCalledWith("actors:ghoul");
}
beforeEach(() => {
  vi.restoreAllMocks();
  vi.clearAllMocks();
  search.mockReset();
  search.mockResolvedValue(result());
  vi.mocked(getRecordDetail).mockImplementation((key) =>
    Promise.resolve({ ...detailFixture(key), artifact_generation: "generation-a" }),
  );
  vi.mocked(discoverFilterEditor).mockResolvedValue({
    ...editorFixture(),
    groups: [],
    common_field_ids: [],
  });
  vi.mocked(discoverFilterCounts).mockResolvedValue({
    search_profile: "bge-base",
    artifact_generation: "generation-a",
    counts: { status: "unavailable", reason: "non_conjunctive_context" },
  });
});
it("keeps the root selection default when a match opens an exact embedded document", async () => {
  const page = result();
  const navigation = activityFixture("Command", "actors:ghoul").navigation;
  page.mode = { kind: "text_search", query: "command", mode: "hybrid" };
  page.rows[0].matching_sections = [{ navigation, label: "Command", preview: null }];
  page.rows[0].matches = [
    {
      navigation,
      label: "Command",
      preview: null,
      lane: "lexical",
      lexical_rank: null,
      semantic_similarity: null,
    },
  ];
  search.mockResolvedValue(page);
  const parent = {
    ...detailFixture("actors:ghoul", "Ghoul"),
    artifact_generation: "generation-a",
    encounter_adjustment: "elite" as const,
  };
  const child = {
    ...detailFixture("actors:ghoul", "Command"),
    artifact_generation: "generation-a",
    selected: navigation,
  };
  vi.mocked(getRecordDetail).mockImplementation((_key, request) =>
    Promise.resolve(request?.embedded_path?.length ? child : parent),
  );
  renderPicker();
  fireEvent.click(await screen.findByRole("link", { name: /^Ghoul/ }));
  fireEvent.click(await screen.findByRole("link", { name: /Command/ }));
  expect(await screen.findByRole("heading", { name: "Command" })).toBeVisible();
  expect(screen.getByLabelText("Selected adjustment")).toHaveTextContent("elite");
  expect(screen.getByLabelText("Prepared adjustment")).toHaveTextContent("elite");
  expect(getRecordDetail).toHaveBeenCalledWith(
    "actors:ghoul",
    expect.objectContaining({ embedded_path: navigation.embedded_path }),
    expect.any(AbortSignal),
  );
  fireEvent.click(screen.getByRole("button", { name: "Add" }));
  await waitFor(() => expect(addSelected).toHaveBeenLastCalledWith("actors:ghoul"));
});

it("inherits lazy contextual counts in Add pickers and leaves text searches count-free", async () => {
  vi.mocked(discoverFilterEditor).mockResolvedValue(
    editorFixture(
      [
        {
          ...fieldFixture("kind", "string", "Kind"),
          choices: ["creature", "spell"],
        },
      ],
      { kind: { creature: "Creature", spell: "Spell" } },
    ),
  );
  vi.mocked(discoverFilterCounts).mockResolvedValue({
    search_profile: "bge-base",
    artifact_generation: "generation-a",
    counts: {
      status: "available",
      value: {
        field: "kind",
        options: [
          { value: "creature", distinct_roots: 4, selected: false },
          { value: "spell", distinct_roots: 0, selected: false },
        ],
        total_values: 2,
        exhaustive: true,
        count_basis: "exact counterfactual displayed roots",
      },
    },
  });
  const fixedFilter: QueryPredicate = {
    kind: "in",
    field: "kind",
    values: ["creature", "hazard"],
  };
  renderPicker({ fixedFilter });
  await screen.findByRole("link", { name: /Ghoul/ });
  expect(search.mock.lastCall?.[0].criteria).toMatchObject({
    kind: "list_records",
    filter: fixedFilter,
  });
  expect(vi.mocked(discoverFilterEditor).mock.lastCall?.[0].filter).toEqual(
    fixedFilter,
  );
  expect(discoverFilterCounts).not.toHaveBeenCalled();
  fireEvent.click(await screen.findByRole("button", { name: "Edit Kind filter" }));
  expect(await screen.findByLabelText("4 matching records")).toBeInTheDocument();
  expect(
    screen.getByRole("button", { name: "Kind: Spell, neutral; activate to include" }),
  ).toBeEnabled();
  expect(discoverFilterCounts).toHaveBeenCalledWith(
    expect.objectContaining({
      context: expect.objectContaining({
        filter: fixedFilter,
        search_profile: "bge-base",
        expected_artifact_generation: "generation-a",
      }),
      values: ["creature", "spell"],
    }),
    expect.any(AbortSignal),
  );
  await submit("dragon");
  await waitFor(() =>
    expect(screen.queryByLabelText("4 matching records")).not.toBeInTheDocument(),
  );
  expect(search.mock.lastCall?.[0].criteria).toMatchObject({
    kind: "text_search",
    query: "dragon",
    filter: fixedFilter,
  });
  expect(discoverFilterCounts).toHaveBeenCalledTimes(1);
});

it("sorts through the shared request while retaining the candidate and Add draft", async () => {
  search.mockImplementation(async (request) => ({
    ...result(),
    sort:
      request.sort ??
      (request.criteria.kind === "list_records"
        ? { kind: "attributes", rules: [{ field: "name", direction: "asc" }] }
        : { kind: "relevance" }),
  }));
  renderPicker();
  fireEvent.click(await screen.findByRole("link", { name: /Ghoul/ }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Add" })).toBeEnabled(),
  );
  fireEvent.change(screen.getByLabelText("Note"), { target: { value: "Keep draft" } });
  fireEvent.click(await screen.findByRole("button", { name: "Results" }));
  await chooseAttributeSort("level", "desc");
  await waitFor(() =>
    expect(search.mock.lastCall?.[0]).toMatchObject({
      sort: {
        kind: "attributes",
        rules: [{ field: "level", direction: "desc" }],
      } satisfies ResultSort,
      page: { number: 1 },
    }),
  );
  await screen.findByRole("link", { name: /Ghoul/ });
  expect(screen.getByLabelText("Note")).toHaveValue("Keep draft");
  await expectRetainedCandidate();
  await submit("dragon");
  await waitFor(() =>
    expect(search.mock.lastCall?.[0]).toMatchObject({
      sort: {
        kind: "attributes",
        rules: [{ field: "level", direction: "desc" }],
      } satisfies ResultSort,
      criteria: { kind: "text_search", query: "dragon" },
    }),
  );
});
it("retains a labelled prior page across sorting but clears it for a different criterion", async () => {
  let finishSort!: (page: SearchResultsPage) => void;
  search.mockImplementation((request) => {
    if (request.criteria.kind === "text_search")
      return new Promise<SearchResultsPage>(() => {});
    if (
      request.sort?.kind === "attributes" &&
      request.sort.rules[0].field === "level" &&
      request.sort.rules[0].direction === "desc"
    )
      return new Promise<SearchResultsPage>((resolve) => {
        finishSort = resolve;
      });
    return Promise.resolve({
      ...result(),
      sort: request.sort ?? {
        kind: "attributes",
        rules: [{ field: "name", direction: "asc" }],
      },
    });
  });
  renderPicker();
  await screen.findByRole("link", { name: /Ghoul/ });
  await chooseAttributeSort("level", "desc");
  await waitFor(() => expect(screen.getByText("Updating results")).toBeVisible());
  expect(screen.getByRole("link", { name: /Ghoul/ })).toBeInTheDocument();
  await act(async () =>
    finishSort({
      ...result("Sorted creature"),
      sort: {
        kind: "attributes",
        rules: [{ field: "level", direction: "desc" }],
      } satisfies ResultSort,
    }),
  );
  await screen.findByRole("link", { name: /Sorted creature/ });
  await submit("different criterion");
  await waitFor(() =>
    expect(search.mock.lastCall?.[0].criteria.kind).toBe("text_search"),
  );
  expect(screen.queryByRole("link", { name: /Sorted creature/ })).toBeNull();
});

it("shows loading without pagination, then completed empty for a queryless picker", async () => {
  let finish!: (value: ReturnType<typeof result>) => void;
  search.mockReturnValue(
    new Promise((resolve) => {
      finish = resolve;
    }),
  );
  renderPicker();
  await waitFor(() =>
    expect(screen.getByRole("status", { name: "Loading records…" })).toBeVisible(),
  );
  expect(screen.queryByRole("group", { name: "Result pages" })).toBeNull();
  expect(screen.queryByText("No matching records")).toBeNull();
  await act(async () =>
    finish({ ...result(), rows: [], page: { ...result().page, count: 0, total: 0 } }),
  );
  expect(await screen.findByText("No matching records")).toBeVisible();
  expect(screen.queryByText("Search for a record")).toBeNull();
  expect(screen.queryByRole("group", { name: "Result pages" })).toBeNull();
});

it("clears the candidate from the shared toolbar while retaining the Add form draft", async () => {
  vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ lg: true });
  renderPicker();
  const row = await screen.findByRole("link", { name: /^Ghoul/ });
  fireEvent.click(row);
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Add" })).toBeEnabled(),
  );
  fireEvent.change(screen.getByLabelText("Note"), {
    target: { value: "Keep my note" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Clear selection" }));
  expect(screen.getByRole("button", { name: "Add" })).toBeDisabled();
  expect(screen.queryByRole("button", { name: "Clear selection" })).toBeNull();
  expect(screen.getByLabelText("Note")).toHaveValue("Keep my note");
  await waitFor(() => expect(row).toHaveFocus());
});

it("keeps clear selection available after returning to narrow results", async () => {
  vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ lg: false });
  renderPicker();
  fireEvent.click(await screen.findByRole("link", { name: /^Ghoul/ }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Add" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Results" }));
  fireEvent.click(screen.getByRole("button", { name: "Clear selection" }));
  expect(screen.getByRole("button", { name: "Add" })).toBeDisabled();
});

it("previews before explicit Add and retains the candidate when a later query has no results", async () => {
  renderPicker();
  const link = await screen.findByRole("link", { name: /Ghoul/ });
  expect(getRecordDetail).not.toHaveBeenCalled();
  fireEvent.click(link);
  await waitFor(() => expect(getRecordDetail).toHaveBeenCalled());
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Add" })).toBeEnabled(),
  );
  search.mockResolvedValue({
    ...result(),
    rows: [],
    page: { ...result().page, count: 0, total: 0 },
  });
  await submit("dragon");
  await waitFor(() => expect(screen.queryByRole("link", { name: /Ghoul/ })).toBeNull());
  await expectRetainedCandidate();
  expect(screen.getByRole("button", { name: "Add" })).toBeEnabled();
});

function multipleCandidates() {
  vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ lg: true });
  search.mockResolvedValue({
    ...result(),
    page: { ...result().page, count: 3, total: 3 },
    rows: [...result().rows, ...result("Ogre").rows, ...result("Dragon").rows],
  });
  vi.mocked(getRecordDetail).mockImplementation((key) =>
    Promise.resolve({
      ...detailFixture(key, key.slice(7)),
      artifact_generation: "generation-a",
    }),
  );
}
it("keeps the document and Add candidate together until a replacement is ready, preserving the draft", async () => {
  multipleCandidates();
  renderPicker();
  fireEvent.click(await screen.findByRole("link", { name: /^Ghoul/ }));
  const preview = screen.getByRole("region", { name: "Record preview" });
  await within(preview).findByRole("heading", { name: "ghoul" });
  const oldTitle = within(preview).getByRole("heading", { name: "ghoul" });
  const frames = [oldTitle.textContent?.trim() ?? "blank"];
  const observer = new MutationObserver(() =>
    frames.push(preview.querySelector("h2")?.textContent?.trim() ?? "blank"),
  );
  observer.observe(preview, { childList: true, characterData: true, subtree: true });
  let finish!: (value: ReturnType<typeof detailFixture>) => void;
  vi.mocked(getRecordDetail).mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        finish = resolve;
      }),
  );
  try {
    fireEvent.click(screen.getByRole("link", { name: /^Ogre/ }));
    expect(screen.getByRole("link", { name: /^Ghoul/ })).toHaveAttribute(
      "aria-current",
      "true",
    );
    expect(oldTitle).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Add" })).toBeDisabled();
    fireEvent.change(screen.getByLabelText("Note"), {
      target: { value: "Keep this draft" },
    });
    fireEvent.submit(screen.getByLabelText("Note").closest("form")!);
    expect(addSelected).not.toHaveBeenCalled();
    await screen.findByText("Loading Ogre…");
    expect(within(preview).getByRole("heading", { name: "ghoul" })).toBe(oldTitle);
    await act(async () =>
      finish({
        ...detailFixture("actors:ogre", "Ogre"),
        artifact_generation: "generation-a",
      }),
    );
    await within(preview).findByRole("heading", { name: "Ogre" });
    expect(screen.getByRole("link", { name: /^Ogre/ })).toHaveAttribute(
      "aria-current",
      "true",
    );
    expect(screen.getByLabelText("Note")).toHaveValue("Keep this draft");
    fireEvent.click(screen.getByRole("button", { name: "Add" }));
    await waitFor(() => expect(addSelected).toHaveBeenLastCalledWith("actors:ogre"));
    expect(frames).not.toContain("blank");
    expect(new Set(frames)).toEqual(new Set(["ghoul", "Ogre"]));
  } finally {
    observer.disconnect();
  }
});

it("keeps a failed request local with Retry and lets Cancel return to the existing candidate", async () => {
  multipleCandidates();
  renderPicker();
  fireEvent.click(await screen.findByRole("link", { name: /^Ghoul/ }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Add" })).toBeEnabled(),
  );
  vi.mocked(getRecordDetail).mockRejectedValueOnce(new Error("Disconnected"));
  fireEvent.click(screen.getByRole("link", { name: /^Ogre/ }));
  expect(await screen.findByText("Could not load Ogre")).toBeVisible();
  expect(screen.getByRole("button", { name: "Add" })).toBeDisabled();
  fireEvent.click(screen.getByRole("button", { name: "Retry loading Ogre" }));
  await screen.findByRole("heading", { name: "ogre" });
  expect(screen.getByRole("button", { name: "Add" })).toBeEnabled();
  vi.mocked(getRecordDetail).mockRejectedValueOnce(new Error("Disconnected"));
  fireEvent.click(screen.getByRole("link", { name: /^Dragon/ }));
  await screen.findByText("Could not load Dragon");
  fireEvent.click(screen.getByRole("button", { name: "Cancel selection" }));
  expect(screen.getByRole("button", { name: "Add" })).toBeEnabled();
  fireEvent.click(screen.getByRole("button", { name: "Add" }));
  await waitFor(() => expect(addSelected).toHaveBeenLastCalledWith("actors:ogre"));
});

it.each(["superseded", "closed", "disabled"] as const)(
  "ignores a late prepared selection when %s",
  async (reason) => {
    multipleCandidates();
    const controls = renderPicker();
    fireEvent.click(await screen.findByRole("link", { name: /^Ghoul/ }));
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Add" })).toBeEnabled(),
    );
    let finish!: (value: ReturnType<typeof detailFixture>) => void;
    vi.mocked(getRecordDetail).mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          finish = resolve;
        }),
    );
    fireEvent.click(screen.getByRole("link", { name: /^Ogre/ }));
    if (reason === "closed") controls.setOpen(false);
    else if (reason === "disabled") controls.setDisabled(true);
    else {
      fireEvent.click(screen.getByRole("link", { name: /^Dragon/ }));
      await screen.findByRole("heading", { name: "dragon" });
    }
    await act(async () =>
      finish({
        ...detailFixture("actors:ogre", "Ogre"),
        artifact_generation: "generation-a",
      }),
    );
    expect(screen.queryByRole("heading", { name: "Ogre" })).toBeNull();
    expect(addSelected).not.toHaveBeenCalled();
    if (reason === "superseded") {
      fireEvent.click(screen.getByRole("button", { name: "Add" }));
      await waitFor(() =>
        expect(addSelected).toHaveBeenLastCalledWith("actors:dragon"),
      );
    }
  },
);

it("rejects a mismatched source generation before caching or selecting the new document", async () => {
  multipleCandidates();
  const { client } = renderPicker();
  fireEvent.click(await screen.findByRole("link", { name: /^Ghoul/ }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Add" })).toBeEnabled(),
  );
  vi.mocked(getRecordDetail).mockResolvedValueOnce({
    ...detailFixture("actors:ogre", "Ogre"),
    artifact_generation: "generation-b",
  });
  fireEvent.click(screen.getByRole("link", { name: /^Ogre/ }));
  expect(await screen.findByText("Reference collection changed")).toBeVisible();
  expect(screen.getByRole("button", { name: "Refresh results" })).toBeEnabled();
  expect(screen.getByRole("link", { name: /^Ghoul/ })).toHaveAttribute(
    "aria-current",
    "true",
  );
  expect(screen.getByRole("button", { name: "Add" })).toBeDisabled();
  expect(
    client
      .getQueryCache()
      .findAll({ queryKey: ["record-detail"] })
      .some(
        (query) =>
          (query.state.data as ReturnType<typeof detailFixture> | undefined)
            ?.artifact_generation === "generation-b",
      ),
  ).toBe(false);
  expect(
    client.getQueryState(
      recordDetailQueryOptions("actors:ogre", undefined, "generation-a").queryKey,
    ),
  ).toMatchObject({ status: "error", data: undefined });
});

it("inherits a profile switch on page one while retaining the explicit candidate, reader and Add draft", async () => {
  const page = result();
  page.page = { ...page.page, total: 100, has_more: true };
  let finishOld!: (value: ReturnType<typeof result>) => void;
  let finishNew!: (value: ReturnType<typeof result>) => void;
  search
    .mockResolvedValueOnce(page)
    .mockReturnValueOnce(
      new Promise((resolve) => {
        finishOld = resolve;
      }),
    )
    .mockReturnValueOnce(
      new Promise((resolve) => {
        finishNew = resolve;
      }),
    );
  const { setProfile } = renderPicker();
  fireEvent.click(await screen.findByRole("link", { name: /Ghoul/ }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Add" })).toBeEnabled(),
  );
  fireEvent.change(screen.getByLabelText("Note"), {
    target: { value: "Keep encounter draft" },
  });
  fireEvent.click(await screen.findByRole("button", { name: "Results" }));
  const pages = await screen.findByRole("group", { name: "Result pages" });
  fireEvent.click(within(pages).getByRole("button", { name: "Next Page" }));
  await waitFor(() => expect(search).toHaveBeenCalledTimes(2));
  setProfile("embeddinggemma");
  await waitFor(() => expect(search).toHaveBeenCalledTimes(3));
  expect(search.mock.calls[2][0]).toMatchObject({
    search_profile: "embeddinggemma",
    page: { number: 1 },
  });
  expect(screen.queryByRole("link", { name: /Ghoul/ })).toBeNull();
  await expectRetainedCandidate();
  expect(screen.getByLabelText("Note")).toHaveValue("Keep encounter draft");
  await act(async () =>
    finishNew(result("New creature", "generation-a", "embeddinggemma")),
  );
  await act(async () => finishOld(result("Late creature")));
  expect(await screen.findByRole("link", { name: /New creature/ })).toBeVisible();
  expect(screen.queryByRole("link", { name: /Late creature/ })).toBeNull();
  await expectRetainedCandidate();
  expect(getRecordDetail).toHaveBeenCalledTimes(1);
});
it("blocks selection and form actions while Add is pending, preserving the draft", async () => {
  const view = renderPicker();
  fireEvent.change(screen.getByLabelText("Note"), {
    target: { value: "Keep this note" },
  });
  fireEvent.click(await screen.findByRole("link", { name: /Ghoul/ }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Add" })).toBeEnabled(),
  );
  view.setDisabled(true);
  expect(screen.getByRole("searchbox")).toBeDisabled();
  expect(screen.getByRole("button", { name: "Add" })).toBeDisabled();
  expect(screen.getByRole("button", { name: "Cancel" })).toBeDisabled();
  view.setDisabled(false);
  expect(screen.getByLabelText("Note")).toHaveValue("Keep this note");
  expect(screen.getByRole("button", { name: "Add" })).toBeEnabled();
  await expectRetainedCandidate();
});
it("distinguishes failed retrieval from empty results and retries the same request", async () => {
  search
    .mockRejectedValueOnce(new Error("Server unavailable"))
    .mockResolvedValue(result());
  renderPicker();
  expect(await screen.findByRole("alert")).toHaveTextContent("Search failed");
  expect(screen.queryByText("No matching records")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Retry search" }));
  expect(await screen.findByRole("link", { name: /Ghoul/ })).toBeInTheDocument();
  expect(search).toHaveBeenCalledTimes(2);
  expect(search.mock.calls[1][0]).toEqual(search.mock.calls[0][0]);
});
it("adopts generation once and explicitly refreshes conflicts without clearing candidate or form", async () => {
  const { client } = renderPicker();
  await screen.findByRole("link", { name: /Ghoul/ });
  expect(search).toHaveBeenCalledTimes(1);
  fireEvent.click(screen.getByRole("link", { name: /Ghoul/ }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Add" })).toBeEnabled(),
  );
  fireEvent.change(screen.getByLabelText("Note"), {
    target: { value: "Unsaved encounter note" },
  });
  client.setQueryData(["encounter", "draft-owner"], { name: "Encounter" });
  search
    .mockRejectedValueOnce(
      new AtlasApiError(409, "The reference collection changed.", {
        code: "artifact_generation_changed",
        message: "The reference collection changed.",
        recoverable_action: "refresh_results",
      }),
    )
    .mockResolvedValue({
      ...result("Dragon", "generation-b"),
      sort: { kind: "relevance" } as const,
    });
  await submit("dragon");
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "The reference collection changed.",
  );
  expect(search.mock.calls[1][0].expected_artifact_generation).toBe("generation-a");
  vi.mocked(getRecordDetail).mockImplementation((key) =>
    Promise.resolve({ ...detailFixture(key), artifact_generation: "generation-b" }),
  );
  fireEvent.click(screen.getByRole("button", { name: "Refresh results" }));
  expect(await screen.findByRole("link", { name: /Dragon/ })).toBeInTheDocument();
  expect(search).toHaveBeenCalledTimes(3);
  expect(search.mock.calls[2][0]).toMatchObject({
    expected_artifact_generation: null,
    page: { number: 1, size: 25 },
    criteria: { query: "dragon", mode: "hybrid" },
  });
  expect(screen.getByLabelText("Note")).toHaveValue("Unsaved encounter note");
  await expectRetainedCandidate();
  expect(client.getQueryData(["encounter", "draft-owner"])).toEqual({
    name: "Encounter",
  });
});

it("runs an inline handoff query with the resolved workspace model and picker evidence", async () => {
  let finish!: (profile: WorkspaceSearchDefault) => void;
  const resolveWorkspaceDefault = vi.fn(
    () =>
      new Promise<WorkspaceSearchDefault>((resolve) => {
        finish = resolve;
      }),
  );
  renderPicker({ initialQuery: "Lantern Ward", resolveWorkspaceDefault });
  await waitFor(() => expect(resolveWorkspaceDefault).toHaveBeenCalledOnce());
  expect(search).not.toHaveBeenCalled();
  const response = result("Lantern Ward", "generation-a", "embeddinggemma");
  response.mode = { kind: "text_search", mode: "hybrid", query: "Lantern Ward" };
  response.sort = { kind: "relevance" };
  search.mockResolvedValue(response);
  await act(async () => finish({ id: "embeddinggemma", generation: "generation-a" }));
  expect(await screen.findByRole("link", { name: /Lantern Ward/ })).toBeVisible();
  expect(screen.getByRole("searchbox", { name: "Search records" })).toHaveValue(
    "Lantern Ward",
  );
  expect(search.mock.calls.map(([request]) => request)).toEqual([
    expect.objectContaining({
      criteria: expect.objectContaining({
        kind: "text_search",
        mode: "hybrid",
        query: "Lantern Ward",
      }),
      result_detail: "evidence",
      search_profile: "embeddinggemma",
      expected_artifact_generation: "generation-a",
    }),
  ]);
});

it("focuses the handoff search field when the model cannot run the query", async () => {
  renderPicker({ initialQuery: "Lantern Ward", usable: false });
  const input = await screen.findByRole("searchbox", { name: "Search records" });
  await waitFor(() => expect(input).toHaveFocus());
  expect(input).toHaveValue("Lantern Ward");
  expect(search).not.toHaveBeenCalled();
});

it.each(["replace", "clear", "close"] as const)(
  "lets %s supersede a handoff waiting for the workspace model",
  async (action) => {
    let finishSeed!: (value: WorkspaceSearchDefault) => void;
    let finishReplacement!: (value: WorkspaceSearchDefault) => void;
    const resolveWorkspaceDefault = vi
      .fn()
      .mockImplementationOnce(
        () =>
          new Promise<WorkspaceSearchDefault>((resolve) => {
            finishSeed = resolve;
          }),
      )
      .mockImplementationOnce(
        () =>
          new Promise<WorkspaceSearchDefault>((resolve) => {
            finishReplacement = resolve;
          }),
      );
    const controls = renderPicker({
      initialQuery: "Lantern",
      usable: false,
      resolveWorkspaceDefault,
    });
    await waitFor(() => expect(resolveWorkspaceDefault).toHaveBeenCalledOnce());
    expect(search).not.toHaveBeenCalled();

    if (action === "replace") {
      const response = result("Dragon", "generation-b", "embeddinggemma");
      response.mode = { kind: "text_search", mode: "hybrid", query: "dragon" };
      response.sort = { kind: "relevance" };
      search.mockResolvedValue(response);
      await submit("dragon");
      expect(resolveWorkspaceDefault).toHaveBeenCalledTimes(2);
      expect(search).not.toHaveBeenCalled();
      await act(async () =>
        finishReplacement({ id: "embeddinggemma", generation: "generation-b" }),
      );
      expect(await screen.findByRole("link", { name: /Dragon/ })).toBeVisible();
      expect(search.mock.calls[0][0]).toMatchObject({
        criteria: { query: "dragon", mode: "hybrid" },
        expected_artifact_generation: "generation-b",
        search_profile: "embeddinggemma",
      });
    } else if (action === "clear") {
      fireEvent.click(screen.getByRole("button", { name: "Clear search" }));
    } else {
      controls.setOpen(false);
      await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
      controls.setOpen(true);
    }
    if (action !== "replace") {
      expect(await screen.findByRole("link", { name: /Ghoul/ })).toBeVisible();
      expect(screen.getByRole("searchbox")).toHaveValue("");
      expect(search.mock.calls[0][0].criteria.kind).toBe("list_records");
    }
    await act(async () => finishSeed({ id: "bge-base", generation: "generation-old" }));
    expect(search).toHaveBeenCalledOnce();
    expect(discoverFilterEditor).toHaveBeenCalled();
    expect(resolveWorkspaceDefault).toHaveBeenCalledTimes(action === "replace" ? 2 : 1);
  },
);

it("discards a pending keyword draft when closing a picker with empty criteria", async () => {
  let finish!: (value: WorkspaceSearchDefault) => void;
  const readiness = new Promise<WorkspaceSearchDefault>((resolve) => {
    finish = resolve;
  });
  const resolveDefault = vi.fn(() => readiness);
  const picker = renderPicker({ resolveWorkspaceDefault: resolveDefault });
  await screen.findByText("Ghoul");
  await submit("unfinished");
  expect(resolveDefault).toHaveBeenCalledOnce();
  expect(screen.getByRole("searchbox", { name: "Search records" })).toHaveValue(
    "unfinished",
  );
  picker.setOpen(false);
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  await act(async () => finish({ id: "bge-base", generation: "generation-a" }));
  picker.setOpen(true);

  const input = await screen.findByRole("searchbox", { name: "Search records" });
  expect(input).toHaveValue("");
  expect(
    search.mock.calls.every(([request]) => request.criteria.kind === "list_records"),
  ).toBe(true);
});

it("keeps a reopened picker independent of an earlier pending refresh", async () => {
  const { client, setOpen } = renderPicker();
  await screen.findByRole("link", { name: /Ghoul/ });
  search.mockRejectedValueOnce(
    new AtlasApiError(409, "Collection changed", {
      code: "artifact_generation_changed",
      message: "Collection changed",
      recoverable_action: "refresh_results",
    }),
  );
  await submit("dragon");
  await screen.findByRole("button", { name: "Refresh results" });
  let finishCancellation!: () => void;
  const cancellation = new Promise<void>((resolve) => {
    finishCancellation = resolve;
  });
  const cancel = client.cancelQueries.bind(client);
  const pendingCancel = vi
    .spyOn(client, "cancelQueries")
    .mockImplementation(async (filters, options) => {
      await cancel(filters, options);
      await cancellation;
    });
  fireEvent.click(screen.getByRole("button", { name: "Refresh results" }));
  await waitFor(() => expect(pendingCancel).toHaveBeenCalledOnce());
  setOpen(false);
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  search.mockResolvedValue(result("New session", "generation-c"));
  setOpen(true);
  expect(await screen.findByRole("link", { name: /New session/ })).toBeInTheDocument();
  expect(search.mock.lastCall?.[0].expected_artifact_generation).toBeNull();
  await act(async () => {
    finishCancellation();
    await cancellation;
  });
  expect(screen.getByRole("link", { name: /New session/ })).toBeInTheDocument();
  expect(
    client
      .getQueryCache()
      .findAll({ queryKey: ["search-picker"] })
      .filter((query) => query.queryKey.includes("generation-c")),
  ).toHaveLength(1);
  expect(search).toHaveBeenCalledTimes(3);
});

it("rereads an externally retired picker against its generation and preserves its candidate and Add draft on conflict", async () => {
  const { client } = renderPicker();
  fireEvent.click(await screen.findByRole("link", { name: /Ghoul/ }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Add" })).toBeEnabled(),
  );
  fireEvent.change(screen.getByLabelText("Note"), {
    target: { value: "Unsubmitted Add note" },
  });
  search.mockRejectedValue(
    new AtlasApiError(409, "Collection changed", {
      code: "artifact_generation_changed",
      message: "Collection changed",
      recoverable_action: "refresh_results",
    }),
  );
  await act(async () => {
    await retireReferenceQueries(client);
  });
  expect(
    await screen.findByRole("button", { name: "Refresh results" }),
  ).toBeInTheDocument();
  expect(search).toHaveBeenCalledTimes(2);
  expect(search.mock.lastCall?.[0].expected_artifact_generation).toBe("generation-a");
  expect(screen.getByLabelText("Note")).toHaveValue("Unsubmitted Add note");
  await expectRetainedCandidate();
});

it("dismisses an open picker on same-href navigation while honoring the pending-write guard", () => {
  const picker = renderPicker();
  picker.setDisabled(true);
  const navigate = () =>
    act(() =>
      atlasRouter.emit({
        type: "onBeforeNavigate",
        fromLocation: atlasRouter.latestLocation,
        toLocation: {
          ...atlasRouter.latestLocation,
          state: {
            ...atlasRouter.latestLocation.state,
            atlasReadingDismissal: "new-search",
          },
        },
        pathChanged: false,
        hrefChanged: false,
        hashChanged: false,
      }),
    );
  navigate();
  expect(cancelPicker).not.toHaveBeenCalled();
  picker.setDisabled(false);
  navigate();
  expect(cancelPicker).toHaveBeenCalledTimes(1);
  picker.setOpen(false);
  navigate();
  expect(cancelPicker).toHaveBeenCalledTimes(1);
});

it("keeps picker selections and drafts through profile/page continuation navigation", async () => {
  renderPicker();
  fireEvent.click(await screen.findByRole("link", { name: /Ghoul/ }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Add" })).toBeEnabled(),
  );
  fireEvent.change(screen.getByLabelText("Note"), {
    target: { value: "Keep this draft" },
  });
  const location = {
    ...atlasRouter.latestLocation,
    state: {
      ...atlasRouter.latestLocation.state,
      atlasReadingDismissal: "prior-search",
    },
  };
  act(() =>
    atlasRouter.emit({
      type: "onBeforeNavigate",
      fromLocation: location,
      toLocation: {
        ...location,
        search: { search_profile: "embeddinggemma", page: "2" },
      },
      pathChanged: false,
      hrefChanged: true,
      hashChanged: false,
    }),
  );
  expect(screen.getByLabelText("Note")).toHaveValue("Keep this draft");
  expect(screen.getByRole("button", { name: "Add" })).toBeEnabled();
  expect(cancelPicker).not.toHaveBeenCalled();
  expect(screen.getByRole("dialog", { name: "Choose record" })).toBeInTheDocument();
});

it("retains backend scope guidance while directions load but invalidates it for a different parent", async () => {
  const scope: SearchConnectionScope = {
    direction: "incoming",
    include_contents: true,
    parent: {
      filter: {
        kind: "any_of",
        children: [
          { kind: "boolean_constant", value: true, clause_id: "edit" },
          { kind: "boolean_constant", value: false },
        ],
      },
      exclude_clause_id: "edit",
      incomplete: false,
      list_ref: null,
      connection_scope: null,
      expected_artifact_generation: "generation-a",
    },
  };
  const broad: SearchResultsPage = {
    ...result(),
    connection_scope: { kind: "all_entries", reason: "non_conjunctive_context" },
  };
  search.mockResolvedValue(broad);
  const view = renderPicker({ connectionScope: scope });
  await waitFor(() =>
    expect(screen.getByLabelText("Discovery scope")).toHaveTextContent("all_entries"),
  );
  await screen.findByRole("link", { name: /Ghoul/ });
  let finishOld!: (value: SearchResultsPage) => void;
  search.mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        finishOld = resolve;
      }),
  );
  view.setConnectionScope({ ...scope, direction: "outgoing" });
  await waitFor(() => expect(finishOld).toBeDefined());
  expect(screen.getByLabelText("Discovery scope")).toHaveTextContent("all_entries");
  expect(screen.queryByRole("link", { name: /Ghoul/ })).toBeNull();
  let finishNew!: (value: SearchResultsPage) => void;
  search.mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        finishNew = resolve;
      }),
  );
  const nextScope: SearchConnectionScope = {
    ...scope,
    direction: "outgoing",
    parent: {
      ...scope.parent,
      filter: { kind: "boolean_constant", value: true },
      exclude_clause_id: null,
    },
  };
  view.setConnectionScope(nextScope);
  await waitFor(() => expect(finishNew).toBeDefined());
  expect(screen.getByLabelText("Discovery scope")).toHaveTextContent("unresolved");
  await act(async () => {
    finishOld(broad);
    finishNew({ ...result(), connection_scope: { kind: "connected" } });
  });
  await waitFor(() =>
    expect(screen.getByLabelText("Discovery scope")).toHaveTextContent("connected"),
  );
  // React Query retains cached data when a background refresh fails.
  search.mockRejectedValue(
    new AtlasApiError(409, "Collection changed", {
      code: "artifact_generation_changed",
      message: "Collection changed",
      recoverable_action: "refresh_results",
    }),
  );
  await act(async () => {
    await view.client.invalidateQueries({ queryKey: ["search-picker"] });
  });
  await screen.findByRole("button", { name: "Close picker" });
  expect(screen.getByLabelText("Discovery scope")).toHaveTextContent("unresolved");
  let failScope!: (error: Error) => void;
  search.mockImplementationOnce(
    () =>
      new Promise((_resolve, reject) => {
        failScope = reject;
      }),
  );
  view.setConnectionScope({ ...nextScope, include_contents: false });
  await waitFor(() => expect(failScope).toBeDefined());
  expect(screen.getByLabelText("Discovery scope")).toHaveTextContent("unresolved");
  // An earlier direction also has a cached page from this now-invalid generation.
  view.setConnectionScope(scope);
  expect(screen.getByLabelText("Discovery scope")).toHaveTextContent("unresolved");
  await act(async () => {
    failScope(
      new AtlasApiError(409, "Collection changed", {
        code: "artifact_generation_changed",
        message: "Collection changed",
        recoverable_action: "refresh_results",
      }),
    );
  });
});
