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
import { App, ConfigProvider, Grid } from "antd";
import { AtlasApiError } from "../../api/apiError";
import { useContext, useState, type ReactNode } from "react";
import type {
  PathbuilderPreviewView,
  PcAbilityView,
  PcReferenceView,
  RecordDetailView,
  SearchResultsPage,
} from "../../generated/atlas";
import { SavedWritesPaused } from "../../shared/feedback/SavedWritesPaused";
import { SearchProfileContext } from "../../shared/search/SearchProfileContext";
import { detailFixture, summaryFixture } from "../../test/fixtures";
import { RouterTestContext } from "../../test/router";
import { CharacterPage } from "./CharacterPage";
import { characterQueryKey } from "./characterQueries";
import type { AbilityView } from "./PcAbilityBrowser";
import { PcCharacterDrawer } from "./PcCharacterDrawer";
import { PathbuilderImport } from "./PathbuilderImport";

const api = vi.hoisted(() => ({
  getCharacter: vi.fn(),
  getCharacterSheet: vi.fn(),
  getEncounterPcReference: vi.fn(),
  searchPage: vi.fn(),
  getRecordDetail: vi.fn(),
  savePcAbilityLink: vi.fn(),
  previewPathbuilderPc: vi.fn(),
  savePathbuilderPc: vi.fn(),
}));
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  ...api,
}));
const ability: PcAbilityView = {
  ability_key: "companion",
  label: "Mature Animal Companion",
  kind: "feat",
  ambiguous: false,
  manual: false,
  automatic_match_available: false,
};
const character: PcReferenceView = {
  character_key: "pc",
  character_revision: 4,
  name: "Simon",
  level: 7,
  imported: true,
  sections: [{ kind: "feat", label: "Feats", abilities: [ability] }],
  parties: [],
};
const recordKey = "feats:companion0000001";
const recordTitle = "Mature Animal Companion (Ranger)";
const resultName = /^Mature Animal Companion \(Ranger\)/;
function page(count = 1): SearchResultsPage {
  return {
    search_content: "all",
    ranking: "score",
    search_profile: "bge-base",
    artifact_generation: "test-generation",
    sort: { kind: "relevance" },
    mode: { kind: "text_search", query: ability.label, mode: "lexical" },
    coverage: null,
    relationship_clauses: [],
    rows: count
      ? [
          {
            record: {
              ...summaryFixture(recordKey, recordTitle),
              kind: "feat",
              kind_label: "Feat",
              level_label: "Level 6",
              publication: "Player Core",
              edition: "remaster",
            },
            matches: [],
            relationship_evidence: [],
          },
        ]
      : [],
    page: {
      number: 1,
      size: 10,
      count,
      total: count,
      has_more: false,
      next_page: null,
    },
  };
}
function Harness({
  client: suppliedClient,
  ready = true,
  children,
}: {
  client?: QueryClient;
  ready?: boolean;
  children?: ReactNode;
}) {
  const context = useContext(SearchProfileContext);
  const [view, setView] = useState<AbilityView | null>(null);
  const [client] = useState(
    () =>
      suppliedClient ??
      new QueryClient({
        defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
      }),
  );
  return (
    <QueryClientProvider client={client}>
      <ConfigProvider theme={{ token: { motion: false } }}>
        <RouterTestContext>
          <SearchProfileContext.Provider
            value={{ ...context, keywordSearch: ready, loading: !ready }}
          >
            <App>
              {children ?? (
                <CharacterPage
                  characterKey="pc"
                  navigation={{ view, open: setView, close: () => setView(null) }}
                />
              )}
            </App>
          </SearchProfileContext.Provider>
        </RouterTestContext>
      </ConfigProvider>
    </QueryClientProvider>
  );
}
beforeEach(() => {
  for (const mock of Object.values(api)) mock.mockReset();
  api.getCharacter.mockResolvedValue(character);
  api.getCharacterSheet.mockResolvedValue({
    character_key: character.character_key,
    character_revision: character.character_revision,
    sheet: {
      name: character.name,
      level: character.level,
      has_additional_details: false,
      sections: [],
    },
  });
  api.getEncounterPcReference.mockResolvedValue({ reference: character });
  api.getRecordDetail.mockResolvedValue(detailFixture(recordKey, recordTitle));
});
afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

