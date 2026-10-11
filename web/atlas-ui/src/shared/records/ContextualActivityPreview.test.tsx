import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { App, ConfigProvider, Grid } from "antd";
import { StrictMode } from "react";
import type {
  ActorActivityView,
  OwnedReadingBatchView,
  OwnedReadingRequest,
} from "../../generated/atlas";
import { activityFixture, numberFact } from "../../test/fixtures";
import { useRecordPreview } from "./useRecordPreview";
import { RecordPreviewPopover } from "./RecordPreviewPopover";
import { ActivityContentPane } from "./ActivityContentPane";
import { atlasRoutePath } from "../../app/routes";

const api = vi.hoisted(() => ({ getOwnedReading: vi.fn(), getRecordDetail: vi.fn() }));
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  ...api,
}));
function response(request: OwnedReadingRequest): OwnedReadingBatchView {
  if (request.selection.kind !== "activity") throw new Error("wrong request");
  return {
    artifact_generation: request.artifact_generation,
    parent: request.parent,
    availability: "value",
    total: 1,
    start: 0,
    end: 1,
    previous: null,
    next: null,
    entries: [
      {
        navigation: request.selection.navigation,
        relationships: [],
        content: [
          {
            locator: {
              record: request.parent.record_key,
              owners: request.selection.navigation.owners,
              field: "/system/description/value",
            },
            role: "description",
            source_fingerprint: "snapshot",
            body: {
              kind: "html",
              html: '<p>Exact local Jaws prose <span data-atlas-interaction="0">Reflex check</span></p>',
              controls: [
                {
                  ordinal: 0,
                  control: {
                    kind: "check",
                    statistic: "Reflex",
                    options: { dc: "15" },
                  },
                },
              ],
            },
          },
        ],
      },
    ],
  };
}
function Flow({
  activity,
  identity = "participant-a",
  generation = "generation-a",
}: {
  activity?: ActorActivityView;
  identity?: string;
  generation?: string | null;
}) {
  const jaws = activityFixture();
  const preview = useRecordPreview({
    identity,
    generation,
    parentRecordKey: jaws.navigation.record_key,
    activities: activity ? [activity] : [],
  });
  return (
    <>
      <button
        onClick={(event) =>
          preview.open(jaws.navigation.record_key, event.currentTarget, jaws.navigation)
        }
      >
        Jaws quick lookup
      </button>
      {preview.recordKey && (
        <RecordPreviewPopover
          anchor={preview.anchor}
          detail={preview.detail}
          loading={preview.loading}
          error={preview.error}
          onRetry={preview.retry}
          onClose={preview.close}
          onOpenFullPage={vi.fn()}
          onReference={preview.open}
          fullPageHref={atlasRoutePath({
            kind: "record",
            recordKey: jaws.navigation.record_key,
            selection: preview.selection,
          })}
          content={
            preview.contextual && (
              <ActivityContentPane
                activity={preview.contextual.activity}
                entry={preview.contextual.entry}
                error={preview.error}
                loading={preview.loading}
                onRetry={preview.contextual.recoverable ? preview.retry : undefined}
                onReference={preview.open}
              />
            )
          }
        />
      )}
    </>
  );
}
function mount(activity = activityFixture()) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: Infinity } },
  });
  const view = (
    value?: ActorActivityView,
    identity = "participant-a",
    generation: string | null = "generation-a",
  ) => (
    <StrictMode>
      <QueryClientProvider client={client}>
        <ConfigProvider theme={{ token: { motion: false } }}>
          <App>
            <Flow activity={value} identity={identity} generation={generation} />
          </App>
        </ConfigProvider>
      </QueryClientProvider>
    </StrictMode>
  );
  const mounted = render(view(activity));
  return {
    ...mounted,
    client,
    update: (
      value?: ActorActivityView,
      identity = "participant-a",
      generation: string | null = "generation-a",
    ) => mounted.rerender(view(value, identity, generation)),
  };
}
beforeEach(() => {
  api.getOwnedReading.mockReset();
  api.getRecordDetail.mockReset();
  api.getOwnedReading.mockImplementation((request) =>
    Promise.resolve(response(request)),
  );
});
afterEach(() => vi.restoreAllMocks());

