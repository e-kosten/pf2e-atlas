import { render, screen } from "@testing-library/react";
import { FactLine } from "./presentationFacts";
import { DamageComponents } from "./ActorReference";
import { RecordPresentation } from "./RecordPresentation";
import {
  actorFixture,
  activityFixture,
  knownFact,
  numberFact,
  summaryFixture,
  unavailableFact,
} from "../../test/fixtures";
import type { PhysicalPriceView, RecordPresentationView } from "../../generated/atlas";

describe("reference fact reading policy", () => {
  it("omits optional baseline whitespace/lists and not-applicable while retaining required, zero, false and unavailable facts", () => {
    render(
      <>
        <FactLine label="Notes" fact={knownFact(" \n\t ")} />
        <FactLine label="Languages" fact={knownFact([])} />
        <FactLine label="Unused" fact={unavailableFact("not_applicable")} required />
        <FactLine label="Formula" fact={knownFact(" ")} required />
        <FactLine label="Weight" fact={numberFact(0)} />
        <FactLine label="Complex" fact={knownFact(false)} />
        <FactLine label="Missing" fact={unavailableFact("missing")} />
        <FactLine label="Null" fact={unavailableFact("null")} />
        <FactLine label="Invalid" fact={unavailableFact("invalid")} />
      </>,
    );
    for (const label of ["Notes", "Languages", "Unused"])
      expect(screen.queryByText(label)).toBeNull();
    expect(screen.getByText("Formula").parentElement).toHaveTextContent("Empty");
    expect(screen.getByText("Weight").parentElement).toHaveTextContent("0");
    expect(screen.getByText("Complex").parentElement).toHaveTextContent("false");
    expect(screen.getByLabelText("Missing is unavailable.")).toHaveTextContent("—");
    for (const state of ["null", "invalid"])
      expect(screen.getByText(`[${state}]`)).toBeVisible();
  });

  it("retains explicit whitespace, empty list and clear changes without inventing missing changes", () => {
    render(
      <>
        <FactLine label="Notes" fact={knownFact(" \t ")} patch />
        <FactLine label="Languages" fact={knownFact([])} patch />
        <FactLine label="Range" fact={unavailableFact("null")} patch />
        <FactLine label="Target" fact={unavailableFact("missing")} patch />
      </>,
    );
    expect(screen.getAllByText("Empty")).toHaveLength(2);
    expect(screen.getByText("Cleared")).toBeVisible();
    expect(screen.queryByText("Target")).toBeNull();
  });

  it("omits baseline damage collections, retains required formula zero/empty and nested empty patch qualifiers", () => {
    const component = {
      id: "authored",
      formula: knownFact("0"),
      damage_type: knownFact(" "),
      kinds: knownFact<string[]>([]),
      category: knownFact(" "),
      materials: knownFact<string[]>([]),
      apply_modifier: knownFact(false),
    };
    const { container, rerender } = render(<DamageComponents damage={knownFact([])} />);
    expect(container).toBeEmptyDOMElement();
    rerender(<DamageComponents damage={knownFact([component])} />);
    expect(container).toHaveTextContent("0 Empty");
    rerender(<DamageComponents damage={knownFact([component])} patch />);
    expect(container).toHaveTextContent("does not apply modifier");
    expect(container.textContent?.match(/Empty/g)).toHaveLength(4);
    rerender(<DamageComponents damage={knownFact([])} patch />);
    expect(container).toHaveTextContent("Damage Empty");
  });

  it("puts modifiers before skills and omits genuinely empty skill/ability headings", () => {
    const actor = actorFixture();
    const presentation: RecordPresentationView = {
      identity: summaryFixture("actors:test", "Test"),
      classification: "Creature 0",
      content: [],
      embedded: [],
      body: { kind: "creature", value: actor },
    };
    actor.abilities = knownFact([
      { key: "str", label: "Strength", modifier: numberFact(0) },
    ]);
    actor.skills = knownFact([
      {
        key: "athletics",
        label: "Athletics",
        modifier: numberFact(0),
        note: knownFact(" "),
        conditional: knownFact([]),
      },
    ]);
    const { rerender } = render(
      <RecordPresentation presentation={presentation} onReference={vi.fn()} />,
    );
    expect(
      screen
        .getByRole("heading", { name: "Ability modifiers" })
        .compareDocumentPosition(screen.getByRole("heading", { name: "Skills" })) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
    actor.skills = knownFact([]);
    actor.abilities = knownFact([]);
    rerender(<RecordPresentation presentation={presentation} onReference={vi.fn()} />);
    expect(screen.queryByRole("heading", { name: "Skills" })).toBeNull();
    expect(screen.queryByRole("heading", { name: "Ability modifiers" })).toBeNull();
  });

  it("uses the typed classification and puts the selected action glyph beside its title once without a raw key", () => {
    const activity = activityFixture("Seize");
    activity.action_cost = knownFact({ glyph: "reaction", label: "reaction" });
    const { container } = render(
      <RecordPresentation
        presentation={{
          identity: summaryFixture("actors:technical-key", "Seize"),
          classification: "Ability",
          content: [],
          embedded: [],
          body: { kind: "activity", value: activity },
        }}
        onReference={vi.fn()}
      />,
    );
    expect(screen.getByRole("heading", { name: "Seize reaction" })).toHaveTextContent(
      "Seize",
    );
    expect(
      container.querySelectorAll('[data-atlas-action-glyph="reaction"]'),
    ).toHaveLength(1);
    expect(
      screen.getByText("Ability", { selector: ".record-reference__kind" }),
    ).toBeVisible();
    expect(screen.queryByText("actors:technical-key")).toBeNull();
  });

  it("retains price constituent null/invalid and known zero without inventing free prices", () => {
    const absent = { ...unavailableFact<number>("missing"), adjustment: null };
    const price: PhysicalPriceView = {
      pp: absent,
      gp: numberFact(0),
      sp: { ...unavailableFact<number>("null"), adjustment: null },
      cp: { ...unavailableFact<number>("invalid"), adjustment: null },
    };
    const presentation: RecordPresentationView = {
      identity: summaryFixture("items:coin", "Coin"),
      classification: "Equipment 0",
      content: [],
      embedded: [],
      body: {
        kind: "physical_reference",
        value: {
          base_type: unavailableFact("missing"),
          details: null,
          quantity: numberFact(1),
          material: unavailableFact("not_applicable"),
          size: unavailableFact("not_applicable"),
          usage: knownFact(" "),
          bulk: numberFact(0),
          price: knownFact(price),
          consumable: null,
          price_per: absent,
        },
      },
    };
    const { rerender } = render(
      <RecordPresentation presentation={presentation} onReference={vi.fn()} />,
    );
    expect(screen.getByText("Price").parentElement).toHaveTextContent(
      "0 gp, [null] sp, [invalid] cp",
    );
    expect(screen.queryByText("Usage")).toBeNull();
    if (presentation.body.kind !== "physical_reference") throw new Error("fixture");
    presentation.body.value.price = knownFact({
      pp: absent,
      gp: absent,
      sp: absent,
      cp: absent,
    });
    rerender(<RecordPresentation presentation={presentation} onReference={vi.fn()} />);
    expect(screen.getByText("Price").parentElement).toHaveTextContent("Not listed");
    presentation.body.value.price = unavailableFact("invalid");
    rerender(<RecordPresentation presentation={presentation} onReference={vi.fn()} />);
    expect(screen.getByText("Price").parentElement).toHaveTextContent("[invalid]");
  });

  it("distinguishes empty table results from unavailable collections and retains required empty table facts", () => {
    const presentation: RecordPresentationView = {
      identity: summaryFixture("tables:test", "Table"),
      classification: "RollTable",
      content: [],
      embedded: [],
      body: {
        kind: "roll_table",
        value: {
          roll_note: "Reference rolls may repeat; results are not consumed.",
          formula: knownFact(" "),
          results: knownFact([]),
        },
      },
    };
    const { rerender } = render(
      <RecordPresentation presentation={presentation} onReference={vi.fn()} />,
    );
    expect(screen.getByText("Formula").parentElement).toHaveTextContent("Empty");
    expect(screen.queryByText("Results")).toBeNull();
    if (presentation.body.kind !== "roll_table") throw new Error("fixture");
    presentation.body.value.results = unavailableFact("missing");
    rerender(<RecordPresentation presentation={presentation} onReference={vi.fn()} />);
    expect(screen.getByText("Results").parentElement).toHaveTextContent("—");
    presentation.body = {
      kind: "table_result",
      value: {
        range: knownFact(""),
        weight: numberFact(0),
        result_type: knownFact(" "),
      },
    };
    rerender(<RecordPresentation presentation={presentation} onReference={vi.fn()} />);
    expect(screen.getByText("Range").parentElement).toHaveTextContent("Empty");
    expect(screen.getByText("Result type").parentElement).toHaveTextContent("Empty");
    expect(screen.getByText("Weight").parentElement).toHaveTextContent("0");
  });
});
