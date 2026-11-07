import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ConfigProvider } from "antd";
import { render, screen, waitFor } from "@testing-library/react";
import { getDefinitionDetail, searchPage } from "../../api/atlasApi";
import { atlasRouter } from "../../app/router";
import type { DefinitionDetailView } from "../../generated/atlas";
import { RouterTestContext } from "../../test/router";
import { SearchProfileProvider } from "../../test/SearchProfileProvider";
import { summaryFixture } from "../../test/fixtures";
import { DefinitionView } from "./DefinitionView";

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getDefinitionDetail: vi.fn(),
  searchPage: vi.fn(),
}));
const detailApi = vi.mocked(getDefinitionDetail);
const searchApi = vi.mocked(searchPage);
function definition(): DefinitionDetailView {
  return {
    artifact_generation: "generation-a",
    definition_key: "language:chthonian",
    label: "Chthonian",
    help: {
      definition_key: "language:chthonian",

      destination: { kind: "definition", key: "language:chthonian" },
    },
    content: {
      body: { kind: "plain", text: "Readable language context." },
      references: [],
    },

    notices: [],

    related_filters: [
      {
        label: "With this language",

        predicate: {
          kind: "set_match",
          field: "languages",
          op: "includes_any",
          values: ["chthonian"],
        },
      },
    ],
    publications: [],
  };
}
function setup(key = "language:chthonian") {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <RouterTestContext>
      <ConfigProvider theme={{ token: { motion: false } }}>
        <QueryClientProvider client={client}>
          <SearchProfileProvider>
            <DefinitionView definitionKey={key} />
          </SearchProfileProvider>
        </QueryClientProvider>
      </ConfigProvider>
    </RouterTestContext>,
  );
}
beforeEach(() => {
  vi.clearAllMocks();
  history.replaceState(null, "", "/definitions/language:chthonian");
  detailApi.mockResolvedValue(definition());
  searchApi.mockResolvedValue({
    search_content: "all",
    ranking: "score",
    sort: { kind: "attributes", rules: [{ field: "name", direction: "asc" }] },
    search_profile: "bge-base",
    artifact_generation: "generation-a",
    mode: { kind: "list_records" },
    coverage: null,
    relationship_clauses: [],
    rows: [
      {
        record: summaryFixture("creatures:speaker", "Speaker"),
        matches: [],
        relationship_evidence: [],
      },
    ],
    page: { number: 1, size: 5, count: 1, total: 1n, has_more: false, next_page: null },
  });
});

it("places the trait-style association block after readable standalone content without Similar", async () => {
  const { container } = setup();
  expect(await screen.findByText("Readable language context.")).toBeVisible();
  expect(await screen.findByRole("tab", { name: "With this language" })).toBeVisible();
  expect(container.textContent!.indexOf("Readable language context.")).toBeLessThan(
    container.textContent!.indexOf("With this language"),
  );
  expect(screen.queryByRole("tab", { name: "Similar" })).toBeNull();
});

it.each(["label_only", "empty"])(
  "uses the generic unavailable destination for a %s direct URL",
  async (kind) => {
    const detail = definition();
    detail.content = null;
    if (kind === "label_only") {
      detail.help = null;
    }
    detailApi.mockResolvedValue(detail);
    setup();
    expect(await screen.findByText("Link not found")).toBeVisible();
    expect(screen.queryByRole("heading", { name: "Chthonian" })).toBeNull();
    expect(
      screen.queryByText("This entry supplies a name without a definition."),
    ).toBeNull();
    expect(screen.queryByRole("region", { name: "Related records" })).toBeNull();
    expect(searchApi).not.toHaveBeenCalled();
  },
);

it("redirects record-backed help to the actual record with its supplied property context", async () => {
  const detail = definition();
  detail.definition_key = "sense:darkvision";
  detail.help!.definition_key = detail.definition_key;
  detail.help!.destination = {
    kind: "record",
    navigation: {
      record_key: "bestiary-ability-glossary:darkvision",
      embedded_path: [],
      field: null,
      passage: null,
      source_fingerprint: null,
    },
  };
  detail.content = null;
  detailApi.mockResolvedValue(detail);
  setup(detail.definition_key);
  await waitFor(() =>
    expect(atlasRouter.latestLocation.pathname).toBe(
      "/records/bestiary-ability-glossary:darkvision",
    ),
  );
  expect(atlasRouter.latestLocation.search).toMatchObject({
    definition_key: "sense:darkvision",
  });
  expect(screen.queryByRole("region", { name: "Related records" })).toBeNull();
});
