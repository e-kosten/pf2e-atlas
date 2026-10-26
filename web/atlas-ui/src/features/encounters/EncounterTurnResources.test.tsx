import { ConfigProvider } from "antd";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { EncounterTurnResources } from "./EncounterTurnResources";
import type { TurnResourcesView } from "../../generated/atlas";
function value(): TurnResourcesView {
  return {
    action_calculation: {
      kind: "this_turn",
      count: {
        label: "Actions",
        base_value: 3n,
        adjusted_value: 1n,
        segments: [],
        adjustments: [{ source: "stunned", label: "Stunned 2", value: -2n }],
        suppressed_adjustments: [],
      },
      changes: [],
    },
    turn_key: "turn:a",
    round_number: 1n,
    regular_total: 1n,
    regular_remaining: 1n,
    extra_total: 0n,
    extra_remaining: 0n,
    extra_uses: [],
    reactions_total: 1n,
    reactions_remaining: 1n,
    start_results: [
      {
        condition_name: "Stunned",
        condition_ref: "conditionitems:dfCMdR4wnpbYNTix",
        outcome: "2 cleared",
      },
    ],
    pending_reason: null,
    forecast: {
      label: "Actions",
      base_value: 3n,
      adjusted_value: 3n,
      segments: [],
      adjustments: [],
      suppressed_adjustments: [],
    },
    can_edit_actions: true,
    can_spend_actions: { available: true },
    can_spend_reactions: { available: true },
  };
}
it("spends only on explicit input and retains the recorded allowance and start result across updates", async () => {
  const onChange = vi.fn().mockResolvedValue(undefined),
    initial = value();
  const { rerender } = render(
    <EncounterTurnResources
      value={initial}
      disabled={false}
      isCurrent
      onChange={onChange}
    />,
  );
  expect(onChange).not.toHaveBeenCalled();
  const availableAction = screen.getByRole("button", { name: "Spend action 1" });
  expect(availableAction).toHaveClass("ant-btn-color-primary", "ant-btn-variant-solid");
  fireEvent.click(availableAction);
  await waitFor(() =>
    expect(onChange).toHaveBeenCalledWith({ kind: "spend", resource: "action" }),
  );
  rerender(
    <EncounterTurnResources
      value={{ ...initial, regular_remaining: 0n }}
      disabled={false}
      isCurrent
      onChange={onChange}
    />,
  );
  const spentAction = screen.getByRole("button", { name: "Restore action 1" });
  expect(spentAction).toHaveAttribute("aria-pressed", "false");
  expect(spentAction).toHaveClass("ant-btn-color-default", "ant-btn-variant-outlined");
  expect(screen.queryByText("0 / 1")).toBeNull();
  expect(screen.getByText(initial.start_results[0].outcome)).toBeVisible();
  expect(onChange).toHaveBeenCalledTimes(1);
});
it("keeps an off-turn reaction usable and distinguishes spent stock from a condition restriction", async () => {
  const onChange = vi.fn().mockResolvedValue(undefined),
    initial = value();
  const { rerender } = render(
    <EncounterTurnResources
      value={initial}
      disabled={false}
      isCurrent={false}
      onChange={onChange}
    />,
  );
  const availableReaction = screen.getByRole("button", { name: "Spend reaction" });
  expect(availableReaction).toHaveClass(
    "ant-btn-color-primary",
    "ant-btn-variant-solid",
  );
  fireEvent.click(availableReaction);
  await waitFor(() =>
    expect(onChange).toHaveBeenCalledWith({ kind: "spend", resource: "reaction" }),
  );
  rerender(
    <EncounterTurnResources
      value={{
        ...initial,
        reactions_remaining: 0n,
        can_spend_reactions: { available: false, reason: "Stunned." },
      }}
      disabled={false}
      isCurrent={false}
      onChange={onChange}
    />,
  );
  const reaction = screen.getByRole("button", { name: "Restore reaction" });
  expect(reaction).toHaveAttribute("aria-pressed", "false");
  expect(reaction.querySelector('[data-atlas-action-glyph="reaction"]')).not.toBeNull();
  expect(reaction).toBeEnabled();
  expect(screen.getByText("Cannot react · Stunned")).toBeVisible();
});
it("shows mid-turn Quickened as a forecast rather than another current action", () => {
  const current = value();
  current.regular_total = 3n;
  current.regular_remaining = 2n;
  current.forecast.adjusted_value = 4n;
  render(
    <EncounterTurnResources
      value={current}
      disabled={false}
      isCurrent
      onChange={vi.fn()}
    />,
  );
  expect(screen.getByRole("button", { name: "Spend action 1" })).toHaveAttribute(
    "aria-pressed",
    "true",
  );
  expect(screen.getByRole("button", { name: "Spend action 2" })).toHaveAttribute(
    "aria-pressed",
    "true",
  );
  expect(screen.getByRole("button", { name: "Restore action 3" })).toHaveAttribute(
    "aria-pressed",
    "false",
  );
  expect(screen.getByText("Next turn")).toBeVisible();
  expect(screen.queryByRole("button", { name: "Spend extra action 1" })).toBeNull();
});

