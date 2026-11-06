import { parseSearchParams, selectedNavigation } from "../../app/urlState";
import { ConfigProvider } from "antd";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { AtlasApiError, getRelationshipOccurrences } from "../../api/atlasApi";
import { summaryFixture } from "../../test/fixtures";
import { RelationshipEvidence } from "./RelationshipEvidence";
import { rootNavigation } from "./useReferenceReading";
import type {
  RecordRelationshipView,
  RelationshipClauseSummaryView,
  RelationshipEvidenceView,
} from "../../generated/atlas";

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getRelationshipOccurrences: vi.fn(),
}));
const api = vi.mocked(getRelationshipOccurrences);
const relationship = {
  direction: "incoming" as const,
  selection: rootNavigation("items:armor"),
  include_contents: true,
};
const constraint: RelationshipClauseSummaryView = {
  clause_id: "link",
  condition: { kind: "record", relationship },
  destination: { kind: "record", navigation: relationship.selection },
  title: "Leather Armor",
  record: summaryFixture("items:armor", "Leather Armor"),
};
const ordinaryDestination = {
  kind: "record" as const,
  navigation: relationship.selection,
};
const occurrence: RecordRelationshipView = {
  source: {
    record: "actors:goblin",
    embedded_path: [],
    field: "/system/description/value",
  },
  source_navigation: {
    ...rootNavigation("actors:goblin"),
    source_fingerprint: "exact-source",
  },
  ordinal: 3,
  origin: "prepared_content",
  kind: "uuid",
  authored_target: "technical:uuid",
  occurrence_path: null,
  audiences: [],
  availability: null,
  status: "resolved_record",
  target: rootNavigation("items:armor"),
  url: null,
  effect_label: null,
  source_title: "Goblin",
  target_title: "Leather Armor",
};
const evidence: RelationshipEvidenceView = {
  clause_id: "link",
  occurrences: [{ kind: "record", occurrence, destination: ordinaryDestination }],
  page: { number: 1, size: 1, count: 1, total: 2n, has_more: true, next_page: 2 },
};
function setup() {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  const refresh = vi.fn(async () => {});
  const view = render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <QueryClientProvider client={client}>
        <RelationshipEvidence
          evidence={evidence}
          filter={{
            kind: "relationship",
            clause_id: constraint.clause_id,
            relationship: relationship,
          }}
          summary={constraint}
          resultKey="actors:goblin"
          generation="generation-a"
          onRefresh={refresh}
        />
      </QueryClientProvider>
    </ConfigProvider>,
  );
  return { ...view, refresh };
}
beforeEach(() => vi.clearAllMocks());

it("shows bounded provenance on demand and requests further pages with the complete checked constraint", async () => {
  api.mockResolvedValue({
    artifact_generation: "generation-a",
    clause: constraint,
    result_record: summaryFixture("actors:goblin", "Goblin"),
    occurrences: [
      {
        kind: "record",
        destination: ordinaryDestination,
        occurrence: { ...occurrence, source_title: "Goblin equipment", ordinal: 4 },
      },
    ],
    page: { ...evidence.page, number: 2, has_more: false, next_page: null },
  });
  const view = setup();
  expect(api).not.toHaveBeenCalled();
  expect(screen.getByText("1 of 2 links")).toBeVisible();
  expect(screen.getByRole("link", { name: "Leather Armor" })).toBeVisible();
  const source = screen.getByRole("link", { name: "Goblin" });
  expect(
    selectedNavigation(
      parseSearchParams(
        new URL(source.getAttribute("href")!, "http://atlas.local").search,
      ),
      occurrence.source_navigation.record_key,
    ),
  ).toEqual(occurrence.source_navigation);
  expect(view.container).not.toHaveTextContent("technical:uuid");
  expect(view.container).not.toHaveTextContent("/system/description/value");
  fireEvent.click(screen.getByRole("button", { name: "Next links page" }));
  expect(await screen.findByRole("link", { name: "Goblin equipment" })).toBeVisible();
  expect(api).toHaveBeenCalledWith(
    {
      clause_id: constraint.clause_id,
      filter: {
        kind: "relationship",
        clause_id: constraint.clause_id,
        relationship: relationship,
      },
      result_record_key: "actors:goblin",
      page: { number: 2, size: 1 },
      expected_artifact_generation: "generation-a",
    },
    expect.any(AbortSignal),
  );
});

