import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { Grid } from "antd";
import { readingScrollGeometry } from "../../test/readingScrollGeometry";
import { StrictMode, useState } from "react";
import { getOwnedReading, getRecordDetail } from "../../api/atlasApi";
import {
  activityFixture,
  actorFixture,
  detailFixture,
  summaryFixture,
  knownFact,
} from "../../test/fixtures";
import { ReferenceBrowser } from "./ReferenceBrowser";
import type { BrowsePositions } from "./useBrowsePositions";
import { ATLAS_ROUTE_CHANGE_EVENT } from "../../app/routes";
import { parentShowsActivity, useReferenceReading } from "./useReferenceReading";
import type {
  OwnedReadingBatchView,
  RecordDetailView,
  SearchResultRow,
} from "../../generated/atlas";

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getRecordDetail: vi.fn(),
  getOwnedReading: vi.fn(),
}));
const getDetail = vi.mocked(getRecordDetail);
const getAbilities = vi.mocked(getOwnedReading);
const navigation = activityFixture("Jaws", "actors:ghoul").navigation;
const rows: SearchResultRow[] = [
  {
    record: summaryFixture("actors:ghoul", "Ghoul"),
    matches: [
      {
        navigation,
        label: "Jaws",
        snippet: null,
        lane: "lexical",
        lexical_rank: null,
        semantic_similarity: null,
      },
    ],
  },
  { record: summaryFixture("spells:heal", "Heal"), matches: [] },
];
function Harness({ positions }: { positions?: BrowsePositions }) {
  const [selected, select] = useState<string | null>(null);
  const reading = useReferenceReading();
  return (
    <ReferenceBrowser
      positions={positions}
      rows={rows}
      selectedRecordKey={selected}
      onSelect={(row) => select(row.record.record_key)}
      reading={reading}
      loading={false}
    />
  );
}
function renderBrowser(positions?: BrowsePositions) {
  return render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <Harness positions={positions} />
    </QueryClientProvider>,
  );
}
beforeEach(() => {
  getDetail.mockReset();
  getAbilities.mockReset();
  getDetail.mockImplementation((key) => Promise.resolve(detailFixture(key)));
});