it("toggles the clicked slot only after canonical refresh and keeps its identity and focus", async () => {
  const initial = { ...value(), regular_total: 3n, regular_remaining: 3n };
  const onChange = vi.fn().mockResolvedValue(undefined);
  const view = (v: TurnResourcesView) => (
    <EncounterTurnResources value={v} disabled={false} isCurrent onChange={onChange} />
  );
  const { rerender } = render(view(initial));
  const middle = screen.getByRole("button", { name: "Spend action 2" });
  middle.focus();
  fireEvent.click(middle);
  expect(middle).toHaveAttribute("aria-pressed", "true");
  await waitFor(() => expect(onChange).toHaveBeenCalledTimes(1));
  rerender(view({ ...initial, regular_remaining: 2n }));
  expect(screen.getByRole("button", { name: "Restore action 2" })).toBe(middle);
  expect(middle).toHaveAttribute("aria-pressed", "false");
  expect(middle).toHaveFocus();
  expect(screen.getByRole("button", { name: "Spend action 3" })).toHaveAttribute(
    "aria-pressed",
    "true",
  );
  fireEvent.click(middle);
  await waitFor(() =>
    expect(onChange).toHaveBeenLastCalledWith({ kind: "restore", resource: "action" }),
  );
  rerender(view(initial));
  expect(screen.getByRole("button", { name: "Spend action 2" })).toBe(middle);
  expect(middle).toHaveFocus();
});
it("keeps unavailable actions honest and does not show a successful spend after failure", async () => {
  const initial = value();
  const onChange = vi.fn().mockRejectedValue(new Error("Connection lost. Retry."));
  const { rerender } = render(
    <EncounterTurnResources
      value={initial}
      disabled={false}
      isCurrent
      onChange={onChange}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Spend action 1" }));
  expect(await screen.findByRole("alert")).toHaveTextContent("Connection lost. Retry.");
  expect(screen.getByRole("button", { name: "Spend action 1" })).toHaveAttribute(
    "aria-pressed",
    "true",
  );
  rerender(
    <EncounterTurnResources
      value={{
        ...initial,
        can_spend_actions: { available: false, reason: "Stunned." },
      }}
      disabled={false}
      isCurrent
      onChange={onChange}
    />,
  );
  expect(screen.getByRole("button", { name: "Spend action 1" })).toBeDisabled();
});
it("explains a modified allowance without duplicating chip controls or confusing the next turn", async () => {
  const initial = value(),
    onChange = vi.fn().mockResolvedValue(undefined);
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <EncounterTurnResources
        value={initial}
        disabled={false}
        isCurrent
        onChange={onChange}
      />
    </ConfigProvider>,
  );
  const trigger = screen.getByRole("button", { name: "Actions: calculation details" });
  trigger.focus();
  fireEvent.click(trigger);
  expect(screen.queryByText(/^This turn:/)).toBeNull();
  await waitFor(() =>
    expect(screen.getByText("This turn", { exact: true })).toBeVisible(),
  );
  const scrollRegion = screen.getByRole("region", { name: "Action calculation" });
  expect(scrollRegion).toHaveAttribute("tabindex", "0");
  await waitFor(() => expect(scrollRegion).toHaveFocus());
  const calculation = screen.getByRole("region", { name: "Actions calculation" });
  await waitFor(() =>
    expect(within(calculation).getByRole("row", { name: "Total 1" })).toBeVisible(),
  );
  expect(screen.queryByRole("spinbutton")).toBeNull();
  expect(screen.queryByRole("button", { name: "Apply" })).toBeNull();
  expect(onChange).not.toHaveBeenCalled();
});

