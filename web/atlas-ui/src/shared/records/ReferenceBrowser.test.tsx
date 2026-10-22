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
import { ConfigProvider, Grid } from "antd";
import { readingScrollGeometry } from "../../test/readingScrollGeometry";
import { StrictMode, useState } from "react";
import { getEmbeddedReading, getRecordDetail } from "../../api/atlasApi";
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
  EmbeddedReadingBatchView,
  RecordDetailView,
  SearchResultRow,
  RelationshipClauseSummaryView,
} from "../../generated/atlas";

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getRecordDetail: vi.fn(),
  getEmbeddedReading: vi.fn(),
}));
const getDetail = vi.mocked(getRecordDetail);
const getAbilities = vi.mocked(getEmbeddedReading);
const navigation = activityFixture("Jaws", "actors:ghoul").navigation;
const rows: SearchResultRow[] = [
  {
    record: summaryFixture("actors:ghoul", "Ghoul"),
    relationship_evidence: [],
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
  {
    record: summaryFixture("spells:heal", "Heal"),
    matches: [],
    relationship_evidence: [],
  },
];
function Harness({
  positions,
  resultRows = rows,
  relationships,
  generation,
}: {
  positions?: BrowsePositions;
  resultRows?: SearchResultRow[];
  relationships?: RelationshipClauseSummaryView[];
  generation?: string;
}) {
  const [selected, select] = useState<string | null>(null);
  const reading = useReferenceReading();
  return (
    <ReferenceBrowser
      positions={positions}
      rows={resultRows}
      relationshipClauses={relationships}
      filter={
        relationships?.length
          ? {
              kind: "all_of",
              children: relationships.map((entry) => ({
                kind: "relationship",
                clause_id: entry.clause_id,
                relationship: entry.relationship,
              })),
            }
          : null
      }
      generation={generation}
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
  "retains landed reading while an inline preview opens after %s content is ready",
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
    const batch: EmbeddedReadingBatchView = {
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
      let finish: (() => void) | undefined;
      if (delayed) {
        const pending = () =>
          new Promise<EmbeddedReadingBatchView>((resolve) => {
            finish = () => resolve(batch);
          });
        if (ability) {
          if (scenario === "retry ability")
            getAbilities.mockRejectedValue(new Error("Ability read rejected"));
          else getAbilities.mockImplementationOnce(pending);
        } else {
          getDetail.mockImplementationOnce(
            () =>
              new Promise((resolve) => {
                finish = () => resolve(parent);
              }),
          );
        }
        await chooseMatch();
        if (scenario === "retry ability") {
          await screen.findByText("Ability read rejected");
          getAbilities.mockImplementation(pending);
          fireEvent.click(screen.getByRole("button", { name: "Retry" }));
        }
        await waitFor(() => expect(finish).toBeDefined());
        await act(async () => finish!());
      } else await chooseMatch();
      const nested = await screen.findByRole("link", { name: "Nested Rule" });
      await waitFor(() => expect(reader.scrollTop).toBe(100));
      reader.scrollTop = 425;
      fireEvent.scroll(reader);
      nested.focus();
      const parentReads = getDetail.mock.calls.filter(
        ([key]) => key === "actors:ghoul",
      ).length;
      const abilityReads = getAbilities.mock.calls.length;
      fireEvent.click(nested);
      await screen.findByRole("heading", { name: "Linked rule" });
      expect(reader.scrollTop).toBe(425);
      expect(nested).toBeInTheDocument();
      fireEvent.click(screen.getByRole("button", { name: "Close reference preview" }));
      await waitFor(() => expect(nested).toHaveFocus());
      expect(reader.scrollTop).toBe(425);
      expect(
        getDetail.mock.calls.filter(([key]) => key === "actors:ghoul"),
      ).toHaveLength(parentReads);
      expect(getAbilities).toHaveBeenCalledTimes(abilityReads);
      expect(document.documentElement.scrollTop).toBe(0);
      if (mobile) {
        fireEvent.click(screen.getByRole("button", { name: "Results" }));
        await waitFor(() =>
          expect(
            screen.getByRole("button", { name: "1 matches in Ghoul" }),
          ).toHaveFocus(),
        );
      }
      // A fresh match selection still lands at its exact activity.
      await chooseMatch();
      await waitFor(() => expect(reader.scrollTop).toBe(100));
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
  "preserves reader scroll and returns preview focus without changing the selected row (desktop=%s)",
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
    expect(reader.scrollTop).toBe(425);
    fireEvent.click(screen.getByRole("button", { name: "Close reference preview" }));
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

it("combines text and relationship evidence in the existing Matches popover and restores its trigger on Escape", async () => {
  const target = {
    record_key: "items:armor",
    embedded_path: [],
    field: null,
    passage: null,
    source_fingerprint: null,
  };
  const constraint: RelationshipClauseSummaryView = {
    clause_id: "link",
    relationship: { direction: "incoming", selection: target, include_contents: true },
    title: "Leather Armor",
    record: summaryFixture("items:armor", "Leather Armor"),
  };
  const row: SearchResultRow = {
    ...rows[0],
    relationship_evidence: [
      {
        clause_id: "link",
        page: {
          number: 1,
          size: 3,
          count: 1,
          total: 1n,
          has_more: false,
          next_page: null,
        },
        occurrences: [
          {
            source: { record: "actors:ghoul", embedded_path: [], field: "description" },
            source_navigation: navigation,
            ordinal: 0,
            origin: "prepared_content",
            kind: "uuid",
            authored_target: null,
            occurrence_path: null,
            audiences: [],
            availability: null,
            status: "resolved_record",
            target,
            url: null,
            effect_label: null,
            source_title: "Ghoul equipment",
            target_title: "Leather Armor",
          },
        ],
      },
    ],
  };
  const view = render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <Harness
          resultRows={[row]}
          relationships={[constraint]}
          generation="generation-a"
        />
      </QueryClientProvider>
    </ConfigProvider>,
  );
  const trigger = screen.getByRole("button", { name: "2 matches in Ghoul" });
  expect(screen.getAllByText("Matches")).toHaveLength(1);
  expect(screen.queryByText("1 link")).toBeNull();
  expect(screen.queryByRole("link", { name: "Ghoul equipment" })).toBeNull();
  fireEvent.click(trigger);
  expect(
    await screen.findByRole("link", { name: "Ghoul equipment" }),
  ).toBeInTheDocument();
  expect(screen.getByRole("link", { name: "Jaws" })).toBeInTheDocument();
  expect(screen.getByRole("link", { name: "Leather Armor" })).toBeInTheDocument();
  expect(view.container).not.toHaveTextContent("Text reference");
  fireEvent.keyDown(document, { key: "Escape" });
  await waitFor(() => expect(trigger).toHaveFocus());
  expect(trigger).toHaveAttribute("aria-expanded", "false");
});
