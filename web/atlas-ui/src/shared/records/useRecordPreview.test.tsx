import { atlasRouter } from "../../app/router";
import {
  act,
  fireEvent,
  render,
  renderHook,
  screen,
  waitFor,
} from "@testing-library/react";
import { ConfigProvider, Grid } from "antd";
import { useRecordPreview } from "./useRecordPreview";
import { EffectReferenceLink } from "./EffectReferenceLink";
import { detailFixture } from "../../test/fixtures";
import { RecordPreviewDrawer } from "./RecordPreviewDrawer";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
function queryWrapper({ children }: { children: import("react").ReactNode }) {
  return (
    <QueryClientProvider client={new QueryClient()}>{children}</QueryClientProvider>
  );
}

vi.mock("./useRecordDetail", () => ({
  useRecordDetail: () => ({
    data: undefined,
    error: null,
    isLoading: false,
    isFetching: false,
  }),
}));

function PreviewFlow() {
  const preview = useRecordPreview();
  const selection = detailFixture("actors:ghoul").selected;
  return (
    <>
      <button
        onClick={(event) =>
          preview.open("actors:ghoul", event.currentTarget, selection)
        }
      >
        Original reference
      </button>
      {preview.destination && (
        <>
          <button
            onClick={(event) =>
              preview.open("rules:nested", event.currentTarget, {
                ...selection,
                record_key: "rules:nested",
              })
            }
          >
            Follow reference
          </button>
          <button onClick={preview.close}>Close preview</button>
        </>
      )}
    </>
  );
}

it("suppresses effect tooltips while reading and restores them after closing", async () => {
  function EffectFlow() {
    const preview = useRecordPreview();
    return (
      <>
        <EffectReferenceLink
          label={{ text: "Effect", title: "Spell Effect: Success" }}
          target={detailFixture("effects:success").selected}
          onReference={preview.open}
        />
        {preview.destination && <button onClick={preview.close}>Close preview</button>}
      </>
    );
  }
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <EffectFlow />
    </ConfigProvider>,
    { wrapper: queryWrapper },
  );
  const link = screen.getByRole("link", { name: "Preview Spell Effect: Success" });
  fireEvent.mouseEnter(link);
  await screen.findByRole("tooltip");
  fireEvent.click(link);
  expect(link).toHaveAttribute("aria-expanded", "true");
  vi.useFakeTimers();
  try {
    fireEvent.mouseLeave(link);
    fireEvent.mouseEnter(link);
    await act(async () => vi.advanceTimersByTimeAsync(400));
    expect(screen.queryByRole("tooltip")).not.toBeInTheDocument();
  } finally {
    vi.useRealTimers();
  }
  fireEvent.click(screen.getByRole("button", { name: "Close preview" }));
  fireEvent.mouseLeave(link);
  fireEvent.mouseEnter(link);
  await screen.findByRole("tooltip");
});

it("returns focus to the original live trigger after following nested references", () => {
  render(<PreviewFlow />, { wrapper: queryWrapper });
  const original = screen.getByRole("button", { name: "Original reference" });
  act(() => original.focus());
  fireEvent.click(original);
  expect(original).toHaveAttribute("aria-expanded", "true");
  const follow = screen.getByRole("button", { name: "Follow reference" });
  act(() => follow.focus());
  fireEvent.click(follow);
  expect(original).toHaveAttribute("aria-expanded", "true");
  const close = screen.getByRole("button", { name: "Close preview" });
  act(() => close.focus());
  fireEvent.click(close);
  expect(original).toHaveFocus();
  expect(original).not.toHaveAttribute("aria-expanded");
  expect(
    screen.queryByRole("button", { name: "Close preview" }),
  ).not.toBeInTheDocument();
});

it("returns focus to the original trigger after dismissing a narrow reference drawer", async () => {
  const breakpoint = vi
    .spyOn(Grid, "useBreakpoint")
    .mockReturnValue({ xs: true, md: false });
  function NarrowPreviewFlow() {
    const preview = useRecordPreview();
    return (
      <>
        <button onClick={(event) => preview.open("actors:ghoul", event.currentTarget)}>
          Ghoul reference
        </button>
        <RecordPreviewDrawer preview={preview} />
      </>
    );
  }
  try {
    render(
      <ConfigProvider theme={{ token: { motion: false } }}>
        <NarrowPreviewFlow />
      </ConfigProvider>,
      { wrapper: queryWrapper },
    );
    const original = screen.getByRole("button", { name: "Ghoul reference" });
    act(() => original.focus());
    fireEvent.click(original);
    await waitFor(() =>
      expect(screen.getByRole("heading", { name: "Reference" })).toHaveFocus(),
    );
    const heading = screen.getByRole("heading", { name: "Reference" });
    act(() => heading.focus());
    fireEvent.keyDown(heading, { key: "Escape", keyCode: 27 });
    await waitFor(() =>
      expect(
        screen.queryByRole("button", { name: "Close reference preview" }),
      ).not.toBeInTheDocument(),
    );
    expect(original).toHaveFocus();
  } finally {
    breakpoint.mockRestore();
  }
});

