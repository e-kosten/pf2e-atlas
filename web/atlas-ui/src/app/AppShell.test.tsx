import { readinessFixture } from "../test/fixtures";
import { fireEvent, render, screen } from "@testing-library/react";
import { AppShell } from "./AppShell";

function renderShell(errorMessage: string | null = null) {
  const navigate = {
    search: vi.fn(),
    encounters: vi.fn(),
    lists: vi.fn(),
  };
  const refresh = vi.fn();
  const changeTheme = vi.fn();
  render(
    <AppShell
      activeView="search"
      colorScheme="light"
      resolvedColorScheme="light"
      onColorSchemeChange={changeTheme}
      onNavigateSearch={navigate.search}
      onNavigateEncounters={navigate.encounters}
      onNavigateLists={navigate.lists}
      readiness={readinessFixture()}
      readinessError={errorMessage ? new Error(errorMessage) : null}
      checkingReadiness={false}
      onCheckReadiness={refresh}
    >
      <main>
        <h2>Search records</h2>
      </main>
    </AppShell>,
  );
  return { navigate, refresh, changeTheme };
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
    fireEvent.click(screen.getByRole("button", { name: "Retry" }));
    expect(refresh).toHaveBeenCalledTimes(1);
    expect(screen.queryByRole("button", { name: "Reference status" })).toBeNull();
  });

  it("offers system appearance without a persistent healthy-status banner", () => {
    const { changeTheme } = renderShell();
    expect(screen.queryByText("Ready to search.")).not.toBeInTheDocument();
    expect(screen.queryByRole("alert")).toBeNull();
    expect(screen.queryByRole("button", { name: "Reference status" })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Theme" }));
    fireEvent.click(screen.getByText("System"));
    expect(changeTheme).toHaveBeenCalledWith("system");
  });
});
