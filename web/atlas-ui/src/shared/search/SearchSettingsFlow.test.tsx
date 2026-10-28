import { App, ConfigProvider } from "antd";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { SearchSettings } from "./SearchSettings";
import { SearchProfileContext } from "./SearchProfileContext";
import {
  DEFAULT_SEARCH_PREFERENCES,
  type SearchPreferences,
} from "./searchPreferences";
import { DEFAULT_SEARCH_STATE, type SearchFormState } from "../filters/searchState";
import { readinessFixture } from "../../test/fixtures";
import { similarityPartsFixture } from "../../test/similarityFixtures";

const api = vi.hoisted(() => ({ parts: vi.fn() }));
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getSimilarityParts: api.parts,
}));
vi.mock("./SimilaritySourceDocument", () => ({
  SimilaritySourceDocument: () => <section aria-label="Source document preview" />,
}));

const seed = { kind: "record" as const, record_key: "hazards:brown-mold" };
const search = {
  ...DEFAULT_SEARCH_STATE,
  similarSeed: seed.record_key,
  similarity: { method: "passages", decay: 0.5, candidate_parts: "any", parts: null },
} satisfies SearchFormState;

beforeEach(() => {
  vi.clearAllMocks();
  localStorage.clear();
  api.parts.mockResolvedValue(similarityPartsFixture("generation-1"));
});

function setup(
  state = search as SearchFormState,
  preferences = DEFAULT_SEARCH_PREFERENCES,
) {
  const apply = vi.fn();
  const remember = vi.fn(() => true);
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <ConfigProvider theme={{ token: { motion: false } }}>
        <App>
          <SearchProfileContext
            value={{
              id: "bge-base",
              defaultId: "bge-base",
              pendingUrlProfile: null,
              profiles: readinessFixture().search_profiles,
              storedVectors: true,
              usable: true,
              loading: false,
              error: null,
              canResetLink: false,
              resetLink: vi.fn(),
              select: vi.fn(),
              preferences,
              applyPreferences: remember,
            }}
          >
            <SearchSettings
              search={state}
              seed={
                state.similarSeed
                  ? state.similarSeed.startsWith("trait:")
                    ? { kind: "trait", trait_key: state.similarSeed }
                    : seed
                  : null
              }
              seedTitle="Brown Mold"
              generation="generation-1"
              onApply={apply}
            />
          </SearchProfileContext>
        </App>
      </ConfigProvider>
    </QueryClientProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: "Search settings" }));
  return { apply, remember };
}

it("keeps advanced controls optional and preserves section edits when returning to settings", async () => {
  const { apply } = setup();
  expect(screen.getByRole("combobox", { name: "Source sections" })).toBeInTheDocument();
  expect(screen.queryByRole("spinbutton", { name: "Match decay" })).toBeNull();
  expect(screen.queryByRole("switch", { name: "Show similarity details" })).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Choose sections" }));
  fireEvent.click(await screen.findByRole("checkbox", { name: "Reset" }));
  fireEvent.click(screen.getByRole("button", { name: "Back to search settings" }));
  expect(screen.getByRole("combobox", { name: "Source sections" })).toBeInTheDocument();
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Choose sections" })).toHaveFocus(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(apply).toHaveBeenCalledWith(
    expect.objectContaining({
      sourceSections: "custom",
      similarityGeneration: "generation-1",
      similarity: expect.objectContaining({ parts: ["description-b"] }),
    }),
    "bge-base",
  );
});

it("resets only the draft until Apply and preserves other family preferences", async () => {
  const preferences: SearchPreferences = {
    ...DEFAULT_SEARCH_PREFERENCES,
    showSimilarityDetails: true,
    showSmallerWeights: true,
    searchContent: "core",
    sourceSectionsByFamily: { hazard: "core", npc: "core" },
    recordSimilarity: {
      method: "content-average",
      decay: 0.85,
      candidateParts: "corresponding",
    },
    traitSimilarity: { method: "content-average", decay: 0.75, candidateParts: "any" },
  };
  const state: SearchFormState = {
    ...search,
    searchContent: "core",
    sourceSections: "custom",
    similarityGeneration: "generation-1",
    similarity: {
      method: "content-average",
      parts: ["description-b"],
      candidate_parts: "corresponding",
    },
  };
  const { apply, remember } = setup(state, preferences);
  fireEvent.click(screen.getByRole("button", { name: "Reset" }));
  expect(apply).not.toHaveBeenCalled();
  expect(remember).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  expect(remember).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Search settings" }));
  fireEvent.click(screen.getByRole("button", { name: "Reset" }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Apply" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(apply).toHaveBeenCalledWith(
    expect.objectContaining({
      searchContent: "all",
      sourceSections: "all",
      similarityGeneration: undefined,
      similarity: {
        method: "passages",
        decay: 0.5,
        candidate_parts: "any",
        parts: null,
      },
    }),
    "bge-base",
  );
  expect(remember).toHaveBeenCalledWith(
    expect.objectContaining({
      showSimilarityDetails: false,
      showSmallerWeights: false,
      sourceSectionsByFamily: { hazard: "all", npc: "core" },
      traitSimilarity: preferences.traitSimilarity,
    }),
  );
});

it("resets text fusion without inventing a Similar selection", () => {
  const { apply } = setup({
    ...DEFAULT_SEARCH_STATE,
    query: "fire",
    ranking: "rrf",
    searchContent: "core",
  });
  fireEvent.click(screen.getByRole("button", { name: "Reset" }));
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(api.parts).not.toHaveBeenCalled();
  expect(apply).toHaveBeenCalledWith(
    expect.objectContaining({
      query: "fire",
      ranking: "score",
      searchContent: "all",
      similarSeed: null,
    }),
    "bge-base",
  );
});

it("keeps Trait definition as the reset method without opening a section editor", () => {
  const { apply } = setup({
    ...DEFAULT_SEARCH_STATE,
    similarSeed: "trait:fire",
    similarity: { method: "baseline" },
  });
  expect(screen.queryByRole("button", { name: "Choose sections" })).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Reset" }));
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(apply).toHaveBeenCalledWith(
    expect.objectContaining({
      similarity: { method: "baseline" },
      sourceSections: undefined,
      similarityGeneration: undefined,
    }),
    "bge-base",
  );
});