it("offers first import for an abilities-only character after the sheet has loaded", async () => {
  let fail!: (error: Error) => void;
  api.getCharacterSheet
    .mockReturnValueOnce(
      new Promise((_, reject) => {
        fail = reject;
      }),
    )
    .mockResolvedValueOnce({
      character_key: character.character_key,
      character_revision: character.character_revision,
    });
  const view = render(
    <SavedWritesPaused.Provider value={false}>
      <Harness />
    </SavedWritesPaused.Provider>,
  );
  await waitFor(() => expect(api.getCharacterSheet).toHaveBeenCalledOnce());
  const button = () =>
    screen.queryByRole("button", { name: "Import from Pathbuilder" });
  expect(button()).not.toBeInTheDocument();
  await act(async () => fail(new Error("Sheet unavailable")));
  const retry = await screen.findByRole("button", { name: "Retry" });
  expect(button()).not.toBeInTheDocument();
  fireEvent.click(retry);
  await waitFor(() => expect(button()).toBeEnabled());
  view.rerender(
    <SavedWritesPaused.Provider value>
      <Harness />
    </SavedWritesPaused.Provider>,
  );
  expect(button()).toBeDisabled();
});

it.each([
  { savedName: "Simon", savedLevel: 7, provenance: null },
  { savedName: "Alias", savedLevel: 7, provenance: "Imported as Simon" },
  { savedName: "Alias", savedLevel: 8, provenance: "Imported as Simon · Level 7" },
  { savedName: "Simon", savedLevel: undefined, provenance: "Imported level 7" },
])(
  "shows imported identity differences for $savedName at level $savedLevel",
  async ({ savedName, savedLevel, provenance }) => {
    api.getCharacter.mockResolvedValueOnce({
      ...character,
      name: savedName,
      level: savedLevel,
    });
    render(<Harness />);
    const stats = within(await screen.findByRole("tabpanel", { name: "Stats" }));
    await stats.findByText("This export has no stats to show.");
    const attribution = stats.queryByText(/^Imported\b/);
    if (provenance === null) expect(attribution).not.toBeInTheDocument();
    else expect(attribution).toHaveTextContent(new RegExp(`^${provenance}$`));
  },
);

it("keeps preparations open for the same caster across a revision refresh and resets them for a different character", async () => {
  const client = new QueryClient();
  const next = { ...character, character_key: "next", name: "New character" };
  const magicSheet = (pc: PcReferenceView) => ({
    character_key: pc.character_key,
    character_revision: pc.character_revision,
    sheet: {
      name: pc.name,
      level: 7,
      has_additional_details: false,
      sections: [
        {
          kind: "magic",
          groups: ["Alarm", "Fear"].map((label) => ({
            label: "Wizard",
            rows: [],
            spell_ranks: [
              {
                label: "Rank 1",
                known: [{ label, count: 1 }],
                prepared: [{ label, count: 2 }],
              },
            ],
          })),
        },
      ],
    },
  });
  api.getCharacterSheet.mockResolvedValueOnce(magicSheet(character));
  const pageFor = (key: string) => (
    <Harness client={client}>
      <CharacterPage
        characterKey={key}
        navigation={{ view: null, open: vi.fn(), close: vi.fn() }}
      />
    </Harness>
  );
  const view = render(pageFor(character.character_key));
  const panels = () => screen.getAllByRole("button", { name: "Prepared at export" });
  fireEvent.click(await screen.findByRole("tab", { name: "Magic" }));
  fireEvent.keyDown(panels()[1], { key: " " });
  expect(panels()[0]).toHaveAttribute("aria-expanded", "false");
  expect(panels()[1]).toHaveAttribute("aria-expanded", "true");

  const revised = { ...character, character_revision: 5 };
  let finish!: (value: ReturnType<typeof magicSheet>) => void;
  api.getCharacter.mockResolvedValueOnce(revised);
  api.getCharacterSheet.mockReturnValueOnce(
    new Promise((resolve) => {
      finish = resolve;
    }),
  );
  await act(async () => {
    await client.invalidateQueries({
      queryKey: characterQueryKey(character.character_key),
    });
  });
  await waitFor(() =>
    expect(screen.queryByRole("tab", { name: "Magic" })).not.toBeInTheDocument(),
  );
  await act(async () => finish(magicSheet(revised)));
  await screen.findByRole("tabpanel", { name: "Magic" });
  expect(panels()[0]).toHaveAttribute("aria-expanded", "false");
  expect(panels()[1]).toHaveAttribute("aria-expanded", "true");

  api.getCharacter.mockResolvedValueOnce(next);
  api.getCharacterSheet.mockResolvedValueOnce(magicSheet(next));
  view.rerender(pageFor(next.character_key));
  fireEvent.click(await screen.findByRole("tab", { name: "Magic" }));
  for (const panel of panels()) expect(panel).toHaveAttribute("aria-expanded", "false");
});

