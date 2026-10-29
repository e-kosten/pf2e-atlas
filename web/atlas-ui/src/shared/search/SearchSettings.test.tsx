import { parseSearchParams, selectedNavigation } from "../../app/urlState";
import { App, ConfigProvider } from "antd";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { SearchProfileContext } from "./SearchProfileContext";
import { DEFAULT_SEARCH_PREFERENCES } from "./searchPreferences";
import { DEFAULT_SEARCH_STATE, type SearchFormState } from "../filters/searchState";
import { readinessFixture } from "../../test/fixtures";
import { SearchSettings } from "./SearchSettings";
import { AtlasApiError } from "../../api/atlasApi";
import type { SimilarityPartsView, SimilaritySeed } from "../../generated/atlas";

vi.mock("./SimilaritySourceDocument", () => ({
  SimilaritySourceDocument: ({
    seed,
    target,
    generation,
    selection,
    onReloadSections,
  }: {
    seed: SimilaritySeed;
    target: { part: { label: string } } | null;
    generation: string;
    selection: { selected: string[] | null } | null;
    onReloadSections: () => void;
  }) => (
    <section
      aria-label="Seed source"
      data-seed={JSON.stringify(seed)}
      data-generation={generation}
      data-selected={JSON.stringify(selection?.selected)}
    >
      {target?.part.label ?? "Whole document"}
      <button onClick={onReloadSections}>Reload sections</button>
    </section>
  ),
}));

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
const inventory = {
  seed,
  artifact_generation: "generation-1",
  search_profile: "bge-base",
  role_recipe: "roles/v1",
  supports_corresponding_parts: true,
  core: {
    family: "hazard",
    selectors: ["part-0"],
    representation: "identity_and_prose",
    fallback_reason: null,
    recipe: "core-sections/v1",
  },
  parts: ["Description", "Reset"].map((label, index) => ({
    kind: "section" as const,
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
    navigation: {
      record_key: seed.record_key,
      embedded_path: [],
      field: `/system/details/${label.toLowerCase()}`,
      passage: null,
      source_fingerprint: "source-1",
    },
  })),
} satisfies SimilarityPartsView;

beforeEach(() => {
  vi.clearAllMocks();
  localStorage.clear();
  api.parts.mockReset().mockResolvedValue(inventory);
});

function setup(
  search: SearchFormState = {
    ...DEFAULT_SEARCH_STATE,
    similarSeed: seed.record_key,
    similarity: options,
  },
  selectedSeed: SimilaritySeed = seed,
  preferences = DEFAULT_SEARCH_PREFERENCES,
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
              preferences,
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
  openAdvancedSettings();
  return { apply, remember };
}

function openAdvancedSettings() {
  if (!screen.queryByRole("combobox", { name: "Search model" }))
    fireEvent.click(screen.getByRole("button", { name: /Advanced settings/ }));
}
function backToSettings() {
  fireEvent.click(screen.getByRole("button", { name: "Back to search settings" }));
  openAdvancedSettings();
}
function chooseSections() {
  fireEvent.click(screen.getByRole("button", { name: "Choose sections" }));
}

function openMatchDisplay() {
  if (!screen.queryByRole("combobox", { name: "Match display" }))
    fireEvent.click(screen.getByRole("button", { name: "Explanation details" }));
}
function chooseMatchDisplay(label: string) {
  openMatchDisplay();
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Match display" }));
  fireEvent.click(screen.getByText(label, { exact: true }));
}

it("defaults to Matching sections and discards a canceled display change", async () => {
  const { apply, remember } = setup();
  openMatchDisplay();
  await waitFor(() =>
    expect(
      within(screen.getByRole("dialog", { name: "Search settings" })).getByTitle(
        "Matching sections",
      ),
    ).toBeVisible(),
  );
  chooseMatchDisplay("Hidden");
  fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  expect(remember).not.toHaveBeenCalled();
  expect(apply).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Search settings" }));
  openAdvancedSettings();
  openMatchDisplay();
  await waitFor(() =>
    expect(
      within(screen.getByRole("dialog", { name: "Search settings" })).getByTitle(
        "Matching sections",
      ),
    ).toBeVisible(),
  );
});