it.each([
  "immediate strike",
  "mobile strike",
  "delayed strike",
  "delayed ability",
  "retry ability",
] as const)(
  "restores landed history after %s content is ready, while a fresh selection lands again",
  async (scenario) => {
    const mobile = scenario === "mobile strike";
    const delayed = scenario.startsWith("delayed") || scenario.startsWith("retry");
    vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ lg: !mobile });
    const ability = scenario.endsWith("ability");
    const activity = {
      ...activityFixture("Jaws", "actors:ghoul"),
      kind: ability ? ("ability" as const) : ("strike" as const),
    };
    const parent = detailFixture(
      "actors:ghoul",
      "Ghoul",
      ability ? undefined : "rules:linked",
    );
    parent.presentation.body = {
      kind: "creature",
      value: { ...actorFixture(), activities: knownFact([activity]) },
    };
    const content = detailFixture("actors:ghoul", "Ghoul", "rules:linked");
    const batch: OwnedReadingBatchView = {
      artifact_generation: parent.artifact_generation,
      parent: parent.selected,
      availability: "value",
      total: 1,
      start: 0,
      end: 1,
      previous: null,
      next: null,
      entries: [
        {
          navigation: activity.navigation,
          content: content.presentation.content,
          relationships: content.relationships,
        },
      ],
    };
    getAbilities.mockResolvedValue(batch);
    getDetail.mockImplementation((key) =>
      Promise.resolve(
        key === "actors:ghoul"
          ? parent
          : detailFixture(key, key === "spells:heal" ? "Heal" : "Linked rule"),
      ),
    );
    let geometry: ReturnType<typeof readingScrollGeometry> | undefined;
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    try {
      render(
        <StrictMode>
          <QueryClientProvider client={client}>
            <Harness />
          </QueryClientProvider>
        </StrictMode>,
      );
      const reader = document.querySelector<HTMLElement>(".reference-reader__scroll")!;
      geometry = readingScrollGeometry(reader);
      async function chooseMatch() {
        fireEvent.click(screen.getByRole("button", { name: "1 matches in Ghoul" }));
        const choices = document.querySelector<HTMLElement>(".reference-match-list")!;
        fireEvent.click(await within(choices).findByRole("link", { name: "Jaws" }));
      }
      await chooseMatch();
      const nested = await screen.findByRole("link", { name: "Nested Rule" });
      await waitFor(() => expect(reader.scrollTop).toBe(100));
      expect(document.documentElement.scrollTop).toBe(0);
      reader.scrollTop = 425;
      fireEvent.scroll(reader);
      nested.focus();
      fireEvent.click(nested);
      await screen.findByRole("heading", { name: "Linked rule" });
      let finish: (() => void) | undefined;
      if (ability) {
        client.removeQueries({ queryKey: ["owned-reading"] });
        if (scenario === "retry ability")
          getAbilities.mockRejectedValue(new Error("Ability read rejected"));
        else
          getAbilities.mockImplementationOnce(
            () =>
              new Promise((resolve) => {
                finish = () => resolve(batch);
              }),
          );
      } else if (scenario === "delayed strike") {
        client.removeQueries({
          queryKey: ["record-detail"],
          predicate: (query) => JSON.stringify(query.queryKey).includes("actors:ghoul"),
        });
        getDetail.mockImplementationOnce(
          () =>
            new Promise((resolve) => {
              finish = () => resolve(parent);
            }),
        );
      }
      fireEvent.click(screen.getByRole("button", { name: "Back" }));
      if (scenario === "retry ability") {
        await screen.findByText("Ability read rejected");
        getAbilities.mockImplementationOnce(
          () =>
            new Promise((resolve) => {
              finish = () => resolve(batch);
            }),
        );
        fireEvent.click(screen.getByRole("button", { name: "Retry" }));
      }
      if (delayed) {
        await waitFor(() => expect(finish).toBeDefined());
        // Emulate the browser clamping a loading reader and delivering its
        // queued scroll event before delayed target content has arrived.
        reader.scrollTop = 0;
        fireEvent.scroll(reader);
        expect(screen.queryByRole("link", { name: "Nested Rule" })).toBeNull();
        await act(async () => finish!());
      }
      const returned = await screen.findByRole("link", { name: "Nested Rule" });
      await waitFor(() => expect(reader.scrollTop).toBe(425));
      expect(returned).toHaveFocus();
      expect(document.documentElement.scrollTop).toBe(0);
      if (ability)
        expect(getAbilities).toHaveBeenLastCalledWith(
          expect.objectContaining({
            selection: { kind: "abilities", cursor: null, target: activity.navigation },
          }),
          expect.any(AbortSignal),
        );
      if (mobile) {
        fireEvent.click(screen.getByRole("button", { name: "Results" }));
        await waitFor(() =>
          expect(
            screen.getByRole("button", { name: "1 matches in Ghoul" }),
          ).toHaveFocus(),
        );
        expect(reader.scrollTop).toBe(425);
      }
      // Re-selecting an already visited match is a deliberate landing, not Back.
      await chooseMatch();
      await waitFor(() => expect(reader.scrollTop).toBe(100));
      if (scenario === "delayed ability") {
        fireEvent.click(screen.getByRole("link", { name: "Nested Rule" }));
        await screen.findByRole("heading", { name: "Linked rule" });
        client.removeQueries({ queryKey: ["owned-reading"] });
        finish = undefined;
        getAbilities.mockImplementationOnce(
          () =>
            new Promise((resolve) => {
              finish = () => resolve(batch);
            }),
        );
        fireEvent.click(screen.getByRole("button", { name: "Back" }));
        await waitFor(() => expect(finish).toBeDefined());
        const heal = screen.getByRole("link", { name: /Heal/ });
        fireEvent.click(heal);
        await screen.findByRole("heading", { name: "Heal" });
        heal.focus();
        await act(async () => finish!());
        expect(screen.getByRole("heading", { name: "Heal" })).toBeVisible();
        expect(heal).toHaveFocus();
        expect(reader.scrollTop).toBe(0);
      }
    } finally {
      geometry?.restore();
      cleanup();
      client.clear();
      vi.restoreAllMocks();
    }
  },
);