async function openAbilities() {
  fireEvent.click(await screen.findByRole("tab", { name: "Abilities" }));
}

it("keeps the linking task visible through preparing, loading, populated and empty searches", async () => {
  let finish!: (value: SearchResultsPage) => void;
  api.searchPage.mockReturnValueOnce(
    new Promise<SearchResultsPage>((resolve) => {
      finish = resolve;
    }),
  );
  const view = render(<Harness ready={false} />);
  await openAbilities();
  fireEvent.click(
    await screen.findByRole("button", { name: "Search for Mature Animal Companion" }),
  );
  const task = within(screen.getByRole("region", { name: "Ability search" }));
  expect(
    task.getByRole("heading", { name: "Find rule for Mature Animal Companion" }),
  ).toBeVisible();
  await waitFor(() => expect(task.getByText("Preparing rule search…")).toBeVisible());
  expect(task.queryByText(/No rules found/)).toBeNull();
  expect(api.searchPage).not.toHaveBeenCalled();
  view.rerender(<Harness />);
  await waitFor(() => expect(api.searchPage).toHaveBeenCalledOnce());
  expect(task.queryByText(/No rules found/)).toBeNull();
  expect(screen.queryByRole("link", { name: "Open full search" })).toBeNull();
  await act(async () => {
    finish(page());
  });
  expect(await task.findByRole("link", { name: resultName })).toBeVisible();
  expect(task.getByText("Player Core")).toBeVisible();
  expect(task.getByText("Feat 6")).toBeVisible();
  api.searchPage.mockResolvedValueOnce(page(0));
  const input = task.getByRole("searchbox", { name: "Search Atlas rules" });
  fireEvent.change(input, { target: { value: "Another name" } });
  fireEvent.keyDown(input, { key: "Enter", keyCode: 13 });
  expect(await task.findByText("No rules found. Try a different name.")).toBeVisible();
  expect(input).toHaveValue("Another name");
  expect(api.searchPage).toHaveBeenLastCalledWith(
    expect.objectContaining({
      criteria: expect.objectContaining({ query: "Another name", mode: "lexical" }),
      page: { number: 1, size: 10 },
    }),
    expect.any(AbortSignal),
  );
});

it("retries a failed rule search in place and restores the ability filter and opener", async () => {
  api.searchPage
    .mockRejectedValueOnce(new Error("Offline"))
    .mockResolvedValueOnce(page());
  render(<Harness />);
  await openAbilities();
  const filter = await screen.findByRole("textbox", { name: "Find an ability" });
  fireEvent.change(filter, { target: { value: "Companion" } });
  const opener = screen.getByRole("button", {
    name: "Search for Mature Animal Companion",
  });
  fireEvent.click(opener);
  expect(await screen.findByText("Could not search Atlas")).toBeVisible();
  const search = screen.getByRole("searchbox", { name: "Search Atlas rules" });
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  expect(await screen.findByRole("link", { name: resultName })).toBeVisible();
  expect(search).toHaveValue(ability.label);
  fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  expect(filter).toHaveValue("Companion");
  await waitFor(() => expect(opener).toHaveFocus());
});

