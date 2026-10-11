import { fireEvent, render, screen } from "@testing-library/react";
import { AppShell } from "./AppShell";
import type { SearchWorkspaceState } from "../features/search/useSearchWorkspace";
import { DEFAULT_SEARCH_STATE } from "../shared/filters/searchState";

function renderShell(errorMessage: string | null = null) {
  const navigate = {
    search: vi.fn(),
    encounters: vi.fn(),
    lists: vi.fn(),
  };
  const refresh = vi.fn();
  const workspace: SearchWorkspaceState = {
    search: DEFAULT_SEARCH_STATE,
    setSearch: vi.fn(),
    activeResultKey: null,
    focusResult: vi.fn(),
    moveResultSelection: vi.fn(),
    openActiveResult: vi.fn(),
    selectRecord: vi.fn(),
    setPageNumber: vi.fn(),
    recordDetail: undefined,
    detailError: null,
    retryDetail: vi.fn(),
    filterEditor: undefined,
    filterDiscoveryContext: {
      kind: "filtered",
      filter: null,
      text: null,
      mode: "hybrid",
    },
    vocabularyScope: { kind: "artifact" },
    filterDiscoveryError: null,
    readiness: {
      data: { status: "ready", message: "Ready to search." },
    } as SearchWorkspaceState["readiness"],
    diagnostics: {
      activeWindowId: null,
      searchDebouncing: false,
      resultRequest: null,
      detailRequest: null,
    },
    resultPage: undefined,
    resultsLoading: false,
    resultsRefreshing: false,
    detailLoading: false,
    filterDiscoveryLoading: false,
    pageNumber: 1,
    selectedRecordKey: null,
    errorMessage,
    refresh,
  };
  render(
    <AppShell
      activeView="search"
      colorScheme="light"
      resolvedColorScheme="light"
      onColorSchemeChange={vi.fn()}
      onNavigateSearch={navigate.search}
      onNavigateEncounters={navigate.encounters}
      onNavigateLists={navigate.lists}
      workspace={workspace}
    >
      <main>
        <h2>Search records</h2>
      </main>
    </AppShell>,
  );
  return { navigate, refresh };
}

describe("application navigation and accessibility", () => {
  it("provides native destinations and handles ordinary navigation once", () => {
    const { navigate } = renderShell();
    const encounters = screen.getByRole("link", { name: "Encounters" });
    expect(encounters).toHaveAttribute("href", "/encounters");
    expect(screen.getByRole("link", { name: "Lists" })).toHaveAttribute(
      "href",
      "/lists",
    );
    expect(screen.getByRole("link", { name: "Search" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    fireEvent.click(encounters);
    expect(navigate.encounters).toHaveBeenCalledTimes(1);
    expect(navigate.search).not.toHaveBeenCalled();
  });

  it.each([{ metaKey: true }, { ctrlKey: true }, { shiftKey: true }, { button: 1 }])(
    "leaves modified and nonprimary clicks to the browser %j",
    (modifiers) => {
      const { navigate } = renderShell();
      const allowedDefault = fireEvent.click(
        screen.getByRole("link", { name: "Lists" }),
        modifiers,
      );
      expect(allowedDefault).toBe(true);
      expect(navigate.lists).not.toHaveBeenCalled();
    },
  );

  it("activates a focused Ant menu item with Enter", () => {
    const { navigate } = renderShell();
    const item = screen.getByRole("menuitem", { name: "Encounters" });
    item.focus();
    fireEvent.keyDown(item, { key: "Enter", code: "Enter", keyCode: 13 });
    expect(navigate.encounters).toHaveBeenCalledTimes(1);
  });

  it("skips to the content without creating a second main landmark", () => {
    renderShell();
    fireEvent.click(screen.getByRole("link", { name: "Skip to main content" }));
    expect(document.activeElement).toHaveAttribute("id", "atlas-main");
    expect(screen.getAllByRole("main")).toHaveLength(1);
  });

  it("names icon actions and announces the actual global error", () => {
    const { refresh } = renderShell("The local service is unavailable.");
    expect(screen.getByRole("alert")).toHaveTextContent(
      "The local service is unavailable.",
    );
    fireEvent.click(screen.getByRole("button", { name: "Refresh" }));
    expect(refresh).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByRole("button", { name: "Diagnostics" }));
    expect(screen.getByRole("region", { name: "Diagnostics" })).toBeVisible();
  });
});
