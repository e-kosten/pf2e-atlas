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

it("keeps its entire notice out of every Explore tab while preserving global status and reading mode", async () => {
  const router = createAtlasRouter(
    createMemoryHistory({
      initialEntries: ["/records/items:seed?record_tab=versions"],
    }),
  );
  function Explore() {
    const { setTab } = useRecordExploreLocation();
    return (
      <div>
        <button onClick={() => setTab("versions")}>Versions</button>
        <button onClick={() => setTab("similar")}>Similar</button>
      </div>
    );
  }
  render(
    <RouterTestContext router={router}>
      <RecordReadingWorkspace
        readingNotice={<p>Compact relationship notice</p>}
        status={<p>Refresh failed; retained rules remain available.</p>}
        explore={<Explore />}
        identity={(action) => action}
      >
        <p>Document prose</p>
      </RecordReadingWorkspace>
    </RouterTestContext>,
  );
  expect(screen.getByText("Compact relationship notice")).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Explore related records" }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Versions" })).toBeVisible(),
  );
  expect(screen.queryByText("Compact relationship notice")).toBeNull();
  expect(
    screen.getByText("Refresh failed; retained rules remain available."),
  ).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Similar" }));
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Read document" })).toBeVisible(),
  );
  expect(screen.queryByText("Compact relationship notice")).toBeNull();
  expect(
    screen.getByText("Refresh failed; retained rules remain available."),
  ).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Read document" }));
  await waitFor(() => expect(screen.getByText("Document prose")).toBeVisible());
  expect(screen.getByText("Compact relationship notice")).toBeVisible();
});
