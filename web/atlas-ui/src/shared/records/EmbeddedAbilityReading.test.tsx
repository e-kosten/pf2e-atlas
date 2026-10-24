import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ConfigProvider, Grid } from "antd";
import {
  EmbeddedAbilityReading,
  type EmbeddedReadingContext,
} from "./EmbeddedAbilityReading";
import {
  activityFixture,
  actorFixture,
  detailFixture,
  knownFact,
} from "../../test/fixtures";
import type {
  EmbeddedReadingBatchView,
  EmbeddedReadingRequest,
} from "../../generated/atlas";
import { recordDetailQueryOptions } from "./recordDetailQuery";
import { RecordReferenceDrawer } from "./RecordReferenceDrawer";
import { atlasRoutePath } from "../../app/routes";
import { AtlasApiError } from "../../api/atlasApi";

const api = vi.hoisted(() => ({
  getEmbeddedReading: vi.fn(),
  getRecordDetail: vi.fn(),
}));
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  ...api,
}));
const abilities = Array.from({ length: 600 }, (_, index) => ({
  ...activityFixture(`Ability ${index}`, "actors:large"),
  kind: "ability" as const,
  navigation: {
    ...activityFixture().navigation,
    record_key: "actors:large",
    embedded_path: [{ collection: "items", identity: { Stable: `ability-${index}` } }],
  },
  action_cost: knownFact({ label: "Reaction", glyph: "reaction" as const }),
}));
const context: EmbeddedReadingContext = {
  generation: "generation-a",
  parent: {
    record_key: "actors:large",
    embedded_path: [],
    field: null,
    passage: null,
    source_fingerprint: null,
  },
};
function batch(request: EmbeddedReadingRequest): EmbeddedReadingBatchView {
  if (request.selection.kind !== "abilities") throw new Error("wrong selection");
  const target = request.selection.target;
  const index = target
    ? abilities.findIndex(
        (activity) => JSON.stringify(activity.navigation) === JSON.stringify(target),
      )
    : 0;
  const start = request.selection.cursor?.offset ?? Math.floor(index / 32) * 32;
  const end = Math.min(start + 32, abilities.length);
  const cursor = (offset: number) => ({
    artifact_generation: request.artifact_generation,
    offset,
  });
  return {
    artifact_generation: request.artifact_generation,
    parent: request.parent,
    availability: "value",
    total: 600,
    start,
    end,
    previous: start ? cursor(Math.max(0, start - 32)) : null,
    next: end < 600 ? cursor(end) : null,
    entries: abilities.slice(start, end).map((activity) => ({
      navigation: activity.navigation,
      content: [
        {
          locator: {
            record: "actors:large",
            embedded_path: activity.navigation.embedded_path,
            field: "/system/description/value",
          },
          role: "description",
          source_fingerprint: null,
          body: { kind: "plain", text: `${activity.title} authored explanation` },
        },
      ],
      relationships: [],
    })),
  };
}
function mount(readingContext = context) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  const view = (value: EmbeddedReadingContext) => (
    <QueryClientProvider client={client}>
      <ConfigProvider theme={{ token: { motion: false } }}>
        <div data-testid="reading-host">
          <EmbeddedAbilityReading
            abilities={abilities}
            context={value}
            onReference={vi.fn()}
          />
        </div>
      </ConfigProvider>
    </QueryClientProvider>
  );
  const result = render(view(readingContext));
  const host = screen.getByTestId("reading-host");
  vi.spyOn(host, "getBoundingClientRect").mockReturnValue(new DOMRect(0, 0, 400, 300));
  return {
    ...result,
    client,
    host,
    changeIdentity: (value: EmbeddedReadingContext) => result.rerender(view(value)),
  };
}
beforeEach(() => {
  api.getEmbeddedReading.mockReset();
  api.getRecordDetail.mockReset();
  api.getEmbeddedReading.mockImplementation((request) =>
    Promise.resolve(batch(request)),
  );
});

