import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ConfigProvider } from "antd";
import type {
  GraphContextView,
  RecordNavigationView,
  RecordRelationshipView,
} from "../../generated/atlas";
import { detailFixture, summaryFixture } from "../../test/fixtures";
import { AtlasApiError } from "../../api/atlasApi";
import { useRecordDetail } from "./useRecordDetail";
import { RelatedRecords } from "./RelatedRecords";

const api = vi.hoisted(() => ({
  getRelatedRecords: vi.fn(),
  openResultWindow: vi.fn(),
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

function relatedFixture() {
  const detail = detailFixture("actors:seed", "Viewed creature");
  return {
    selection: detail.selected,
    artifact_generation: detail.artifact_generation,
    seed: detail.record,
    references: { kind: "content" as const, content: graphFixture() },
    counterparts: { kind: "empty" as const },
    variants: { kind: "empty" as const },
  };
}

describe("shared related disclosure", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    api.getRelatedRecords.mockResolvedValue(relatedFixture());
    api.openResultWindow.mockResolvedValue({
      window_id: 1n,
      mode: { kind: "similar_records", seed: summaryFixture("actors:seed") },
      rows: [],
      page: {
        number: 1,
        size: 5,
        count: 0,
        total: 0n,
        has_more: false,
        next_page: null,
      },
      coverage: {
        exhaustive: false,
        semantic_unit_window: 100,
        candidate_roots: 12,
        count_basis: "internal-count-basis",
      },
    });
  });
  it("requests one bounded availability operation on opening and hides empty category controls", async () => {
    renderRelated();
    expect(api.getRelatedRecords).not.toHaveBeenCalled();
    openRelated();
    expect(await screen.findByRole("link", { name: "Referenced spell" })).toBeVisible();
    expect(api.getRelatedRecords).toHaveBeenCalledTimes(1);
    expect(screen.queryByRole("tab", { name: "Remaster" })).toBeNull();
    expect(screen.queryByRole("tab", { name: "Variants" })).toBeNull();
    expect(screen.queryByRole("tab", { name: "Similar" })).toBeNull();
    expect(api.openResultWindow).not.toHaveBeenCalled();
  });
  it("preserves distinct incoming occurrences and checked navigation; modified links stay native", async () => {
    const onReference = renderRelated();
    openRelated();
    const incoming = await screen.findAllByRole("link", { name: "Referring creature" });
    expect(incoming).toHaveLength(2);
    const expected = graphFixture().backlinks.occurrences[0].source_navigation;
    expect(
      JSON.parse(
        new URL(
          incoming[0].getAttribute("href")!,
          "http://atlas.local",
        ).searchParams.get("selection")!,
      ),
    ).toEqual(expected);
    fireEvent.click(incoming[0]);
    expect(onReference).toHaveBeenCalledWith("actors:referrer", incoming[0], expected);
    onReference.mockClear();
    expect(fireEvent.click(incoming[0], { metaKey: true })).toBe(true);
    expect(onReference).not.toHaveBeenCalled();
    expect(screen.queryByText("Compendium.raw.internal.uuid")).toBeNull();
  });
  it("reloads the exact record query when the artifact generation changes", async () => {
    const oldDetail = detailFixture("actors:seed", "Viewed creature");
    const newDetail = { ...oldDetail, artifact_generation: "new-preparation" };
    api.getRecordDetail.mockResolvedValueOnce(oldDetail).mockResolvedValue(newDetail);
    api.getRelatedRecords.mockRejectedValueOnce(
      new AtlasApiError(400, "Generation changed", {
        code: "invalid_request",
        message: "Generation changed",
        recoverable_action: "reload_record",
      }),
    );
    function Reader() {
      const detail = useRecordDetail("actors:seed");
      return detail.data ? (
        <RelatedRecords detail={detail.data} onReference={vi.fn()} />
      ) : null;
    }
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    render(
      <ConfigProvider theme={{ token: { motion: false } }}>
        <QueryClientProvider client={client}>
          <Reader />
        </QueryClientProvider>
      </ConfigProvider>,
    );
    fireEvent.click(await screen.findByRole("button", { name: "Related records" }));
    fireEvent.click(await screen.findByRole("button", { name: "Reload record" }));
    await waitFor(() =>
      expect(api.getRelatedRecords).toHaveBeenLastCalledWith({
        selection: newDetail.selected,
        artifact_generation: "new-preparation",
      }),
    );
    expect(api.getRecordDetail).toHaveBeenCalledTimes(2);
    client.clear();
  });

  it("uses local retry and distinguishes unavailable checks from verified empty", async () => {
    api.getRelatedRecords.mockRejectedValueOnce(new Error("Service unavailable"));
    renderRelated();
    openRelated();
    expect(await screen.findByRole("alert")).toHaveTextContent("Service unavailable");
    api.getRelatedRecords.mockResolvedValue({
      ...relatedFixture(),
      references: { kind: "empty" },
      variants: { kind: "unavailable", reason: "Compatibility facts are unavailable" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Retry" }));
    expect(
      await screen.findByText(/Compatibility facts are unavailable/),
    ).toBeVisible();
    await waitFor(() => expect(screen.queryByRole("alert")).toBeNull());
    expect(screen.queryByRole("tab")).toBeNull();
  });
  it("sends complete owned selection and generation and uses root discovery separately", async () => {
    const detail = detailFixture("actors:seed", "Viewed creature");
    detail.selected = {
      ...detail.selected,
      owners: [{ collection: "items", identity: { Stable: "embedded" } }],
      field: "/system/description/value",
    };
    renderRelated(detail);
    openRelated();
    await waitFor(() =>
      expect(api.getRelatedRecords).toHaveBeenCalledWith({
        selection: detail.selected,
        artifact_generation: detail.artifact_generation,
      }),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Similar to parent record: Viewed creature" }),
    );
    expect(await screen.findByText("No similar records found.")).toBeVisible();
    expect(api.openResultWindow).toHaveBeenCalledWith(
      {
        mode: { kind: "similar_records", seed_record_key: "actors:seed", filter: null },
        page: { number: 1, size: 5 },
      },
      expect.any(AbortSignal),
    );
    const link = screen.getByRole("link", { name: "See more similar records" });
    expect(
      new URL(link.getAttribute("href")!, "http://atlas.local").searchParams.get(
        "similar",
      ),
    ).toBe("actors:seed");
    expect(fireEvent.click(link, { metaKey: true })).toBe(true);
    expect(screen.queryByText("About these suggestions")).toBeNull();
    expect(screen.queryByText(/internal-count-basis/)).toBeNull();
  });
  it("comparison uses shared family panes without recursive related queries", async () => {
    api.getRelatedRecords.mockResolvedValue({
      ...relatedFixture(),
      variants: {
        kind: "content",
        content: {
          seed: summaryFixture("actors:seed"),
          base_name: "Viewed creature",
          variants: [
            summaryFixture("actors:seed", "Viewed creature"),
            summaryFixture("actors:variant", "Greater creature"),
          ],
          evidence: [],
          ambiguous: false,
          truncated: false,
        },
      },
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
    fireEvent.click(await screen.findByRole("tab", { name: "Variants" }));
    fireEvent.mouseDown(
      await screen.findByRole("combobox", { name: "Compare variant" }),
    );
    const choices = screen.getAllByText("Greater creature");
    fireEvent.click(choices[choices.length - 1]);
    const dialog = await screen.findByRole("dialog");
    await waitFor(() =>
      expect(
        within(dialog).getByRole("heading", { name: "Greater creature" }),
      ).toBeVisible(),
    );
    expect(
      within(dialog).queryByRole("button", { name: "Related records" }),
    ).toBeNull();
    expect(api.getRelatedRecords).toHaveBeenCalledTimes(1);
  });
});
