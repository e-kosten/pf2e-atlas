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
import { App, ConfigProvider } from "antd";
import { useContext, useState } from "react";
import type {
  PcAbilityView,
  PcReferenceView,
  SearchResultsPage,
} from "../../generated/atlas";
import { SearchProfileContext } from "../../shared/search/SearchProfileContext";
import { detailFixture, summaryFixture } from "../../test/fixtures";
import { RouterTestContext } from "../../test/router";
import { CharacterReference } from "./PcCharacterDrawer";

const api = vi.hoisted(() => ({
  getCharacter: vi.fn(),
  searchPage: vi.fn(),
  getRecordDetail: vi.fn(),
  savePcAbilityLink: vi.fn(),
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
};
const character: PcReferenceView = {
  character_key: "pc",
  character_revision: 4,
  name: "Simon",
  level: 7,
  imported: true,
  abilities: [ability],
  sections: [{ kind: "feat", label: "Feats", abilities: [ability] }],
  parties: [],
  resolved_count: 0,
  unresolved_count: 1,
};
const recordKey = "feats:companion0000001";
const recordTitle = "Mature Animal Companion (Ranger)";
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
function Harness({ ready = true }: { ready?: boolean }) {
  const context = useContext(SearchProfileContext);
  const [client] = useState(
    () =>
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
              <CharacterReference
                location={{ kind: "character", characterKey: "pc" }}
                name="Simon"
                mode="page"
              />
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
  api.getRecordDetail.mockResolvedValue(detailFixture(recordKey, recordTitle));
});
afterEach(cleanup);

it("keeps the linking task visible through preparing, loading, populated and empty searches", async () => {
  let finish!: (value: SearchResultsPage) => void;
  api.searchPage.mockReturnValueOnce(
    new Promise<SearchResultsPage>((resolve) => {
      finish = resolve;
    }),
  );
  const view = render(<Harness ready={false} />);
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
  expect(await task.findByRole("button", { name: recordTitle })).toBeVisible();
  expect(task.getByText("Player Core")).toBeVisible();
  expect(task.getByText("Feat Level 6")).toBeVisible();
  api.searchPage.mockResolvedValueOnce(page(0));
  const input = task.getByRole("textbox", { name: "Search Atlas rules" });
  fireEvent.change(input, { target: { value: "Another name" } });
  fireEvent.click(task.getByRole("button", { name: "Search" }));
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
  const filter = await screen.findByRole("textbox", { name: "Find an ability" });
  fireEvent.change(filter, { target: { value: "Companion" } });
  const opener = screen.getByRole("button", {
    name: "Search for Mature Animal Companion",
  });
  fireEvent.click(opener);
  expect(await screen.findByText("Could not search Atlas")).toBeVisible();
  const search = screen.getByRole("textbox", { name: "Search Atlas rules" });
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  expect(await screen.findByRole("button", { name: recordTitle })).toBeVisible();
  expect(search).toHaveValue(ability.label);
  fireEvent.click(screen.getByRole("button", { name: "Back to abilities" }));
  expect(filter).toHaveValue("Companion");
  await waitFor(() => expect(opener).toHaveFocus());
});

it("returns from a candidate to the same query and result, then saves the guarded rule link", async () => {
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
    abilities: [linkedAbility],
    sections: [{ kind: "feat", label: "Feats", abilities: [linkedAbility] }],
    resolved_count: 1,
    unresolved_count: 0,
  };
  api.savePcAbilityLink.mockResolvedValue(linkedCharacter);
  render(<Harness />);
  fireEvent.click(
    await screen.findByRole("button", { name: "Search for Mature Animal Companion" }),
  );
  const result = await screen.findByRole("button", { name: recordTitle });
  fireEvent.click(result);
  expect(await screen.findByRole("button", { name: "Link this rule" })).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Back to results" }));
  expect(screen.getByRole("textbox", { name: "Search Atlas rules" })).toHaveValue(
    ability.label,
  );
  await waitFor(() => expect(result).toHaveFocus());
  fireEvent.click(result);
  const link = await screen.findByRole("button", { name: "Link this rule" });
  await waitFor(() => expect(link).toBeEnabled());
  api.getCharacter.mockResolvedValue(linkedCharacter);
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
});

it("does not announce a cached empty result as final while its refresh is pending", async () => {
  api.searchPage.mockResolvedValueOnce(page(0));
  render(<Harness />);
  const opener = await screen.findByRole("button", {
    name: "Search for Mature Animal Companion",
  });
  fireEvent.click(opener);
  expect(
    await screen.findByText("No rules found. Try a different name."),
  ).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Back to abilities" }));
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
  expect(await screen.findByRole("button", { name: recordTitle })).toBeVisible();
});
