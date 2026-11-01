import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { SearchProfileProvider } from "../../test/SearchProfileProvider";
import { navigateDestination } from "../../app/navigation";
import { SelectionSearch } from "./SelectionSearch";

vi.mock("../../app/navigation", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../app/navigation")>()),
  navigateDestination: vi.fn(),
}));

beforeEach(() => {
  vi.clearAllMocks();
  // JSDOM supplies ranges and selection, but no text layout.
  Object.defineProperty(Range.prototype, "getClientRects", {
    configurable: true,
    value: () => [new DOMRect(40, 100, 120, 20)],
  });
});
afterEach(() => {
  cleanup();
  window.getSelection()?.removeAllRanges();
});

function preview(dialog = false) {
  const content = (
    <article data-selection-search>
      <h2 tabIndex={-1}>Reference</h2>
      <p> Fire &amp; ice? </p>
      <label>
        Notes
        <input defaultValue="Private draft" />
      </label>
      <div contentEditable suppressContentEditableWarning>
        Editable draft
      </div>
    </article>
  );
  return render(
    <SearchProfileProvider id="embeddinggemma">
      {dialog ? (
        <section role="dialog" aria-label="Preview">
          {content}
        </section>
      ) : (
        content
      )}
      <p>Outside reading</p>
      <button>Another action</button>
      <SelectionSearch />
    </SearchProfileProvider>,
  );
}

function select(element: Element, end = element) {
  const range = document.createRange();
  range.setStart(element.firstChild!, 0);
  range.setEnd(end.firstChild!, end.firstChild!.textContent!.length);
  act(() => {
    const selection = window.getSelection()!;
    selection.removeAllRanges();
    selection.addRange(range);
    document.dispatchEvent(new Event("selectionchange"));
  });
}
const searchAction = () => screen.findByRole("link", { name: "Search Atlas" });
const settled = () => act(() => new Promise((resolve) => setTimeout(resolve, 230)));

it("offers a fresh native search link, preserving punctuation and the chosen model, without searching on selection", async () => {
  preview();
  select(screen.getByText("Fire & ice?"));
  const action = await searchAction();
  const url = new URL(action.getAttribute("href")!, "http://localhost");
  expect(url.pathname).toBe("/search");
  expect(Object.fromEntries(url.searchParams)).toEqual({
    q: "Fire & ice?",
    search_profile: "embeddinggemma",
  });
  expect(navigateDestination).not.toHaveBeenCalled();
  const originalSelection = window.getSelection()?.toString();
  fireEvent.mouseDown(action);
  expect(window.getSelection()?.toString()).toBe(originalSelection);
  fireEvent.click(action);
  expect(navigateDestination).toHaveBeenCalledExactlyOnceWith(
    expect.objectContaining({
      to: "/search",
      search: { q: "Fire & ice?", search_profile: "embeddinggemma" },
    }),
    { dismissReadingOverlays: true },
  );
  expect(screen.queryByRole("link", { name: "Search Atlas" })).not.toBeInTheDocument();
});

it.each([{ metaKey: true }, { ctrlKey: true }, { shiftKey: true }, { button: 1 }])(
  "leaves modified/native activation to the browser: %j",
  async (modifiers) => {
    preview();
    select(screen.getByText("Fire & ice?"));
    const action = await searchAction();
    const allowed = fireEvent.click(action, modifiers);
    expect(allowed).toBe(true);
    expect(navigateDestination).not.toHaveBeenCalled();
  },
);

it("waits for the drag to finish and replaces the query on a later selection", async () => {
  preview();
  fireEvent.pointerDown(screen.getByText("Fire & ice?"));
  select(screen.getByText("Fire & ice?"));
  await settled();
  expect(screen.queryByRole("link", { name: "Search Atlas" })).not.toBeInTheDocument();
  fireEvent.pointerUp(document);
  await searchAction();
  select(screen.getByRole("heading", { name: "Reference" }));
  await waitFor(() =>
    expect(
      new URL(
        screen.getByRole("link", { name: "Search Atlas" }).getAttribute("href")!,
        "http://localhost",
      ).searchParams.get("q"),
    ).toBe("Reference"),
  );
});

it.each(["Outside reading", "Editable draft"])(
  "ignores selection in %s",
  async (text) => {
    preview();
    select(screen.getByText(text));
    await settled();
    expect(
      screen.queryByRole("link", { name: "Search Atlas" }),
    ).not.toBeInTheDocument();
  },
);