it("keeps the current rule and footer mounted until the next result is prepared", async () => {
  vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ lg: true });
  const first = page();
  const other = summaryFixture("feats:companion0000002", "Another companion rule");
  api.searchPage.mockResolvedValue({
    ...first,
    rows: [...first.rows, { record: other, matches: [], relationship_evidence: [] }],
  });
  let finish!: (detail: RecordDetailView) => void;
  let pending = true;
  api.getRecordDetail.mockImplementation((key: string) => {
    if (key !== other.record_key)
      return Promise.resolve(detailFixture(recordKey, recordTitle));
    if (!pending) return Promise.resolve(detailFixture(other.record_key, other.title));
    pending = false;
    return new Promise<RecordDetailView>((resolve) => {
      finish = resolve;
    });
  });
  render(<Harness />);
  await openAbilities();
  fireEvent.click(
    await screen.findByRole("button", { name: "Search for Mature Animal Companion" }),
  );
  fireEvent.click(await screen.findByRole("link", { name: resultName }));
  const heading = await screen.findByRole("heading", { name: recordTitle });
  const link = screen.getByRole("button", { name: "Link this rule" });
  await waitFor(() => expect(link).toBeEnabled());
  fireEvent.click(screen.getByRole("link", { name: /^Another companion rule/ }));
  expect(heading).toBeVisible();
  expect(link).toBeDisabled();
  expect(screen.queryByText("Loading Another companion rule…")).toBeNull();
  expect(await screen.findByText("Loading Another companion rule…")).toBeVisible();
  expect(heading).toBeVisible();
  expect(link).toBeDisabled();
  await act(async () => finish(detailFixture(other.record_key, other.title)));
  expect(await screen.findByRole("heading", { name: other.title })).toBeVisible();
  expect(screen.getByRole("button", { name: "Link this rule" })).toBe(link);
  await waitFor(() => expect(link).toBeEnabled());
});

it.each(["character page", "encounter drawer"])(
  "returns to the same result, then saves a guarded rule link in the %s",
  async (host) => {
    vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ lg: false });
    api.searchPage.mockResolvedValue(page());
    const linkedAbility = {
      ...ability,
      manual: true,
      record_key: recordKey,
      record_title: recordTitle,
    };
    const linkedCharacter = {
      ...character,
      character_revision: 5,
      sections: [{ kind: "feat", label: "Feats", abilities: [linkedAbility] }],
    };
    api.savePcAbilityLink.mockResolvedValue(linkedCharacter);
    render(
      <Harness>
        {host === "encounter drawer" ? (
          <PcCharacterDrawer
            location={{
              encounterKey: "encounter",
              participantKey: "participant",
              revision: 2,
            }}
            name="Simon"
          />
        ) : undefined}
      </Harness>,
    );
    if (host === "character page") await openAbilities();
    fireEvent.click(
      await screen.findByRole("button", { name: "Search for Mature Animal Companion" }),
    );
    const result = await screen.findByRole("link", { name: resultName });
    fireEvent.keyDown(result, { key: "Enter" });
    expect(await screen.findByRole("button", { name: "Link this rule" })).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Results" }));
    expect(screen.getByRole("searchbox", { name: "Search Atlas rules" })).toHaveValue(
      ability.label,
    );
    await waitFor(() => expect(result).toHaveFocus());
    fireEvent.click(result);
    const link = await screen.findByRole("button", { name: "Link this rule" });
    await waitFor(() => expect(link).toBeEnabled());
    api.getCharacter.mockResolvedValue(linkedCharacter);
    api.getEncounterPcReference.mockResolvedValue({ reference: linkedCharacter });
    fireEvent.click(link);
    await waitFor(() =>
      expect(api.savePcAbilityLink).toHaveBeenCalledWith("pc", "companion", {
        expected_character_revision: 4,
        selection: {
          kind: "manual",
          record_key: recordKey,
          expected_artifact_generation: "test-generation",
        },
      }),
    );
    await waitFor(() =>
      expect(screen.queryByRole("region", { name: "Ability search" })).toBeNull(),
    );
    await waitFor(() =>
      expect(screen.getByRole("button", { name: ability.label })).toHaveFocus(),
    );
    if (host === "encounter drawer") expect(api.getCharacter).not.toHaveBeenCalled();
  },
);

