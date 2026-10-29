import { recordDetailRequest } from "./recordDetailQuery";
import type { ReactNode } from "react";
import { retireReferenceQueries } from "../queries/referenceQueries";
import {
  act,
  fireEvent,
  render,
  renderHook,
  screen,
  waitFor,
} from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { RecordDetailView, RecordNavigationView } from "../../generated/atlas";
import { detailFixture } from "../../test/fixtures";
import { useRecordDetail } from "./useRecordDetail";
import { RecordDetailPane } from "./RecordDetailPane";

const api = vi.hoisted(() => ({ getRecordDetail: vi.fn() }));
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  getRecordDetail: api.getRecordDetail,
}));

function pendingDetail() {
  let resolve!: (value: RecordDetailView) => void;
  let reject!: (reason: Error) => void;
  const promise = new Promise<RecordDetailView>((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
}
function Harness({
  recordKey,
  selection,
}: {
  recordKey: string;
  selection?: RecordNavigationView;
}) {
  const query = useRecordDetail(recordKey, selection);
  return (
    <RecordDetailPane
      detail={query.data}
      loading={query.isLoading || query.isFetching}
      error={query.error}
      onRetry={() => {
        void query.refetch();
      }}
      onReference={vi.fn()}
      showRelated={false}
    />
  );
}
function client() {
  return new QueryClient({ defaultOptions: { queries: { retry: false, gcTime: 0 } } });
}

describe("address-safe detail refresh", () => {
  beforeEach(() => api.getRecordDetail.mockReset());

  it("retains same-address cached content during refetch failure and recovers through Retry", async () => {
    const queryClient = client();
    const refresh = pendingDetail();
    api.getRecordDetail
      .mockResolvedValueOnce(detailFixture("spell:heal", "Heal"))
      .mockReturnValueOnce(refresh.promise)
      .mockResolvedValueOnce(detailFixture("spell:heal", "Updated Heal"));
    render(
      <QueryClientProvider client={queryClient}>
        <Harness recordKey="spell:heal" />
      </QueryClientProvider>,
    );
    expect(await screen.findByRole("heading", { name: "Heal" })).toBeVisible();
    await act(async () => {
      void queryClient.invalidateQueries({ queryKey: ["record-detail"] });
    });
    expect(await screen.findByLabelText("Updating record")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Heal" })).toBeVisible();
    await act(async () => refresh.reject(new Error("Temporary service failure")));
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Previously loaded content remains below.",
    );
    expect(screen.getByRole("heading", { name: "Heal" })).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Retry" }));
    expect(await screen.findByRole("heading", { name: "Updated Heal" })).toBeVisible();
    expect(screen.queryByRole("alert")).toBeNull();
    expect(api.getRecordDetail).toHaveBeenCalledTimes(3);
  });

  const initial: RecordNavigationView = {
    record_key: "actors:one",
    embedded_path: [],
    field: "/system/description/value",
    passage: null,
    source_fingerprint: "snapshot-a",
  };
  it.each([
    ["root", "actors:two", { ...initial, record_key: "actors:two" }],
    [
      "owner",
      "actors:one",
      {
        ...initial,
        embedded_path: [
          { collection: "items", identity: { SnapshotLocal: { index: 0 } } },
        ],
      },
    ],
    ["field", "actors:one", { ...initial, field: "/system/details/publicNotes" }],
    ["passage", "actors:one", { ...initial, passage: { kind: "identity" } }],
    ["fingerprint", "actors:one", { ...initial, source_fingerprint: "snapshot-b" }],
  ] satisfies Array<[string, string, RecordNavigationView]>)(
    "does not display the previous address after changing %s",
    async (_part, nextKey, nextSelection) => {
      const queryClient = client();
      const next = pendingDetail();
      api.getRecordDetail
        .mockResolvedValueOnce(detailFixture("actors:one", "Previous selection"))
        .mockReturnValueOnce(next.promise);
      const view = render(
        <QueryClientProvider client={queryClient}>
          <Harness recordKey="actors:one" selection={initial} />
        </QueryClientProvider>,
      );
      expect(
        await screen.findByRole("heading", { name: "Previous selection" }),
      ).toBeVisible();
      view.rerender(
        <QueryClientProvider client={queryClient}>
          <Harness recordKey={nextKey} selection={nextSelection} />
        </QueryClientProvider>,
      );
      await waitFor(() => expect(api.getRecordDetail).toHaveBeenCalledTimes(2));
      expect(screen.queryByRole("heading", { name: "Previous selection" })).toBeNull();
      expect(screen.getByRole("status")).toHaveAccessibleName("Loading record…");
      await act(async () =>
        next.resolve({
          ...detailFixture(nextKey, "Next selection"),
          selected: nextSelection,
        }),
      );
      expect(
        await screen.findByRole("heading", { name: "Next selection" }),
      ).toBeVisible();
      expect(api.getRecordDetail.mock.calls[1]).toEqual([
        nextKey,
        recordDetailRequest(nextSelection),
        expect.any(AbortSignal),
      ]);
    },
  );
});

it("discards retained preview content when its reference query is retired", async () => {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const old = detailFixture("spells:one");
  api.getRecordDetail.mockResolvedValueOnce(old);
  const { result } = renderHook(() => useRecordDetail("spells:one"), {
    wrapper: ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={client}>{children}</QueryClientProvider>
    ),
  });
  await waitFor(() => expect(result.current.data).toEqual(old));
  api.getRecordDetail.mockRejectedValue(new Error("New collection offline"));
  await act(async () => retireReferenceQueries(client));
  await waitFor(() =>
    expect(result.current.error?.message).toBe("New collection offline"),
  );
  expect(result.current.data).toBeUndefined();
});
