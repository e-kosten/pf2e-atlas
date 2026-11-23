import {
  DesktopTabContext,
  type DesktopTabRuntime,
  type TabCloseGuard,
} from "../../app/DesktopTabContext";
import {
  markdownSource,
  replaceMarkdown,
  selectMarkdown,
} from "../../test/markdownEditor";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { App, ConfigProvider } from "antd";
import type { RecordSummaryView } from "../../generated/atlas";
import { detailFixture } from "../../test/fixtures";
import { AddToRecordButton } from "./AddToRecordButton";

const api = vi.hoisted(() => ({
  getSavedLists: vi.fn(),
  getEncounters: vi.fn(),
  getEncounter: vi.fn(),
  getRecordDetail: vi.fn(),
  createSavedList: vi.fn(),
  createEncounterWithAddition: vi.fn(),
  previewDraftEncounterAddition: vi.fn(),
  addSavedListItem: vi.fn(),
  addEncounterRecordParticipant: vi.fn(),
}));
vi.mock("../../api/atlasApi", async () => ({
  ...api,
  ...(await import("../../api/apiError")),
}));
const creature: RecordSummaryView = {
  record_key: "creatures:goblin",
  title: "Goblin",
  kind: "creature",
  kind_label: "Creature",
  source_type: null,
  level_label: "1",
  level_basis: null,
  rarity: null,
  traits: [],
  publication: null,
  pack: null,
};
const list = { list_key: "list_ambush", slug: "ambush", name: "Ambush", tags: [] };
const encounter = {
  encounter_key: "enc_ambush",
  slug: "ambush",
  name: "Ambush",
  status: "draft",
};
function setup(record = creature, tab: DesktopTabRuntime | null = null) {
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <ConfigProvider theme={{ token: { motion: false } }}>
        <App>
          <DesktopTabContext.Provider value={tab}>
            <AddToRecordButton recordKey={record.record_key} record={record} />
          </DesktopTabContext.Provider>
        </App>
      </ConfigProvider>
    </QueryClientProvider>,
  );
  const trigger = screen.getByRole("button", {
    name:
      record.kind === "creature" || record.kind === "hazard"
        ? "Add to list or encounter"
        : "Add to list",
  });
  fireEvent.click(trigger);
  return trigger;
}
beforeEach(() => {
  for (const mock of Object.values(api)) mock.mockReset();
  api.getSavedLists.mockResolvedValue({ lists: [] });
  api.getEncounters.mockResolvedValue({ encounters: [] });
  api.getEncounter.mockResolvedValue(null);
  api.previewDraftEncounterAddition.mockResolvedValue({
    artifact_generation: "generation",
    projected: {
      opposition: [{ name: "Goblin" }],
      party_size: null,
      party_level: null,
    },
  });
  api.getRecordDetail.mockResolvedValue({
    ...detailFixture(creature.record_key),
    encounter_adjustment: "normal",
  });
});
it("creates and adds to a list in one submission, preserving the browsing location and returning focus", async () => {
  api.createSavedList.mockResolvedValue({ list });
  api.addSavedListItem.mockResolvedValue({
    list_key: list.list_key,
    slug: list.slug,
    outcome: "added",
  });
  const location = window.location.href;
  const trigger = setup();
  fireEvent.click(screen.getByRole("button", { name: "New list" }));
  const name = screen.getByRole("textbox", { name: "New list name" });
  await waitFor(() => expect(name).toHaveFocus());
  fireEvent.change(name, { target: { value: "森林" } });
  fireEvent.click(screen.getByRole("button", { name: "Create & add" }));
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(api.createSavedList).toHaveBeenCalledWith({
    name: "森林",
    tags: [],
  });
  expect(api.addSavedListItem).toHaveBeenCalledWith(list.list_key, {
    record_ref: creature.record_key,
  });
  expect(window.location.href).toBe(location);
  expect(trigger).toHaveFocus();
  const feedback = await screen.findByRole("alert");
  expect(feedback).toHaveTextContent("Added to Ambush");
  expect(feedback).toHaveStyle({ pointerEvents: "none" });
  const destination = within(feedback).getByRole("link", { name: "Ambush" });
  expect(destination).toHaveStyle({ pointerEvents: "auto" });
  expect(destination).toHaveAttribute("href", "/lists/ambush");
  fireEvent.click(destination, { ctrlKey: true });
  expect(window.location.href).toBe(location);
});
it("retains the reviewed creation request across a lost response and retries it unchanged", async () => {
  api.createEncounterWithAddition
    .mockRejectedValueOnce(new Error("Disconnected"))
    .mockResolvedValueOnce({ encounter_key: encounter.encounter_key, available: true });
  const trigger = setup();
  fireEvent.click(screen.getByRole("tab", { name: "Encounters" }));
  fireEvent.click(screen.getByRole("button", { name: "New encounter" }));
  const name = screen.getByRole("textbox", { name: "New encounter name" });
  fireEvent.change(name, { target: { value: "Ambush" } });
  const submit = screen.getByRole("button", { name: "Create & add" });
  await waitFor(() => expect(submit).toBeEnabled());
  expect(screen.getByText("1 × Goblin")).toBeVisible();
  fireEvent.click(submit);
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Encounter creation could not be confirmed",
  );
  expect(name).toBeDisabled();
  fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  fireEvent.click(trigger);
  fireEvent.click(screen.getByRole("button", { name: "Retry to confirm" }));
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(api.createEncounterWithAddition).toHaveBeenCalledTimes(2);
  expect(api.createEncounterWithAddition.mock.calls[1]).toEqual(
    api.createEncounterWithAddition.mock.calls[0],
  );
  expect(api.createEncounterWithAddition).toHaveBeenCalledWith({
    name: "Ambush",
    expected_artifact_generation: "generation",
    addition: {
      request_id: expect.any(String),
      record_ref: creature.record_key,
      quantity: 1,
    },
  });
  expect(api.addEncounterRecordParticipant).not.toHaveBeenCalled();
  fireEvent.click(trigger);
  expect(screen.queryByRole("button", { name: "Retry to confirm" })).toBeNull();
  expect(screen.getByRole("tab", { name: "Lists" })).toHaveAttribute(
    "aria-selected",
    "true",
  );
});

