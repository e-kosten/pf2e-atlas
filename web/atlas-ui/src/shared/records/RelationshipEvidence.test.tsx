import { ConfigProvider } from "antd";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { AtlasApiError, getRelationshipOccurrences } from "../../api/atlasApi";
import { summaryFixture } from "../../test/fixtures";
import { RelationshipEvidence } from "./RelationshipEvidence";
import { rootNavigation } from "./useReferenceReading";
import type {
  RecordRelationshipView,
  RelationshipConstraintSummaryView,
  RelationshipEvidenceView,
} from "../../generated/atlas";

vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getRelationshipOccurrences: vi.fn(),
}));
const api = vi.mocked(getRelationshipOccurrences);
const constraint: RelationshipConstraintSummaryView = {
  constraint: {
    direction: "incoming",
    selection: rootNavigation("items:armor"),
    include_contents: true,
  },
  title: "Leather Armor",
  record: summaryFixture("items:armor", "Leather Armor"),
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
  origin_label: "Description link",
};
const evidence: RelationshipEvidenceView = {
  constraint_index: 0,
  occurrences: [occurrence],
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
    constraint,
    result_record: summaryFixture("actors:goblin", "Goblin"),
    occurrences: [{ ...occurrence, source_title: "Goblin equipment", ordinal: 4 }],
    page: { ...evidence.page, number: 2, has_more: false, next_page: null },
  });
  const view = setup();
  expect(api).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "2 links" }));
  expect(screen.getByText("1 of 2 links")).toBeVisible();
  const source = screen.getByRole("link", { name: "Goblin" });
  expect(
    JSON.parse(
      new URL(source.getAttribute("href")!, "http://atlas.local").searchParams.get(
        "selection",
      )!,
    ),
  ).toEqual(occurrence.source_navigation);
  expect(view.container).not.toHaveTextContent("technical:uuid");
  expect(view.container).not.toHaveTextContent("/system/description/value");
  fireEvent.click(screen.getByRole("button", { name: "Next links page" }));
  expect(await screen.findByRole("link", { name: "Goblin equipment" })).toBeVisible();
  expect(api).toHaveBeenCalledWith(
    {
      constraint: constraint.constraint,
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
  fireEvent.click(screen.getByRole("button", { name: "2 links" }));
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
    constraint,
    result_record: summaryFixture("actors:goblin", "Goblin"),
    occurrences: [{ ...occurrence, source_title: "Old evidence page", ordinal: 4 }],
    page: { ...evidence.page, number: 2, has_more: false, next_page: null },
  });
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  const content = (
    summary: RelationshipConstraintSummaryView,
    preview: RelationshipEvidenceView,
  ) => (
    <ConfigProvider theme={{ token: { motion: false } }}>
      <QueryClientProvider client={client}>
        <RelationshipEvidence
          evidence={preview}
          summary={summary}
          resultKey="actors:goblin"
          generation="generation-a"
        />
      </QueryClientProvider>
    </ConfigProvider>
  );
  const view = render(content(constraint, evidence));
  fireEvent.click(screen.getByRole("button", { name: "2 links" }));
  fireEvent.click(screen.getByRole("button", { name: "Next links page" }));
  expect(await screen.findByRole("link", { name: "Old evidence page" })).toBeVisible();
  const next = {
    ...constraint,
    title: "Stunned",
    constraint: {
      ...constraint.constraint,
      selection: rootNavigation("conditions:stunned"),
    },
  };
  view.rerender(
    content(next, {
      ...evidence,
      occurrences: [
        { ...occurrence, source_title: "New exact evidence", target_title: "Stunned" },
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
    constraint,
    result_record: summaryFixture("actors:goblin", "Goblin"),
    occurrences: [],
    page: evidence.page,
  });
  const view = setup();
  fireEvent.click(screen.getByRole("button", { name: "2 links" }));
  fireEvent.click(screen.getByRole("button", { name: "Next links page" }));
  fireEvent.click(await screen.findByRole("button", { name: "Refresh results" }));
  await waitFor(() => expect(view.refresh).toHaveBeenCalledOnce());
});