it("previews the whole summary without selecting on focus or inventing radio controls", async () => {
  renderBrowser();
  const link = screen.getByRole("link", { name: /Ghoul/ });
  fireEvent.focus(link);
  expect(getDetail).not.toHaveBeenCalled();
  expect(screen.queryByRole("radio")).toBeNull();
  fireEvent.click(link);
  await waitFor(() =>
    expect(getDetail).toHaveBeenCalledWith(
      "actors:ghoul",
      undefined,
      expect.any(AbortSignal),
    ),
  );
  expect(link).toHaveAttribute("aria-current", "true");
});
it.each([{ metaKey: true }, { ctrlKey: true }, { button: 1 }])(
  "preserves native modified summary navigation %j",
  (modifiers) => {
    renderBrowser();
    expect(
      fireEvent.click(screen.getByRole("link", { name: /Ghoul/ }), modifiers),
    ).toBe(true);
    expect(getDetail).not.toHaveBeenCalled();
  },
);
it("keeps match controls out of the summary anchor and restores focus on Escape", async () => {
  renderBrowser();
  const trigger = screen.getByRole("button", { name: "1 matches in Ghoul" });
  expect(trigger.closest("a")).toBeNull();
  fireEvent.click(trigger);
  await waitFor(() =>
    expect(screen.getByRole("link", { name: "Jaws" })).toBeInTheDocument(),
  );
  fireEvent.keyDown(document, { key: "Escape" });
  expect(trigger).toHaveFocus();
  expect(trigger).toHaveAttribute("aria-expanded", "false");
  expect(getDetail).not.toHaveBeenCalled();
});
it("keeps typed inline activities in their parent but excludes spells and precise passages", () => {
  const parent = detailFixture("actors:ghoul");
  parent.presentation.body = {
    kind: "creature",
    value: {
      ...actorFixture(),
      activities: knownFact([activityFixture("Jaws", "actors:ghoul")]),
    },
  };
  expect(parentShowsActivity(parent, navigation)).toBe(true);
  expect(
    parentShowsActivity(parent, { ...navigation, field: "/system/description/value" }),
  ).toBe(false);
  const other = structuredClone(parent) as RecordDetailView;
  if (other.presentation.body.kind === "creature")
    other.presentation.body.value.activities.value![0].kind = "spell";
  expect(parentShowsActivity(other, navigation)).toBe(false);
});

it.each([true, false])(
  "snapshots only visible scroll hosts before navigation (desktop=%s)",
  async (desktop) => {
    vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ lg: desktop });
    const positions = {
      readList: () => 0,
      saveList: vi.fn(),
      readReader: () => 0,
      saveReader: vi.fn(),
      readReaderFocus: vi.fn(),
      saveReaderFocus: vi.fn(),
    };
    renderBrowser(positions);
    fireEvent.click(screen.getByRole("link", { name: /Ghoul/ }));
    await waitFor(() => expect(getDetail).toHaveBeenCalledTimes(1));
    const list = document.querySelector(".reference-results__scroll")!;
    const reader = document.querySelector(".reference-reader__scroll")!;
    list.scrollTop = 211;
    reader.scrollTop = 400;
    fireEvent(window, new Event(ATLAS_ROUTE_CHANGE_EVENT));
    if (desktop) expect(positions.saveList).toHaveBeenLastCalledWith(211);
    else expect(positions.saveList).not.toHaveBeenCalled();
    expect(positions.saveReader).toHaveBeenLastCalledWith(
      expect.stringContaining("actors:ghoul"),
      400,
    );
  },
);

it.each([true, false])(
  "restores prior reader scroll/focus and mobile result focus without changing the selected row (desktop=%s)",
  async (desktop) => {
    vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ lg: desktop });
    getDetail.mockImplementation((key) =>
      Promise.resolve(
        detailFixture(
          key,
          key === "actors:ghoul" ? "Ghoul" : "Linked rule",
          key === "actors:ghoul" ? "rules:linked" : undefined,
        ),
      ),
    );
    renderBrowser();
    const summary = screen.getByRole("link", { name: /Ghoul/ });
    const list = document.querySelector<HTMLElement>(".reference-results__scroll")!;
    list.scrollTop = 230;
    fireEvent.scroll(list);
    fireEvent.keyDown(summary, { key: " " });
    const nested = await screen.findByRole("link", { name: "Nested Rule" });
    const reader = document.querySelector<HTMLElement>(".reference-reader__scroll")!;
    reader.scrollTop = 425;
    fireEvent.scroll(reader);
    nested.focus();
    fireEvent.click(nested);
    await screen.findByRole("heading", { name: "Linked rule" });
    expect(summary).toHaveAttribute("aria-current", "true");
    expect(reader.scrollTop).toBe(0);
    fireEvent.click(screen.getByRole("button", { name: "Back" }));
    const returned = await screen.findByRole("link", { name: "Nested Rule" });
    await waitFor(() => expect(reader.scrollTop).toBe(425));
    expect(returned).toHaveFocus();
    if (!desktop) {
      fireEvent.click(screen.getByRole("button", { name: "Results" }));
      await waitFor(() => expect(summary).toHaveFocus());
      expect(list.scrollTop).toBe(230);
    }
  },
);
