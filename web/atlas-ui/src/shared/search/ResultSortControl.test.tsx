import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { ConfigProvider } from "antd";
import { useState } from "react";
import {
  DEFAULT_SEARCH_STATE,
  searchStateQueryString,
  type SearchFormState,
} from "../filters/searchState";
import { ResultSortControl } from "./ResultSortControl";
import { chooseAttributeSort, selectSortOption } from "../../test/sortControls";

function setup(
  initial: SearchFormState = DEFAULT_SEARCH_STATE,
  surface: "search" | "saved_list" = "search",
  syncUrl = false,
) {
  const changed = vi.fn();
  function Harness() {
    const [search, setSearch] = useState(initial);
    return (
      <ResultSortControl
        search={search}
        surface={surface}
        setSearch={(next) => {
          changed(next);
          if (syncUrl)
            history.replaceState(null, "", "/search" + searchStateQueryString(next));
          setSearch(next);
        }}
      />
    );
  }
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <Harness />
    </ConfigProvider>,
  );
  return changed;
}
async function open() {
  fireEvent.click(screen.getByRole("button", { name: /^Sort:/ }));
  await screen.findByRole("dialog", { name: "Sort results" });
}

it("shows Sort explicitly and records choosing the displayed automatic default", async () => {
  const changed = setup();
  await open();
  await selectSortOption("Sort field 1", "Name");
  expect(changed).toHaveBeenLastCalledWith({
    ...DEFAULT_SEARCH_STATE,
    sort: { kind: "attributes", rules: [{ field: "name", direction: "asc" }] },
  });
});
it("edits an ordered unique chain with accessible moves/removal and resets the last rule to automatic", async () => {
  const changed = setup();
  await chooseAttributeSort("level", "desc");
  await open();
  fireEvent.click(screen.getByRole("button", { name: "Add another sort" }));
  await selectSortOption("Sort field 2", "Unit price");
  fireEvent.click(screen.getByRole("button", { name: "Add another sort" }));
  expect(screen.getByRole("button", { name: "Add another sort" })).toBeDisabled();
  expect(changed.mock.lastCall?.[0].sort.rules).toEqual([
    { field: "level", direction: "desc" },
    { field: "unit_price", direction: "asc" },
    { field: "name", direction: "asc" },
  ]);
  fireEvent.click(screen.getByRole("button", { name: "Move sort rule 2 up" }));
  expect(changed.mock.lastCall?.[0].sort.rules[0].field).toBe("unit_price");
  for (let remaining = 3; remaining > 0; remaining--)
    fireEvent.click(screen.getByRole("button", { name: "Remove sort rule 1" }));
  expect(changed.mock.lastCall?.[0].sort).toBeUndefined();
});
it("offers Relevance only with semantic criteria and Saved order only in lists", async () => {
  setup({ ...DEFAULT_SEARCH_STATE, query: "fire" }, "saved_list");
  await open();
  fireEvent.mouseDown(screen.getByRole("combobox", { name: "Sort mode" }));
  expect(screen.getByRole("option", { name: "Saved order" })).toBeInTheDocument();
  expect(screen.queryByRole("option", { name: "Relevance" })).toBeNull();
});
it("generates and shuffles a seed, commits edited seeds only on Enter/blur, and rejects malformed drafts", async () => {
  const changed = setup();
  vi.spyOn(crypto, "getRandomValues").mockImplementation((array) => {
    (array as Uint32Array)[0] = 42;
    return array;
  });
  await open();
  await selectSortOption("Sort mode", "Random");
  expect(changed.mock.lastCall?.[0].sort).toEqual({ kind: "random", seed: 42 });
  let input = screen.getByRole("textbox", { name: "Random seed" });
  const calls = changed.mock.calls.length;
  input.focus();
  fireEvent.change(input, { target: { value: "0" } });
  expect(changed).toHaveBeenCalledTimes(calls);
  fireEvent.keyDown(input, { key: "Enter" });
  expect(changed.mock.lastCall?.[0].sort).toEqual({ kind: "random", seed: 0 });
  expect(screen.getByRole("textbox", { name: "Random seed" })).toBe(input);
  expect(input).toHaveFocus();
  fireEvent.change(input, { target: { value: "4294967295" } });
  fireEvent.blur(input);
  expect(changed.mock.lastCall?.[0].sort).toEqual({ kind: "random", seed: 4294967295 });
  input = screen.getByRole("textbox", { name: "Random seed" });
  fireEvent.change(input, { target: { value: "12oops" } });
  await act(async () => fireEvent.blur(input));
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Random seed must be a whole number",
  );
  expect(changed.mock.lastCall?.[0].sort.seed).toBe(4294967295);
  fireEvent.click(screen.getByRole("button", { name: "Shuffle" }));
  expect(changed.mock.lastCall?.[0].sort).toEqual({ kind: "random", seed: 42 });
  expect(screen.getByRole("textbox", { name: "Random seed" })).toHaveValue("42");
  expect(screen.queryByRole("alert")).toBeNull();
});
it("copies the authoritative seed and restores focus on Escape", async () => {
  const copy = vi.fn().mockResolvedValue(undefined);
  Object.defineProperty(navigator, "clipboard", {
    configurable: true,
    value: { writeText: copy },
  });
  setup({ ...DEFAULT_SEARCH_STATE, sort: { kind: "random", seed: 0 } });
  await open();
  fireEvent.click(screen.getByRole("button", { name: "Copy seed" }));
  expect(copy).toHaveBeenCalledWith("0");
  fireEvent.keyDown(screen.getByRole("dialog", { name: "Sort results" }), {
    key: "Escape",
  });
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Sort: Random" })).toHaveFocus(),
  );
});