it("keeps a generation conflict paused until the actual parent owner reloads", async () => {
  api.getEmbeddedReading.mockRejectedValue(
    new AtlasApiError(409, "The reference collection changed", {
      code: "artifact_generation_changed",
      message: "The reference collection changed",
      recoverable_action: "reload_record",
    }),
  );
  const reloadParent = vi.fn();
  mount({ ...context, reloadParent, reloadParentLabel: "Reload encounter" });
  expect(await screen.findByText("The reference collection changed")).toBeVisible();
  const retry = screen.getByRole("button", { name: "Retry" });
  expect(retry).toBeDisabled();
  fireEvent.click(retry);
  expect(api.getEmbeddedReading).toHaveBeenCalledTimes(1);
  fireEvent.click(screen.getByRole("button", { name: "Reload encounter" }));
  expect(reloadParent).toHaveBeenCalledTimes(1);
  expect(api.getEmbeddedReading).toHaveBeenCalledTimes(1);
});

it("reads a finite default batch inline, preserves native permalinks and never seeds full detail", async () => {
  const { client } = mount();
  await screen.findByText("Ability 0 authored explanation");
  expect(screen.getByText("Ability 31 authored explanation")).toBeVisible();
  expect(
    screen.getByRole("heading", { name: /Ability 0 Reaction/, level: 4 }),
  ).toBeVisible();
  expect(screen.queryByRole("heading", { name: "Description" })).toBeNull();
  expect(screen.queryByText("Ability 32 authored explanation")).toBeNull();
  await act(async () => {
    await new Promise((resolve) => setTimeout(resolve, 30));
  });
  expect(api.getEmbeddedReading).toHaveBeenCalledTimes(1); // Visible idle sentinel cannot drain 600.
  expect(api.getRecordDetail).not.toHaveBeenCalled();
  expect(
    client.getQueryData(
      recordDetailQueryOptions("actors:large", abilities[0].navigation).queryKey,
    ),
  ).toBeUndefined();
  const link = screen.getByRole("link", { name: "Ability 0" });
  expect(link).toHaveAttribute(
    "href",
    atlasRoutePath({
      kind: "record",
      recordKey: "actors:large",
      selection: abilities[0].navigation,
    }),
  );
  fireEvent.click(link, { ctrlKey: true });
  expect(api.getEmbeddedReading).toHaveBeenCalledTimes(1);
  expect(
    document.querySelectorAll('[data-atlas-action-glyph="reaction"]'),
  ).toHaveLength(32);
});

it("advances only for its actual scrolling host, halts on collapse, and retains text through local retry", async () => {
  const { host } = mount();
  await screen.findByText("Ability 0 authored explanation");
  const unrelated = document.createElement("div");
  document.body.append(unrelated);
  unrelated.scrollTop = 100;
  fireEvent.scroll(unrelated);
  expect(api.getEmbeddedReading).toHaveBeenCalledTimes(1);
  fireEvent.click(screen.getByRole("button", { name: "Abilities" }));
  host.scrollTop = 100;
  fireEvent.scroll(host);
  expect(api.getEmbeddedReading).toHaveBeenCalledTimes(1);
  fireEvent.click(screen.getByRole("button", { name: "Abilities" }));
  await screen.findByText("Ability 0 authored explanation");
  api.getEmbeddedReading.mockRejectedValueOnce(new Error("Late batch failed"));
  host.scrollTop = 200;
  fireEvent.scroll(host);
  await screen.findByText("Late batch failed");
  expect(screen.getByText("Ability 0 authored explanation")).toBeVisible();
  host.scrollTop = 300;
  fireEvent.scroll(host);
  expect(api.getEmbeddedReading).toHaveBeenCalledTimes(2);
  fireEvent.click(screen.getByRole("button", { name: "Retry" }));
  await screen.findByText("Ability 32 authored explanation");
  expect(api.getEmbeddedReading).toHaveBeenCalledTimes(3);
  unrelated.remove();
});

it("lands directly on a late checked owner with a readable backward and forward path", async () => {
  const { host } = mount({ ...context, landing: abilities[400].navigation });
  await screen.findByText("Ability 400 authored explanation");
  expect(api.getEmbeddedReading).toHaveBeenCalledTimes(1);
  expect(api.getEmbeddedReading.mock.calls[0][0].selection.target).toEqual(
    abilities[400].navigation,
  );
  expect(screen.queryByText("Ability 0 authored explanation")).toBeNull();
  expect(
    screen.getByText("Earlier abilities are outside this reading range."),
  ).toBeVisible();
  expect(screen.getByRole("heading", { name: /Ability 400 Reaction/ })).toHaveFocus();
  fireEvent.click(screen.getByRole("button", { name: "Read earlier abilities" }));
  await screen.findByText("Ability 352 authored explanation");
  expect(screen.getByText("Ability 400 authored explanation")).toBeVisible();
  host.scrollTop = 200;
  fireEvent.scroll(host);
  await screen.findByText("Ability 416 authored explanation");
  expect(api.getEmbeddedReading).toHaveBeenCalledTimes(3);
});

