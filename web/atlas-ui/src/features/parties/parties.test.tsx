import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { App, ConfigProvider } from "antd";
import type { ReactNode } from "react";
import { EncounterAddMenu } from "../encounters/EncounterAddMenu";
import { PartyManager } from "./PartyManager";
const api = vi.hoisted(() => ({
  getParties: vi.fn(),
  createParty: vi.fn(),
  updateParty: vi.fn(),
  deleteParty: vi.fn(),
}));
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  ...api,
}));
const party = {
  party_key: "party",
  name: "Campaign",
  revision: 4,
  members: [
    { member_key: "pc-a", name: "Aster" },
    { member_key: "pc-b", name: "Bram" },
  ],
};
function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return (
    <QueryClientProvider client={client}>
      <ConfigProvider theme={{ token: { motion: false } }}>
        <App>{children}</App>
      </ConfigProvider>
    </QueryClientProvider>
  );
}
beforeEach(() => {
  for (const mock of Object.values(api)) mock.mockReset();
  api.getParties.mockResolvedValue({ parties: [party] });
});
afterEach(cleanup);
it("adds the whole chosen party in one click without a modal or subset fields", async () => {
  const onAdd = vi.fn().mockResolvedValue(undefined);
  render(
    <EncounterAddMenu
      onAddCreature={vi.fn()}
      onAddPc={vi.fn()}
      disabled={false}
      onAdd={onAdd}
    />,
    { wrapper },
  );
  expect(api.getParties).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Add participants" }));
  const item = await screen.findByRole("menuitem", { name: /Campaign.*Aster, Bram/ });
  await waitFor(() => expect(item).not.toHaveAttribute("aria-disabled", "true"));
  expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  expect(screen.queryByRole("checkbox")).not.toBeInTheDocument();
  fireEvent.click(item);
  await waitFor(() =>
    expect(onAdd).toHaveBeenCalledExactlyOnceWith({
      party_key: "party",
      expected_revision: 4,
    }),
  );
});
it("prevents duplicate additions while the write and refresh remain pending", async () => {
  let finish!: () => void;
  const onAdd = vi.fn(
    () =>
      new Promise<void>((resolve) => {
        finish = resolve;
      }),
  );
  render(
    <EncounterAddMenu
      onAddCreature={vi.fn()}
      onAddPc={vi.fn()}
      disabled={false}
      onAdd={onAdd}
    />,
    { wrapper },
  );
  fireEvent.click(screen.getByRole("button", { name: "Add participants" }));
  const item = await screen.findByRole("menuitem", { name: /Campaign.*Aster, Bram/ });
  await waitFor(() => expect(item).not.toHaveAttribute("aria-disabled", "true"));
  fireEvent.click(item);
  fireEvent.click(item);
  expect(onAdd).toHaveBeenCalledTimes(1);
  expect(screen.getByRole("button", { name: "Add participants" })).toBeDisabled();
  finish();
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Add participants" })).toBeEnabled(),
  );
});
it("offers a local read retry when parties cannot load", async () => {
  api.getParties.mockRejectedValueOnce(new Error("Offline"));
  render(
    <EncounterAddMenu
      onAddCreature={vi.fn()}
      onAddPc={vi.fn()}
      disabled={false}
      onAdd={vi.fn()}
    />,
    { wrapper },
  );
  fireEvent.click(screen.getByRole("button", { name: "Add participants" }));
  fireEvent.click(
    await screen.findByRole("menuitem", { name: "Parties could not load. Retry" }),
  );
  await waitFor(() => expect(api.getParties).toHaveBeenCalledTimes(2));
});
it("opens party creation directly from an empty menu", async () => {
  api.getParties.mockResolvedValue({ parties: [] });
  render(
    <EncounterAddMenu
      onAddCreature={vi.fn()}
      onAddPc={vi.fn()}
      disabled={false}
      onAdd={vi.fn()}
    />,
    { wrapper },
  );
  fireEvent.click(screen.getByRole("button", { name: "Add participants" }));
  fireEvent.click(await screen.findByRole("menuitem", { name: "Create Party…" }));
  expect(await screen.findByRole("dialog", { name: "New Party" })).toBeInTheDocument();
});
it("keeps a failed party creation draft and permits an explicit retry", async () => {
  api.createParty
    .mockRejectedValueOnce(new Error("Could not save"))
    .mockResolvedValue(party);
  const onClose = vi.fn();
  render(<PartyManager initialCreate onClose={onClose} />, { wrapper });
  const dialog = await screen.findByRole("dialog", { name: "New Party" });
  fireEvent.change(within(dialog).getByLabelText("Party name"), {
    target: { value: "Campaign" },
  });
  fireEvent.click(within(dialog).getByRole("button", { name: "Add PC" }));
  fireEvent.change(within(dialog).getByLabelText("PC 1 name"), {
    target: { value: "Aster" },
  });
  fireEvent.change(within(dialog).getByRole("spinbutton", { name: "Party level" }), {
    target: { value: "5" },
  });
  fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));
  expect(await screen.findByText("Party could not be saved")).toBeInTheDocument();
  expect(within(dialog).getByLabelText("PC 1 name")).toHaveValue("Aster");
  expect(onClose).not.toHaveBeenCalled();
  fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));
  await waitFor(() => expect(onClose).toHaveBeenCalledOnce());
  expect(api.createParty).toHaveBeenLastCalledWith({
    name: "Campaign",
    members: [{ name: "Aster", level: 5 }],
  });
});

