import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import type { RecordNavigationView, SearchResultsPage } from "../../generated/atlas";
import { detailFixture, editorFixture, summaryFixture } from "../../test/fixtures";
import { ResultTable } from "./ResultTable";
import { useSearchWorkspace } from "./useSearchWorkspace";

const api = vi.hoisted(() => ({
  discoverFilterEditor: vi.fn(),
  discoverFilterValues: vi.fn(),
  discoverFilterCounts: vi.fn(),
  getRecordDetail: vi.fn(),
  getReadiness: vi.fn(),
  searchPage: vi.fn(),
}));
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  ...api,
}));

const selection: RecordNavigationView = {
  record_key: "actors:caster",
  owners: [{ collection: "items", identity: { Stable: "spell-local" } }],
  field: "/system/description/value",
  passage: { kind: "identity" },
  source_fingerprint: null,
};
const matchLabel =
  "A creature’s locally authored Fireball description and targeting details";
const page: SearchResultsPage = {
  artifact_generation: "generation-a",
  mode: { kind: "text_search", query: "Fireball", mode: "hybrid" },
  page: { number: 1, size: 25, count: 2, total: 2n, has_more: false, next_page: null },
  coverage: null,
  rows: [
    {
      record: summaryFixture("spells:fireball", "Fireball"),
      matches: [
        {
          navigation: selection,
          lane: "semantic",
          label: matchLabel,
          snippet: null,
          lexical_rank: null,
          semantic_similarity: null,
        },
      ],
    },
    { record: summaryFixture("spells:heal", "Heal"), matches: [] },
  ],
};

function WorkspaceResults() {
  const workspace = useSearchWorkspace();
  return <ResultTable workspace={workspace} />;
}
function renderResults() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <WorkspaceResults />
    </QueryClientProvider>,
  );
}
const originalScroll = Object.getOwnPropertyDescriptor(
  Element.prototype,
  "scrollIntoView",
);
beforeAll(() => {
  Object.defineProperty(Element.prototype, "scrollIntoView", {
    configurable: true,
    value: vi.fn(),
  });
});
afterAll(() => {
  if (originalScroll)
    Object.defineProperty(Element.prototype, "scrollIntoView", originalScroll);
  else Reflect.deleteProperty(Element.prototype, "scrollIntoView");
});

describe("ResultTable committed selection", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    history.replaceState(null, "", "/search?q=Fireball");
    api.getReadiness.mockResolvedValue({ status: "ready", message: "Ready" });
    api.discoverFilterEditor.mockResolvedValue(editorFixture());
    api.discoverFilterCounts.mockResolvedValue({
      counts: { status: "unavailable", reason: "non_conjunctive_context" },
    });
    api.searchPage.mockResolvedValue(page);
    api.getRecordDetail.mockImplementation((key: string) =>
      Promise.resolve(detailFixture(key)),
    );
  });

  it("leaves the default keyboard target unselected and opens detail on row activation", async () => {
    renderResults();
    const fireball = (await screen.findByRole("link", { name: "Fireball" })).closest(
      "tr",
    )!;
    expect(fireball).toHaveAttribute("aria-selected", "false");
    expect(fireball.closest("tr")).toHaveAttribute("data-active-result", "true");
    expect(api.getRecordDetail).not.toHaveBeenCalled();
    expect(screen.queryByRole("radio")).not.toBeInTheDocument();
    fireEvent.click(fireball);
    await waitFor(() =>
      expect(api.getRecordDetail).toHaveBeenCalledWith(
        "spells:fireball",
        undefined,
        expect.any(AbortSignal),
      ),
    );
    expect(fireball).toHaveAttribute("aria-selected", "true");
    expect(window.location.pathname).toBe("/search/records/spells:fireball");
    expect(new URLSearchParams(window.location.search).get("q")).toBe("Fireball");
    expect(screen.queryByText("spells:fireball")).not.toBeInTheDocument();
  });

  it("keeps the committed row selected while hover and keyboard focus move to another row", async () => {
    renderResults();
    const fireball = (await screen.findByRole("link", { name: "Fireball" })).closest(
      "tr",
    )!;
    fireEvent.click(fireball);
    await waitFor(() => expect(api.getRecordDetail).toHaveBeenCalledTimes(1));
    const heal = screen.getByRole("link", { name: "Heal" }).closest("tr")!;
    const healRow = heal.closest("tr")!;
    fireEvent.mouseEnter(healRow);
    fireEvent.focus(healRow);
    expect(healRow).toHaveAttribute("data-active-result", "true");
    expect(fireball).toHaveAttribute("aria-selected", "true");
    expect(heal).toHaveAttribute("aria-selected", "false");
    expect(api.getRecordDetail).toHaveBeenCalledTimes(1);
    expect(window.location.pathname).toBe("/search/records/spells:fireball");
    fireEvent.keyDown(healRow, { key: "Enter" });
    await waitFor(() =>
      expect(api.getRecordDetail).toHaveBeenCalledWith(
        "spells:heal",
        undefined,
        expect.any(AbortSignal),
      ),
    );
    expect(heal).toHaveAttribute("aria-selected", "true");
    expect(fireball).toHaveAttribute("aria-selected", "false");
    expect(window.location.pathname).toBe("/search/records/spells:heal");
  });

  it("retains row click and Space selection without treating embedded link clicks as row selection", async () => {
    renderResults();
    const fireball = (await screen.findByRole("link", { name: "Fireball" })).closest(
      "tr",
    )!;
    fireEvent.click(fireball);
    await waitFor(() =>
      expect(api.getRecordDetail).toHaveBeenCalledWith(
        "spells:fireball",
        undefined,
        expect.any(AbortSignal),
      ),
    );
    const heal = screen.getByRole("link", { name: "Heal" }).closest("tr")!;
    fireEvent.keyDown(heal.closest("tr")!, { key: " " });
    await waitFor(() =>
      expect(api.getRecordDetail).toHaveBeenCalledWith(
        "spells:heal",
        undefined,
        expect.any(AbortSignal),
      ),
    );
    expect(heal).toHaveAttribute("aria-selected", "true");
    expect(fireball).toHaveAttribute("aria-selected", "false");
    const match = screen.getByRole("link", { name: matchLabel });
    const requestCount = api.getRecordDetail.mock.calls.length;
    fireEvent.click(match);
    expect(window.location.pathname).toBe("/records/actors:caster");
    expect(
      JSON.parse(new URLSearchParams(window.location.search).get("selection")!),
    ).toEqual(selection);
    expect(api.getRecordDetail).toHaveBeenCalledTimes(requestCount);
  });

  it.each([{ metaKey: true }, { ctrlKey: true }, { shiftKey: true }, { button: 1 }])(
    "keeps modified name and Match clicks native and retains the full checked address %j",
    async (modifiers) => {
      renderResults();
      const match = await screen.findByRole("link", { name: matchLabel });
      const href = new URL(match.getAttribute("href")!, window.location.origin);
      expect(href.pathname).toBe("/records/actors:caster");
      expect(JSON.parse(href.searchParams.get("selection")!)).toEqual(selection);
      expect(fireEvent.click(match, modifiers)).toBe(true);
      const name = screen.getByRole("link", { name: "Fireball" });
      expect(name).toHaveAttribute("href", "/records/spells:fireball");
      expect(fireEvent.click(name, modifiers)).toBe(true);
      expect(window.location.pathname).toBe("/search");
      expect(api.getRecordDetail).not.toHaveBeenCalled();
    },
  );
});
