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
  const field = seed.closest(".result-sort__seed-input");
  expect(field).not.toBeNull();
  expect(shuffle.parentElement).toBe(field!.parentElement);
  fireEvent.keyDown(dialog, { key: "Escape" });
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Sort: Random" })).toHaveFocus(),
  );
  expect(screen.queryByRole("button", { name: "Shuffle" })).toBeNull();
});

it("copies a valid committed draft through the field's icon and confirms success without replacing the input", async () => {
  const copy = vi.fn().mockResolvedValue(undefined);
  Object.defineProperty(navigator, "clipboard", {
    configurable: true,
    value: { writeText: copy },
  });
  const changed = setup({
    ...DEFAULT_SEARCH_STATE,
    sort: { kind: "random", seed: 42 },
  });
  await open();
  const input = screen.getByRole("textbox", { name: "Random seed" });
  const field = input.closest(".result-sort__seed-input")!;
  const button = within(field as HTMLElement).getByRole("button", {
    name: "Copy seed",
  });
  act(() => input.focus());
  fireEvent.change(input, { target: { value: "43" } });
  fireEvent.mouseDown(button);
  expect(button).toHaveFocus();
  await act(async () => fireEvent.click(button));
  expect(changed.mock.lastCall?.[0].sort).toEqual({ kind: "random", seed: 43 });
  expect(copy).toHaveBeenCalledWith("43");
  expect(screen.getByRole("textbox", { name: "Random seed" })).toBe(input);
  expect(screen.getByRole("status")).toHaveTextContent("Seed copied");
  expect(within(button).getByRole("img", { name: "check" })).toBeInTheDocument();
  fireEvent.mouseEnter(button);
  expect(await screen.findByRole("tooltip", { name: "Copied" })).toBeInTheDocument();
});

it("copies an edited seed immediately after a prior success while keeping the pointer target stable", async () => {
  const copy = vi.fn().mockResolvedValue(undefined);
  Object.defineProperty(navigator, "clipboard", {
    configurable: true,
    value: { writeText: copy },
  });
  const changed = setup({
    ...DEFAULT_SEARCH_STATE,
    sort: { kind: "random", seed: 42 },
  });
  await open();
  const input = screen.getByRole("textbox", { name: "Random seed" });
  const button = screen.getByRole("button", { name: "Copy seed" });
  await act(async () => fireEvent.click(button));
  expect(copy).toHaveBeenLastCalledWith("42");
  expect(within(button).getByRole("img", { name: "check" })).toBeInTheDocument();
  act(() => input.focus());
  fireEvent.change(input, { target: { value: "43" } });
  fireEvent.mouseDown(button);
  expect(screen.getByRole("button", { name: "Copy seed" })).toBe(button);
  expect(button).toHaveFocus();
  await act(async () => fireEvent.click(button));
  expect(changed.mock.lastCall?.[0].sort).toEqual({ kind: "random", seed: 43 });
  expect(copy.mock.calls).toEqual([["42"], ["43"]]);
  expect(screen.getByRole("status")).toHaveTextContent("Seed copied");
});

it("disables copy for an invalid seed draft and reports actual clipboard failure without success feedback", async () => {
  const copy = vi.fn().mockRejectedValue(new Error("Clipboard denied"));
  Object.defineProperty(navigator, "clipboard", {
    configurable: true,
    value: { writeText: copy },
  });
  setup({ ...DEFAULT_SEARCH_STATE, sort: { kind: "random", seed: 42 } });
  await open();
  const input = screen.getByRole("textbox", { name: "Random seed" });
  const button = screen.getByRole("button", { name: "Copy seed" });
  act(() => input.focus());
  fireEvent.change(input, { target: { value: "invalid" } });
  fireEvent.mouseDown(button);
  expect(button).toBeDisabled();
  fireEvent.click(button);
  expect(copy).not.toHaveBeenCalled();
  fireEvent.change(input, { target: { value: "0" } });
  fireEvent.keyDown(input, { key: "Enter" });
  expect(button).toBeEnabled();
  await act(async () => fireEvent.click(button));
  expect(copy).toHaveBeenCalledWith("0");
  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Unable to copy seed. Select and copy it.",
  );
  expect(screen.getByRole("status")).not.toHaveTextContent("Seed copied");
  expect(within(button).queryByRole("img", { name: "check" })).toBeNull();
  fireEvent.change(input, { target: { value: "1" } });
  fireEvent.keyDown(input, { key: "Enter" });
  expect(screen.queryByRole("alert")).toBeNull();
  fireEvent.change(input, { target: { value: "0" } });
  fireEvent.keyDown(input, { key: "Enter" });
  expect(screen.queryByRole("alert")).toBeNull();
});