it("keeps occurrence failure local and delegates generation recovery to the search owner", async () => {
  api.mockRejectedValue(
    new AtlasApiError(409, "Changed", {
      code: "artifact_generation_changed",
      message: "Changed",
    }),
  );
  const view = setup();
  fireEvent.click(screen.getByRole("button", { name: "Next links page" }));
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Previously loaded links remain below.",
  );
  expect(screen.getByRole("link", { name: "Goblin" })).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Refresh results" }));
  await waitFor(() => expect(view.refresh).toHaveBeenCalledOnce());
  expect(api).toHaveBeenCalledOnce();
});

it("resets evidence paging and retained content when the exact relationship changes", async () => {
  api.mockResolvedValue({
    artifact_generation: "generation-a",
    clause: constraint,
    result_record: summaryFixture("actors:goblin", "Goblin"),
    occurrences: [
      {
        kind: "record",
        destination: ordinaryDestination,
        occurrence: { ...occurrence, source_title: "Old evidence page", ordinal: 4 },
      },
    ],
    page: { ...evidence.page, number: 2, has_more: false, next_page: null },
  });
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  const content = (
    summary: RelationshipClauseSummaryView,
    preview: RelationshipEvidenceView,
  ) => (
    <ConfigProvider theme={{ token: { motion: false } }}>
      <QueryClientProvider client={client}>
        <RelationshipEvidence
          evidence={preview}
          filter={{
            kind: "relationship",
            clause_id: summary.clause_id,
            relationship:
              summary.condition.kind === "record"
                ? summary.condition.relationship
                : relationship,
          }}
          summary={summary}
          resultKey="actors:goblin"
          generation="generation-a"
        />
      </QueryClientProvider>
    </ConfigProvider>
  );
  const view = render(content(constraint, evidence));
  fireEvent.click(screen.getByRole("button", { name: "Next links page" }));
  expect(await screen.findByRole("link", { name: "Old evidence page" })).toBeVisible();
  const next = {
    ...constraint,
    title: "Stunned",
    condition: {
      kind: "record" as const,
      relationship: {
        ...relationship,
        selection: rootNavigation("conditions:stunned"),
      },
    },
  };
  view.rerender(
    content(next, {
      ...evidence,
      occurrences: [
        {
          kind: "record",
          destination: ordinaryDestination,
          occurrence: {
            ...occurrence,
            source_title: "New exact evidence",
            target_title: "Stunned",
          },
        },
      ],
    }),
  );
  expect(await screen.findByRole("link", { name: "New exact evidence" })).toBeVisible();
  expect(screen.queryByRole("link", { name: "Old evidence page" })).toBeNull();
  expect(screen.getByRole("textbox", { name: "Go to" })).toHaveValue("1");
  expect(api).toHaveBeenCalledOnce();
});

it("treats a response generation mismatch as explicit result refresh recovery", async () => {
  api.mockResolvedValue({
    artifact_generation: "generation-b",
    clause: constraint,
    result_record: summaryFixture("actors:goblin", "Goblin"),
    occurrences: [],
    page: evidence.page,
  });
  const view = setup();
  fireEvent.click(screen.getByRole("button", { name: "Next links page" }));
  fireEvent.click(await screen.findByRole("button", { name: "Refresh results" }));
  await waitFor(() => expect(view.refresh).toHaveBeenCalledOnce());
});

it.each([
  { name: "result record", resultKey: "actors:other", clause: constraint },
  {
    name: "clause occurrence",
    resultKey: "actors:goblin",
    clause: { ...constraint, clause_id: "other" },
  },
  {
    name: "direction",
    resultKey: "actors:goblin",
    clause: {
      ...constraint,
      condition: {
        kind: "record" as const,
        relationship: { ...relationship, direction: "outgoing" as const },
      },
    },
  },
  {
    name: "contents scope",
    resultKey: "actors:goblin",
    clause: {
      ...constraint,
      condition: {
        kind: "record" as const,
        relationship: { ...relationship, include_contents: false },
      },
    },
  },
  {
    name: "source fingerprint",
    resultKey: "actors:goblin",
    clause: {
      ...constraint,
      condition: {
        kind: "record" as const,
        relationship: {
          ...relationship,
          selection: {
            ...relationship.selection,
            source_fingerprint: "foreign",
          },
        },
      },
    },
  },
])(
  "rejects evidence for a different $name while retaining the prior page",
  async ({ resultKey, clause }) => {
    api.mockResolvedValue({
      artifact_generation: "generation-a",
      clause,
      result_record: summaryFixture(resultKey, "Foreign result"),
      occurrences: [
        {
          kind: "record",
          destination: ordinaryDestination,
          occurrence: { ...occurrence, source_title: "Foreign evidence" },
        },
      ],
      page: { ...evidence.page, number: 2, has_more: false, next_page: null },
    });
    setup();
    fireEvent.click(screen.getByRole("button", { name: "Next links page" }));
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Unable to update links",
    );
    expect(screen.getByRole("link", { name: "Goblin" })).toBeVisible();
    expect(screen.queryByRole("link", { name: "Foreign evidence" })).toBeNull();
    expect(screen.getByRole("button", { name: "Retry" })).toBeEnabled();
    expect(api).toHaveBeenCalledOnce();
  },
);