it("uses one shared level for every PC, including PCs added after entering it", async () => {
  api.createParty.mockResolvedValue(party);
  render(<PartyManager initialCreate onClose={vi.fn()} />, { wrapper });
  const dialog = await screen.findByRole("dialog", { name: "New Party" });
  expect(within(dialog).getByRole("radio", { name: "Same level" })).toBeChecked();
  fireEvent.change(within(dialog).getByLabelText("Party name"), {
    target: { value: "Shared" },
  });
  fireEvent.change(within(dialog).getByRole("spinbutton", { name: "Party level" }), {
    target: { value: "7" },
  });
  for (const [index, name] of ["Aster", "Bram"].entries()) {
    fireEvent.click(within(dialog).getByRole("button", { name: "Add PC" }));
    fireEvent.change(within(dialog).getByLabelText(`PC ${index + 1} name`), {
      target: { value: name },
    });
  }
  expect(within(dialog).queryByRole("spinbutton", { name: "PC 1 level" })).toBeNull();
  fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));
  await waitFor(() =>
    expect(api.createParty).toHaveBeenCalledWith({
      name: "Shared",
      members: [
        { name: "Aster", level: 7 },
        { name: "Bram", level: 7 },
      ],
    }),
  );
});

it("retains mixed individual drafts while switching level entry modes", async () => {
  api.getParties.mockResolvedValue({
    parties: [
      {
        ...party,
        members: party.members.map((member, index) => ({
          ...member,
          level: index === 0 ? 4 : 6,
        })),
      },
    ],
  });
  api.updateParty.mockResolvedValue(party);
  render(<PartyManager onClose={vi.fn()} />, { wrapper });
  fireEvent.click(await screen.findByRole("button", { name: "Campaign" }));
  const dialog = await screen.findByRole("dialog", { name: "Edit Party" });
  expect(
    within(dialog).getByRole("radio", { name: "Individual levels" }),
  ).toBeChecked();
  fireEvent.click(within(dialog).getByRole("radio", { name: "Same level" }));
  fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));
  await within(dialog).findByText("Use 1–20.");
  expect(api.updateParty).not.toHaveBeenCalled();
  fireEvent.change(within(dialog).getByRole("spinbutton", { name: "Party level" }), {
    target: { value: "9" },
  });
  fireEvent.click(within(dialog).getByRole("radio", { name: "Individual levels" }));
  expect(within(dialog).getByRole("spinbutton", { name: "PC 1 level" })).toHaveValue(
    "4",
  );
  expect(within(dialog).getByRole("spinbutton", { name: "PC 2 level" })).toHaveValue(
    "6",
  );
  fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));
  await waitFor(() =>
    expect(api.updateParty).toHaveBeenCalledWith("party", {
      name: "Campaign",
      expected_revision: 4,
      members: [
        { member_key: "pc-a", name: "Aster", level: 4 },
        { member_key: "pc-b", name: "Bram", level: 6 },
      ],
    }),
  );
});

it("seeds individual entry from the shared level then preserves subsequent individual edits", async () => {
  render(<PartyManager initialCreate onClose={vi.fn()} />, { wrapper });
  const dialog = await screen.findByRole("dialog", { name: "New Party" });
  fireEvent.change(within(dialog).getByRole("spinbutton", { name: "Party level" }), {
    target: { value: "5" },
  });
  fireEvent.click(within(dialog).getByRole("button", { name: "Add PC" }));
  fireEvent.click(within(dialog).getByRole("radio", { name: "Individual levels" }));
  expect(within(dialog).getByRole("spinbutton", { name: "PC 1 level" })).toHaveValue(
    "5",
  );
  fireEvent.change(within(dialog).getByRole("spinbutton", { name: "PC 1 level" }), {
    target: { value: "6" },
  });
  fireEvent.click(within(dialog).getByRole("radio", { name: "Same level" }));
  expect(within(dialog).getByRole("spinbutton", { name: "Party level" })).toHaveValue(
    "6",
  );
  fireEvent.click(within(dialog).getByRole("radio", { name: "Individual levels" }));
  expect(within(dialog).getByRole("spinbutton", { name: "PC 1 level" })).toHaveValue(
    "6",
  );
});
