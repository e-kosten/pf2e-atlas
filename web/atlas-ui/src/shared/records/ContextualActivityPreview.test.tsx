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
  RecordDetailView,
  EncounterReferenceRequest,
} from "../../generated/atlas";
import {
  activityFixture,
  actorFixture,
  detailFixture,
  knownFact,
  numberFact,
  unavailableFact,
} from "../../test/fixtures";
import { useRecordPreview } from "./useRecordPreview";
import { RecordPreviewPopover } from "./RecordPreviewPopover";
import { atlasRoutePath } from "../../app/routes";

const api = vi.hoisted(() => ({
  getEncounterReference: vi.fn(),
  getRecordDetail: vi.fn(),
}));
let currentActivity = activityFixture();
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  ...api,
}));
function response(request: EncounterReferenceRequest): RecordDetailView {
  const detail = detailFixture(request.selection.record_key, "Current actor");
  return {
    ...detail,
    artifact_generation: request.artifact_generation,
    selected: currentActivity.navigation,
    presentation: {
      ...detail.presentation,
      identity: { ...detail.record, title: currentActivity.title },
      body: { kind: "activity", value: currentActivity },
      content:
        currentActivity.kind === "casting_entry"
          ? []
          : [
              {
                locator: {
                  record: request.selection.record_key,
                  owners: request.selection.owners,
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
    encounterRef: "test-encounter",
    identity,
    generation,
    parentRecordKey: jaws.navigation.record_key,
    presentation: activity
      ? {
          ...detailFixture(jaws.navigation.record_key).presentation,
          body: {
            kind: "creature",
            value: { ...actorFixture(), activities: knownFact([activity]) },
          },
        }
      : undefined,
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
        />
      )}
    </>
  );
}
function mount(activity = activityFixture()) {
  currentActivity = activity;
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
    ) => {
      if (value) currentActivity = value;
      mounted.rerender(view(value, identity, generation));
    },
  };
}
beforeEach(() => {
  currentActivity = activityFixture();
  api.getEncounterReference.mockReset();
  api.getRecordDetail.mockReset();
  api.getEncounterReference.mockImplementation((_encounter, _participant, request) =>
    Promise.resolve(response(request)),
  );
});
afterEach(() => vi.restoreAllMocks());

it("renders a complete casting entry, its local spell roster, and one title through the shared reader", async () => {
  vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ xl: true });
  const entry = activityFixture("Staff of Fire (Major)");
  entry.kind = "casting_entry";
  entry.family = "spellcastingEntry";
  entry.action_cost = unavailableFact("not_applicable");
  const spell = activityFixture("Local Fireball");
  spell.kind = "spell";
  spell.base_rank = numberFact(3);
  spell.authored_cast_rank = numberFact(5);
  entry.casting_context = {
    parent: { ...entry.navigation, owners: [] },
    parent_title: "Current actor",
    spells: knownFact([spell]),
  };
  mount(entry);
  fireEvent.click(screen.getByRole("button", { name: "Jaws quick lookup" }));
  await waitFor(() =>
    expect(
      screen.getByRole("heading", { name: "Staff of Fire (Major)", level: 2 }),
    ).toBeVisible(),
  );
  expect(
    screen.getAllByRole("heading", { name: "Staff of Fire (Major)" }),
  ).toHaveLength(1);
  expect(screen.getByRole("button", { name: /Local Fireball/ })).toBeVisible();
  expect(screen.queryByRole("heading", { name: "Description" })).toBeNull();
  expect(screen.queryByText("Reference")).toBeNull();
  expect(api.getRecordDetail).not.toHaveBeenCalled();
});

it("loads complete selected views in an isolated participant cache and refreshes changed runtime facts", async () => {
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
  await waitFor(() => expect(screen.getByText("+17")).toBeVisible());
  expect(screen.queryByText("+9")).toBeNull();
  expect(
    screen.getByText("Clumsy 1: Current condition penalty is included."),
  ).toBeVisible();
  expect(api.getEncounterReference).toHaveBeenCalledTimes(2);
  expect(api.getRecordDetail).not.toHaveBeenCalled();
  expect(
    client
      .getQueryCache()
      .findAll({ queryKey: ["record-detail"] })
      .map((query) => JSON.stringify(query.state.data))
      .join(),
  ).not.toContain('"attack"');
  update(undefined);
  expect(
    screen.getByText("This participant's reference is no longer available."),
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
  expect(api.getEncounterReference).toHaveBeenCalledTimes(1);
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