it("keeps Shuffle beside the seed in the popover and out of the toolbar", async () => {
  setup({ ...DEFAULT_SEARCH_STATE, sort: { kind: "random", seed: 42 } });
  expect(screen.queryByRole("button", { name: "Shuffle" })).toBeNull();
  await open();
  const dialog = screen.getByRole("dialog", { name: "Sort results" });
  const seed = within(dialog).getByRole("textbox", { name: "Random seed" });
  const shuffle = within(dialog).getByRole("button", { name: "Shuffle" });
  expect(shuffle.parentElement).toBe(seed.parentElement);
  fireEvent.keyDown(dialog, { key: "Escape" });
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Sort: Random" })).toHaveFocus(),
  );
  expect(screen.queryByRole("button", { name: "Shuffle" })).toBeNull();
});

it("cancels an uncommitted seed on Escape before dismissal or focus restoration can blur-commit it", async () => {
  history.replaceState(null, "", "/search?sort=random&seed=42");
  const changed = setup(
    { ...DEFAULT_SEARCH_STATE, sort: { kind: "random", seed: 42 } },
    "search",
    true,
  );
  await open();
  const input = screen.getByRole("textbox", { name: "Random seed" });
  act(() => input.focus());
  fireEvent.change(input, { target: { value: "43" } });
  fireEvent.keyDown(input, { key: "Escape", keyCode: 27, which: 27 });
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Sort: Random" })).toHaveFocus(),
  );
  expect(changed).not.toHaveBeenCalled();
  expect(new URLSearchParams(location.search).get("seed")).toBe("42");
  expect(screen.getByRole("button", { name: "Sort: Random" })).toHaveAttribute(
    "aria-expanded",
    "false",
  );
  await open();
  expect(screen.getByRole("textbox", { name: "Random seed" })).toHaveValue("42");
});

it("lets an expanded Select consume Escape before dismissing the Sort dialog", async () => {
  setup();
  await open();
  const input = screen.getByRole("combobox", { name: "Sort mode" });
  fireEvent.mouseDown(input);
  expect(input).toHaveAttribute("aria-expanded", "true");
  await screen.findByRole("option", { name: "By field" });
  fireEvent.keyDown(input, { key: "Escape", keyCode: 27, which: 27 });
  await waitFor(() => expect(input).toHaveAttribute("aria-expanded", "false"));
  expect(screen.getByRole("button", { name: /^Sort:/ })).toHaveAttribute(
    "aria-expanded",
    "true",
  );
  fireEvent.keyDown(input, { key: "Escape", keyCode: 27, which: 27 });
  await waitFor(() =>
    expect(screen.getByRole("button", { name: /^Sort:/ })).toHaveFocus(),
  );
  expect(screen.getByRole("button", { name: /^Sort:/ })).toHaveAttribute(
    "aria-expanded",
    "false",
  );
});