it("refreshes existing choices after an uncertain create so the same list can be selected", async () => {
  const later = { ...list, list_key: "list_second", name: "Cavern", slug: "cavern" };
  let committed = false;
  api.getSavedLists.mockImplementation(async () => ({
    lists: committed ? [list, later] : [list],
  }));
  api.createSavedList.mockImplementation(async () => {
    committed = true;
    throw new Error("Lost response");
  });
  api.addSavedListItem.mockResolvedValue({ ...later, outcome: "added" });
  setup();
  fireEvent.click(screen.getByRole("button", { name: "New list" }));
  fireEvent.change(screen.getByRole("textbox", { name: "New list name" }), {
    target: { value: later.name },
  });
  fireEvent.click(screen.getByRole("button", { name: "Create & add" }));
  expect(await screen.findByText("List creation could not be confirmed")).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Check list directory" }));
  expect(await screen.findByRole("link", { name: "Cavern (cavern)" })).toHaveAttribute(
    "href",
    "/lists/cavern",
  );
  fireEvent.click(screen.getByRole("button", { name: "Choose existing list" }));
  const picker = screen.getByRole("combobox", { name: "Saved list" });
  fireEvent.mouseDown(picker);
  fireEvent.change(picker, { target: { value: later.name } });
  fireEvent.click(await screen.findByText(later.name, { exact: true }));
  fireEvent.click(screen.getByRole("button", { name: /^Add$/ }));
  await waitFor(() =>
    expect(api.addSavedListItem).toHaveBeenCalledWith(later.list_key, {
      record_ref: creature.record_key,
    }),
  );
  expect(api.createSavedList).toHaveBeenCalledOnce();
});
it("reuses the new list after its record attachment fails", async () => {
  api.createSavedList.mockResolvedValue({ list });
  api.addSavedListItem
    .mockRejectedValueOnce(new Error("Disconnected"))
    .mockResolvedValueOnce({ ...list, outcome: "added" });
  setup();
  fireEvent.click(screen.getByRole("button", { name: "New list" }));
  fireEvent.change(screen.getByRole("textbox", { name: "New list name" }), {
    target: { value: list.name },
  });
  fireEvent.click(screen.getByRole("button", { name: "Create & add" }));
  expect(await screen.findByRole("alert")).toHaveTextContent("Ambush was created");
  fireEvent.click(screen.getByRole("button", { name: /^Add$/ }));
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(api.createSavedList).toHaveBeenCalledOnce();
  expect(api.addSavedListItem.mock.calls[0]).toEqual(
    api.addSavedListItem.mock.calls[1],
  );
});

