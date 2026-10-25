import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ConfigProvider, Grid } from "antd";
import { useState, type ComponentProps, type ReactElement } from "react";
import { CommonChoicePicker } from "./CommonChoicePicker";
import { editorFixture, fieldFixture } from "../../test/fixtures";
import type { FilterVocabularyScope, QueryPredicate } from "../../generated/atlas";

const api = vi.hoisted(() => ({
  discoverFilterValues: vi.fn(),
  discoverFilterCounts: vi.fn(),
}));
const popup = vi.hoisted(() => ({
  deferred: false,
  mount: null as (() => void) | null,
  complete: null as ((open: boolean) => void) | null,
}));
vi.mock("../../api/atlasApi", () => api);
vi.mock("antd", async (importOriginal) => {
  const original = await importOriginal<typeof import("antd")>();
  const react = await import("react");
  function Popover(props: ComponentProps<typeof original.Popover>) {
    const [mounted, setMounted] = react.useState(false);
    if (!popup.deferred) return <original.Popover {...props} />;
    // Model native portal/motion readiness independently of the parent render.
    // Keep hidden content mounted during closing, as a popup motion can do.
    popup.mount = () => setMounted(true);
    popup.complete = props.afterOpenChange ?? null;
    return (
      <>
        {react.cloneElement(props.children as ReactElement<{ onClick: () => void }>, {
          onClick: () => props.onOpenChange?.(!props.open),
        })}
        {mounted && <div hidden={!props.open}>{props.content as ReactElement}</div>}
      </>
    );
  }
  return { ...original, Popover };
});
function picker({
  closed = true,
  initial = null,
  scope = { kind: "artifact" },
  withCounts = false,
}: {
  closed?: boolean;
  initial?: QueryPredicate | null;
  scope?: FilterVocabularyScope;
  withCounts?: boolean;
} = {}) {
  const definition = {
    ...fieldFixture("traits", "set", "Traits"),
    choices: closed ? ["fire", "cold", "magical"] : [],
  };
  const field = editorFixture([definition]).groups[0].fields[0];
  const onChange = vi.fn();
  function Harness() {
    const [clause, setClause] = useState(initial);
    return (
      <CommonChoicePicker
        field={field}
        clause={clause}
        scope={scope}
        countLimit={128}
        countContext={
          withCounts
            ? { kind: "filtered", filter: clause, text: null, mode: "hybrid" }
            : undefined
        }
        onChange={(next) => {
          setClause(next);
          onChange(next);
        }}
      />
    );
  }
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <QueryClientProvider
        client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
      >
        <Harness />
      </QueryClientProvider>
    </ConfigProvider>,
  );
  return onChange;
}
function open() {
  fireEvent.click(screen.getByRole("button", { name: "Edit Traits filter" }));
}
describe("compact common choice picker", () => {
  beforeEach(() => {
    popup.deferred = false;
    popup.mount = null;
    popup.complete = null;
    vi.clearAllMocks();
    api.discoverFilterCounts.mockResolvedValue({
      search_profile: "bge-base",
      artifact_generation: "generation-a",
      counts: {
        status: "available",
        value: {
          field: "traits",
          options: [
            { value: "fire", distinct_roots: 4n, selected: false },
            { value: "cold", distinct_roots: 0n, selected: false },
          ],
          total_values: 2n,
          exhaustive: true,
          count_basis: "exact counterfactual displayed roots",
        },
      },
    });
    vi.spyOn(Grid, "useBreakpoint").mockReturnValue({ lg: true });
    api.discoverFilterValues.mockImplementation(async (request) =>
      request.kind === "contextual"
        ? {
            kind: "contextual",
            search_profile: "bge-base",
            artifact_generation: "generation-a",
            outcome: { status: "unavailable", reason: "non_conjunctive_context" },
          }
        : {
            kind: "vocabulary",
            scope: request.scope,
            values: {
              field: request.field_id,
              selected_values: request.selected_values,
              total_values: 90n,
              options: request.offset
                ? [{ value: "second-page" }]
                : [{ value: "fire" }, { value: "cold" }],
              basis: "product_key_scope_before_edition_preference",
            },
          },
    );
  });
  afterEach(() => vi.restoreAllMocks());
  it("loads counts only for the open picker and permits excluding zero choices", async () => {
    picker({ withCounts: true });
    expect(api.discoverFilterCounts).not.toHaveBeenCalled();
    open();
    expect(await screen.findByLabelText("4 matching records")).toBeInTheDocument();
    expect(screen.getByLabelText("0 matching records")).toBeInTheDocument();
    expect(
      screen.getByRole("button", {
        name: "Traits: Cold, neutral; activate to exclude",
      }),
    ).toBeEnabled();
    expect(
      screen.getByRole("button", {
        name: "Traits: Magical, neutral; activate to include",
      }),
    ).toBeEnabled();
    expect(api.discoverFilterCounts).toHaveBeenCalledTimes(1);
    fireEvent.keyDown(screen.getByRole("textbox", { name: "Search traits choices" }), {
      key: "Escape",
    });
    expect(screen.queryByLabelText("4 matching records")).not.toBeInTheDocument();
  });
  it("counts the displayed open vocabulary page and selected literals independently", async () => {
    picker({
      closed: false,
      withCounts: true,
      initial: {
        kind: "set_match",
        field: "traits",
        clause_id: "traits",
        op: "includes_any",
        values: ["retained"],
      },
    });
    open();
    await waitFor(() =>
      expect(api.discoverFilterCounts).toHaveBeenLastCalledWith(
        expect.objectContaining({
          clause_id: "traits",
          values: ["retained", "fire", "cold"],
        }),
        expect.any(AbortSignal),
      ),
    );
    fireEvent.click(screen.getByRole("button", { name: "More values" }));
    await waitFor(() =>
      expect(api.discoverFilterCounts).toHaveBeenLastCalledWith(
        expect.objectContaining({ values: ["retained", "second-page"] }),
        expect.any(AbortSignal),
      ),
    );
  });
  it("preserves an early focused and edited row when native opening completes", async () => {
    popup.deferred = true;
    picker();
    const trigger = screen.getByRole("button", { name: "Edit Traits filter" });
    trigger.focus();
    open();
    act(() => popup.mount?.());
    const row = screen.getByRole("button", {
      name: "Traits: Fire, neutral; activate to include",
    });
    row.focus();
    fireEvent.click(row);
    await waitFor(() =>
      expect(row).toHaveAccessibleName("Traits: Fire, included; activate to exclude"),
    );
    expect(row).toHaveStyle({
      color: "var(--filter-include-text)",
      background: "var(--filter-include-bg)",
    });
    expect(row).toHaveFocus();
    act(() => popup.complete?.(true));
    expect(row).toHaveFocus();
    expect(
      screen.getByRole("button", {
        name: "Traits: Fire, included; activate to exclude",
      }),
    ).toBe(row);
    fireEvent.keyDown(row, { key: "Escape" });
    open();
    act(() => popup.complete?.(true));
    expect(
      screen.getByRole("textbox", { name: "Search traits choices" }),
    ).toHaveFocus();
  });
  it("does not autofocus over a pointer interaction during native opening", () => {
    popup.deferred = true;
    picker();
    const trigger = screen.getByRole("button", { name: "Edit Traits filter" });
    trigger.focus();
    open();
    act(() => popup.mount?.());
    const row = screen.getByRole("button", {
      name: "Traits: Fire, neutral; activate to include",
    });
    fireEvent.pointerDown(row);
    act(() => popup.complete?.(true));
    // Pointer-down precedes the browser's focus/click actions. The completion
    // must leave that interaction in charge rather than moving focus itself.
    expect(trigger).toHaveFocus();
  });
  it("focuses the search after first portal readiness and reopening without stealing row or closing focus", async () => {
    popup.deferred = true;
    picker();
    const trigger = screen.getByRole("button", { name: "Edit Traits filter" });
    trigger.focus();
    open();
    expect(screen.queryByRole("textbox")).not.toBeInTheDocument();
    expect(trigger).toHaveFocus();
    act(() => popup.mount?.());
    const search = screen.getByRole("textbox", { name: "Search traits choices" });
    expect(trigger).toHaveFocus();
    act(() => popup.complete?.(true));
    await waitFor(() => expect(search).toHaveFocus());
    const row = screen.getByRole("button", {
      name: "Traits: Fire, neutral; activate to include",
    });
    row.focus();
    fireEvent.click(row);
    await waitFor(() =>
      expect(row).toHaveAccessibleName("Traits: Fire, included; activate to exclude"),
    );
    expect(row).toHaveFocus();
    fireEvent.keyDown(row, { key: "Escape" });
    expect(trigger).toHaveFocus();
    // A delayed opening completion after closing must not steal focus, even
    // while Ant retains the mounted input during the closing animation.
    act(() => popup.complete?.(true));
    expect(trigger).toHaveFocus();
    open();
    expect(trigger).toHaveFocus();
    act(() => popup.complete?.(true));
    expect(search).toHaveFocus();
    fireEvent.keyDown(search, { key: "Escape" });
    act(() => popup.complete?.(false));
    expect(trigger).toHaveFocus();
  });
  it("shows only active selections when closed and keeps searched catalog rows in order through edits", async () => {
    const change = picker();
    expect(screen.queryByText("fire")).not.toBeInTheDocument();
    expect(api.discoverFilterValues).not.toHaveBeenCalled();
    open();
    const menu = screen.getByRole("dialog", { name: "Traits choices" });
    const search = within(menu).getByRole("textbox", { name: "Search traits choices" });
    await waitFor(() => expect(search).toHaveFocus());
    fireEvent.change(search, { target: { value: "fi" } });
    expect(within(menu).queryByText("cold")).not.toBeInTheDocument();
    const row = within(menu).getByRole("button", {
      name: "Traits: Fire, neutral; activate to include",
    });
    row.focus();
    fireEvent.click(row);
    await waitFor(() =>
      expect(row).toHaveAccessibleName("Traits: Fire, included; activate to exclude"),
    );
    expect(row).toHaveFocus();
    expect(row).not.toHaveAttribute("aria-pressed");
    expect(row).toHaveTextContent(/^Fire$/);
    fireEvent.click(row);
    await waitFor(() =>
      expect(row).toHaveAccessibleName("Traits: Fire, excluded; activate to clear"),
    );
    expect(row).toHaveStyle({
      color: "var(--filter-exclude-text)",
      background: "var(--filter-exclude-bg)",
    });
    expect(change).toHaveBeenLastCalledWith(
      expect.objectContaining({
        kind: "set_match",
        op: "excludes_any",
        values: ["fire"],
      }),
    );
    fireEvent.change(search, { target: { value: "" } });
    await waitFor(() =>
      expect(
        within(menu)
          .getAllByRole("button")
          .map((button) => button.textContent),
      ).toEqual(["Fire", "Cold", "Magical"]),
    );
    fireEvent.keyDown(row, { key: "Escape" });
    await waitFor(() =>
      expect(
        screen.queryByRole("dialog", { name: "Traits choices" }),
      ).not.toBeInTheDocument(),
    );
    expect(screen.getByRole("button", { name: "Edit Traits filter" })).toHaveFocus();
    const remove = screen.getByRole("button", {
      name: "Remove excluded Fire from Traits",
    });
    expect(remove).toHaveTextContent("Fire");
    expect(remove).toHaveStyle({
      color: "var(--filter-exclude-text)",
      background: "var(--filter-exclude-bg)",
    });
    expect(remove.style.borderColor).toBe("var(--filter-exclude-border)");
    expect(screen.queryByText("cold")).not.toBeInTheDocument();
    fireEvent.click(remove);
    expect(change).toHaveBeenLastCalledWith(null);
  });
  it("retains an absent selected row after clearing it until the menu closes", async () => {
    picker({
      initial: {
        kind: "set_match",
        clause_id: "traits",
        field: "traits",
        op: "includes_any",
        values: ["absent-selected"],
      },
    });
    open();
    const row = screen.getByRole("button", {
      name: "Traits: Absent-selected, included; activate to exclude",
    });
    fireEvent.click(row);
    await waitFor(() =>
      expect(row).toHaveAccessibleName(
        "Traits: Absent-selected, excluded; activate to clear",
      ),
    );
    fireEvent.click(row);
    await waitFor(() =>
      expect(row).toHaveAccessibleName(
        "Traits: Absent-selected, neutral; activate to include",
      ),
    );
    expect(
      screen.queryByRole("button", { name: /Remove .*absent-selected/ }),
    ).not.toBeInTheDocument();
    fireEvent.keyDown(row, { key: "Escape" });
    open();
    expect(
      screen.queryByRole("button", { name: /Traits: absent-selected/ }),
    ).not.toBeInTheDocument();
  });
  it("uses bounded server search/page requests in the saved-list scope while retaining absent selections", async () => {
    picker({
      closed: false,
      scope: { kind: "saved_list", list_ref: "research" },
      initial: {
        kind: "set_match",
        clause_id: "traits",
        field: "traits",
        op: "includes_any",
        values: ["absent-selected"],
      },
    });
    expect(api.discoverFilterValues).not.toHaveBeenCalled();
    open();
    const menu = screen.getByRole("dialog", { name: "Traits choices" });
    await within(menu).findByRole("button", {
      name: "Traits: Fire, neutral; activate to include",
    });
    expect(screen.getByText("90 values in this list")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "More values" }));
    await within(menu).findByRole("button", {
      name: "Traits: Second-page, neutral; activate to include",
    });
    expect(api.discoverFilterValues).toHaveBeenLastCalledWith(
      expect.objectContaining({
        scope: { kind: "saved_list", list_ref: "research" },
        offset: 40,
        limit: 40,
        selected_values: ["absent-selected"],
      }),
      expect.any(AbortSignal),
    );
    expect(
      within(menu).getByRole("button", {
        name: "Traits: Absent-selected, included; activate to exclude",
      }),
    ).toBeInTheDocument();
    fireEvent.change(screen.getByRole("textbox", { name: "Search traits choices" }), {
      target: { value: "fir" },
    });
    await waitFor(() =>
      expect(api.discoverFilterValues).toHaveBeenLastCalledWith(
        expect.objectContaining({
          text: "fir",
          offset: 0,
          limit: 40,
          selected_values: ["absent-selected"],
        }),
        expect.any(AbortSignal),
      ),
    );
  });
  it("retains newly edited off-page rows in edit order through neutral and another include cycle", async () => {
    picker({ closed: false });
    open();
    const menu = screen.getByRole("dialog", { name: "Traits choices" });
    const cold = await within(menu).findByRole("button", {
      name: "Traits: Cold, neutral; activate to include",
    });
    fireEvent.click(cold);
    const fire = within(menu).getByRole("button", {
      name: "Traits: Fire, neutral; activate to include",
    });
    fireEvent.click(fire);
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "More values" })).toBeEnabled(),
    );
    fireEvent.click(screen.getByRole("button", { name: "More values" }));
    await within(menu).findByRole("button", {
      name: "Traits: Second-page, neutral; activate to include",
    });
    const retained = within(menu).getByRole("button", {
      name: "Traits: Cold, included; activate to exclude",
    });
    const names = () =>
      within(menu)
        .getAllByRole("button")
        .filter((button) => button.classList.contains("common-choice-row"))
        .map((button) => button.textContent);
    expect(names()).toEqual(["Second-page", "Cold", "Fire"]);
    retained.focus();
    fireEvent.click(retained);
    await waitFor(() =>
      expect(retained).toHaveAccessibleName(
        "Traits: Cold, excluded; activate to clear",
      ),
    );
    expect(names()).toEqual(["Second-page", "Cold", "Fire"]);
    expect(retained).toHaveFocus();
    fireEvent.click(retained);
    await waitFor(() =>
      expect(retained).toHaveAccessibleName(
        "Traits: Cold, neutral; activate to include",
      ),
    );
    expect(names()).toEqual(["Second-page", "Cold", "Fire"]);
    expect(retained).toBeInTheDocument();
    expect(retained).toHaveFocus();
    fireEvent.click(retained);
    await waitFor(() =>
      expect(retained).toHaveAccessibleName(
        "Traits: Cold, included; activate to exclude",
      ),
    );
    expect(names()).toEqual(["Second-page", "Cold", "Fire"]);
    expect(retained).toHaveFocus();
    // Neutral history belongs to this open session, not subsequent visits.
    fireEvent.click(retained);
    fireEvent.click(retained);
    await waitFor(() =>
      expect(retained).toHaveAccessibleName(
        "Traits: Cold, neutral; activate to include",
      ),
    );
    fireEvent.keyDown(retained, { key: "Escape" });
    open();
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "More values" })).toBeEnabled(),
    );
    fireEvent.click(screen.getByRole("button", { name: "More values" }));
    await screen.findByRole("button", {
      name: "Traits: Second-page, neutral; activate to include",
    });
    expect(
      screen.queryByRole("button", {
        name: "Traits: Cold, neutral; activate to include",
      }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", {
        name: "Traits: Fire, included; activate to exclude",
      }),
    ).toBeInTheDocument();
  });
  it("preserves mounted suggestion rows and focus while a selection refetch is pending", async () => {
    let finish: ((value: unknown) => void) | undefined;
    const initialResponse = api.discoverFilterValues.getMockImplementation()!;
    api.discoverFilterValues.mockImplementation((request) =>
      request.selected_values.length
        ? new Promise((resolve) => {
            finish = resolve;
          })
        : initialResponse(request),
    );
    picker({ closed: false });
    open();
    const row = await screen.findByRole("button", {
      name: "Traits: Fire, neutral; activate to include",
    });
    row.focus();
    fireEvent.click(row);
    await waitFor(() => expect(finish).toBeDefined());
    expect(row).toBeInTheDocument();
    expect(row).toHaveFocus();
    expect(
      screen.getByRole("button", {
        name: "Traits: Cold, neutral; activate to include",
      }),
    ).toBeInTheDocument();
    finish!({
      kind: "vocabulary",
      scope: { kind: "artifact" },
      values: {
        field: "traits",
        selected_values: ["fire"],
        total_values: 90n,
        options: [{ value: "fire" }, { value: "cold" }],
        basis: "product_key_scope_before_edition_preference",
      },
    });
    await waitFor(() =>
      expect(row).toHaveAccessibleName("Traits: Fire, included; activate to exclude"),
    );
    expect(row).toHaveFocus();
  });
  it("keeps the requested server page after the initial debounce settles", async () => {
    vi.useFakeTimers();
    try {
      picker({ closed: false });
      open();
      await act(async () => {
        await vi.advanceTimersByTimeAsync(1);
      });
      fireEvent.click(screen.getByRole("button", { name: "More values" }));
      await act(async () => {
        await vi.advanceTimersByTimeAsync(251);
      });
      expect(screen.getByRole("button", { name: "Previous values" })).toBeEnabled();
      expect(
        screen.getByRole("button", {
          name: "Traits: Second-page, neutral; activate to include",
        }),
      ).toBeInTheDocument();
    } finally {
      vi.useRealTimers();
    }
  });
  it("keeps selected criteria and offers a local retry when vocabulary fails", async () => {
    api.discoverFilterValues.mockRejectedValueOnce(
      new Error("Vocabulary service unavailable"),
    );
    const change = picker({
      closed: false,
      initial: {
        kind: "set_match",
        clause_id: "traits",
        field: "traits",
        op: "excludes_any",
        values: ["absent-selected"],
      },
    });
    open();
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Vocabulary service unavailable",
    );
    expect(
      screen.getByRole("button", {
        name: "Traits: Absent-selected, excluded; activate to clear",
      }),
    ).toBeInTheDocument();
    expect(change).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Retry values" }));
    expect(
      await screen.findByRole("button", {
        name: "Traits: Fire, neutral; activate to include",
      }),
    ).toBeInTheDocument();
    expect(change).not.toHaveBeenCalled();
  });
  it("uses a mobile Drawer and returns focus to the native trigger after dismissal", async () => {
    vi.mocked(Grid.useBreakpoint).mockReturnValue({ xs: true });
    picker();
    const trigger = screen.getByRole("button", { name: "Edit Traits filter" });
    trigger.focus();
    open();
    expect(await screen.findByRole("dialog", { name: "Traits" })).toBeInTheDocument();
    expect(
      screen.getByRole("textbox", { name: "Search traits choices" }),
    ).not.toHaveFocus();
    fireEvent.click(screen.getByRole("button", { name: "Close" }));
    await waitFor(() => expect(trigger).toHaveFocus());
  });
});
