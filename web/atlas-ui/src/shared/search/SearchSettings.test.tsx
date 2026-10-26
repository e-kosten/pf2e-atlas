import { App, ConfigProvider } from "antd";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { SearchProfileContext } from "./SearchProfileContext";
import { DEFAULT_SEARCH_PREFERENCES } from "./searchPreferences";
import { DEFAULT_SEARCH_STATE, type SearchFormState } from "../filters/searchState";
import { readinessFixture } from "../../test/fixtures";
import { SearchSettings } from "./SearchSettings";
import type { SimilarityPartsView, SimilaritySeed } from "../../generated/atlas";

const api = vi.hoisted(() => ({ parts: vi.fn() }));
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getSimilarityParts: api.parts,
}));

const seed = {
  kind: "record",
  record_key: "hazards:brown-mold",
} satisfies SimilaritySeed;
const options = {
  method: "passages",
  decay: 0.5,
  parts: null,
  candidate_parts: "any",
} as const;
const inventory: SimilarityPartsView = {
  seed,
  artifact_generation: "generation-1",
  search_profile: "bge-base",
  role_recipe: "roles/v1",
  supports_corresponding_parts: true,
  parts: ["Description", "Reset"].map((label, index) => ({
    label,
    selector: `part-${index}`,
    owner_label: "Brown Mold",
    owners: [],
    field: `/system/details/${label.toLowerCase()}`,
    section_ordinal: 0,
    role: index === 0 ? "description" : "reset",
    role_label: label,
    has_vectors: true,
    label_only: false,
  })),
};

beforeEach(() => {
  vi.clearAllMocks();
  localStorage.clear();
  api.parts.mockResolvedValue(inventory);
});

function setup(
  search: SearchFormState = {
    ...DEFAULT_SEARCH_STATE,
    similarSeed: seed.record_key,
    similarity: options,
  },
  selectedSeed: SimilaritySeed = seed,
) {
  const apply = vi.fn();
  const remember = vi.fn(() => true);
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={client}>
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
              preferences: DEFAULT_SEARCH_PREFERENCES,
              applyPreferences: remember,
            }}
          >
            <SearchSettings
              search={search}
              seed={search.similarSeed ? selectedSeed : null}
              seedTitle={selectedSeed.kind === "trait" ? "Fire" : "Brown Mold"}
              onApply={apply}
              generation="generation-1"
            />
          </SearchProfileContext>
        </App>
      </ConfigProvider>
    </QueryClientProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: "Search settings" }));
  return { apply, remember };
}

it("keeps edits local until Apply and discards them on Cancel", () => {
  const { apply, remember } = setup();
  fireEvent.change(screen.getByRole("spinbutton", { name: "Match decay" }), {
    target: { value: "0.75" },
  });
  expect(apply).not.toHaveBeenCalled();
  expect(remember).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  fireEvent.click(screen.getByRole("button", { name: "Search settings" }));
  expect(
    Number(
      (screen.getByRole("spinbutton", { name: "Match decay" }) as HTMLInputElement)
        .value,
    ),
  ).toBe(0.5);
});

it("applies the full model and settings draft once and remembers general preferences", () => {
  const { apply, remember } = setup({ ...DEFAULT_SEARCH_STATE, query: "heal" });
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Search model" }));
  fireEvent.click(screen.getByText("EmbeddingGemma"));
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Search ranking" }));
  fireEvent.click(screen.getByText("Rank-based"));
  expect(apply).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(apply).toHaveBeenCalledExactlyOnceWith(
    expect.objectContaining({ ranking: "rrf", query: "heal" }),
    "embeddinggemma",
  );
  expect(remember).toHaveBeenCalledExactlyOnceWith(
    expect.objectContaining({ model: "embeddinggemma", ranking: "rrf" }),
  );
});

it("rejects a manually typed decay outside the supported range", () => {
  const { apply } = setup();
  fireEvent.change(screen.getByRole("spinbutton", { name: "Match decay" }), {
    target: { value: "2" },
  });
  expect(screen.getByRole("button", { name: "Apply" })).toBeDisabled();
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(apply).not.toHaveBeenCalled();
});

it("discovers exact parts and carries their collection guard when applying", async () => {
  const { apply } = setup();
  fireEvent.click(screen.getByText("Sections to compare"));
  const description = await screen.findByRole("checkbox", { name: "Description" });
  fireEvent.click(description);
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(apply).toHaveBeenCalledWith(
    expect.objectContaining({
      similarity: { ...options, parts: ["part-1"] },
      similarityGeneration: "generation-1",
    }),
    "bge-base",
  );
});

it("retains an exact selection and prevents Apply when inventory cannot validate it", async () => {
  api.parts.mockRejectedValue(new Error("Collection unavailable"));
  const { apply } = setup({
    ...DEFAULT_SEARCH_STATE,
    similarSeed: seed.record_key,
    similarity: { ...options, parts: ["old-part"] },
    similarityGeneration: "old-generation",
  });
  await screen.findByText("Collection unavailable");
  expect(screen.getByRole("button", { name: "Apply" })).toBeDisabled();
  expect(apply).not.toHaveBeenCalled();
});

it("omits controls that do not apply to the baseline", async () => {
  setup();
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Comparison method" }));
  fireEvent.click(screen.getByText("Name and identity"));
  await waitFor(() =>
    expect(screen.queryByRole("spinbutton", { name: "Match decay" })).toBeNull(),
  );
  expect(screen.queryByText("Sections to compare")).toBeNull();
});