it("cannot submit a stale hazard quantity while its replacement preview is pending", async () => {
  let finish!: (value: unknown) => void;
  const preview = {
    artifact_generation: "g",
    projected: { opposition: [{ name: "Rune" }], party_level: null, party_size: null },
  };
  api.previewDraftEncounterAddition
    .mockResolvedValueOnce(preview)
    .mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          finish = resolve;
        }),
    );
  setup({ ...creature, kind: "hazard" });
  fireEvent.click(screen.getByRole("tab", { name: "Encounters" }));
  fireEvent.click(screen.getByRole("button", { name: "New encounter" }));
  expect(await screen.findByText("1 × Rune")).toBeVisible();
  fireEvent.change(screen.getByRole("spinbutton", { name: "Quantity" }), {
    target: { value: "2" },
  });
  const submit = screen.getByRole("button", { name: "Create & add" });
  await waitFor(() => expect(submit).toBeDisabled());
  expect(screen.queryByText("1 × Rune")).toBeNull();
  await act(async () =>
    finish({
      ...preview,
      projected: { ...preview.projected, opposition: [{ name: "2 × Rune" }] },
    }),
  );
  expect(await screen.findByText("2 × Rune")).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
  expect(api.createEncounterWithAddition).not.toHaveBeenCalled();
});
it("guards a pending create-and-add against repeat submissions and dismissal", async () => {
  let finish!: (value: unknown) => void;
  api.createSavedList.mockReturnValue(
    new Promise((resolve) => {
      finish = resolve;
    }),
  );
  api.addSavedListItem.mockResolvedValue({
    list_key: list.list_key,
    slug: list.slug,
    outcome: "added",
  });
  setup();
  fireEvent.click(screen.getByRole("button", { name: "New list" }));
  const name = screen.getByRole("textbox", { name: "New list name" });
  fireEvent.change(name, { target: { value: "Ambush" } });
  fireEvent.submit(name.closest("form")!);
  fireEvent.submit(name.closest("form")!);
  await waitFor(() => expect(api.createSavedList).toHaveBeenCalledOnce());
  expect(screen.getByRole("button", { name: "Cancel" })).toBeDisabled();
  expect(screen.getByRole("button", { name: "Close" })).toBeDisabled();
  expect(screen.getByRole("tab", { name: "Encounters" })).toHaveAttribute(
    "aria-disabled",
    "true",
  );
  fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });
  expect(screen.getByRole("dialog")).toBeVisible();
  await act(async () => finish({ list }));
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(api.addSavedListItem).toHaveBeenCalledOnce();
});
it.each(["spell", "equipment", "action"])(
  "offers only lists for a %s, including the action tooltip",
  async (kind) => {
    const trigger = setup({ ...creature, kind, record_key: `${kind}:example` });
    expect(trigger).toHaveAccessibleName("Add to list");
    expect(screen.getByRole("dialog", { name: "Add to list" })).toBeVisible();
    expect(screen.queryByRole("tab", { name: "Encounters" })).toBeNull();
    expect(
      screen.queryByText("Only creatures and hazards can be added to encounters."),
    ).toBeNull();
    expect(screen.getByRole("button", { name: "New list" })).toBeEnabled();
    expect(api.getEncounters).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    act(() => trigger.focus());
    expect(await screen.findByRole("tooltip")).toHaveTextContent(/^Add to list$/);
  },
);
it.each(["creature", "hazard"])("keeps both destinations for a %s", (kind) => {
  setup({ ...creature, kind });
  expect(screen.getByRole("tab", { name: "Lists" })).toBeVisible();
  expect(screen.getByRole("tab", { name: "Encounters" })).toBeVisible();
});