it("keeps extra action directly toggleable with its visible restriction and no separate menu", async () => {
  const initial = {
    ...value(),
    extra_total: 1n,
    extra_remaining: 1n,
    extra_uses: [
      {
        kind: "restricted" as const,
        actions: ["stride" as const, "strike" as const],
        guidance: null,
      },
    ],
  };
  const onChange = vi.fn().mockResolvedValue(undefined);
  const view = (v: TurnResourcesView) => (
    <ConfigProvider theme={{ token: { motion: false } }}>
      <EncounterTurnResources
        value={v}
        disabled={false}
        isCurrent
        onChange={onChange}
      />
    </ConfigProvider>
  );
  const { rerender } = render(view(initial));
  expect(screen.queryByText("Extra action", { exact: true })).toBeNull();
  expect(screen.getByText("Extra action: stride, strike")).toBeVisible();
  expect(screen.queryByRole("button", { name: /Extra action: details/ })).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Spend extra action 1" }));
  await waitFor(() =>
    expect(onChange).toHaveBeenCalledWith({ kind: "spend", resource: "extra_action" }),
  );
  rerender(view({ ...initial, extra_remaining: 0n }));
  fireEvent.click(screen.getByRole("button", { name: "Restore extra action 1" }));
  await waitFor(() =>
    expect(onChange).toHaveBeenLastCalledWith({
      kind: "restore",
      resource: "extra_action",
    }),
  );
  fireEvent.click(screen.getByRole("button", { name: "Actions: calculation details" }));
  await waitFor(() =>
    expect(screen.getByRole("region", { name: "Actions calculation" })).toBeVisible(),
  );
  expect(screen.queryByText(/^This turn:/)).toBeNull();
  expect(screen.queryByRole("spinbutton")).toBeNull();
  expect(screen.queryByText("Ordinary remaining")).toBeNull();
});

it.each(["typed", "wire"])(
  "keeps an unmodified three-action allowance plain while chips spend and restore (%s)",
  (representation) => {
    const initial = {
      ...value(),
      regular_total: 3n,
      regular_remaining: 2n,
      start_results: [],
      action_calculation: {
        kind: "this_turn" as const,
        count: value().forecast,
        changes: [],
      },
    };
    const supplied =
      representation === "wire"
        ? JSON.parse(
            JSON.stringify(initial, (_, item) =>
              typeof item === "bigint" ? Number(item) : item,
            ),
          )
        : initial;
    render(
      <EncounterTurnResources
        value={supplied}
        disabled={false}
        isCurrent
        onChange={vi.fn()}
      />,
    );
    expect(screen.getByText("Actions", { exact: true }).tagName).toBe("STRONG");
    expect(screen.queryByRole("button", { name: /Actions:/ })).toBeNull();
    expect(screen.getByRole("button", { name: "Spend action 1" })).toBeEnabled();
    expect(screen.getByRole("button", { name: "Restore action 3" })).toBeEnabled();
  },
);
it("keeps explanations when modifiers cancel to the base allowance", async () => {
  const initial = {
    ...value(),
    regular_total: 3n,
    regular_remaining: 3n,
    start_results: [],
  };
  initial.forecast.adjustments = [
    { source: "slowed", label: "Slowed", value: -1n },
    { source: "quickened", label: "Quickened", value: 1n },
  ];
  initial.action_calculation = {
    kind: "this_turn",
    count: initial.forecast,
    changes: [],
  };
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <EncounterTurnResources
        value={initial}
        disabled={false}
        isCurrent
        onChange={vi.fn()}
      />
    </ConfigProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: "Actions: calculation details" }));
  const calculation = await screen.findByRole("region", {
    name: "Actions calculation",
  });
  await waitFor(() =>
    expect(within(calculation).getByRole("row", { name: "Slowed -1" })).toBeVisible(),
  );
  expect(within(calculation).getByRole("row", { name: "Quickened +1" })).toBeVisible();
});