it("remembers a display-only change without submitting another search", async () => {
  const { apply, remember } = setup();
  chooseMatchDisplay("Detailed");
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Apply" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(remember).toHaveBeenCalledWith(
    expect.objectContaining({ matchDisplay: "detailed" }),
  );
  expect(apply).not.toHaveBeenCalled();
});

it("resets match display to Matching sections", async () => {
  setup();
  chooseMatchDisplay("Detailed");
  fireEvent.click(screen.getByRole("button", { name: "Reset" }));
  await waitFor(() =>
    expect(
      within(screen.getByRole("dialog", { name: "Search settings" })).getByTitle(
        "Matching sections",
      ),
    ).toBeVisible(),
  );
});

it("remembers smaller weights as a presentation preference with Cancel and Reset support", async () => {
  const { apply, remember } = setup(undefined, seed, {
    ...DEFAULT_SEARCH_PREFERENCES,
    matchDisplay: "detailed",
  });
  const choice = () => screen.getByRole("switch", { name: "Show smaller weights" });
  openMatchDisplay();
  expect(choice()).not.toBeChecked();
  fireEvent.click(choice());
  fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  expect(remember).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Search settings" }));
  openAdvancedSettings();
  openMatchDisplay();
  expect(choice()).not.toBeChecked();
  fireEvent.click(choice());
  fireEvent.click(screen.getByRole("button", { name: "Reset" }));
  expect(screen.queryByRole("switch", { name: "Show smaller weights" })).toBeNull();
  chooseMatchDisplay("Detailed");
  expect(choice()).not.toBeChecked();
  fireEvent.click(choice());
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Apply" })).toBeEnabled(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(remember).toHaveBeenCalledExactlyOnceWith(
    expect.objectContaining({ showSmallerWeights: true, matchDisplay: "detailed" }),
  );
  expect(apply).not.toHaveBeenCalled();
});