it("retires copied feedback and late clipboard results when the seed changes, including A → B → A", async () => {
  let finish!: () => void;
  const copy = vi
    .fn()
    .mockReturnValueOnce(
      new Promise<void>((resolve) => {
        finish = resolve;
      }),
    )
    .mockResolvedValue(undefined);
  Object.defineProperty(navigator, "clipboard", {
    configurable: true,
    value: { writeText: copy },
  });
  setup({ ...DEFAULT_SEARCH_STATE, sort: { kind: "random", seed: 42 } });
  await open();
  const input = screen.getByRole("textbox", { name: "Random seed" });
  const button = screen.getByRole("button", { name: "Copy seed" });
  fireEvent.click(button);
  const change = (seed: string) => {
    fireEvent.change(input, { target: { value: seed } });
    fireEvent.keyDown(input, { key: "Enter" });
  };
  change("43");
  change("42");
  await act(async () => finish());
  expect(screen.getByRole("status")).not.toHaveTextContent("Seed copied");
  await act(async () => fireEvent.click(button));
  expect(screen.getByRole("status")).toHaveTextContent("Seed copied");
  change("43");
  change("42");
  expect(screen.getByRole("status")).not.toHaveTextContent("Seed copied");
  expect(screen.getByRole("textbox", { name: "Random seed" })).toBe(input);
});

it("shows Copied after success without requiring hover/focus and dismisses the confirmation", async () => {
  const copy = vi.fn().mockResolvedValue(undefined);
  Object.defineProperty(navigator, "clipboard", {
    configurable: true,
    value: { writeText: copy },
  });
  setup({ ...DEFAULT_SEARCH_STATE, sort: { kind: "random", seed: 42 } });
  await open();
  vi.useFakeTimers();
  try {
    await act(async () =>
      fireEvent.click(screen.getByRole("button", { name: "Copy seed" })),
    );
    await act(async () => vi.advanceTimersByTime(0));
    expect(screen.getByRole("status")).toHaveTextContent("Seed copied");
    expect(screen.getByRole("tooltip", { name: "Copied" })).toBeInTheDocument();
    await act(async () => vi.advanceTimersByTime(2000));
    expect(screen.getByRole("status")).toBeEmptyDOMElement();
    expect(screen.queryByRole("tooltip", { name: /^(Copied|Copy seed)$/ })).toBeNull();
    expect(
      within(screen.getByRole("button", { name: "Copy seed" })).queryByRole("img", {
        name: "check",
      }),
    ).toBeNull();
  } finally {
    vi.useRealTimers();
  }
});

it("retires a success timer when the seed changes without closing the new seed's tooltip early", async () => {
  const copy = vi.fn().mockResolvedValue(undefined);
  Object.defineProperty(navigator, "clipboard", {
    configurable: true,
    value: { writeText: copy },
  });
  setup({ ...DEFAULT_SEARCH_STATE, sort: { kind: "random", seed: 42 } });
  await open();
  vi.useFakeTimers();
  try {
    const button = screen.getByRole("button", { name: "Copy seed" });
    const input = screen.getByRole("textbox", { name: "Random seed" });
    await act(async () => fireEvent.click(button));
    await act(async () => vi.advanceTimersByTime(1000));
    fireEvent.change(input, { target: { value: "43" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(screen.getByRole("status")).toBeEmptyDOMElement();
    await act(async () => fireEvent.click(button));
    await act(async () => vi.advanceTimersByTime(600));
    expect(copy).toHaveBeenLastCalledWith("43");
    expect(screen.getByRole("tooltip", { name: "Copied" })).toBeInTheDocument();
    await act(async () => vi.advanceTimersByTime(1400));
    expect(screen.queryByRole("tooltip", { name: /^(Copied|Copy seed)$/ })).toBeNull();
  } finally {
    vi.useRealTimers();
  }
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