it("suspends an empty part draft when switching to the baseline", async () => {
  const { apply } = setup();
  fireEvent.click(screen.getByText("Sections to compare"));
  await screen.findByRole("checkbox", { name: "Description" });
  fireEvent.click(screen.getByRole("checkbox", { name: "All sections" }));
  expect(screen.getByRole("button", { name: "Apply" })).toBeDisabled();
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Comparison method" }));
  fireEvent.click(screen.getByText("Name and identity"));
  expect(screen.getByRole("button", { name: "Apply" })).toBeEnabled();
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(apply).toHaveBeenCalledWith(
    expect.objectContaining({
      similarity: { method: "baseline" },
      similarityGeneration: undefined,
    }),
    "bge-base",
  );
});

it("explains model and both text ranking choices without exposing Similar controls", async () => {
  const { apply, remember } = setup({ ...DEFAULT_SEARCH_STATE, query: "heal" });
  const modelHelp = screen.getByRole("button", { name: "About embedding model" });
  fireEvent.click(modelHelp);
  await waitFor(() =>
    expect(
      screen.getByText(
        /Changes which embedding model is used for search and document ranking/,
      ),
    ).toBeVisible(),
  );
  fireEvent.keyDown(modelHelp, { key: "Escape" });
  const rankingHelp = screen.getByRole("button", { name: "About ranking" });
  fireEvent.click(rankingHelp);
  await waitFor(() =>
    expect(
      screen.getByText(/Search combines word matches with meaning matches/),
    ).toBeVisible(),
  );
  fireEvent.keyDown(rankingHelp, { key: "Escape" });
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Search ranking" }));
  fireEvent.click(screen.getByText("Rank-based"));
  fireEvent.click(rankingHelp);
  await waitFor(() =>
    expect(
      screen.getByText(
        /Rank-based considers how high each record appears in those two lists/,
      ),
    ).toBeVisible(),
  );
  expect(screen.queryByRole("combobox", { name: "Comparison method" })).toBeNull();
  expect(
    screen.queryByRole("button", { name: "About sections to compare" }),
  ).toBeNull();
  fireEvent.keyDown(rankingHelp, { key: "Escape" });
  fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  expect(apply).not.toHaveBeenCalled();
  expect(remember).not.toHaveBeenCalled();
});

it("explains selected Similar choices and keeps help separate from opening Sections to compare", async () => {
  setup();
  expect(screen.queryByRole("combobox", { name: "Search ranking" })).toBeNull();
  const methodHelp = screen.getByRole("button", { name: "About comparison method" });
  fireEvent.click(methodHelp);
  await waitFor(() =>
    expect(
      screen.getByText(/Looks for passages similar to each selected section/),
    ).toBeVisible(),
  );
  fireEvent.keyDown(methodHelp, { key: "Escape" });
  const decayHelp = screen.getByRole("button", { name: "About match decay" });
  fireEvent.click(decayHelp);
  await waitFor(() =>
    expect(screen.getByText(/0 uses only the strongest section/)).toBeVisible(),
  );
  fireEvent.keyDown(decayHelp, { key: "Escape" });
  const partsHelp = screen.getByRole("button", { name: "About sections to compare" });
  fireEvent.click(partsHelp);
  await waitFor(() =>
    expect(
      screen.getByText(
        /Choose which sections of this record to use when finding similar records/,
      ),
    ).toBeVisible(),
  );
  expect(api.parts).not.toHaveBeenCalled();
  fireEvent.keyDown(partsHelp, { key: "Escape" });
  fireEvent.click(screen.getByText("Sections to compare"));
  await screen.findByRole("checkbox", { name: "Description" });
  const candidateHelp = screen.getByRole("button", { name: "About candidate content" });
  fireEvent.click(candidateHelp);
  await waitFor(() =>
    expect(
      screen.getByText(/Any selected section can match any content/),
    ).toBeVisible(),
  );
  fireEvent.keyDown(candidateHelp, { key: "Escape" });
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Candidate content" }));
  fireEvent.click(screen.getByText("Corresponding sections"));
  fireEvent.click(candidateHelp);
  await waitFor(() =>
    expect(
      screen.getByText(
        /If a result lacks that type of section, it contributes no match/,
      ),
    ).toBeVisible(),
  );
  fireEvent.keyDown(candidateHelp, { key: "Escape" });
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Comparison method" }));
  fireEvent.click(screen.getByText("Content average"));
  fireEvent.click(methodHelp);
  await waitFor(() =>
    expect(
      screen.getByText(
        /Specific details can become less prominent when several different sections are combined/,
      ),
    ).toBeVisible(),
  );
  expect(screen.queryByRole("spinbutton", { name: "Match decay" })).toBeNull();
});

it("names the trait baseline by its content while preserving its wire value", async () => {
  const { apply } = setup(
    {
      ...DEFAULT_SEARCH_STATE,
      similarSeed: "trait:fire",
      similarity: { method: "baseline" },
    },
    { kind: "trait", trait_key: "trait:fire" },
  );
  await waitFor(() => expect(screen.getByText("Trait definition")).toBeVisible());
  expect(screen.queryByText("Name and identity")).toBeNull();
  expect(screen.queryByText("Current baseline")).toBeNull();
  expect(screen.queryByText(/Uses the trait’s definition, or its label/)).toBeNull();
  const methodHelp = screen.getByRole("button", { name: "About comparison method" });
  fireEvent.click(methodHelp);
  await waitFor(() =>
    expect(
      screen.getByText(/Uses the trait’s definition as the starting point/),
    ).toBeVisible(),
  );
  fireEvent.keyDown(methodHelp, { key: "Escape" });
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(apply).toHaveBeenCalledExactlyOnceWith(
    expect.objectContaining({ similarity: { method: "baseline" } }),
    "bge-base",
  );
});