it("bounds preview history and restores exact selected copy, landing, source link and scroll before final opener dismissal", () => {
  const { result } = renderHook(() => useRecordPreview(), { wrapper: queryWrapper });
  const opener = document.createElement("button");
  document.body.append(opener);
  const first = {
    ...detailFixture("actors:ghoul").selected,
    embedded_path: [{ collection: "items", identity: { SnapshotLocal: { index: 7 } } }],
    field: "system.description.value",
    source_fingerprint: "copy-a",
  };
  const host = document.createElement("section");
  host.className = "record-preview-drawer__body";
  const drawer = document.createElement("div");
  drawer.className = "record-preview-drawer";
  const source = document.createElement("a");
  source.href = "/records/rules:next";
  host.append(source);
  drawer.append(host);
  document.body.append(drawer);
  act(() => result.current.open(first.record_key, opener, first, first));
  host.scrollTop = 314;
  act(() => result.current.open("rules:next", source));
  act(() => result.current.readingHistory.onBack());
  expect(result.current.selection).toEqual(first);
  expect(result.current.landing).toEqual(first);
  expect(result.current.readingHistory.restore).toEqual({
    scroll: 314,
    focusHref: "/records/rules:next",
    focusRegion: "body",
  });
  for (let index = 0; index < 30; index++)
    act(() => result.current.open(`rules:${index}`, source));
  for (let index = 0; index < 24; index++)
    act(() => result.current.readingHistory.onBack());
  expect(result.current.destination).toMatchObject({
    kind: "record",
    navigation: { record_key: "rules:5" },
  });
  expect(result.current.readingHistory.canGoBack).toBe(false);
  act(() => result.current.close());
  expect(opener).toHaveFocus();
  expect(result.current.readingHistory.canGoBack).toBe(false);
  drawer.remove();
  opener.remove();
});

it("retains an exact contextual document during canonical refresh, replaces fresh context, and isolates participant/generation changes", async () => {
  const api = await import("../../api/atlasApi");
  const initial = detailFixture("actors:mage", "Fear");
  const fresh = detailFixture("actors:mage", "Updated Fear");
  let finish: (detail: typeof initial) => void = () => {};
  const read = vi
    .spyOn(api, "getEncounterReference")
    .mockResolvedValueOnce(initial)
    .mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          finish = resolve;
        }),
    );
  const context = {
    encounterRef: "encounter:one",
    identity: "participant:one",
    generation: "generation:one",
    parentRecordKey: "actors:mage",
    presentation: initial.presentation,
  };
  const { result, rerender } = renderHook((props) => useRecordPreview(props), {
    wrapper: queryWrapper,
    initialProps: context,
  });
  act(() => result.current.open("actors:mage", undefined, initial.selected));
  await waitFor(() => expect(result.current.detail).toBe(initial));
  rerender({ ...context, presentation: fresh.presentation });
  await waitFor(() => expect(read).toHaveBeenCalledTimes(2));
  expect(result.current.loading).toBe(true);
  expect(result.current.detail).toBe(initial);
  await act(async () => finish(fresh));
  await waitFor(() => expect(result.current.detail).toBe(fresh));
  rerender({ ...context, identity: "participant:two" });
  expect(result.current.detail).toBeUndefined();
  expect(result.current.error?.message).toMatch(/participant has changed/);
  rerender({ ...context, generation: "generation:two" });
  expect(result.current.detail).toBeUndefined();
  expect(result.current.error?.message).toMatch(/reference collection has changed/);
  read.mockRestore();
});