it("keeps edits local until Apply and discards them on Cancel", () => {
  const { apply, remember } = setup();
  fireEvent.change(screen.getByRole("spinbutton", { name: "Match decay" }), {
    target: { value: "0.75" },
  });
  expect(apply).not.toHaveBeenCalled();
  expect(remember).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  fireEvent.click(screen.getByRole("button", { name: "Search settings" }));
  openAdvancedSettings();
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
  chooseSections();
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

it("flattens single-section entries while retaining independent source links and multi-section selection", async () => {
  const ability = {
    ...inventory.parts[0],
    selector: "part-ability",
    owner_label: "Leech Warmth",
    owners: [{ collection: "/items", identity: { Stable: "leech-warmth" } }],
    label: "Ability",
    role: "ability",
    role_label: "Ability",
  } satisfies SimilarityPartsView["parts"][number];
  api.parts.mockResolvedValue({ ...inventory, parts: [...inventory.parts, ability] });
  const { apply } = setup();
  chooseSections();
  const choice = await screen.findByRole("checkbox", { name: "Leech Warmth: Ability" });
  expect(
    screen.queryByRole("checkbox", { name: "All sections in Leech Warmth" }),
  ).toBeNull();
  const link = screen.getByRole("link", { name: "Leech Warmth" });
  expect(
    selectedNavigation(
      parseSearchParams(new URL(link.getAttribute("href")!, location.origin).search),
      inventory.seed.kind === "record" ? inventory.seed.record_key : "",
    ),
  ).toEqual(ability.navigation);
  fireEvent.click(link);
  expect(screen.getByRole("region", { name: "Seed source" })).toHaveTextContent(
    "Ability",
  );
  expect(choice).toBeChecked();
  fireEvent.click(screen.getByRole("radio", { name: "Search settings" }));
  fireEvent.click(choice);
  const group = screen.getByRole("checkbox", { name: "All sections in Brown Mold" });
  expect(group).toBeChecked();
  fireEvent.click(screen.getByRole("checkbox", { name: "Description" }));
  expect(group).toBePartiallyChecked();
  fireEvent.click(group);
  expect(screen.getByRole("checkbox", { name: "Description" })).toBeChecked();
  expect(screen.getByRole("checkbox", { name: "Reset" })).toBeChecked();
  expect(choice).not.toBeChecked();
  expect(within(link.parentElement!).getByText("Ability")).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(apply).toHaveBeenCalledExactlyOnceWith(
    expect.objectContaining({
      similarity: { ...options, parts: expect.arrayContaining(["part-0", "part-1"]) },
      similarityGeneration: "generation-1",
    }),
    "bge-base",
  );
  expect(apply.mock.calls[0][0].similarity.parts).toHaveLength(2);
});

it("retains an exact selection and prevents Apply when inventory cannot validate it", async () => {
  api.parts.mockRejectedValue(new Error("Collection unavailable"));
  const { apply } = setup({
    ...DEFAULT_SEARCH_STATE,
    similarSeed: seed.record_key,
    similarity: { ...options, parts: ["old-part"] },
    similarityGeneration: "old-generation",
  });
  await screen.findByText("Unable to load comparison sections");
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
  expect(screen.queryByRole("button", { name: "Choose sections" })).toBeNull();
});

it("suspends an empty part draft when switching to the baseline", async () => {
  const { apply } = setup();
  chooseSections();
  await screen.findByRole("checkbox", { name: "Description" });
  fireEvent.click(screen.getByRole("checkbox", { name: "Select all" }));
  expect(screen.getByRole("button", { name: "Apply" })).toBeDisabled();
  backToSettings();
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

it("explains match decay for the selected Similar method", async () => {
  setup();
  fireEvent.click(screen.getByRole("button", { name: "About match decay" }));
  await waitFor(() =>
    expect(screen.getByText(/0 uses only the strongest section/)).toBeVisible(),
  );
});

it("updates section-matching help when the selected matching mode changes", async () => {
  setup();
  const help = screen.getByRole("button", { name: "About section matching" });
  fireEvent.click(help);
  await waitFor(() =>
    expect(
      screen.getByText(/Any selected section can match any content/),
    ).toBeVisible(),
  );
  fireEvent.keyDown(help, { key: "Escape" });
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Section matching" }));
  fireEvent.click(screen.getByText("Corresponding sections"));
  fireEvent.click(help);
  await waitFor(() =>
    expect(
      screen.getByText(
        /If a result lacks that type of section, it contributes no match/,
      ),
    ).toBeVisible(),
  );
});

it("keeps sections help separate from opening and changing the section selection", async () => {
  setup();
  chooseSections();
  const description = await screen.findByRole("checkbox", { name: "Description" });
  const help = screen.getByRole("button", { name: "About sections from brown mold" });
  fireEvent.click(help);
  await screen.findByText(
    /Choose which sections of this record to use when finding similar records/,
  );
  expect(api.parts).toHaveBeenCalledTimes(1);
  expect(description).toBeChecked();
  expect(screen.getByRole("radio", { name: "Search settings" })).toBeChecked();
});

it("updates comparison help and hides decay when Content average is selected", async () => {
  setup();
  expect(screen.queryByRole("combobox", { name: "Search ranking" })).toBeNull();
  const help = screen.getByRole("button", { name: "About comparison method" });
  fireEvent.click(help);
  await waitFor(() =>
    expect(
      screen.getByText(/Looks for passages similar to each selected section/),
    ).toBeVisible(),
  );
  fireEvent.keyDown(help, { key: "Escape" });
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Comparison method" }));
  fireEvent.click(screen.getByText("Content average"));
  fireEvent.click(help);
  await waitFor(() =>
    expect(
      screen.getByText(
        /Specific details can become less prominent when several different sections are combined/,
      ),
    ).toBeVisible(),
  );
  expect(screen.queryByRole("spinbutton", { name: "Match decay" })).toBeNull();
});

it("resets traits to their default without replacing the record preference", () => {
  const preferences = {
    ...DEFAULT_SEARCH_PREFERENCES,
    recordSimilarity: {
      method: "content-average" as const,
      decay: 0.25,
      candidateParts: "corresponding" as const,
    },
    traitSimilarity: {
      method: "passages" as const,
      decay: 0.75,
      candidateParts: "any" as const,
    },
  };
  const { apply, remember } = setup(
    { ...DEFAULT_SEARCH_STATE, similarSeed: "trait:fire", similarity: options },
    { kind: "trait", trait_key: "trait:fire" },
    preferences,
  );
  fireEvent.click(screen.getByRole("button", { name: "Reset" }));
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(apply).toHaveBeenCalledWith(
    expect.objectContaining({ similarity: { method: "baseline" } }),
    "bge-base",
  );
  expect(remember).toHaveBeenCalledWith(
    expect.objectContaining({
      recordSimilarity: preferences.recordSimilarity,
      traitSimilarity: { method: "baseline", decay: 0.5, candidateParts: "any" },
    }),
  );
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
  await waitFor(() => expect(screen.getByTitle("Trait definition")).toBeVisible());
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

it("opens a section's source separately from its checkbox and preserves the complete draft across views", async () => {
  const { apply } = setup();
  fireEvent.change(screen.getByRole("spinbutton", { name: "Match decay" }), {
    target: { value: "0.75" },
  });
  chooseSections();
  const description = await screen.findByRole("checkbox", { name: "Description" });
  const link = screen.getByRole("link", { name: "Description" });
  expect(
    selectedNavigation(
      parseSearchParams(new URL(link.getAttribute("href")!, location.origin).search),
      inventory.seed.kind === "record" ? inventory.seed.record_key : "",
    ),
  ).toEqual(inventory.parts[0].navigation);
  fireEvent.click(link);
  const source = screen.getByRole("region", { name: "Seed source" });
  expect(source).toHaveTextContent("Description");
  await waitFor(() =>
    expect(
      screen.getByText("Only highlighted sections are used for comparison."),
    ).toBeVisible(),
  );
  expect(source).toHaveAttribute("data-generation", "generation-1");
  expect(description).toBeChecked();
  expect(screen.getByRole("radio", { name: "Source document" })).toBeChecked();
  fireEvent.click(screen.getByRole("radio", { name: "Search settings" }));
  fireEvent.click(description);
  expect(description).not.toBeChecked();
  fireEvent.click(screen.getByRole("radio", { name: "Source document" }));
  expect(source).toHaveTextContent("Description");
  expect(apply).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(apply).toHaveBeenCalledExactlyOnceWith(
    expect.objectContaining({
      similarity: { ...options, decay: 0.75, parts: ["part-1"] },
    }),
    "bge-base",
  );
});

it("keeps modified section activation native and leaves selection and the active view unchanged", async () => {
  setup();
  chooseSections();
  const description = await screen.findByRole("checkbox", { name: "Description" });
  const event = new MouseEvent("click", {
    bubbles: true,
    cancelable: true,
    ctrlKey: true,
  });
  screen.getByRole("link", { name: "Description" }).dispatchEvent(event);
  expect(event.defaultPrevented).toBe(false);
  expect(description).toBeChecked();
  expect(screen.getByRole("radio", { name: "Search settings" })).toBeChecked();
});

it("reloads the source and inventory generation together while preserving unrelated draft settings", async () => {
  const next = {
    ...inventory,
    artifact_generation: "generation-2",
    parts: inventory.parts.map((part) => ({
      ...part,
      selector: `new-${part.selector}`,
    })),
  };
  api.parts.mockResolvedValueOnce(inventory).mockResolvedValue(next);
  const { apply } = setup();
  fireEvent.change(screen.getByRole("spinbutton", { name: "Match decay" }), {
    target: { value: "0.75" },
  });
  chooseSections();
  fireEvent.click(await screen.findByRole("checkbox", { name: "Description" }));
  fireEvent.click(screen.getByRole("link", { name: "Reset" }));
  fireEvent.click(screen.getByRole("button", { name: "Reload sections" }));
  await waitFor(() =>
    expect(screen.getByRole("region", { name: "Seed source" })).toHaveAttribute(
      "data-generation",
      "generation-2",
    ),
  );
  expect(screen.getByRole("region", { name: "Seed source" })).toHaveTextContent(
    "Whole document",
  );
  expect(screen.getByRole("button", { name: "Apply" })).toBeDisabled();
  fireEvent.click(screen.getByRole("radio", { name: "Search settings" }));
  expect(screen.getByRole("checkbox", { name: "Description" })).not.toBeChecked();
  fireEvent.click(screen.getByRole("checkbox", { name: "Description" }));
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(apply).toHaveBeenCalledExactlyOnceWith(
    expect.objectContaining({
      similarityGeneration: "generation-2",
      similarity: { ...options, decay: 0.75, parts: ["new-part-0"] },
    }),
    "bge-base",
  );
  expect(api.parts).toHaveBeenLastCalledWith(
    expect.objectContaining({ expected_artifact_generation: null }),
    expect.any(AbortSignal),
  );
});

it("retires loaded source when a model-switch inventory reports a new generation, preserving the draft until reload", async () => {
  api.parts.mockResolvedValueOnce(inventory).mockRejectedValue(
    new AtlasApiError(409, "The reference collection changed.", {
      code: "artifact_generation_changed",
      message: "The reference collection changed.",
      recoverable_action: "refresh_results",
    }),
  );
  setup();
  fireEvent.change(screen.getByRole("spinbutton", { name: "Match decay" }), {
    target: { value: "0.75" },
  });
  chooseSections();
  fireEvent.click(await screen.findByRole("checkbox", { name: "Description" }));
  expect(
    screen.getByRole("region", { name: "Seed source", hidden: true }),
  ).toHaveAttribute("data-generation", "generation-1");
  backToSettings();
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Search model" }));
  fireEvent.click(screen.getByText("EmbeddingGemma"));
  await screen.findByText(
    "The reference collection changed. Reload sections to continue.",
  );
  expect(
    screen.queryByRole("region", { name: "Seed source", hidden: true }),
  ).toBeNull();
  expect(screen.getByRole("button", { name: "Reload sections" })).toBeVisible();
  expect(screen.getByRole("spinbutton", { name: "Match decay" })).toHaveValue("0.75");
  chooseSections();
  expect(screen.getByText("1 selected section")).toBeVisible();
  expect(screen.getByRole("button", { name: "Apply" })).toBeDisabled();
  fireEvent.click(screen.getByRole("radio", { name: "Source document" }));
  expect(screen.getByText("Reference collection changed")).toBeVisible();
  expect(screen.getByRole("button", { name: "Reload sections" })).toBeVisible();
  backToSettings();
  // An older model's cached inventory cannot resurrect retired source content.
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Search model" }));
  fireEvent.click(screen.getByText("BGE Base"));
  expect(
    screen.queryByRole("region", { name: "Seed source", hidden: true }),
  ).toBeNull();
  expect(screen.getByRole("button", { name: "Reload sections" })).toBeVisible();
});

it("updates only the local source selection immediately and restores the applied selection after Cancel", async () => {
  const { apply } = setup();
  chooseSections();
  const description = await screen.findByRole("checkbox", { name: "Description" });
  const source = screen.getByRole("region", { name: "Seed source", hidden: true });
  expect(source).toHaveAttribute("data-selected", "null");
  const inventoryRequests = api.parts.mock.calls.length;
  fireEvent.click(description);
  expect(source).toHaveAttribute("data-selected", '["part-1"]');
  fireEvent.click(screen.getByRole("checkbox", { name: "Reset" }));
  expect(source).toHaveAttribute("data-selected", "[]");
  expect(api.parts).toHaveBeenCalledTimes(inventoryRequests);
  expect(apply).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  fireEvent.click(screen.getByRole("button", { name: "Search settings" }));
  openAdvancedSettings();
  chooseSections();
  await screen.findByRole("checkbox", { name: "Description" });
  expect(
    screen.getByRole("region", { name: "Seed source", hidden: true }),
  ).toHaveAttribute("data-selected", "null");
});

it("keeps the active settings summary without a customization marker", () => {
  setup({ ...DEFAULT_SEARCH_STATE, query: "heal", ranking: "rrf" });
  expect(
    screen.getByRole("button", { name: "Search settings" }),
  ).toHaveAccessibleDescription(/Rank-based ranking/);
  expect(screen.queryByText(/Customized/)).toBeNull();
  expect(document.querySelector(".ant-badge")).toBeNull();
});

function coreInventory(): SimilarityPartsView {
  return {
    ...inventory,
    core: { ...inventory.core, selectors: ["identity-root", "part-0"] },
    parts: [
      ...inventory.parts,
      {
        kind: "identity",
        selector: "identity-root",
        label: "Name and identity",
        owner_label: "Brown Mold",
        has_vectors: true,
        navigation: {
          record_key: seed.record_key,
          embedded_path: [],
          field: null,
          passage: null,
          source_fingerprint: "source-1",
        },
      },
    ],
  };
}

it("remembers an authoritative All preset for the backend's family with sections collapsed", async () => {
  const npcSeed = { kind: "record", record_key: "creatures:lich" } as const;
  api.parts.mockResolvedValue({
    ...coreInventory(),
    seed: npcSeed,
    core: { ...coreInventory().core, family: "npc" },
  } satisfies SimilarityPartsView);
  const { apply, remember } = setup(
    {
      ...DEFAULT_SEARCH_STATE,
      similarSeed: npcSeed.record_key,
      similarity: options,
      sourceSections: "all",
    },
    npcSeed,
    {
      ...DEFAULT_SEARCH_PREFERENCES,
      sourceSectionsByFamily: { npc: "core", hazard: "core" },
    },
  );
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Apply" })).toBeEnabled(),
  );
  expect(screen.queryByRole("checkbox", { name: "Select all" })).toBeNull();
  expect(screen.queryByRole("region", { name: "Seed source" })).toBeNull();
  expect(remember).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(remember).toHaveBeenCalledExactlyOnceWith(
    expect.objectContaining({
      sourceSectionsByFamily: { npc: "all", hazard: "core" },
    }),
  );
  expect(apply).toHaveBeenCalledExactlyOnceWith(
    expect.objectContaining({ sourceSections: "all" }),
    "bge-base",
  );
});

it("blocks early All Apply and exposes loading, failure and retry with sections collapsed", async () => {
  let rejectInventory!: (error: Error) => void;
  let resolveInventory!: (value: SimilarityPartsView) => void;
  api.parts
    .mockImplementationOnce(
      () =>
        new Promise<SimilarityPartsView>((_, reject) => {
          rejectInventory = reject;
        }),
    )
    .mockImplementationOnce(
      () =>
        new Promise<SimilarityPartsView>((resolve) => {
          resolveInventory = resolve;
        }),
    );
  const { apply, remember } = setup();
  const applyButton = screen.getByRole("button", { name: "Apply" });
  expect(applyButton).toBeDisabled();
  await waitFor(() =>
    expect(screen.getByText("Loading comparison sections…")).toBeVisible(),
  );
  fireEvent.click(applyButton);
  expect(apply).not.toHaveBeenCalled();
  expect(remember).not.toHaveBeenCalled();
  await act(async () => rejectInventory(new Error("Collection unavailable")));
  expect(await screen.findByText("Unable to load comparison sections")).toBeVisible();
  expect(screen.queryByText("Collection unavailable")).toBeNull();
  expect(applyButton).toBeDisabled();
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  await waitFor(() => expect(api.parts).toHaveBeenCalledTimes(2));
  await waitFor(() =>
    expect(screen.getByText("Loading comparison sections…")).toBeVisible(),
  );
  fireEvent.click(applyButton);
  expect(remember).not.toHaveBeenCalled();
  await act(async () => resolveInventory(coreInventory()));
  await waitFor(() => expect(applyButton).toBeEnabled());
  expect(screen.queryByText("Unable to load comparison sections")).toBeNull();
  expect(screen.queryByRole("checkbox", { name: "Select all" })).toBeNull();
  expect(screen.queryByRole("region", { name: "Seed source" })).toBeNull();
  fireEvent.click(applyButton);
  expect(remember).toHaveBeenCalledExactlyOnceWith(
    expect.objectContaining({ sourceSectionsByFamily: { hazard: "all" } }),
  );
  expect(apply).toHaveBeenCalledTimes(1);
});

it("includes identity once in All and lets the backend resolve the preset on Apply", async () => {
  api.parts.mockResolvedValue(coreInventory());
  const { apply } = setup();
  chooseSections();
  const identity = await screen.findByRole("checkbox", {
    name: "Brown Mold: Name and identity",
  });
  expect(identity).toBeChecked();
  expect(
    screen.getAllByRole("checkbox", { name: "Brown Mold: Name and identity" }),
  ).toHaveLength(1);
  expect(screen.getByRole("checkbox", { name: "Description" })).toBeChecked();
  expect(screen.getByRole("checkbox", { name: "Reset" })).toBeChecked();
  expect(screen.getByRole("checkbox", { name: "Select all" })).toBeChecked();
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(apply).toHaveBeenCalledExactlyOnceWith(
    expect.objectContaining({
      sourceSections: "all",
      similarity: { ...options, parts: null },
      similarityGeneration: undefined,
    }),
    "bge-base",
  );
});

it("reflects the complete selection in Select all across Core, Custom and All", async () => {
  api.parts.mockResolvedValue(coreInventory());
  setup({
    ...DEFAULT_SEARCH_STATE,
    similarSeed: seed.record_key,
    similarity: options,
    sourceSections: "core",
  });
  chooseSections();
  const identity = await screen.findByRole("checkbox", {
    name: "Brown Mold: Name and identity",
  });
  const selectAll = screen.getByRole("checkbox", { name: "Select all" });
  expect(selectAll).toBePartiallyChecked();
  fireEvent.click(selectAll);
  expect(identity).toBeChecked();
  expect(screen.getByRole("checkbox", { name: "Reset" })).toBeChecked();
  expect(selectAll).toBeChecked();
  fireEvent.click(identity);
  expect(selectAll).toBePartiallyChecked();
  fireEvent.click(identity);
  expect(selectAll).toBeChecked();
  backToSettings();
  await waitFor(() => expect(screen.getByTitle("Custom")).toBeVisible());
});

it("blocks All when its required identity is unavailable instead of omitting it", async () => {
  const current = coreInventory();
  api.parts.mockResolvedValue({
    ...current,
    parts: current.parts.map((part) =>
      part.kind === "identity" ? { ...part, has_vectors: false } : part,
    ),
  });
  const { apply } = setup();
  chooseSections();
  await screen.findByText(
    "Name and identity is unavailable for this model. Choose another model or update the collection.",
  );
  expect(screen.getByRole("button", { name: "Apply" })).toBeDisabled();
  expect(
    screen.getByRole("checkbox", { name: "Brown Mold: Name and identity" }),
  ).toBeDisabled();
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(apply).not.toHaveBeenCalled();
});

it("distinguishes source and result content and explains their selected draft options", async () => {
  api.parts.mockResolvedValue(coreInventory());
  const { apply, remember } = setup();
  await waitFor(() => {
    expect(
      screen.getByText("Content from Brown Mold used for comparison."),
    ).toBeVisible();
  });
  const sourceHelp = screen.getByRole("button", { name: "About source sections" });
  const contentHelp = screen.getByRole("button", { name: "About search content" });
  fireEvent.click(sourceHelp);
  await screen.findByText(
    "Uses Brown Mold’s name and identity plus all available document sections.",
  );
  fireEvent.keyDown(sourceHelp, { key: "Escape" });
  fireEvent.click(contentHelp);
  await screen.findByText(
    "Searches all available content, including details like a creature's spells.",
  );
  fireEvent.keyDown(contentHelp, { key: "Escape" });

  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Source sections" }));
  fireEvent.click(screen.getByText("Core sections"));
  fireEvent.click(sourceHelp);
  await screen.findByText(
    "Uses Brown Mold’s name and identity plus the core sections chosen for this record kind.",
  );
  fireEvent.keyDown(sourceHelp, { key: "Escape" });
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Search content" }));
  fireEvent.click(screen.getAllByText("Core sections").slice(-1)[0]);
  fireEvent.click(contentHelp);
  await screen.findByText(
    "Limits content searches to a core set of fields, like name and description, while omitting details like a creature's spells.",
  );
  fireEvent.keyDown(contentHelp, { key: "Escape" });

  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Source sections" }));
  fireEvent.click(screen.getByText("Custom"));
  backToSettings();
  fireEvent.click(screen.getByRole("button", { name: "About source sections" }));
  await screen.findByText("Uses only the selected content from Brown Mold.");
  expect(apply).not.toHaveBeenCalled();
  expect(remember).not.toHaveBeenCalled();
});

it("describes the searched content for ordinary text without source-selection copy", async () => {
  setup({ ...DEFAULT_SEARCH_STATE, query: "fire" });

  expect(screen.queryByText("Content from Brown Mold used for comparison.")).toBeNull();
});

it("applies Core source and result controls atomically without storing exact selectors", async () => {
  api.parts.mockResolvedValue(coreInventory());
  const { apply, remember } = setup();
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Source sections" }));
  fireEvent.click(screen.getByText("Core sections"));
  chooseSections();
  await screen.findByRole("checkbox", { name: "Brown Mold: Name and identity" });
  expect(screen.getByRole("checkbox", { name: "Description" })).toBeChecked();
  expect(screen.getByRole("checkbox", { name: "Reset" })).not.toBeChecked();
  expect(
    screen.getByRole("checkbox", { name: "Brown Mold: Name and identity" }),
  ).toBeChecked();
  backToSettings();
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Search content" }));
  fireEvent.click(screen.getAllByText("Core sections").slice(-1)[0]);
  expect(apply).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(apply).toHaveBeenCalledExactlyOnceWith(
    expect.objectContaining({
      sourceSections: "core",
      searchContent: "core",
      similarity: { ...options, parts: null },
      similarityGeneration: undefined,
    }),
    "bge-base",
  );
  expect(remember).toHaveBeenCalledExactlyOnceWith(
    expect.objectContaining({
      searchContent: "core",
      sourceSectionsByFamily: { hazard: "core" },
    }),
  );
});
it("turns a Core checkbox edit into generation-bound Custom and permits deselecting identity", async () => {
  api.parts.mockResolvedValue(coreInventory());
  const { apply, remember } = setup({
    ...DEFAULT_SEARCH_STATE,
    similarSeed: seed.record_key,
    similarity: options,
    sourceSections: "core",
  });
  chooseSections();
  const identity = await screen.findByRole("checkbox", {
    name: "Brown Mold: Name and identity",
  });
  fireEvent.click(identity);
  backToSettings();
  await waitFor(() => expect(screen.getByTitle("Custom")).toBeVisible());
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(apply).toHaveBeenCalledExactlyOnceWith(
    expect.objectContaining({
      sourceSections: "custom",
      similarity: { ...options, parts: ["part-0"] },
      similarityGeneration: "generation-1",
    }),
    "bge-base",
  );
  expect(remember).toHaveBeenCalledWith(
    expect.objectContaining({ sourceSectionsByFamily: {} }),
  );
});
it("shows identity-only Core and keeps a missing required identity actionable", async () => {
  const current = coreInventory();
  api.parts.mockResolvedValue({
    ...current,
    core: {
      ...current.core,
      selectors: ["identity-root"],
      representation: "identity_only",
      fallback_reason: "no_core_prose",
    },
    parts: current.parts.map((part) =>
      part.kind === "identity" ? { ...part, has_vectors: false } : part,
    ),
  });
  const { apply } = setup({
    ...DEFAULT_SEARCH_STATE,
    similarSeed: seed.record_key,
    similarity: options,
    sourceSections: "core",
  });
  await screen.findByText("Name and identity only");
  await waitFor(() =>
    expect(screen.getByText(/Some core content is unavailable/)).toBeVisible(),
  );
  expect(screen.getByRole("button", { name: "Apply" })).toBeDisabled();
  fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  expect(apply).not.toHaveBeenCalled();
});
