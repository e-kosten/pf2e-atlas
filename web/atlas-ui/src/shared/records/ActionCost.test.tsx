import { render, screen } from "@testing-library/react";
import { ActionCost } from "./ActionCost";
import { ActorActivity } from "./ActorReference";
import { SpellChanges, SpellReference } from "./SpellReference";
import { RecordPresentation } from "./RecordPresentation";
import {
  activityFixture,
  detailFixture,
  knownFact,
  spellFixture,
  unavailableFact,
} from "../../test/fixtures";
import type { ActionGlyphView, SpellChangeView } from "../../generated/atlas";

describe("authored action costs", () => {
  it.each<[ActionGlyphView, string]>([
    ["one_action", "1 action"],
    ["two_actions", "2 actions"],
    ["three_actions", "3 actions"],
    ["reaction", "reaction"],
    ["free_action", "free action"],
  ])("renders %s as decoration with one readable label", (glyph, label) => {
    const { container } = render(<ActionCost fact={knownFact({ glyph, label })} />);
    expect(screen.getByText(label)).toHaveClass("sr-only");
    const mark = container.querySelector("[data-atlas-action-glyph]");
    expect(mark).toHaveAttribute("data-atlas-action-glyph", glyph);
    expect(mark).toHaveAttribute("aria-hidden", "true");
    expect(mark).toBeEmptyDOMElement();
    expect(container.textContent).toBe(label);
  });

  it.each(["1 to 3 actions", "1 minute", "passive"])(
    "keeps %s as readable supplied text",
    (label) => {
      const { container } = render(
        <ActionCost fact={knownFact({ glyph: null, label })} />,
      );
      expect(screen.getByText(label)).toBeVisible();
      expect(screen.getByText(label)).not.toHaveClass("sr-only");
      expect(container.querySelector("[data-atlas-action-glyph]")).toBeNull();
    },
  );

  it.each([
    ["missing", "Action cost not listed"],
    ["null", "Action cost not specified"],
    ["invalid", "Action cost unavailable"],
  ] as const)("keeps %s required cost explicit", (state, label) => {
    render(<ActionCost fact={unavailableFact(state)} />);
    expect(screen.getByText(label)).toBeVisible();
  });

  it("preserves clear, empty and absent patch meanings", () => {
    const { rerender, container } = render(
      <ActionCost fact={unavailableFact("null")} label="Cast" patch />,
    );
    expect(screen.getByText("Cleared")).toBeVisible();
    rerender(
      <ActionCost fact={knownFact({ glyph: null, label: "" })} label="Cast" patch />,
    );
    expect(screen.getByText("Empty")).toBeVisible();
    rerender(<ActionCost fact={unavailableFact("missing")} label="Cast" patch />);
    expect(container).toBeEmptyDOMElement();
    rerender(<ActionCost fact={unavailableFact("not_applicable")} />);
    expect(container).toBeEmptyDOMElement();
    rerender(<ActionCost fact={knownFact({ glyph: null, label: "" })} label="Cast" />);
    expect(container).toBeEmptyDOMElement();
  });

  it("renders spell casting cost once in its casting group", () => {
    const { container } = render(<SpellReference spell={spellFixture()} />);
    expect(
      container.querySelectorAll('[data-atlas-action-glyph="two_actions"]'),
    ).toHaveLength(1);
    expect(screen.getByText("2 actions")).toHaveClass("sr-only");
    expect(screen.queryByText("[object Object]")).toBeNull();
  });

  it("uses typed costs for a selected activity and an owned link", () => {
    const activity = activityFixture();
    const { container, rerender } = render(
      <ActorActivity activity={activity} onReference={vi.fn()} selected />,
    );
    expect(
      container.querySelector('[data-atlas-action-glyph="one_action"]'),
    ).not.toBeNull();
    expect(screen.getByText("1 action")).toHaveClass("sr-only");
    const presentation = detailFixture("actors:party", "Party").presentation;
    presentation.owned = [
      {
        navigation: activity.navigation,
        title: "Ready",
        family: "action",
        usage: null,
        action_cost: knownFact({ glyph: "reaction", label: "reaction" }),
      },
    ];
    rerender(<RecordPresentation presentation={presentation} onReference={vi.fn()} />);
    expect(
      container.querySelector('[data-atlas-action-glyph="reaction"]'),
    ).not.toBeNull();
    expect(screen.getByText("reaction")).toHaveClass("sr-only");
  });

  it("shows a typed glyph in an authored spell patch without losing explicit clears", () => {
    const changes: SpellChangeView = {
      cast: knownFact({ glyph: "free_action", label: "free action" }),
      range: unavailableFact("null"),
      target: unavailableFact("missing"),
      area: unavailableFact("missing"),
      defense: unavailableFact("missing"),
      duration: unavailableFact("missing"),
      sustained: unavailableFact("missing"),
      traits: unavailableFact("missing"),
      traditions: unavailableFact("missing"),
      damage: unavailableFact("missing"),
      heightening: unavailableFact("missing"),
    };
    const { container } = render(<SpellChanges changes={changes} />);
    expect(
      container.querySelector('[data-atlas-action-glyph="free_action"]'),
    ).not.toBeNull();
    expect(screen.getByText("Cleared")).toBeVisible();
  });
});