it("composes cached authored prose with current adjusted facts and explanations on every update", async () => {
  vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ xl: true });
  const jaws = activityFixture();
  const { update, client } = mount(jaws);
  fireEvent.click(screen.getByRole("button", { name: "Jaws quick lookup" }));
  await waitFor(() => expect(screen.getByText(/Exact local Jaws prose/)).toBeVisible());
  expect(screen.getByText("+9")).toBeVisible();
  update({
    ...jaws,
    attack: numberFact(17),
    notes: [
      {
        source: "Clumsy",
        label: "Clumsy 1",
        reason: "Current condition penalty is included.",
      },
    ],
  });
  expect(screen.getByText("+17")).toBeVisible();
  expect(screen.queryByText("+9")).toBeNull();
  expect(
    screen.getByText("Clumsy 1: Current condition penalty is included."),
  ).toBeVisible();
  expect(api.getOwnedReading).toHaveBeenCalledTimes(1);
  expect(api.getRecordDetail).not.toHaveBeenCalled();
  expect(
    client
      .getQueryCache()
      .findAll({ queryKey: ["owned-reading"] })
      .map((query) => JSON.stringify(query.state.data))
      .join(),
  ).not.toContain('"attack"');
  update(undefined);
  expect(
    screen.getByText(
      "This activity is no longer available on the current participant.",
    ),
  ).toBeVisible();
  expect(screen.queryByText("+17")).toBeNull();
  expect(screen.queryByText(/Exact local Jaws prose/)).toBeNull();
  expect(api.getRecordDetail).not.toHaveBeenCalled();
});

it("keeps changed participants and stale generations explicit without baseline fallback", async () => {
  vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ xl: true });
  const jaws = activityFixture();
  const { update } = mount(jaws);
  fireEvent.click(screen.getByRole("button", { name: "Jaws quick lookup" }));
  await waitFor(() => expect(screen.getByText(/Exact local Jaws prose/)).toBeVisible());
  update(jaws, "participant-b");
  expect(screen.getByText(/selected participant has changed/)).toBeVisible();
  update(jaws, "participant-a", "generation-b");
  expect(screen.getByText(/reference collection has changed/)).toBeVisible();
  expect(api.getOwnedReading).toHaveBeenCalledTimes(1);
  expect(api.getRecordDetail).not.toHaveBeenCalled();
});

it.each(["desktop", "narrow"])(
  "%s contextual preview preserves nested native Escape and original trigger focus",
  async (viewport) => {
    vi.spyOn(Grid, "useBreakpoint").mockReturnValue({
      xl: viewport === "desktop",
      xs: viewport === "narrow",
    });
    mount();
    const trigger = screen.getByRole("button", { name: "Jaws quick lookup" });
    act(() => trigger.focus());
    fireEvent.click(trigger);
    const full = await screen.findByRole("link", { name: "Open reference full page" });
    await waitFor(() => expect(full).toHaveFocus());
    const marker = await screen.findByRole("button", { name: "Reflex check" });
    act(() => marker.focus());
    fireEvent.keyDown(marker, { key: "Enter" });
    const modal = await screen.findByRole("dialog", { name: "Reflex" });
    const ok = within(modal).getByRole("button", { name: "OK" });
    act(() => ok.focus());
    fireEvent.keyDown(ok, { key: "Escape", keyCode: 27 });
    await waitFor(() => expect(modal).not.toBeInTheDocument());
    expect(marker).toHaveFocus();
    const close = screen.getByRole("button", { name: "Close reference preview" });
    act(() => close.focus());
    fireEvent.keyDown(close, { key: "Escape", keyCode: 27 });
    await waitFor(() =>
      expect(
        screen.queryByRole("link", { name: "Open reference full page" }),
      ).toBeNull(),
    );
    expect(trigger).toHaveFocus();
    expect(api.getRecordDetail).not.toHaveBeenCalled();
  },
);
