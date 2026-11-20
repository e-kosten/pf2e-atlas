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
import type { PartyView } from "../../generated/atlas";
import { RouterTestContext } from "../../test/router";
import { RenameParty, SetPartyLevels } from "./PartyEditors";

const api = vi.hoisted(() => ({
  getParties: vi.fn(),
  updateParty: vi.fn(),
}));
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  ...api,
}));
const party: PartyView = {
  party_key: "p",
  name: "Campaign",
  revision: 3,
  members: [
    {
      character_key: "a",
      name: "Aster",
      level: 7,
      character_revision: 2,
      imported: true,
    },
    {
      character_key: "b",
      name: "Bram",
      level: 7,
      character_revision: 5,
      imported: false,
    },
  ],
};
function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return (
    <QueryClientProvider client={client}>
      <ConfigProvider theme={{ token: { motion: false } }}>
        <RouterTestContext>
          <App>{children}</App>
        </RouterTestContext>
      </ConfigProvider>
    </QueryClientProvider>
  );
}
beforeEach(() => {
  vi.clearAllMocks();
  api.getParties.mockResolvedValue({ parties: [party] });
  api.updateParty.mockResolvedValue(party);
});
afterEach(cleanup);

it("closes a reverted rename directly, including whitespace-only differences", async () => {
  const close = vi.fn();
  render(<RenameParty party={party} onClose={close} />, { wrapper });
  const dialog = within(await screen.findByRole("dialog", { name: "Rename party" }));
  const name = dialog.getByRole("textbox", { name: "Party name" });
  fireEvent.change(name, { target: { value: "A changed name" } });
  await waitFor(() =>
    expect(dialog.getByRole("button", { name: "Rename" })).toBeEnabled(),
  );
  fireEvent.change(name, { target: { value: " Campaign " } });
  await waitFor(() =>
    expect(dialog.getByRole("button", { name: "Rename" })).toBeDisabled(),
  );
  fireEvent.click(dialog.getByRole("button", { name: "Cancel" }));
  await waitFor(() => expect(close).toHaveBeenCalledOnce());
  expect(screen.queryByText("Discard this draft?")).toBeNull();
  expect(api.updateParty).not.toHaveBeenCalled();
});

it.each(["pristine", "same level", "cleared target", "empty selection"])(
  "closes %s level work without a discard prompt",
  async (scenario) => {
    const close = vi.fn();
    render(<SetPartyLevels party={party} onClose={close} />, { wrapper });
    const dialog = within(
      await screen.findByRole("dialog", { name: "Set character levels" }),
    );
    const level = dialog.getByRole("spinbutton", { name: "New level" });
    if (scenario === "same level") fireEvent.change(level, { target: { value: "7" } });
    if (scenario === "cleared target") {
      fireEvent.change(level, { target: { value: "8" } });
      fireEvent.change(level, { target: { value: "" } });
    }
    if (scenario === "empty selection") {
      fireEvent.change(level, { target: { value: "8" } });
      fireEvent.click(dialog.getByRole("checkbox", { name: "Aster" }));
      fireEvent.click(dialog.getByRole("checkbox", { name: "Bram" }));
    }
    expect(dialog.getByRole("button", { name: "Set character levels" })).toBeDisabled();
    fireEvent.click(dialog.getByRole("button", { name: "Cancel" }));
    await waitFor(() => expect(close).toHaveBeenCalledOnce());
    expect(screen.queryByText("Discard this draft?")).toBeNull();
    expect(api.updateParty).not.toHaveBeenCalled();
  },
);

it("updates only effective changes when the selected characters have different levels", async () => {
  const mixed = {
    ...party,
    members: [party.members[0], { ...party.members[1], level: 8 }],
  };
  render(<SetPartyLevels party={mixed} onClose={vi.fn()} />, { wrapper });
  const dialog = within(
    await screen.findByRole("dialog", { name: "Set character levels" }),
  );
  fireEvent.change(dialog.getByRole("spinbutton", { name: "New level" }), {
    target: { value: "7" },
  });
  const save = dialog.getByRole("button", { name: "Set 1 character to level 7" });
  await waitFor(() => expect(save).toBeEnabled());
  fireEvent.click(save);
  await waitFor(() =>
    expect(api.updateParty).toHaveBeenCalledWith("p", {
      expected_revision: 3,
      name: "Campaign",
      members: [
        { character_key: "a", expected_character_revision: 2 },
        { character_key: "b", expected_character_revision: 5 },
      ],
      level_updates: [{ character_key: "b", expected_character_revision: 5, level: 7 }],
    }),
  );
});

it.each(["-", "1."])(
  "protects unfinished numeric text %s and retains it when returning to edit",
  async (text) => {
    const close = vi.fn();
    render(<SetPartyLevels party={party} onClose={close} />, { wrapper });
    const dialog = within(
      await screen.findByRole("dialog", { name: "Set character levels" }),
    );
    const level = dialog.getByRole("spinbutton", { name: "New level" });
    fireEvent.keyDown(level, { key: text.slice(-1) });
    fireEvent.change(level, { target: { value: text } });
    fireEvent.keyUp(level, { key: text.slice(-1) });
    expect(level).toHaveValue(text);
    fireEvent.keyDown(level, { key: "Enter" });
    expect(level).toHaveValue(text);
    expect(dialog.getByRole("button", { name: "Set character levels" })).toBeDisabled();
    fireEvent.click(dialog.getByRole("button", { name: "Cancel" }));
    const keep = await dialog.findByRole("button", { name: "Keep editing" });
    await waitFor(() => expect(keep).toHaveFocus());
    expect(close).not.toHaveBeenCalled();
    fireEvent.click(keep);
    await waitFor(() => expect(level).toBeVisible());
    expect(level).toHaveValue(text);
    fireEvent.click(dialog.getByRole("button", { name: "Cancel" }));
    fireEvent.click(await dialog.findByRole("button", { name: "Discard" }));
    await waitFor(() => expect(close).toHaveBeenCalledOnce());
    expect(api.updateParty).not.toHaveBeenCalled();
  },
);

it("keeps a submitted level edit open until its write completes", async () => {
  let finish!: (value: PartyView) => void;
  api.updateParty.mockReturnValue(
    new Promise<PartyView>((resolve) => {
      finish = resolve;
    }),
  );
  const close = vi.fn();
  render(<SetPartyLevels party={party} onClose={close} />, { wrapper });
  const dialog = within(
    await screen.findByRole("dialog", { name: "Set character levels" }),
  );
  fireEvent.change(dialog.getByRole("spinbutton", { name: "New level" }), {
    target: { value: "8" },
  });
  const save = dialog.getByRole("button", { name: "Set 2 characters to level 8" });
  await waitFor(() => expect(save).toBeEnabled());
  fireEvent.click(save);
  await waitFor(() =>
    expect(dialog.getByRole("button", { name: "Cancel" })).toBeDisabled(),
  );
  expect(dialog.queryByRole("button", { name: "Close" })).toBeNull();
  expect(close).not.toHaveBeenCalled();
  finish(party);
  await waitFor(() => expect(close).toHaveBeenCalledOnce());
});
