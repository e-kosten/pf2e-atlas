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
import { AtlasApiError } from "../../api/atlasApi";

const api = vi.hoisted(() => ({
  getEncounterReference: vi.fn(),
  getRecordDetail: vi.fn(),
  getRelatedRecords: vi.fn(),
  searchPage: vi.fn(),
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
  reloadEncounter,
}: {
  activity?: ActorActivityView;
  identity?: string;
  generation?: string | null;
  reloadEncounter?: () => Promise<unknown>;
}) {
  const jaws = activity ?? activityFixture();
  const preview = useRecordPreview({
    encounterRef: "test-encounter",
    identity,
    generation,
    reloadEncounter,
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
          onReload={preview.reload}
          reloadLabel={preview.reloadLabel}
          retryLabel={preview.retryLabel}
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
function mount(activity = activityFixture(), reloadEncounter?: () => Promise<unknown>) {
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
            <Flow
              activity={value}
              identity={identity}
              generation={generation}
              reloadEncounter={reloadEncounter}
            />
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
  api.getRelatedRecords.mockReset();
  api.searchPage.mockReset();
  api.getRelatedRecords.mockImplementation((request) =>
    Promise.resolve({
      selection: request.selection,
      artifact_generation: request.artifact_generation,
      seed: detailFixture(request.selection.record_key, "Current actor").record,
      references: { kind: "empty" },
      counterparts: { kind: "empty" },
      variants: { kind: "empty" },
    }),
  );
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

it.each(["Related", "Similar"])(
  "%s conflict in a successful contextual preview reloads the encounter before a fresh selection can be opened",
  async (category) => {
    vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ xl: true });
    const conflict = new AtlasApiError(409, "The reference collection changed", {
      code: "artifact_generation_changed",
      message: "The reference collection changed",
      recoverable_action: "reload_record",
      details: {
        expected_artifact_generation: "generation-a",
        actual_artifact_generation: "generation-b",
      },
    });
    if (category === "Related") api.getRelatedRecords.mockRejectedValue(conflict);
    else api.searchPage.mockRejectedValue(conflict);
    let finishReload!: () => void;
    const reloadEncounter = vi.fn(
      () =>
        new Promise<void>((resolve) => {
          finishReload = resolve;
        }),
    );
    const activityA = activityFixture();
    const { update } = mount(activityA, reloadEncounter);
    fireEvent.click(screen.getByRole("button", { name: "Jaws quick lookup" }));
    await waitFor(() =>
      expect(screen.getByText(/Exact local Jaws prose/)).toBeVisible(),
    );
    fireEvent.click(screen.getByRole("button", { name: "Related records" }));
    if (category === "Similar") {
      fireEvent.click(
        await screen.findByRole("button", {
          name: "Similar to parent record: Current actor",
        }),
      );
      await waitFor(() =>
        expect(api.searchPage).toHaveBeenCalledWith(
          expect.objectContaining({ expected_artifact_generation: "generation-a" }),
          expect.any(AbortSignal),
        ),
      );
    }
    const reloadButton = await screen.findByRole("button", {
      name: "Reload encounter",
    });
    const similarRequestCount = api.searchPage.mock.calls.length;
    const relatedRequestCount = api.getRelatedRecords.mock.calls.length;
    fireEvent.click(reloadButton);
    expect(reloadEncounter).toHaveBeenCalledTimes(1);
    expect(screen.queryByText(/Exact local Jaws prose/)).toBeNull();

    const activityB = {
      ...activityA,
      navigation: {
        ...activityA.navigation,
        owners: [{ collection: "/items", identity: { SnapshotLocal: { index: 7 } } }],
        source_fingerprint: "snapshot-b",
      },
    };
    update(activityB, "participant-a", "generation-b");
    await act(async () => {
      finishReload();
    });
    await waitFor(() =>
      expect(
        screen.getByText(/Close this preview and reopen the activity/),
      ).toBeVisible(),
    );
    expect(screen.queryByRole("button", { name: "Retry" })).toBeNull();
    expect(api.getEncounterReference).toHaveBeenCalledTimes(1);
    expect(api.searchPage).toHaveBeenCalledTimes(similarRequestCount);
    expect(api.getRelatedRecords).toHaveBeenCalledTimes(relatedRequestCount);
    fireEvent.click(screen.getByRole("button", { name: "Close reference preview" }));
    fireEvent.click(screen.getByRole("button", { name: "Jaws quick lookup" }));
    await waitFor(() =>
      expect(screen.getByText(/Exact local Jaws prose/)).toBeVisible(),
    );
    expect(api.getEncounterReference).toHaveBeenCalledTimes(2);
    expect(api.getEncounterReference.mock.calls[0][2]).toEqual(
      expect.objectContaining({
        artifact_generation: "generation-a",
        selection: expect.objectContaining({
          owners: activityA.navigation.owners,
          source_fingerprint: activityA.navigation.source_fingerprint,
        }),
      }),
    );
    expect(api.getEncounterReference.mock.calls[1][2]).toEqual(
      expect.objectContaining({
        artifact_generation: "generation-b",
        selection: expect.objectContaining({
          owners: activityB.navigation.owners,
          source_fingerprint: "snapshot-b",
        }),
      }),
    );
    expect(api.getRecordDetail).not.toHaveBeenCalled();
  },
);

it("retries a failed encounter refresh through its owner and reopens an initial conflict using a fresh exact selection", async () => {
  vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ xl: true });
  const conflict = new AtlasApiError(409, "The reference collection changed", {
    code: "artifact_generation_changed",
    message: "The reference collection changed",
    recoverable_action: "reload_record",
  });
  api.getEncounterReference.mockImplementation((_encounter, _participant, request) =>
    request.artifact_generation === "generation-a"
      ? Promise.reject(conflict)
      : Promise.resolve(response(request)),
  );
  let finishReload!: () => void;
  const reloadEncounter = vi
    .fn()
    .mockRejectedValueOnce(new Error("Encounter refresh unavailable"))
    .mockImplementationOnce(
      () =>
        new Promise<void>((resolve) => {
          finishReload = resolve;
        }),
    );
  const activityA = activityFixture();
  const { update } = mount(activityA, reloadEncounter);
  fireEvent.click(screen.getByRole("button", { name: "Jaws quick lookup" }));
  fireEvent.click(await screen.findByRole("button", { name: "Reload encounter" }));
  await waitFor(() =>
    expect(screen.getByText("Encounter refresh unavailable")).toBeVisible(),
  );
  fireEvent.click(screen.getByRole("button", { name: "Reload encounter" }));
  const activityB = {
    ...activityA,
    navigation: {
      ...activityA.navigation,
      owners: [{ collection: "/items", identity: { SnapshotLocal: { index: 11 } } }],
      source_fingerprint: "initial-conflict-snapshot-b",
    },
  };
  update(activityB, "participant-a", "generation-b");
  await act(async () => {
    finishReload();
  });
  await waitFor(() =>
    expect(
      screen.getByText(/Close this preview and reopen the activity/),
    ).toBeVisible(),
  );
  expect(reloadEncounter).toHaveBeenCalledTimes(2);
  expect(api.getEncounterReference).toHaveBeenCalledTimes(1);
  expect(screen.queryByRole("button", { name: "Reload encounter" })).toBeNull();
  expect(screen.queryByRole("button", { name: "Retry" })).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Close reference preview" }));
  fireEvent.click(screen.getByRole("button", { name: "Jaws quick lookup" }));
  await waitFor(() => expect(screen.getByText(/Exact local Jaws prose/)).toBeVisible());
  expect(api.getEncounterReference).toHaveBeenCalledTimes(2);
  expect(api.getEncounterReference.mock.calls[1][2]).toEqual(
    expect.objectContaining({
      artifact_generation: "generation-b",
      selection: expect.objectContaining({
        owners: activityB.navigation.owners,
        source_fingerprint: "initial-conflict-snapshot-b",
      }),
    }),
  );
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
