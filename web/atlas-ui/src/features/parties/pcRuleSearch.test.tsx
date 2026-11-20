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
import { AtlasApiError } from "../../api/apiError";
import { useContext, useState, type ReactNode } from "react";
import type {
  PcAbilityView,
  PcReferenceView,
  SearchResultsPage,
} from "../../generated/atlas";
import { SearchProfileContext } from "../../shared/search/SearchProfileContext";
import { detailFixture, summaryFixture } from "../../test/fixtures";
import { RouterTestContext } from "../../test/router";
import { CharacterReference } from "./PcCharacterDrawer";
import { PathbuilderImport } from "./PathbuilderImport";

const api = vi.hoisted(() => ({
  getCharacter: vi.fn(),
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
function Harness({
  ready = true,
  children,
}: {
  ready?: boolean;
  children?: ReactNode;
}) {
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
              {children ?? (
                <CharacterReference
                  location={{ kind: "character", characterKey: "pc" }}
                  name="Simon"
                  mode="page"
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

it.each(["cancel", "retry", "new generation"] as const)(
  "stages an ambiguous rule link until import submission (%s)",
  async (outcome) => {
    const ambiguous = { ...ability, ambiguous: true };
    const draft = {
      ...character,
      abilities: [ambiguous],
      sections: [{ kind: "feat", label: "Feats", abilities: [ambiguous] }],
      retained_manual_count: 0,
      artifact_generation: "test-generation",
    };
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
    fireEvent.click(await screen.findByRole("button", { name: recordTitle }));
    fireEvent.click(await screen.findByRole("button", { name: "Link this rule" }));
    expect(await screen.findByText("1 linked · 0 unlinked")).toBeVisible();
    expect(api.savePcAbilityLink).not.toHaveBeenCalled();
    expect(api.savePathbuilderPc).not.toHaveBeenCalled();

    if (outcome === "cancel") {
      fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
      fireEvent.click(await screen.findByRole("button", { name: "Discard" }));
      await waitFor(() => expect(closed).toHaveBeenCalledOnce());
      expect(api.savePathbuilderPc).not.toHaveBeenCalled();
    } else if (outcome === "new generation") {
      fireEvent.click(screen.getByRole("button", { name: "Add character" }));
      fireEvent.click(
        await screen.findByRole("button", { name: "Review latest changes" }),
      );
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
