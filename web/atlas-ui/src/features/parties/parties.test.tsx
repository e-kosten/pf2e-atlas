import { AtlasApiError } from "../../api/apiError";

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
import { RouterTestContext } from "../../test/router";
import { PartyPage } from "./PartyPage";
import { PartiesPage } from "./PartiesPage";
const api = vi.hoisted(() => ({
  getParties: vi.fn(),
  getCharacters: vi.fn(),
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
    {
      character_key: "pc-a",
      name: "Aster",
      character_revision: 2,
      imported: true,
      level: 5,
    },
    { character_key: "pc-b", name: "Bram", character_revision: 3, imported: false },
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
  for (const mock of Object.values(api)) mock.mockReset();
  api.getParties.mockResolvedValue({ parties: [party] });
});
afterEach(cleanup);
it("adds the whole chosen party in one click without a modal or subset fields", async () => {
  const onAdd = vi.fn().mockResolvedValue(undefined);
  render(
    <EncounterAddMenu
      encounterRevision={9}
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
      expected_encounter_revision: 9,
      request_id: expect.any(String),
      characters: [
        { character_key: "pc-a", expected_revision: 2 },
        { character_key: "pc-b", expected_revision: 3 },
      ],
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
      encounterRevision={9}
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
      encounterRevision={9}
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

it("retains the new party name after a failed save and retries the empty roster", async () => {
  api.createParty
    .mockRejectedValueOnce(new Error("Unavailable"))
    .mockResolvedValue({ party_key: "party" });
  render(<PartiesPage />, { wrapper });
  fireEvent.click(await screen.findByRole("button", { name: "New party" }));
  const dialog = await screen.findByRole("dialog", { name: "New party" });
  const name = within(dialog).getByLabelText("Party name");
  fireEvent.change(name, { target: { value: "My group" } });
  fireEvent.click(within(dialog).getByRole("button", { name: "Create party" }));
  expect(await within(dialog).findByText("Party could not be saved")).toBeVisible();
  expect(name).toHaveValue("My group");
  fireEvent.click(within(dialog).getByRole("button", { name: "Create party" }));
  await waitFor(() => expect(api.createParty).toHaveBeenCalledTimes(2));
  expect(api.createParty).toHaveBeenLastCalledWith({ name: "My group", members: [] });
});

it("shows shared memberships during level review and updates only selected characters", async () => {
  api.getCharacters.mockResolvedValue({
    characters: party.members.map((member) => ({
      ...member,
      revision: member.character_revision,
      parties:
        member.character_key === "pc-a"
          ? [{ party_key: "other", name: "Second table" }]
          : [],
    })),
  });
  render(<PartyPage partyKey="party" />, { wrapper });
  expect(await screen.findByRole("link", { name: "Aster" })).toHaveAttribute(
    "href",
    "/characters/pc-a",
  );
  expect(screen.queryByText(/Second table/)).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Party actions" }));
  fireEvent.click(await screen.findByRole("button", { name: "Set character levels…" }));
  const dialog = await screen.findByRole("dialog", { name: "Set character levels" });
  expect(await within(dialog).findByRole("checkbox", { name: "Aster" })).toBeChecked();
  expect(within(dialog).getByText("Also updates Second table")).toBeVisible();
  expect(within(dialog).getByText("Level 5")).toBeVisible();
  fireEvent.click(within(dialog).getByRole("checkbox", { name: "Bram" }));
  fireEvent.change(within(dialog).getByRole("spinbutton", { name: "New level" }), {
    target: { value: "7" },
  });
  fireEvent.click(
    within(dialog).getByRole("button", { name: "Set 1 character to level 7" }),
  );
  await waitFor(() =>
    expect(api.updateParty).toHaveBeenCalledWith("party", {
      expected_revision: 4,
      name: "Campaign",
      members: [
        { character_key: "pc-a", expected_character_revision: 2 },
        { character_key: "pc-b", expected_character_revision: 3 },
      ],
      level_updates: [
        { character_key: "pc-a", expected_character_revision: 2, level: 7 },
      ],
    }),
  );
});

it("reviews a concurrent roster change before retrying retained level and character choices", async () => {
  const asCharacters = (members: typeof party.members) => ({
    characters: members.map((m) => ({
      ...m,
      revision: m.character_revision,
      parties: [],
    })),
  });
  api.getCharacters.mockResolvedValue(asCharacters(party.members));
  api.updateParty
    .mockRejectedValueOnce(new AtlasApiError(409, "The party changed."))
    .mockResolvedValue({ party_key: "party" });
  render(<PartyPage partyKey="party" />, { wrapper });
  fireEvent.click(await screen.findByRole("button", { name: "Party actions" }));
  fireEvent.click(await screen.findByRole("button", { name: "Set character levels…" }));
  const dialog = within(
    await screen.findByRole("dialog", { name: "Set character levels" }),
  );
  fireEvent.click(dialog.getByRole("checkbox", { name: "Bram" }));
  fireEvent.change(dialog.getByRole("spinbutton", { name: "New level" }), {
    target: { value: "7" },
  });
  const save = dialog.getByRole("button", { name: "Set 1 character to level 7" });
  await waitFor(() => expect(save).toBeEnabled());
  fireEvent.click(save);
  const review = await dialog.findByRole("button", { name: "Review latest changes" });
  expect(save).toBeDisabled();
  const newMember = {
    character_key: "pc-c",
    character_revision: 8,
    name: "Cale",
    level: 3,
    imported: false,
  };
  const members = [newMember, { ...party.members[0], character_revision: 6, level: 4 }];
  api.getParties.mockResolvedValue({
    parties: [{ ...party, name: "Saved elsewhere", revision: 9, members }],
  });
  api.getCharacters.mockResolvedValue(asCharacters(members));
  fireEvent.click(review);
  expect(await dialog.findByRole("status")).toHaveTextContent(
    "Latest changes reviewed",
  );
  expect(dialog.getByRole("spinbutton", { name: "New level" })).toHaveValue("7");
  expect(dialog.getByRole("checkbox", { name: "Aster" })).toBeChecked();
  expect(dialog.getByText("Level 4")).toBeVisible();
  expect(dialog.getByRole("checkbox", { name: "Cale" })).not.toBeChecked();
  expect(dialog.queryByRole("checkbox", { name: "Bram" })).toBeNull();
  expect(api.updateParty).toHaveBeenCalledTimes(1);
  await waitFor(() => expect(save).toBeEnabled());
  fireEvent.click(save);
  await waitFor(() =>
    expect(api.updateParty).toHaveBeenLastCalledWith("party", {
      name: "Saved elsewhere",
      expected_revision: 9,
      members: [
        { character_key: "pc-c", expected_character_revision: 8 },
        { character_key: "pc-a", expected_character_revision: 6 },
      ],
      level_updates: [
        { character_key: "pc-a", expected_character_revision: 6, level: 7 },
      ],
    }),
  );
});

it("reviews the saved name and current roster while retaining a separate rename draft", async () => {
  api.updateParty.mockRejectedValueOnce(new AtlasApiError(409, "The party changed."));
  render(<PartyPage partyKey="party" />, { wrapper });
  fireEvent.click(await screen.findByRole("button", { name: "Party actions" }));
  fireEvent.click(await screen.findByRole("button", { name: "Rename party" }));
  const dialog = within(await screen.findByRole("dialog", { name: "Rename party" }));
  const input = dialog.getByRole("textbox", { name: "Party name" });
  expect(dialog.queryByRole("spinbutton")).toBeNull();
  const save = dialog.getByRole("button", { name: "Rename" });
  expect(save).toBeDisabled();
  fireEvent.change(input, { target: { value: "My draft" } });
  await waitFor(() => expect(save).toBeEnabled());
  fireEvent.click(save);
  const review = await dialog.findByRole("button", { name: "Review latest changes" });
  api.getParties.mockResolvedValue({
    parties: [
      { ...party, name: "Saved elsewhere", revision: 9, members: [party.members[0]] },
    ],
  });
  fireEvent.click(review);
  expect(await dialog.findByText("Saved name: Saved elsewhere")).toBeVisible();
  expect(input).toHaveValue("My draft");
  expect(api.updateParty).toHaveBeenCalledTimes(1);
  await waitFor(() => expect(save).toBeEnabled());
  fireEvent.click(save);
  await waitFor(() =>
    expect(api.updateParty).toHaveBeenLastCalledWith("party", {
      name: "My draft",
      expected_revision: 9,
      members: [{ character_key: "pc-a", expected_character_revision: 2 }],
      level_updates: [],
    }),
  );
});