it("blocks adding a creature while its adjustment lookup is unavailable and lets the user retry", async () => {
  api.getRecordDetail.mockRejectedValueOnce(new Error("Disconnected"));
  setup();
  fireEvent.click(screen.getByRole("tab", { name: "Encounters" }));
  fireEvent.click(screen.getByRole("button", { name: "New encounter" }));
  expect(await screen.findByText("Creature adjustment could not load")).toBeVisible();
  expect(screen.getByText("Unavailable")).toBeVisible();
  expect(screen.getByRole("button", { name: "Create & add" })).toBeDisabled();
  fireEvent.click(screen.getByRole("button", { name: /^Retry$/ }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Create & add" })).toBeEnabled(),
  );
  expect(screen.getByText("Normal", { exact: true })).toBeVisible();
});

it("protects an unsaved new list name when its empty note is expanded", async () => {
  const guards = new Set<TabCloseGuard>();
  setup(creature, {
    id: "tab",
    active: true,
    activation: 0,
    panel: () => document.body,
    title: vi.fn(),
    theme: vi.fn(),
    guard: (guard) => {
      guards.add(guard);
      return () => guards.delete(guard);
    },
  });
  fireEvent.click(screen.getByRole("button", { name: "New list" }));
  fireEvent.change(screen.getByRole("textbox", { name: "New list name" }), {
    target: { value: "Expedition" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Add a note" }));
  fireEvent.click(screen.getByRole("button", { name: "Expand editor" }));
  expect([...guards].map((guard) => guard())).toContain("dirty");
  fireEvent.click(screen.getByRole("button", { name: "Collapse editor" }));
  expect([...guards].map((guard) => guard())).toContain("dirty");
});
it("resizes the same note editor and returns to the destination without submitting", async () => {
  setup();
  fireEvent.click(screen.getByRole("button", { name: "New list" }));
  fireEvent.change(screen.getByRole("textbox", { name: "New list name" }), {
    target: { value: "Expedition" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Add a note" }));
  const note = screen.getByRole("textbox", { name: "Note" });
  replaceMarkdown(note, "Keep a map");
  selectMarkdown(note, 7, 10);
  const expand = screen.getByRole("button", { name: "Expand editor" });
  fireEvent.click(expand);
  const document = screen.getByRole("dialog", { name: "List entry note" });
  expect(within(document).getByRole("textbox", { name: "Note" })).toBe(note);
  expect(within(document).queryByRole("button", { name: "Create & add" })).toBeNull();
  fireEvent.submit(note.closest("form")!);
  fireEvent.click(within(document).getByRole("button", { name: "Bold" }));
  expect(markdownSource(note)).toBe("Keep a **map**");
  fireEvent.click(within(document).getByRole("button", { name: "Undo" }));
  expect(markdownSource(note)).toBe("Keep a map");
  fireEvent.click(within(document).getByRole("radio", { name: "Preview" }));
  fireEvent.keyDown(document, { key: "Escape" });
  expect(expand).toHaveFocus();
  expect(screen.getByRole("region", { name: "Markdown preview" })).toHaveTextContent(
    "Keep a map",
  );
  expect(screen.getByRole("textbox", { name: "New list name" })).toHaveValue(
    "Expedition",
  );
  fireEvent.click(expand);
  fireEvent.click(screen.getByRole("button", { name: "Collapse editor" }));
  expect(expand).toHaveFocus();
  expect(api.createSavedList).not.toHaveBeenCalled();
  expect(api.addSavedListItem).not.toHaveBeenCalled();
});
type Dismissal = "Escape" | "backdrop" | "Cancel" | "Close";

function dismiss(dialog: HTMLElement, via: Dismissal) {
  if (via === "Escape") {
    fireEvent.keyDown(dialog, { key: "Escape" });
  } else if (via === "backdrop") {
    const backdrop = dialog.closest(".ant-modal-wrap")!;
    fireEvent.mouseDown(backdrop);
    fireEvent.click(backdrop);
  } else {
    fireEvent.click(within(dialog).getByRole("button", { name: via }));
  }
}

const dismissals = ["Escape", "backdrop", "Cancel", "Close"] as const;

it.each(dismissals)("closes an empty add form via %s", async (via) => {
  setup();
  dismiss(screen.getByRole("dialog", { name: "Add to…" }), via);
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(screen.queryByText("Discard this draft?")).not.toBeInTheDocument();
});

it.each(dismissals)("asks before discarding a list note via %s", async (via) => {
  const opener = setup();
  const dialog = screen.getByRole("dialog", { name: "Add to…" });
  const note = "A note with enough detail to keep.\nSecond line.";
  expect(within(dialog).queryByRole("textbox", { name: "Note" })).toBeNull();
  fireEvent.click(within(dialog).getByRole("button", { name: "Add a note" }));
  const field = within(dialog).getByRole("textbox", { name: "Note" });
  expect(field).toHaveFocus();
  replaceMarkdown(field, note);
  fireEvent.click(within(dialog).getByRole("button", { name: "Hide note" }));
  expect(
    within(dialog).getByRole("button", { name: "Edit note (included)" }),
  ).toBeVisible();
  fireEvent.click(within(dialog).getByRole("tab", { name: "Encounters" }));
  fireEvent.click(within(dialog).getByRole("tab", { name: "Lists" }));
  fireEvent.click(within(dialog).getByRole("button", { name: "Edit note (included)" }));
  expect(markdownSource(within(dialog).getByRole("textbox", { name: "Note" }))).toBe(
    note,
  );

  dismiss(dialog, via);
  expect(screen.getAllByRole("dialog")).toEqual([dialog]);
  const keep = within(dialog).getByRole("button", { name: "Keep editing" });
  expect(keep).toHaveFocus();
  if (via === "Escape") dismiss(dialog, "Escape");
  else fireEvent.click(keep);
  expect(markdownSource(within(dialog).getByRole("textbox", { name: "Note" }))).toBe(
    note,
  );

  dismiss(dialog, via);
  fireEvent.click(within(dialog).getByRole("button", { name: "Discard" }));
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  fireEvent.click(opener);
  expect(screen.getByRole("button", { name: "Add a note" })).toBeVisible();
});
