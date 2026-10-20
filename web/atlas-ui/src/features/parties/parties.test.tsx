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
  fireEvent.change(within(dialog).getByLabelText("PC 1"), {
    target: { value: "Aster" },
  });
  fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));
  expect(await screen.findByText("Could not save")).toBeInTheDocument();
  expect(within(dialog).getByLabelText("PC 1")).toHaveValue("Aster");
  expect(onClose).not.toHaveBeenCalled();
  fireEvent.click(within(dialog).getByRole("button", { name: "Save" }));
  await waitFor(() => expect(onClose).toHaveBeenCalledOnce());
  expect(api.createParty).toHaveBeenLastCalledWith({
    name: "Campaign",
    members: [{ name: "Aster" }],
  });
});
