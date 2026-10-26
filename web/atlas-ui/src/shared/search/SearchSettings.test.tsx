import { App } from "antd";
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
) {
  const apply = vi.fn();
  const remember = vi.fn(() => true);
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  render(
    <QueryClientProvider client={client}>
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
            seed={search.similarSeed ? seed : null}
            onApply={apply}
            generation="generation-1"
          />
        </SearchProfileContext>
      </App>
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
  fireEvent.click(screen.getByText("Compare parts"));
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
  fireEvent.click(screen.getByText("Current baseline"));
  await waitFor(() =>
    expect(screen.queryByRole("spinbutton", { name: "Match decay" })).toBeNull(),
  );
  expect(screen.queryByText("Compare parts")).toBeNull();
});

it("suspends an empty part draft when switching to the baseline", async () => {
  const { apply } = setup();
  fireEvent.click(screen.getByText("Compare parts"));
  await screen.findByRole("checkbox", { name: "Description" });
  fireEvent.click(screen.getByRole("checkbox", { name: "All parts" }));
  expect(screen.getByRole("button", { name: "Apply" })).toBeDisabled();
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Comparison method" }));
  fireEvent.click(screen.getByText("Current baseline"));
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