it("ignores a selection crossing reading scopes or an editable control", async () => {
  preview();
  select(screen.getByText("Fire & ice?"), screen.getByText("Outside reading"));
  await settled();
  expect(screen.queryByRole("link", { name: "Search Atlas" })).not.toBeInTheDocument();
  select(screen.getByRole("heading"), screen.getByText("Editable draft"));
  await settled();
  expect(screen.queryByRole("link", { name: "Search Atlas" })).not.toBeInTheDocument();
});

it("does not revive the previous reading selection when an input receives focus", async () => {
  preview();
  select(screen.getByText("Fire & ice?"));
  await searchAction();
  act(() => screen.getByRole("textbox").focus());
  fireEvent.pointerUp(document);
  await settled();
  expect(screen.queryByRole("link", { name: "Search Atlas" })).not.toBeInTheDocument();
});

it("keeps the keyboard action inside a dialog and lets Escape dismiss only the action", async () => {
  preview(true);
  const origin = screen.getByRole("heading");
  act(() => origin.focus());
  select(screen.getByText("Fire & ice?"));
  const action = await searchAction();
  expect(screen.getByRole("dialog")).toContainElement(action);
  expect(origin).toHaveFocus();
  const originalSelection = window.getSelection()?.toString();
  fireEvent.keyDown(document, { key: "Tab" });
  expect(action).toHaveFocus();
  // WebKit clears native selection when keyboard focus moves to the action.
  act(() => window.getSelection()?.removeAllRanges());
  fireEvent(document, new Event("selectionchange"));
  const outerEscape = vi.fn();
  document.addEventListener("keydown", outerEscape);
  fireEvent.keyDown(action, { key: "Escape" });
  document.removeEventListener("keydown", outerEscape);
  expect(outerEscape).not.toHaveBeenCalled();
  expect(origin).toHaveFocus();
  expect(screen.getByRole("dialog")).toBeInTheDocument();
  expect(window.getSelection()?.toString()).toBe(originalSelection);
  fireEvent(document, new Event("selectionchange"));
  await settled();
  expect(screen.queryByRole("link", { name: "Search Atlas" })).not.toBeInTheDocument();
  act(() => {
    const range = window.getSelection()!.getRangeAt(0);
    range.setEnd(range.endContainer, range.endOffset - 3);
    document.dispatchEvent(new Event("selectionchange"));
  });
  const nextAction = await searchAction();
  expect(
    new URL(nextAction.getAttribute("href")!, "http://localhost").searchParams.get("q"),
  ).toBe(window.getSelection()!.toString().trim());
});

it.each(["scroll", "resize", "pointerdown", "clear"])(
  "dismisses on %s",
  async (event) => {
    preview();
    select(screen.getByText("Fire & ice?"));
    await searchAction();
    if (event === "resize") fireEvent(window, new Event("resize"));
    else if (event === "pointerdown") fireEvent.pointerDown(screen.getByRole("button"));
    else if (event === "scroll") fireEvent.scroll(screen.getByRole("article"));
    else {
      act(() => window.getSelection()?.removeAllRanges());
      fireEvent(document, new Event("selectionchange"));
    }
    await waitFor(() =>
      expect(
        screen.queryByRole("link", { name: "Search Atlas" }),
      ).not.toBeInTheDocument(),
    );
  },
);

it("offers no action in disabled reading content and hides an existing action when a write begins", async () => {
  preview();
  const article = screen.getByRole("article");
  article.setAttribute("data-selection-search-disabled", "true");
  select(screen.getByText("Fire & ice?"));
  await settled();
  expect(screen.queryByRole("link", { name: "Search Atlas" })).not.toBeInTheDocument();
  article.removeAttribute("data-selection-search-disabled");
  select(screen.getByText("Fire & ice?"));
  await searchAction();
  act(() => article.setAttribute("data-selection-search-disabled", "true"));
  await waitFor(() =>
    expect(
      screen.queryByRole("link", { name: "Search Atlas" }),
    ).not.toBeInTheDocument(),
  );
});

it("positions the initial action after a reader's queued layout scroll", async () => {
  preview();
  select(screen.getByText("Fire & ice?"));
  fireEvent.scroll(screen.getByRole("article"));
  await act(() => new Promise((resolve) => setTimeout(resolve, 120)));
  fireEvent.scroll(screen.getByRole("article"));
  await act(() => new Promise((resolve) => setTimeout(resolve, 120)));
  fireEvent.scroll(screen.getByRole("article"));
  expect(await searchAction()).toBeInTheDocument();
});