it("renders definition evidence with supplied titles and verifies its exact target on paging", async () => {
  const definition: RelationshipClauseSummaryView = {
    clause_id: "language",
    condition: {
      kind: "definition",
      reference: {
        target: { kind: "definition", key: "language:chthonian" },
        include_contents: true,
      },
    },
    destination: { kind: "definition", key: "language:chthonian" },
    title: "Chthonian",
  };
  const proof: RelationshipEvidenceView = {
    ...evidence,
    clause_id: "language",
    occurrences: [
      {
        kind: "definition",
        occurrence: {
          source: { kind: "record", navigation: rootNavigation("actors:goblin") },
          source_title: "Goblin",
          source_field: "/system/details/languages/value",
          target: definition.destination,
          target_title: "Chthonian",
          occurrence_path: "/system/details/languages/value/0",
          evidence: "typed_property",
          label: null,
          visible: true,
        },
      },
    ],
  };
  api.mockResolvedValue({
    artifact_generation: "generation-a",
    clause: {
      ...definition,
      condition: {
        kind: "definition",
        reference: {
          target: { kind: "definition", key: "language:common" },
          include_contents: true,
        },
      },
    },
    result_record: summaryFixture("actors:goblin", "Goblin"),
    occurrences: [],
    page: { ...proof.page, number: 2 },
  });
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <RelationshipEvidence
        evidence={proof}
        filter={{
          kind: "definition_reference",
          clause_id: "language",
          reference:
            definition.condition.kind === "definition"
              ? definition.condition.reference
              : neverReference(),
        }}
        summary={definition}
        resultKey="actors:goblin"
        generation="generation-a"
      />
    </QueryClientProvider>,
  );
  expect(screen.getByText("Chthonian")).toBeVisible();
  expect(screen.queryByText("language:chthonian")).toBeNull();
  expect(screen.queryByRole("link", { name: "Chthonian" })).toBeNull();
  expect(screen.getByRole("link", { name: "Goblin" })).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Next links page" }));
  expect(await screen.findByRole("alert")).toHaveTextContent("Unable to update links");
  expect(screen.getByText("Chthonian")).toBeVisible();
});
function neverReference(): never {
  throw new Error("Expected definition fixture");
}

it("retains context-bearing record destinations as supplied titles until contextual reading is activated", () => {
  const destination = {
    kind: "record" as const,
    navigation: rootNavigation("abilities:darkvision"),
    reading_context: "general_definition" as const,
  };
  const reference = {
    target: { kind: "definition" as const, key: "sense:darkvision" },
    include_contents: true,
  };
  render(
    <QueryClientProvider client={new QueryClient()}>
      <RelationshipEvidence
        evidence={{
          ...evidence,
          occurrences: [
            {
              kind: "record",
              destination,
              occurrence: { ...occurrence, target_title: "Darkvision" },
            },
          ],
        }}
        filter={{ kind: "definition_reference", clause_id: "link", reference }}
        summary={{
          clause_id: "link",
          condition: { kind: "definition", reference },
          destination,
          title: "Darkvision",
        }}
        resultKey="actors:goblin"
        generation="generation-a"
      />
    </QueryClientProvider>,
  );
  expect(screen.getByRole("link", { name: "Goblin" })).toBeVisible();
  expect(screen.getByText("Darkvision", { exact: true })).toBeVisible();
  expect(screen.queryByRole("link", { name: "Darkvision" })).toBeNull();
});