it("does not announce a cached empty result as final while its refresh is pending", async () => {
  api.searchPage.mockResolvedValueOnce(page(0));
  render(<Harness />);
  await openAbilities();
  const opener = await screen.findByRole("button", {
    name: "Search for Mature Animal Companion",
  });
  fireEvent.click(opener);
  expect(
    await screen.findByText("No rules found. Try a different name."),
  ).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  let finish!: (value: SearchResultsPage) => void;
  api.searchPage.mockReturnValueOnce(
    new Promise<SearchResultsPage>((resolve) => {
      finish = resolve;
    }),
  );
  fireEvent.click(opener);
  await waitFor(() => expect(api.searchPage).toHaveBeenCalledTimes(2));
  expect(screen.queryByText("No rules found. Try a different name.")).toBeNull();
  await act(async () => {
    finish(page());
  });
  expect(await screen.findByRole("link", { name: resultName })).toBeVisible();
});

it.each(["cancel", "retry", "new generation"] as const)(
  "stages an ambiguous rule link until import submission (%s)",
  async (outcome) => {
    const ambiguous = { ...ability, ambiguous: true };
    const draft: PathbuilderPreviewView = {
      ...character,
      level: 7,
      sheet: {
        name: character.name,
        level: 7,
        has_additional_details: false,
        sections: [],
      },
      sections: [{ kind: "feat", label: "Feats", abilities: [ambiguous] }],
      artifact_generation: "test-generation",
    };
    let failReview!: (error: Error) => void;
    api.previewPathbuilderPc
      .mockResolvedValueOnce(draft)
      .mockResolvedValue({ ...draft, artifact_generation: "new-generation" });
    api.searchPage.mockResolvedValue(page());
    api.savePathbuilderPc
      .mockRejectedValueOnce(
        outcome === "new generation"
          ? new AtlasApiError(409, "Generation changed")
          : new Error("Response lost"),
      )
      .mockResolvedValueOnce({ character_key: "pc", available: true });
    const closed = vi.fn();
    render(
      <Harness>
        <PathbuilderImport onClose={closed} />
      </Harness>,
    );
    fireEvent.change(screen.getByLabelText("Pathbuilder JSON"), {
      target: { value: "{}" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Continue" }));

    expect(await screen.findByText("Multiple matches")).toBeVisible();
    fireEvent.click(
      screen.getByRole("button", { name: "Link rule for Mature Animal Companion" }),
    );
    fireEvent.click(await screen.findByRole("link", { name: resultName }));
    const link = await screen.findByRole("button", { name: "Link this rule" });
    await waitFor(() => expect(link).toBeEnabled());
    fireEvent.click(link);
    expect(await screen.findByText("1 linked · 0 unlinked")).toBeVisible();
    expect(api.savePcAbilityLink).not.toHaveBeenCalled();
    expect(api.savePathbuilderPc).not.toHaveBeenCalled();

    if (outcome === "cancel") {
      fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
      fireEvent.click(await screen.findByRole("button", { name: "Discard" }));
      await waitFor(() => expect(closed).toHaveBeenCalledOnce());
      expect(api.savePathbuilderPc).not.toHaveBeenCalled();
    } else if (outcome === "new generation") {
      api.previewPathbuilderPc.mockReturnValueOnce(
        new Promise<PathbuilderPreviewView>((_resolve, reject) => {
          failReview = reject;
        }),
      );
      fireEvent.click(screen.getByRole("button", { name: "Add character" }));
      const review = screen.getByRole("region", { name: "Import review" });
      fireEvent.click(
        await screen.findByRole("button", { name: "Review latest changes" }),
      );
      await waitFor(() => expect(api.previewPathbuilderPc).toHaveBeenCalledTimes(2));
      expect(review).toBeVisible();
      expect(within(review).getByText("1 linked · 0 unlinked")).toBeVisible();
      expect(screen.getByRole("button", { name: "Add character" })).toBeDisabled();
      await act(async () => failReview(new Error("Review failed")));
      const retry = await screen.findByRole("button", { name: "Retry review" });
      expect(within(review).getByText("1 linked · 0 unlinked")).toBeVisible();
      expect(screen.getByRole("button", { name: "Add character" })).toBeDisabled();
      fireEvent.click(retry);
      expect(
        await screen.findByText("Atlas rules changed. Choose your draft links again."),
      ).toBeVisible();
      expect(screen.getByText("0 linked · 1 unlinked")).toBeVisible();
      fireEvent.click(screen.getByRole("button", { name: "Add character" }));
      await waitFor(() => expect(api.savePathbuilderPc).toHaveBeenCalledTimes(2));
      expect(api.savePathbuilderPc.mock.calls[1][0]).toMatchObject({
        expected_artifact_generation: "new-generation",
      });
      expect(api.savePathbuilderPc.mock.calls[1][0].rule_links).toBeUndefined();
    } else {
      fireEvent.click(screen.getByRole("button", { name: "Add character" }));
      const retry = await screen.findByRole("button", { name: "Retry to confirm" });
      expect(
        screen.getByRole("button", { name: /Change link for Mature/ }),
      ).toBeDisabled();
      expect(screen.getByRole("button", { name: "Edit import" })).toBeDisabled();
      fireEvent.click(retry);
      await waitFor(() => expect(api.savePathbuilderPc).toHaveBeenCalledTimes(2));
      const firstRequest = api.savePathbuilderPc.mock.calls[0][0];
      expect(firstRequest).toMatchObject({
        json: "{}",
        rule_links: { companion: recordKey },
        expected_artifact_generation: "test-generation",
      });
      expect(api.savePathbuilderPc.mock.calls[1][0]).toEqual(firstRequest);
    }
  },
);

it("keeps rule review from replacing an unconfirmed import request", async () => {
  api.previewPathbuilderPc.mockResolvedValue({
    name: character.name,
    level: 7,
    sheet: {
      name: character.name,
      level: 7,
      has_additional_details: false,
      sections: [],
    },
    sections: character.sections,
    artifact_generation: "older-generation",
  });
  api.searchPage.mockResolvedValue(page());
  api.savePathbuilderPc
    .mockRejectedValueOnce(new Error("Response lost"))
    .mockResolvedValueOnce({ character_key: "pc", available: true });
  render(
    <Harness>
      <PathbuilderImport onClose={vi.fn()} />
    </Harness>,
  );
  fireEvent.change(screen.getByLabelText("Pathbuilder JSON"), {
    target: { value: "{}" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Continue" }));
  fireEvent.click(
    await screen.findByRole("button", {
      name: "Link rule for Mature Animal Companion",
    }),
  );
  fireEvent.click(await screen.findByRole("link", { name: resultName }));
  const link = await screen.findByRole("button", { name: "Link this rule" });
  await waitFor(() => expect(link).toBeEnabled());
  fireEvent.click(link);
  const review = await screen.findByRole("button", { name: "Review latest changes" });
  expect(review).toBeEnabled();
  fireEvent.click(screen.getByRole("button", { name: "Add character" }));
  const retry = await screen.findByRole("button", { name: "Retry to confirm" });
  expect(review).toBeDisabled();
  fireEvent.click(retry);
  await waitFor(() => expect(api.savePathbuilderPc).toHaveBeenCalledTimes(2));
  expect(api.savePathbuilderPc.mock.calls[1]).toEqual(
    api.savePathbuilderPc.mock.calls[0],
  );
  expect(api.previewPathbuilderPc).toHaveBeenCalledOnce();
});
