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
import { AddToRecordButton } from "./AddToRecordButton";

const api = vi.hoisted(() => ({
  getSavedLists: vi.fn(),
  getEncounters: vi.fn(),
  createSavedList: vi.fn(),
  createEncounter: vi.fn(),
  addSavedListItem: vi.fn(),
  addEncounterRecordParticipant: vi.fn(),
}));
vi.mock("../../api/atlasApi", () => api);
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
function setup(record = creature) {
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <ConfigProvider theme={{ token: { motion: false } }}>
        <App>
          <AddToRecordButton recordKey={record.record_key} record={record} />
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
  fireEvent.change(name, { target: { value: "Ambush" } });
  fireEvent.click(screen.getByRole("button", { name: "Create & add" }));
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(api.createSavedList).toHaveBeenCalledWith({
    name: "Ambush",
    slug: "ambush",
    tags: [],
  });
  expect(api.addSavedListItem).toHaveBeenCalledWith({
    list_ref: list.list_key,
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
it("retains a created destination after an add failure and retries only the add", async () => {
  api.createEncounter.mockResolvedValue({ encounter });
  api.addEncounterRecordParticipant
    .mockRejectedValueOnce(new Error("Disconnected"))
    .mockResolvedValueOnce({ encounter_key: encounter.encounter_key });
  setup();
  fireEvent.click(screen.getByRole("tab", { name: "Encounters" }));
  fireEvent.click(screen.getByRole("button", { name: "New encounter" }));
  fireEvent.change(screen.getByRole("textbox", { name: "New encounter name" }), {
    target: { value: "Ambush" },
  });
  fireEvent.click(screen.getByRole("button", { name: "Create & add" }));
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Ambush was created, but the record could not be added.",
  );
  expect(
    screen.getByRole("combobox", { name: "Encounter" }).parentElement,
  ).toHaveTextContent("Ambush");
  fireEvent.click(screen.getByRole("button", { name: /^Add$/ }));
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(api.createEncounter).toHaveBeenCalledOnce();
  expect(api.addEncounterRecordParticipant).toHaveBeenCalledTimes(2);
  expect(api.addEncounterRecordParticipant).toHaveBeenLastCalledWith({
    encounter_ref: encounter.encounter_key,
    record_ref: creature.record_key,
    quantity: 1,
  });
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
