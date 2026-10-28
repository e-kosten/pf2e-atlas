import { createMemoryHistory } from "@tanstack/react-router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { App, ConfigProvider } from "antd";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { atlasRouter } from "../../app/router";
import { listsDestination } from "../../app/destinations";
import { RouterTestContext } from "../../test/router";
import type {
  SavedListIndexView,
  EncounterIndexView as EncounterIndex,
} from "../../generated/atlas";
import { ListIndexView } from "./ListIndexView";
import { EncounterIndexView } from "../encounters/EncounterIndexView";

const api = vi.hoisted(() => ({ getSavedLists: vi.fn(), getEncounters: vi.fn() }));
vi.mock("../../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../../api/atlasApi")>()),
  ...api,
}));

const listIndex: SavedListIndexView = {
  lists: [
    {
      list_key: "list_castle",
      slug: "castle",
      name: "Castle Prep",
      tags: ["prep", "boss"],
      item_count: 1,
      created_at: "2026-01-01",
      updated_at: "2026-01-01",
    },
    {
      list_key: "list_retired",
      slug: "retired",
      name: "Castle Retired",
      tags: ["prep", "retired"],
      item_count: 1,
      created_at: "2026-01-01",
      updated_at: "2026-01-01",
    },
    {
      list_key: "list_riverside",
      slug: "riverside",
      name: "Riverside",
      tags: ["travel"],
      item_count: 1,
      created_at: "2026-01-01",
      updated_at: "2026-01-01",
    },
  ],
};
const encounters: EncounterIndex = {
  encounters: [
    {
      encounter_key: "enc_current",
      slug: "current",
      name: "Current Ambush",
      status: "draft",
      round_number: 0n,
      participant_count: 0,
      created_at: "2026-01-01",
      updated_at: "2026-01-01",
    },
    {
      encounter_key: "enc_old",
      slug: "old",
      name: "Old Battle",
      status: "archived",
      round_number: 0n,
      participant_count: 0,
      created_at: "2026-01-01",
      updated_at: "2026-01-01",
    },
  ],
};

beforeEach(() => {
  api.getSavedLists.mockReset().mockResolvedValue(listIndex);
  api.getEncounters.mockReset().mockResolvedValue(encounters);
});

async function startAt(href: string) {
  atlasRouter.update({ history: createMemoryHistory({ initialEntries: [href] }) });
  await atlasRouter.load();
}

function mount(kind: "lists" | "encounters") {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={client}>
      <ConfigProvider theme={{ token: { motion: false } }}>
        <RouterTestContext>
          <App>
            {kind === "lists" ? (
              <ListIndexView route={{}} />
            ) : (
              <EncounterIndexView route={{}} />
            )}
          </App>
        </RouterTestContext>
      </ConfigProvider>
    </QueryClientProvider>,
  );
}

it("restores list name and included/excluded tag filters from the URL on a fresh mount", async () => {
  await startAt("/lists?q=Castle&tag=prep&tag=travel&exclude_tag=retired");
  const first = mount("lists");
  expect(await screen.findByRole("link", { name: /^Castle Prep/ })).toBeVisible();
  expect(screen.queryByRole("link", { name: /^Castle Retired/ })).toBeNull();
  expect(screen.queryByRole("link", { name: /^Riverside/ })).toBeNull();
  expect(screen.getByRole("textbox", { name: "Search saved lists" })).toHaveValue(
    "Castle",
  );
  expect(
    screen.getByRole("button", { name: "Remove excluded retired from Tags" }),
  ).toBeVisible();
  first.unmount();
  mount("lists");
  expect(await screen.findByRole("link", { name: /^Castle Prep/ })).toBeVisible();
  expect(screen.getByRole("textbox", { name: "Search saved lists" })).toHaveValue(
    "Castle",
  );
  expect(
    screen.getByRole("button", { name: "Remove included travel from Tags" }),
  ).toBeVisible();
});

it("replaces live list filter edits and omits cleared defaults without losing the input draft", async () => {
  await startAt("/lists");
  mount("lists");
  await screen.findByText("Castle Prep");
  const length = atlasRouter.history.length;
  const input = screen.getByRole("textbox", { name: "Search saved lists" });
  fireEvent.change(input, { target: { value: " Castle " } });
  await waitFor(() => expect(atlasRouter.state.location.searchStr).toBe("?q=Castle"));
  expect(input).toHaveValue(" Castle ");
  expect(atlasRouter.history.length).toBe(length);
  expect(screen.queryByText("Riverside")).toBeNull();
  fireEvent.change(input, { target: { value: "" } });
  await waitFor(() => expect(atlasRouter.state.location.searchStr).toBe(""));
  expect(await screen.findByText("Riverside")).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Edit Tags filter" }));
  fireEvent.click(
    await screen.findByRole("button", {
      name: "Tags: boss, neutral; activate to include",
    }),
  );
  await waitFor(() =>
    expect(
      new URLSearchParams(atlasRouter.state.location.searchStr).getAll("tag"),
    ).toEqual(["boss"]),
  );
  expect(atlasRouter.history.length).toBe(length);
  fireEvent.click(
    screen.getByRole("button", { name: "Remove included boss from Tags" }),
  );
  await waitFor(() => expect(atlasRouter.state.location.searchStr).toBe(""));
});

it("restores list filter state through browser Back and Forward", async () => {
  await startAt("/lists?tag=prep");
  mount("lists");
  await screen.findByText("Castle Prep");
  await act(async () => {
    await atlasRouter.navigate(listsDestination({ include: ["travel"] }));
  });
  expect(await screen.findByText("Riverside")).toBeVisible();
  expect(screen.queryByText("Castle Prep")).toBeNull();
  await act(async () => atlasRouter.history.back());
  expect(await screen.findByText("Castle Prep")).toBeVisible();
  expect(screen.queryByText("Riverside")).toBeNull();
  await act(async () => atlasRouter.history.forward());
  expect(await screen.findByText("Riverside")).toBeVisible();
});

it("restores archived encounters and pushes tab changes while omitting the Active default", async () => {
  await startAt("/encounters?view=archived");
  const first = mount("encounters");
  expect(await screen.findByText("Old Battle")).toBeVisible();
  expect(screen.getByRole("tab", { name: "Archived", selected: true })).toBeVisible();
  expect(screen.queryByText("Current Ambush")).toBeNull();
  const length = atlasRouter.history.length;
  fireEvent.click(screen.getByRole("tab", { name: "Active" }));
  expect(await screen.findByText("Current Ambush")).toBeVisible();
  expect(atlasRouter.state.location.searchStr).toBe("");
  expect(atlasRouter.history.length).toBe(length + 1);
  await act(async () => atlasRouter.history.back());
  expect(await screen.findByText("Old Battle")).toBeVisible();
  expect(atlasRouter.state.location.searchStr).toBe("?view=archived");
  first.unmount();
  mount("encounters");
  expect(await screen.findByText("Old Battle")).toBeVisible();
  await act(async () => atlasRouter.history.forward());
  expect(await screen.findByText("Current Ambush")).toBeVisible();
});