it("closes a retained linked preview on an explicit reading departure to the current href", () => {
  const { result } = renderHook(() => useRecordPreview(), { wrapper: queryWrapper });
  act(() => result.current.open("actors:ghoul"));
  expect(result.current.destination).toMatchObject({
    kind: "record",
    navigation: { record_key: "actors:ghoul" },
  });
  act(() =>
    atlasRouter.emit({
      type: "onBeforeNavigate",
      fromLocation: atlasRouter.latestLocation,
      toLocation: {
        ...atlasRouter.latestLocation,
        state: {
          ...atlasRouter.latestLocation.state,
          atlasReadingDismissal: "new-search",
        },
      },
      pathChanged: false,
      hrefChanged: false,
      hashChanged: false,
    }),
  );
  expect(result.current.destination).toBeNull();
  expect(result.current.selection).toBeUndefined();
});

it("retains the original owner callback through nested record/help reading and returns only once", () => {
  const { result } = renderHook(() => useRecordPreview(), { wrapper: queryWrapper });
  const firstOwner = vi.fn();
  const nestedOwner = vi.fn();
  const anchor = document.createElement("a");
  const navigation = detailFixture("rules:first").selected;
  const help = {
    definition_key: null,

    destination: { kind: "record" as const, navigation },
  };
  act(() => result.current.openHelp(help, anchor, undefined, firstOwner));
  act(() =>
    result.current.openHelp(
      {
        ...help,
        destination: {
          kind: "record",
          navigation: { ...navigation, record_key: "rules:second" },
        },
      },
      anchor,
      undefined,
      nestedOwner,
    ),
  );
  act(() => result.current.open("rules:third", anchor));
  act(() => result.current.close());
  act(() => result.current.close());
  expect(firstOwner).toHaveBeenCalledExactlyOnceWith("return");
  expect(nestedOwner).not.toHaveBeenCalled();
});

it("abandons a suspended owner on explicit navigation without resuming or focusing its opener", () => {
  const { result } = renderHook(() => useRecordPreview(), { wrapper: queryWrapper });
  const finish = vi.fn();
  const anchor = document.createElement("a");
  const focus = vi.spyOn(anchor, "focus");
  const navigation = detailFixture("rules:first").selected;
  act(() =>
    result.current.openHelp(
      {
        definition_key: null,

        destination: { kind: "record", navigation },
      },
      anchor,
      undefined,
      finish,
    ),
  );
  act(() =>
    atlasRouter.emit({
      type: "onBeforeNavigate",
      fromLocation: atlasRouter.latestLocation,
      toLocation: {
        ...atlasRouter.latestLocation,
        state: {
          ...atlasRouter.latestLocation.state,
          atlasReadingDismissal: "leave-owner",
        },
      },
      pathChanged: false,
      hrefChanged: false,
      hashChanged: false,
    }),
  );
  expect(finish).toHaveBeenCalledExactlyOnceWith("navigate");
  expect(focus).not.toHaveBeenCalled();
  expect(result.current.destination).toBeNull();
});

it("keeps mixed trait and record history with exact owned addresses and original opener", async () => {
  const api = await import("../../api/atlasApi");
  const help = {
    definition_key: "trait:agile",

    destination: { kind: "definition" as const, key: "trait:agile" },
  };
  const read = vi.spyOn(api, "getDefinitionDetail").mockResolvedValue({
    definition_key: "trait:agile",
    label: "Agile",
    artifact_generation: "generation-a",
    help,
    content: { body: { kind: "plain", text: "Agile rules." }, references: [] },

    notices: [],

    related_filters: [],
    publications: [],
  });
  const { result } = renderHook(() => useRecordPreview(), { wrapper: queryWrapper });
  const opener = document.createElement("button");
  document.body.append(opener);
  const selection = {
    ...detailFixture("actors:ghoul").selected,
    spell_rank: 5,
    embedded_path: [{ collection: "items", identity: { SnapshotLocal: { index: 2 } } }],
    source_fingerprint: "verified-source",
  };
  try {
    act(() => result.current.open("actors:ghoul", opener, selection, selection));
    act(() => result.current.openHelp(help, opener, "generation-a"));
    expect(result.current.destination).toEqual(help.destination);
    act(() => result.current.open("rules:condition", opener));
    act(() => result.current.readingHistory.onBack());
    expect(result.current.destination).toEqual(help.destination);
    act(() => result.current.readingHistory.onBack());
    expect(result.current.destination).toMatchObject({
      kind: "record",
      navigation: { record_key: "actors:ghoul" },
    });
    expect(result.current.selection).toEqual(selection);
    expect(result.current.landing).toEqual(selection);
    expect(result.current.readingHistory.canGoBack).toBe(false);
    act(() => result.current.close());
    expect(result.current.destination).toBeNull();
    expect(opener).toHaveFocus();
  } finally {
    opener.remove();
    read.mockRestore();
  }
});