it("cancels a pending old-generation read when identity changes and discards its late result", async () => {
  let resolveOld: ((value: EmbeddedReadingBatchView) => void) | undefined;
  api.getEmbeddedReading.mockImplementationOnce(
    () =>
      new Promise<EmbeddedReadingBatchView>((resolve) => {
        resolveOld = resolve;
      }),
  );
  const { changeIdentity } = mount();
  await waitFor(() => expect(api.getEmbeddedReading).toHaveBeenCalledTimes(1));
  const [oldRequest, signal] = api.getEmbeddedReading.mock.calls[0];
  changeIdentity({ ...context, generation: "generation-b" });
  await screen.findByText("Ability 0 authored explanation");
  expect(signal.aborted).toBe(true);
  await act(async () => {
    resolveOld?.({ ...batch(oldRequest), entries: [] });
  });
  expect(screen.getByText("Ability 0 authored explanation")).toBeVisible();
  expect(api.getEmbeddedReading).toHaveBeenCalledTimes(2);
});

it.each(["desktop", "narrow", "comparison"])(
  "uses the actual %s scrolling container and never drains a visible idle sentinel",
  async (surface) => {
    vi.spyOn(Grid, "useBreakpoint").mockReturnValue({
      md: surface !== "narrow",
      xs: surface === "narrow",
    });
    const client = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const detail = detailFixture(context.parent.record_key);
    detail.artifact_generation = context.generation;
    detail.presentation.body = {
      kind: "creature",
      value: { ...actorFixture(), activities: knownFact(abilities) },
    };
    const content = (
      <EmbeddedAbilityReading
        abilities={abilities}
        context={context}
        onReference={vi.fn()}
      />
    );
    render(
      <QueryClientProvider client={client}>
        <ConfigProvider theme={{ token: { motion: false } }}>
          {surface === "comparison" ? (
            <div className="comparison-test-host">{content}</div>
          ) : (
            <RecordReferenceDrawer
              anchor={document.createElement("button")}
              detail={detail}
              loading={false}
              onClose={vi.fn()}
              onOpenFullPage={vi.fn()}
              onReference={vi.fn()}
            />
          )}
        </ConfigProvider>
      </QueryClientProvider>,
    );
    await waitFor(() =>
      expect(screen.getByText("Ability 0 authored explanation")).toBeVisible(),
    );
    const host = document.querySelector<HTMLElement>(
      surface === "comparison"
        ? ".comparison-test-host"
        : ".record-preview-drawer__body",
    )!;
    vi.spyOn(host, "getBoundingClientRect").mockReturnValue(
      new DOMRect(0, 0, 400, 300),
    );
    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 30));
    });
    expect(api.getEmbeddedReading).toHaveBeenCalledTimes(1);
    host.scrollTop = 120;
    fireEvent.scroll(host);
    await screen.findByText("Ability 32 authored explanation");
    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 30));
    });
    expect(api.getEmbeddedReading).toHaveBeenCalledTimes(2);
    vi.restoreAllMocks();
  },
);

it("reuses immutable generation content on remount without refetching every loaded batch", async () => {
  const mounted = mount();
  await screen.findByText("Ability 0 authored explanation");
  mounted.host.scrollTop = 120;
  fireEvent.scroll(mounted.host);
  await screen.findByText("Ability 32 authored explanation");
  mounted.unmount();
  render(
    <QueryClientProvider client={mounted.client}>
      <ConfigProvider theme={{ token: { motion: false } }}>
        <EmbeddedAbilityReading
          abilities={abilities}
          context={context}
          onReference={vi.fn()}
        />
      </ConfigProvider>
    </QueryClientProvider>,
  );
  await screen.findByText("Ability 32 authored explanation");
  await act(async () => {
    await new Promise((resolve) => setTimeout(resolve, 30));
  });
  expect(api.getEmbeddedReading).toHaveBeenCalledTimes(2);
});
