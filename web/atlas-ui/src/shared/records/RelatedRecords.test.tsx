import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ConfigProvider } from "antd";
import type {
  GraphContextView,
  RecordNavigationView,
  RecordRelationshipView,
} from "../../generated/atlas";
import { detailFixture, summaryFixture } from "../../test/fixtures";
import { RelatedRecords } from "./RelatedRecords";

const api = vi.hoisted(() => ({
  getRecordGraph: vi.fn(),
  getRecordRemaster: vi.fn(),
  getRecordVariants: vi.fn(),
  getSimilarRecords: vi.fn(),
  getRecordDetail: vi.fn(),
}));
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  ...api,
}));

function navigation(key: string): RecordNavigationView {
  return {
    record_key: key,
    owners: [],
    field: null,
    passage: null,
    source_fingerprint: null,
  };
}
function graphFixture(): GraphContextView {
  const seed = summaryFixture("actors:seed", "Viewed creature");
  const source = {
    record: "actors:referrer",
    owners: [{ collection: "items", identity: { SnapshotLocal: { index: 0 } } }],
    field: "/system/description/value",
  };
  const incoming: RecordRelationshipView = {
    source,
    source_navigation: {
      ...navigation(source.record),
      owners: source.owners,
      field: source.field,
      source_fingerprint: "snapshot-current",
    },
    ordinal: 4,
    origin: "prepared_content",
    kind: "uuid",
    authored_target: "Compendium.raw.internal.uuid",
    occurrence_path: null,
    audiences: [],
    availability: null,
    status: "resolved_record",
    target: navigation(seed.record_key),
    url: null,
  };
  const outgoing = {
    ...incoming,
    source: { record: seed.record_key, owners: [], field: source.field },
    source_navigation: { ...navigation(seed.record_key), field: source.field },
    target: navigation("spell:target"),
  };
  return {
    seed,
    outgoing: {
      records: [summaryFixture("spell:target", "Referenced spell")],
      occurrences: [outgoing],
      truncated: false,
    },
    backlinks: {
      records: [summaryFixture("actors:referrer", "Referring creature")],
      occurrences: [incoming, { ...incoming, ordinal: 5 }],
      truncated: true,
    },
  };
}
function renderRelated(detail = detailFixture("actors:seed", "Viewed creature")) {
  const onReference = vi.fn();
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <QueryClientProvider client={queryClient}>
        <RelatedRecords detail={detail} onReference={onReference} />
      </QueryClientProvider>
    </ConfigProvider>,
  );
  return onReference;
}
function openRelated() {
  fireEvent.click(screen.getByRole("button", { name: "Related records" }));
}

