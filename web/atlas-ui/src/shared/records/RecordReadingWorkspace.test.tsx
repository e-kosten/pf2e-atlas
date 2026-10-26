import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { createMemoryHistory, useRouterState } from "@tanstack/react-router";
import { createAtlasRouter } from "../../app/router";
import { RouterTestContext } from "../../test/router";
import { RecordReadingWorkspace } from "./RecordReadingWorkspace";
import { useRecordExploreLocation } from "./recordExploreLocation";

it("restores reading mode and the selected Explore tab through Back and Forward", async () => {
  const router = createAtlasRouter(
    createMemoryHistory({ initialEntries: ["/records/items:seed"] }),
  );
  function Explore() {
    const { tab, setTab } = useRecordExploreLocation();
    return (
      <div>
        Explore: {tab}
        <button onClick={() => setTab("similar")}>Similar</button>
        <button onClick={() => void router.navigate({ to: "/search" })}>
          Search all
        </button>
      </div>
    );
  }
  function Workspace() {
    const path = useRouterState({ select: (state) => state.location.pathname });
    if (path === "/search") return <div>Search results</div>;
    return (
      <RecordReadingWorkspace
        identity={(action) => <header>{action}</header>}
        explore={<Explore />}
      >
        <p>Document prose</p>
      </RecordReadingWorkspace>
    );
  }
  render(
    <RouterTestContext router={router}>
      <Workspace />
    </RouterTestContext>,
  );
  await waitFor(() => expect(screen.getByText("Document prose")).toBeVisible());
  fireEvent.click(screen.getByRole("button", { name: "Explore related records" }));
  await waitFor(() => expect(screen.getByText("Explore: incoming")).toBeVisible());
  fireEvent.click(screen.getByRole("button", { name: "Similar" }));
  await waitFor(() => expect(screen.getByText("Explore: similar")).toBeVisible());
  fireEvent.click(screen.getByRole("button", { name: "Search all" }));
  await waitFor(() => expect(screen.getByText("Search results")).toBeVisible());
  await act(async () => router.history.back());
  await waitFor(() => expect(screen.getByText("Explore: similar")).toBeVisible());
  expect(screen.queryByText("Document prose")).not.toBeVisible();
  await act(async () => router.history.back());
  await waitFor(() => expect(screen.getByText("Document prose")).toBeVisible());
  await act(async () => router.history.forward());
  await waitFor(() => expect(screen.getByText("Explore: similar")).toBeVisible());
});