it("opens the retained Stunned rules without restoring a cleared condition or spending resources", () => {
  const onChange = vi.fn(),
    onReference = vi.fn();
  render(
    <EncounterTurnResources
      value={value()}
      disabled={false}
      isCurrent
      onChange={onChange}
      onReference={onReference}
    />,
  );
  const link = screen.getByRole("link", { name: "Stunned" });
  expect(link).toHaveAttribute("href", "/records/conditionitems:dfCMdR4wnpbYNTix");
  fireEvent.click(link);
  expect(onReference).toHaveBeenCalledWith("conditionitems:dfCMdR4wnpbYNTix", link);
  expect(onChange).not.toHaveBeenCalled();
  fireEvent.click(link, { ctrlKey: true });
  expect(onReference).toHaveBeenCalledTimes(1);
});

it("keeps this turn's overlap explanation while conditions change and switches to the forecast at turn end", async () => {
  const initial = value();
  initial.regular_total = 2n;
  initial.regular_remaining = 2n;
  initial.action_calculation = {
    kind: "this_turn",
    count: {
      ...initial.forecast,
      adjusted_value: 2n,
      adjustments: [{ source: "stunned", label: "Stunned 1", value: -1n }],
      suppressed_adjustments: [
        {
          source: "slowed",
          label: "Slowed 1",
          value: -1n,
          reason: "Overlaps Stunned; no additional action lost.",
        },
      ],
    },
    changes: [],
  };
  const view = (v: TurnResourcesView, current: boolean) => (
    <ConfigProvider theme={{ token: { motion: false } }}>
      <EncounterTurnResources
        value={v}
        disabled={false}
        isCurrent={current}
        onChange={vi.fn()}
      />
    </ConfigProvider>
  );
  const { rerender } = render(view(initial, true));
  fireEvent.click(screen.getByRole("button", { name: "Actions: calculation details" }));
  const table = await screen.findByRole("region", { name: "Actions calculation" });
  await waitFor(() =>
    expect(within(table).getByRole("row", { name: "Stunned 1 -1" })).toBeVisible(),
  );
  expect(
    within(table).getByText("Overlaps Stunned; no additional action lost."),
  ).toBeVisible();
  const changed = {
    ...initial,
    regular_remaining: 1n,
    forecast: {
      ...initial.forecast,
      adjusted_value: 1n,
      adjustments: [{ source: "slowed", label: "Slowed 2", value: -2n }],
    },
  };
  rerender(view(changed, true));
  await waitFor(() =>
    expect(screen.getByText("This turn", { exact: true })).toBeVisible(),
  );
  expect(within(table).getByRole("row", { name: "Total 2" })).toBeVisible();
  expect(within(table).queryByText("Slowed 2")).toBeNull();
  rerender(
    view(
      {
        ...changed,
        action_calculation: { kind: "next_turn", count: changed.forecast },
      },
      false,
    ),
  );
  await waitFor(() =>
    expect(within(table).getByRole("row", { name: "Total 1" })).toBeVisible(),
  );
  expect(within(table).getByRole("row", { name: "Slowed 2 -2" })).toBeVisible();
  expect(within(table).queryByText("Stunned 1")).toBeNull();
});

it("keeps later automatic losses separate from the initial allocation", async () => {
  const initial = value();
  initial.action_calculation = {
    kind: "this_turn",
    count: initial.forecast,
    changes: [{ source: "stunned-during-0", label: "Stunned 1", value: -1n }],
  };
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <EncounterTurnResources
        value={initial}
        disabled={false}
        isCurrent
        onChange={vi.fn()}
      />
    </ConfigProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: "Actions: calculation details" }));
  const start = await screen.findByRole("region", { name: "Actions calculation" });
  await waitFor(() =>
    expect(within(start).getByRole("row", { name: "Total 3" })).toBeVisible(),
  );
  const later = screen.getByRole("region", { name: "Automatic action changes" });
  expect(within(later).getByRole("row", { name: "Stunned 1 -1" })).toBeVisible();
});

it("does not reconstruct an older turn's missing calculation from today's conditions", async () => {
  const initial = value();
  initial.action_calculation = { kind: "this_turn", count: null, changes: [] };
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <EncounterTurnResources
        value={initial}
        disabled={false}
        isCurrent
        onChange={vi.fn()}
      />
    </ConfigProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: "Actions: calculation details" }));
  await waitFor(() =>
    expect(
      screen.getByText("Allocation details were not recorded for this turn."),
    ).toBeVisible(),
  );
  expect(screen.queryByRole("region", { name: "Actions calculation" })).toBeNull();
});