describe("shared related disclosure", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    api.getRecordGraph.mockResolvedValue(graphFixture());
    api.getRecordRemaster.mockResolvedValue({
      seed: summaryFixture("actors:seed"),
      links: [],
    });
    api.getRecordVariants.mockResolvedValue({
      seed: summaryFixture("actors:seed"),
      base_name: "Viewed creature",
      variants: [],
      evidence: [],
      ambiguous: false,
      truncated: false,
    });
    api.getSimilarRecords.mockResolvedValue({
      seed: summaryFixture("actors:seed"),
      results: {
        rows: [],
        page: {
          number: 1,
          size: 25,
          count: 0,
          total: 0,
          has_more: false,
          next_page: null,
        },
        coverage: {
          exhaustive: false,
          semantic_unit_window: 100,
          candidate_roots: 12,
          count_basis: "internal-count-basis",
        },
      },
    });
  });

  it("starts no requests closed, and fetches only the selected tab", async () => {
    renderRelated();
    expect(api.getRecordGraph).not.toHaveBeenCalled();
    expect(api.getRecordRemaster).not.toHaveBeenCalled();
    expect(api.getRecordVariants).not.toHaveBeenCalled();
    expect(api.getSimilarRecords).not.toHaveBeenCalled();
    openRelated();
    expect(await screen.findByRole("link", { name: "Referenced spell" })).toBeVisible();
    expect(api.getRecordGraph).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole("tab", { name: "Remaster" }));
    expect(
      await screen.findByText(
        "No verified legacy or remastered counterpart was found.",
      ),
    ).toBeVisible();
    expect(api.getRecordRemaster).toHaveBeenCalledTimes(1);
    expect(api.getRecordVariants).not.toHaveBeenCalled();
    expect(api.getSimilarRecords).not.toHaveBeenCalled();
  });

  it("preserves distinct incoming occurrences and uses checked source owner/field/generation", async () => {
    const onReference = renderRelated();
    openRelated();
    const incoming = await screen.findAllByRole("link", { name: "Referring creature" });
    expect(incoming).toHaveLength(2);
    const expected = graphFixture().backlinks.occurrences[0].source_navigation;
    const url = new URL(incoming[0].getAttribute("href")!, "http://atlas.local");
    expect(JSON.parse(url.searchParams.get("selection")!)).toEqual(expected);
    fireEvent.click(incoming[0]);
    expect(onReference).toHaveBeenCalledWith("actors:referrer", incoming[0], expected);
    fireEvent.click(screen.getByRole("link", { name: "Referenced spell" }));
    expect(onReference).toHaveBeenLastCalledWith(
      "spell:target",
      expect.any(HTMLElement),
      navigation("spell:target"),
    );
    expect(screen.queryByText("Compendium.raw.internal.uuid")).toBeNull();
    expect(screen.getByText(/additional references may exist/)).toBeVisible();
  });

  it("keeps modified occurrence clicks native", async () => {
    const onReference = renderRelated();
    openRelated();
    const link = await screen.findByRole("link", { name: "Referenced spell" });
    expect(fireEvent.click(link, { metaKey: true })).toBe(true);
    expect(onReference).not.toHaveBeenCalled();
  });

  it("uses local retry and separates empty directions", async () => {
    api.getRecordGraph.mockRejectedValueOnce(new Error("Graph service unavailable"));
    renderRelated();
    openRelated();
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Graph service unavailable",
    );
    api.getRecordGraph.mockResolvedValue({
      ...graphFixture(),
      outgoing: { records: [], occurrences: [], truncated: false },
      backlinks: { records: [], occurrences: [], truncated: false },
    });
    fireEvent.click(screen.getByRole("button", { name: "Retry" }));
    expect(await screen.findByText("No references were found here.")).toBeVisible();
    expect(screen.getByText("No references to this record were found.")).toBeVisible();
    await waitFor(() => expect(screen.queryByRole("alert")).toBeNull());
  });

  it("explains parent scope and bounded similarity without internal retrieval terms", async () => {
    const detail = detailFixture("actors:seed", "Viewed creature");
    detail.selected = {
      ...navigation("actors:seed"),
      owners: [{ collection: "items", identity: { Stable: "embedded" } }],
    };
    renderRelated(detail);
    openRelated();
    fireEvent.click(screen.getByRole("tab", { name: "Similar" }));
    expect(await screen.findByText("No suggestions found.")).toBeVisible();
    expect(
      screen.getByText(/suggestions are for the parent record, Viewed creature/),
    ).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "About these suggestions" }));
    expect(screen.getByText("More matches may exist.")).toBeVisible();
    expect(screen.queryByText(/internal-count-basis/)).toBeNull();
    expect(api.getSimilarRecords).toHaveBeenCalledWith("actors:seed");
  });

  it("comparison uses shared family panes without nested related disclosures", async () => {
    api.getRecordVariants.mockResolvedValue({
      seed: summaryFixture("actors:seed"),
      base_name: "Viewed creature",
      variants: [
        summaryFixture("actors:seed", "Viewed creature"),
        summaryFixture("actors:variant", "Greater creature"),
      ],
      evidence: [],
      ambiguous: false,
      truncated: false,
    });
    api.getRecordDetail.mockImplementation((key: string) =>
      Promise.resolve(
        detailFixture(
          key,
          key === "actors:seed" ? "Viewed creature" : "Greater creature",
        ),
      ),
    );
    renderRelated();
    openRelated();
    fireEvent.click(screen.getByRole("tab", { name: "Variants" }));
    const selector = await screen.findByRole("combobox", { name: "Compare variant" });
    fireEvent.mouseDown(selector);
    const choices = screen.getAllByText("Greater creature");
    fireEvent.click(choices[choices.length - 1]);
    const dialog = await screen.findByRole("dialog");
    await waitFor(() => {
      expect(
        within(dialog).getByRole("heading", { name: "Viewed creature" }),
      ).toBeVisible();
      expect(
        within(dialog).getByRole("heading", { name: "Greater creature" }),
      ).toBeVisible();
    });
    expect(
      within(dialog).queryByRole("button", { name: "Related records" }),
    ).toBeNull();
  });
});
