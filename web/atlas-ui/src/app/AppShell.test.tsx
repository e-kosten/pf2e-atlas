import { RouterTestContext } from "../test/router";
import { readinessFixture } from "../test/fixtures";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { AppShell } from "./AppShell";
import {
  checkResponseCompatibility,
  clearCompatibilityFailure,
} from "../api/compatibility";
import { reloadFrontend } from "../shared/browser/reloadFrontend";
vi.mock("../shared/browser/reloadFrontend", () => ({ reloadFrontend: vi.fn() }));

function renderShell(
  errorMessage: string | null = null,
  resolvedColorScheme: "light" | "dark" = "light",
) {
  const refresh = vi.fn();
  const changeTheme = vi.fn();
  render(
    <AppShell
      activeView="search"
      colorScheme="light"
      resolvedColorScheme={resolvedColorScheme}
      onColorSchemeChange={changeTheme}
      readiness={readinessFixture()}
      readinessError={errorMessage ? new Error(errorMessage) : null}
      checkingReadiness={false}
      onCheckReadiness={refresh}
    >
      <main>
        <h2>Search records</h2>
      </main>
    </AppShell>,
    { wrapper: RouterTestContext },
  );
  return { refresh, changeTheme };
}

describe("application navigation and accessibility", () => {
  beforeEach(() => {
    history.replaceState(null, "", "/search");
    clearCompatibilityFailure();
    vi.clearAllMocks();
  });
  it("provides native destinations and handles ordinary navigation once", async () => {
    renderShell();
    const entries = history.length;
    const brand = screen.getByRole("link", { name: "PF2e Atlas" });
    expect(brand).toHaveAttribute("href", "/search");
    expect(brand).toHaveStyle({ color: "var(--brand-text)" });
    expect(brand.querySelector("svg.atlas-mark")).toHaveAttribute(
      "aria-hidden",
      "true",
    );
    expect(brand).toHaveTextContent("PF2e Atlas");
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
    await waitFor(() => expect(location.pathname).toBe("/encounters"));
    expect(history.length).toBe(entries + 1);
  });

  it.each([{ metaKey: true }, { ctrlKey: true }, { shiftKey: true }, { button: 1 }])(
    "leaves modified and nonprimary clicks to the browser %j",
    (modifiers) => {
      renderShell();
      const entries = history.length;
      const allowedDefault = fireEvent.click(
        screen.getByRole("link", { name: "Lists" }),
        modifiers,
      );
      expect(allowedDefault).toBe(true);
      expect(location.pathname).toBe("/search");
      expect(history.length).toBe(entries);
    },
  );

  it("activates a focused Ant menu item with Enter", async () => {
    renderShell();
    const entries = history.length;
    const item = screen.getByRole("menuitem", { name: "Encounters" });
    item.focus();
    fireEvent.keyDown(item, { key: "Enter", code: "Enter", keyCode: 13 });
    await waitFor(() => expect(location.pathname).toBe("/encounters"));
    expect(history.length).toBe(entries + 1);
  });

  it("gives Ant navigation the resolved shell appearance", () => {
    renderShell(null, "dark");
    expect(screen.getByRole("menu", { name: "Atlas views" })).toHaveClass(
      "ant-menu-dark",
    );
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

  it("retains the mounted page and its draft when a later request detects an update", () => {
    render(
      <AppShell
        activeView="search"
        colorScheme="light"
        resolvedColorScheme="light"
        onColorSchemeChange={() => {}}
        readiness={readinessFixture()}
        readinessError={null}
        checkingReadiness={false}
        onCheckReadiness={() => {}}
      >
        <main>
          <input aria-label="Draft" defaultValue="unsaved text" />
        </main>
      </AppShell>,
      { wrapper: RouterTestContext },
    );
    const draft = screen.getByRole("textbox", { name: "Draft" });
    act(() => {
      expect(() =>
        checkResponseCompatibility(
          new Response("{}", { headers: { "x-atlas-api-contract": "other" } }),
        ),
      ).toThrow();
    });
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Atlas was updated. Reload to continue.",
    );
    expect(screen.getByRole("textbox", { name: "Draft" })).toBe(draft);
    expect(draft).toHaveValue("unsaved text");
    expect(reloadFrontend).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Reload page" }));
    expect(reloadFrontend).toHaveBeenCalledTimes(1);
  });

  it("offers system appearance without a persistent healthy-status banner", () => {
    const { changeTheme } = renderShell();
    expect(screen.queryByText("Ready to search.")).not.toBeInTheDocument();
    expect(screen.queryByRole("alert")).toBeNull();
    expect(screen.queryByRole("button", { name: "Reference status" })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Theme" }));
    expect(screen.getByRole("radiogroup", { name: "Appearance" })).toBeInTheDocument();
    fireEvent.click(screen.getByText("System"));
    expect(changeTheme).toHaveBeenCalledWith("system");
  });
});
