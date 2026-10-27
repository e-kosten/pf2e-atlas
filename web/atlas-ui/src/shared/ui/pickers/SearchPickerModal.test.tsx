import { chooseAttributeSort } from "../../../test/sortControls";
import type { ResultSort } from "../../../generated/atlas";
import { SearchProfileProvider } from "../../../test/SearchProfileProvider";
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
  getSimilarityParts,
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
import {
  similarityConfigurationFixture,
  similarityPartsFixture,
} from "../../../test/similarityFixtures";

vi.mock("../../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../../api/atlasApi")>()),
  searchPage: vi.fn(),
  getRecordDetail: vi.fn(),
  discoverFilterEditor: vi.fn(),
  discoverFilterCounts: vi.fn(),
  getSimilarityParts: vi.fn(),
}));
const search = vi.mocked(searchPage);
const addSelected = vi.fn();
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
      total: 1n,
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
  similar = false,
}: {
  disabled?: boolean;
  open?: boolean;
  similar?: boolean;
}) {
  const [form] = Form.useForm();
  const [selected, setSelected] = useState<string | null>(null);
  return (
    <SearchPickerModal
      title="Choose record"
      disabled={disabled}
      open={open}
      form={form}
      buildRequest={(query) => ({
        criteria: similar
          ? {
              kind: "similar_records",
              seed_record_key: "hazards:brown-mold",
              filter: null,
            }
          : query.trim()
            ? { kind: "text_search", query, mode: "hybrid", filter: null }
            : { kind: "list_records", filter: null },
        expected_artifact_generation: null,
        page: { number: 1, size: 25 },
      })}
      emptyPrompt="Search for a record"
      emptyResults="No matching records"
      selectedRecordKey={selected}
      onSelectionChange={(selection) =>
        setSelected(selection?.record.record_key ?? null)
      }
      onCancel={vi.fn()}
      onFinish={() => addSelected(selected)}
      okButtonProps={{ disabled: !selected }}
    >
      {(detail) => (
        <>
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
function renderPicker(similar = false) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: 20_000 } },
  });
  let profile = "bge-base";
  let disabled = false;
  let open = true;
  const content = () => (
    <ConfigProvider theme={{ token: { motion: false } }}>
      <QueryClientProvider client={client}>
        <SearchProfileProvider id={profile}>
          <App>
            <Picker disabled={disabled} open={open} similar={similar} />
          </App>
        </SearchProfileProvider>
      </QueryClientProvider>
    </ConfigProvider>
  );
  const view = render(content());
  return {
    client,
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
  page.rows[0].matching_sections = [{ navigation, label: "Command", snippet: null }];
  page.rows[0].matches = [
    {
      navigation,
      label: "Command",
      snippet: null,
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
    Promise.resolve(request?.embedded_path.length ? child : parent),
  );
  renderPicker();
  fireEvent.click(await screen.findByRole("link", { name: /^Ghoul/ }));
  fireEvent.click(await screen.findByRole("link", { name: /Command/ }));
  expect(await screen.findByRole("heading", { name: "Command" })).toBeVisible();
  expect(screen.getByLabelText("Selected adjustment")).toHaveTextContent("elite");
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
    editorFixture([
      {
        ...fieldFixture("kind", "string", "Kind"),
        choices: ["creature", "spell"],
      },
    ]),
  );
  vi.mocked(discoverFilterCounts).mockResolvedValue({
    search_profile: "bge-base",
    artifact_generation: "generation-a",
    counts: {
      status: "available",
      value: {
        field: "kind",
        options: [
          { value: "creature", distinct_roots: 4n, selected: false },
          { value: "spell", distinct_roots: 0n, selected: false },
        ],
        total_values: 2n,
        exhaustive: true,
        count_basis: "exact counterfactual displayed roots",
      },
    },
  });
  renderPicker();
  await screen.findByRole("link", { name: /Ghoul/ });
  expect(discoverFilterCounts).not.toHaveBeenCalled();
  fireEvent.click(await screen.findByRole("button", { name: "Edit Kind filter" }));
  expect(await screen.findByLabelText("4 matching records")).toBeInTheDocument();
  expect(
    screen.getByRole("button", { name: "Kind: Spell, neutral; activate to exclude" }),
  ).toBeEnabled();
  expect(discoverFilterCounts).toHaveBeenCalledWith(
    expect.objectContaining({
      context: expect.objectContaining({
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
    expect(screen.getByRole("status", { name: "Searching…" })).toBeVisible(),
  );
  expect(screen.queryByTitle("Next Page")).toBeNull();
  expect(screen.queryByText("No matching records")).toBeNull();
  await act(async () =>
    finish({ ...result(), rows: [], page: { ...result().page, count: 0, total: 0n } }),
  );
  expect(await screen.findByText("No matching records")).toBeVisible();
  expect(screen.queryByText("Search for a record")).toBeNull();
  expect(screen.queryByTitle("Next Page")).toBeNull();
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
    page: { ...result().page, count: 0, total: 0n },
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
    page: { ...result().page, count: 3, total: 3n },
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
  const frames: string[] = [];
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
    await within(preview).findByText("Loading Ogre…");
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
          (
            query.state.data as
              { detail?: ReturnType<typeof detailFixture> } | undefined
          )?.detail?.artifact_generation === "generation-b",
      ),
  ).toBe(false);
});

it("inherits a profile switch on page one while retaining the explicit candidate, reader and Add draft", async () => {
  const page = result();
  page.page = { ...page.page, total: 100n, has_more: true };
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
  fireEvent.click(screen.getByTitle("Next Page"));
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
  expect(await screen.findByRole("alert")).toHaveTextContent("Server unavailable");
  expect(screen.queryByText("No matching records")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Retry search" }));
  expect(await screen.findByRole("link", { name: /Ghoul/ })).toBeInTheDocument();
  expect(search).toHaveBeenCalledTimes(2);
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

it("applies reloaded B parts once while preserving the picker candidate and unsaved form", async () => {
  let finishOld!: (page: SearchResultsPage) => void;
  const oldPage = new Promise<SearchResultsPage>((resolve) => {
    finishOld = resolve;
  });
  search.mockImplementation((request) => {
    const generation = request.expected_artifact_generation ?? "generation-a";
    const similarity =
      request.criteria.kind === "similar_records"
        ? request.criteria.similarity
        : undefined;
    const page = {
      ...result(generation === "generation-b" ? "Dragon" : "Ghoul", generation),
      sort: { kind: "relevance" } as const,
      ...(similarity ? { similarity: similarityConfigurationFixture(similarity) } : {}),
    };
    return search.mock.calls.length === 2 ? oldPage : Promise.resolve(page);
  });
  vi.mocked(getSimilarityParts)
    .mockRejectedValueOnce(
      new AtlasApiError(409, "Collection changed", {
        code: "artifact_generation_changed",
        message: "Collection changed",
        recoverable_action: "refresh_results",
      }),
    )
    .mockResolvedValue(similarityPartsFixture());
  const { client } = renderPicker(true);
  fireEvent.click(await screen.findByRole("link", { name: /Ghoul/ }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Add" })).toBeEnabled(),
  );
  fireEvent.change(screen.getByLabelText("Note"), {
    target: { value: "Unsaved encounter note" },
  });
  client.setQueryData(["encounter", "draft-owner"], { name: "Encounter" });
  act(() => {
    void client.invalidateQueries({ queryKey: ["search-picker"] });
  });
  await waitFor(() => expect(search).toHaveBeenCalledTimes(2));
  vi.mocked(getRecordDetail).mockImplementation((key) =>
    Promise.resolve({ ...detailFixture(key), artifact_generation: "generation-b" }),
  );
  fireEvent.click(screen.getByRole("button", { name: "Search settings" }));
  fireEvent.click(screen.getByText("Sections to compare"));
  fireEvent.click(await screen.findByRole("button", { name: "Reload sections" }));
  await screen.findByRole("checkbox", { name: "Description" });
  const selectAll = screen.getByRole("checkbox", { name: "Select all", exact: true });
  expect(selectAll).toBeChecked();
  fireEvent.click(selectAll);
  fireEvent.click(screen.getByRole("checkbox", { name: "Description" }));
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  // The reader remains open; return to results to inspect the new collection.
  fireEvent.click(screen.getByRole("button", { name: "Results" }));
  expect(await screen.findByRole("link", { name: /Dragon/ })).toBeInTheDocument();
  expect(search).toHaveBeenCalledTimes(3);
  expect(search.mock.lastCall?.[0]).toMatchObject({
    expected_artifact_generation: "generation-b",
    criteria: { similarity: { parts: ["description-b"] } },
    page: { number: 1 },
  });
  expect(search.mock.calls[1][1]?.aborted).toBe(true);
  await act(async () => {
    finishOld(result("Old A"));
    await new Promise((resolve) => setTimeout(resolve, 350));
  });
  expect(screen.queryByRole("link", { name: /Old A/ })).toBeNull();
  expect(screen.queryByRole("button", { name: "Refresh results" })).toBeNull();
  expect(screen.getByLabelText("Note")).toHaveValue("Unsaved encounter note");
  await expectRetainedCandidate();
  expect(client.getQueryData(["encounter", "draft-owner"])).toEqual({
    name: "Encounter",
  });
  expect(search).toHaveBeenCalledTimes(3);
  expect(
    client
      .getQueryCache()
      .findAll({ queryKey: ["search-picker"] })
      .some((query) => query.queryKey.includes("generation-a")),
  ).toBe(false);
});
