import { fireEvent, render, screen, within } from "@testing-library/react";
import { RecordPresentation } from "./RecordPresentation";
import { DamageComponents } from "./ActorReference";
import { SpellChanges, SpellReference } from "./SpellReference";
import {
  actorFixture,
  activityFixture,
  detailFixture,
  knownFact,
  numberFact,
  spellFixture,
  summaryFixture,
  unavailableFact,
} from "../../test/fixtures";
import type { RecordPresentationView, SpellChangeView } from "../../generated/atlas";

function actorPresentation(): RecordPresentationView {
  const actor = actorFixture();
  actor.resistances = knownFact([
    {
      damage_type: knownFact("all damage"),
      magnitude: numberFact(10),
      exceptions: knownFact(["force", "ghost touch", "vitality"]),
      double_against: knownFact(["non-magical attacks"]),
    },
  ]);
  actor.movement = knownFact([
    { movement_type: knownFact("fly"), feet: numberFact(30) },
  ]);
  actor.activities = knownFact([activityFixture()]);
  return {
    classification: "Creature 1",
    identity: {
      ...summaryFixture("actors:ghoul", "Ghoul"),
      kind: "creature",
      kind_label: "Creature",
      rarity: "uncommon",
      publication: "Monster Core",
      level_label: "1",
    },
    body: { kind: "creature", value: actor },
    content: [],
    owned: [],
  };
}

describe("semantic family presentation", () => {
  it.each([
    { compact: false, encounter: false },
    { compact: true, encounter: false },
    { compact: false, encounter: true },
  ])("reuses identity, qualified defenses and units across contexts %j", (context) => {
    const { container } = render(
      <RecordPresentation
        presentation={actorPresentation()}
        onReference={vi.fn()}
        {...context}
      />,
    );
    expect(screen.getByRole("heading", { name: "Ghoul" })).toBeVisible();
    expect(screen.getByText("Monster Core")).toBeVisible();
    expect(screen.getByText("Uncommon")).toBeVisible();
    expect(screen.getByText(/except force, ghost touch, vitality/)).toBeVisible();
    expect(screen.getByText(/double against non-magical attacks/)).toBeVisible();
    expect(screen.getByText("+0")).toBeVisible();
    expect(screen.getByText("25 feet")).toBeVisible();
    expect(screen.getByText("30 feet")).toBeVisible();
    expect(container.querySelectorAll(".actor-reference__defenses")).toHaveLength(1);
    expect(screen.getByRole("heading", { name: "Strikes" })).toBeVisible();
    expect(screen.queryByText("HP") !== null).toBe(!context.encounter);
  });

  it("preserves exact selected identity and owner navigation", () => {
    const onReference = vi.fn();
    const presentation = {
      ...detailFixture("actors:ghoul").presentation,
      identity: summaryFixture("actors:ghoul", "Ghoul Fever"),
    };
    const selected = activityFixture().navigation;
    render(
      <RecordPresentation
        presentation={presentation}
        root={summaryFixture("actors:ghoul", "Ghoul")}
        selection={selected}
        onReference={onReference}
      />,
    );
    expect(screen.getByRole("heading", { name: "Ghoul Fever" })).toBeVisible();
    const parent = screen.getByRole("link", { name: "Return to Ghoul" });
    const destination = new URL(parent.getAttribute("href")!, window.location.origin);
    expect(destination.pathname).toBe("/records/actors:ghoul");
    expect(JSON.parse(destination.searchParams.get("landing")!)).toEqual(selected);
    expect(destination.searchParams.has("selection")).toBe(false);
    expect(onReference).not.toHaveBeenCalled();
  });

  it("offers the explicit original record separately from the customized embedded copy", () => {
    const onReference = vi.fn();
    const detail = detailFixture(
      "actors:caster",
      "Customized staff",
      "equipment:catalog-staff",
    );
    const selected = activityFixture("Customized staff", "actors:caster").navigation;
    const relationship = {
      ...detail.relationships[0],
      origin: "structured",
      kind: "CompendiumProvenance",
      status: "resolved_record",
    };
    const view = (available = true) => (
      <RecordPresentation
        presentation={detail.presentation}
        root={summaryFixture("actors:caster", "Caster")}
        selection={selected}
        relationships={[
          available
            ? relationship
            : { ...relationship, status: "unresolved", target: null },
        ]}
        onReference={onReference}
      />
    );
    const { rerender } = render(view());
    expect(screen.getByRole("heading", { name: "Customized staff" })).toBeVisible();
    const original = screen.getByRole("link", { name: "View original" });
    const destination = new URL(original.getAttribute("href")!, window.location.origin);
    expect(destination.pathname).toBe("/records/equipment:catalog-staff");
    expect(JSON.parse(destination.searchParams.get("selection")!)).toEqual(
      relationship.target,
    );
    fireEvent.click(original);
    expect(window.location.pathname).toBe("/records/equipment:catalog-staff");
    onReference.mockClear();
    fireEvent.click(original, { ctrlKey: true });
    expect(onReference).not.toHaveBeenCalled();
    rerender(view(false));
    expect(screen.queryByRole("link", { name: "View original" })).toBeNull();
  });

  it("shows selected activity traits once in its identity and retains traits on owned activities", () => {
    const disease = { kind: "trait", value: "disease", label: "Disease" };
    const activity = { ...activityFixture("Ghoul Fever"), traits: [disease] };
    const presentation = {
      ...actorPresentation(),
      identity: { ...summaryFixture("actors:ghoul", "Ghoul Fever"), traits: [disease] },
      body: { kind: "activity" as const, value: activity },
    };
    const { rerender } = render(
      <RecordPresentation presentation={presentation} onReference={vi.fn()} />,
    );
    expect(screen.getAllByText("Disease")).toHaveLength(1);
    expect(screen.queryByRole("link", { name: "Ghoul Fever" })).toBeNull();
    const parent = actorPresentation();
    if (parent.body.kind !== "creature") throw new Error("fixture family");
    parent.body.value.activities = knownFact([activity]);
    rerender(<RecordPresentation presentation={parent} onReference={vi.fn()} />);
    expect(screen.getAllByText("Disease")).toHaveLength(1);
    expect(screen.getByRole("link", { name: "Ghoul Fever" })).toBeVisible();
  });

  it("groups owned spells by their exact casting entry and retains authored DC and attack", () => {
    const presentation = actorPresentation();
    if (presentation.body.kind !== "creature") throw new Error("fixture family");
    const entry = {
      ...activityFixture("Arcane Prepared"),
      kind: "casting_entry" as const,
      attack: numberFact(23),
      difficulty_class: numberFact(33),
      casting_tradition: knownFact("arcane"),
      preparation: knownFact("prepared"),
    };
    const spell = {
      ...activityFixture("Customized Fireball"),
      kind: "spell" as const,
      casting_entry: entry.navigation,
      association: knownFact("Matched authored entry"),
      navigation: {
        ...entry.navigation,
        owners: [{ collection: "items", identity: { SnapshotLocal: { index: 1 } } }],
      },
    };
    presentation.body.value.activities = knownFact([entry, spell]);
    const callback = vi.fn();
    render(<RecordPresentation presentation={presentation} onReference={callback} />);
    const group = screen.getByRole("heading", { name: "Spellcasting" }).parentElement!;
    expect(within(group).getByText("33")).toBeVisible();
    expect(within(group).getByText("+23")).toBeVisible();
    expect(within(group).getByText("prepared")).toBeVisible();
    expect(group.querySelector(".actor-casting__spells")).toHaveTextContent(
      "Customized Fireball",
    );
    fireEvent.click(within(group).getByRole("link", { name: "Customized Fireball" }));
    expect(callback).toHaveBeenCalledWith(
      "actors:ghoul",
      expect.any(HTMLElement),
      spell.navigation,
    );
  });

  it.each([false, true])(
    "orders lowercase hazard content roles without duplication (compact=%s)",
    (compact) => {
      const actor = actorFixture();
      actor.activities = knownFact([activityFixture()]);
      const fields = ["description", "stealth", "disable", "routine", "reset"].map(
        (role) => ({
          locator: { record: "hazards:pit", owners: [], field: role },
          role,
          source_fingerprint: null,
          body: { kind: "plain" as const, text: `${role} instructions` },
        }),
      );
      render(
        <RecordPresentation
          presentation={{
            classification: "Reference",
            identity: summaryFixture("hazards:pit", "Hidden Spiked Pit"),
            content: fields,
            owned: [],
            body: {
              kind: "hazard",
              value: {
                actor,
                defense_block: knownFact(true),
                show_saves: true,
                complex: knownFact(false),
                stealth: numberFact(0),
                hardness: numberFact(0),
              },
            },
          }}
          onReference={vi.fn()}
          compact={compact}
        />,
      );
      expect(screen.getByText("Simple")).toBeVisible();
      expect(
        within(screen.getByText("Stealth").parentElement!).getByText("+0"),
      ).toBeVisible();
      expect(screen.queryByText("Perception")).toBeNull();
      expect(screen.queryByText("Languages")).toBeNull();
      expect(screen.getAllByText("stealth instructions")).toHaveLength(1);
      expect(screen.getByRole("heading", { name: "Detection" })).toBeVisible();
      for (const role of ["description", "disable", "routine", "reset"])
        expect(screen.getAllByText(`${role} instructions`)).toHaveLength(1);
      if (compact) return;
      const description = screen.getByText("description instructions");
      const detection = screen.getByText("Stealth");
      expect(
        description.compareDocumentPosition(detection) &
          Node.DOCUMENT_POSITION_FOLLOWING,
      ).toBeTruthy();
      const disable = screen.getByText("disable instructions");
      const hardness = screen.getByText("Hardness");
      const routine = screen.getByText("routine instructions");
      expect(
        disable.compareDocumentPosition(hardness) & Node.DOCUMENT_POSITION_FOLLOWING,
      ).toBeTruthy();
      expect(
        screen
          .getByRole("heading", { name: "Strikes" })
          .compareDocumentPosition(routine) & Node.DOCUMENT_POSITION_FOLLOWING,
      ).toBeTruthy();
      expect(screen.getByRole("heading", { name: "Reset" })).toBeVisible();
    },
  );

  it.each(["invalid", "missing", "null"] as const)(
    "retains required IWR magnitude (%s), explicit null/invalid qualifiers and known zero",
    (state) => {
      const presentation = actorPresentation();
      if (presentation.body.kind !== "creature") throw new Error("fixture family");
      presentation.body.value.resistances = knownFact([
        {
          damage_type: knownFact("fire"),
          magnitude: { state, value: null, adjustment: null },
          exceptions: unavailableFact(state),
          double_against: unavailableFact(state),
        },
        {
          damage_type: knownFact("cold"),
          magnitude: numberFact(0),
          exceptions: knownFact([]),
          double_against: unavailableFact("not_applicable"),
        },
      ]);
      render(<RecordPresentation presentation={presentation} onReference={vi.fn()} />);
      const row = screen.getByText("Resistances").parentElement!;
      expect(row).toHaveTextContent(
        state === "missing"
          ? "fire [missing]; cold 0"
          : `fire [${state}] (exceptions [${state}]) (double against [${state}]); cold 0`,
      );
      expect(row).not.toHaveTextContent("not_applicable");
    },
  );

  it("retains unavailable damage qualifiers and authored clears without fabricating patch fields", () => {
    const damage = knownFact([
      {
        id: "0",
        formula: knownFact("1d6"),
        damage_type: knownFact("fire"),
        kinds: unavailableFact<string[]>("invalid"),
        category: unavailableFact<string>("null"),
        materials: unavailableFact<string[]>("missing"),
        apply_modifier: knownFact(false),
      },
    ]);
    const { container, rerender } = render(<DamageComponents damage={damage} />);
    expect(container).toHaveTextContent(
      "1d6 fire; kinds [invalid]; materials [missing]",
    );
    rerender(<DamageComponents damage={damage} patch />);
    expect(container).toHaveTextContent("kinds [invalid]; category cleared");
    expect(container).not.toHaveTextContent("materials");
  });

  it.each(["invalid", "null"] as const)(
    "retains unavailable enclosing spell and defense qualifier states (%s)",
    (state) => {
      const spell = spellFixture();
      spell.heightening = unavailableFact(state);
      spell.forms = unavailableFact(state);
      spell.defense = knownFact({
        statistic: knownFact("Reflex"),
        basic: unavailableFact(state),
        passive: unavailableFact(state),
      });
      render(
        <RecordPresentation
          presentation={{
            classification: "Reference",
            identity: summaryFixture("spells:test", "Spell"),
            content: [],
            owned: [],
            body: { kind: "spell", value: spell },
          }}
          onReference={vi.fn()}
        />,
      );
      expect(screen.getByText("Heightening").parentElement).toHaveTextContent(
        `[${state}]`,
      );
      expect(screen.getByText("Spell forms").parentElement).toHaveTextContent(
        `[${state}]`,
      );
      expect(screen.getByText("Defense").parentElement).toHaveTextContent(
        `Reflex; basic save [${state}]; passive defense [${state}]`,
      );
      expect(screen.queryByText("Basic Reflex")).toBeNull();
    },
  );

  it("renders authored interval heightening and partial form changes without calculating or fabricating missing fields", () => {
    const spell = spellFixture();
    const missing = <T,>() => unavailableFact<T>();
    const changes: SpellChangeView = {
      cast: missing(),
      range: unavailableFact("null"),
      target: missing(),
      area: missing(),
      defense: missing(),
      duration: missing(),
      sustained: missing(),
      traits: missing(),
      traditions: missing(),
      damage: missing(),
      heightening: knownFact({
        kind: missing(),
        interval: { state: "missing", value: null, adjustment: null },
        area: { state: "missing", value: null, adjustment: null },
        damage: knownFact([{ id: "0", formula: knownFact("1d8+8") }]),
        levels: missing(),
      }),
    };
    spell.heightening = knownFact({
      kind: "interval",
      interval: numberFact(1),
      area: { state: "not_applicable", value: null, adjustment: null },
      damage: knownFact([{ id: "0", formula: knownFact("1d8") }]),
    });
    spell.forms = knownFact([
      {
        id: "living",
        name: knownFact("Heal the Living"),
        sort: numberFact(0),
        changes: knownFact(changes),
      },
    ]);
    render(
      <RecordPresentation
        presentation={{
          classification: "Reference",
          identity: summaryFixture("spells:heal", "Heal"),
          content: [],
          owned: [],
          body: { kind: "spell", value: spell },
        }}
        onReference={vi.fn()}
      />,
    );
    expect(screen.getByText("Basic Reflex")).toBeVisible();
    expect(screen.getByText("20 feet burst")).toBeVisible();
    expect(screen.getByText("2 actions")).toBeVisible();
    expect(screen.queryByText("None")).toBeNull();
    const form = screen.getByRole("heading", {
      name: "Heal the Living",
    }).parentElement!;
    expect(within(form).getByText("1d8+8")).toBeVisible();
    expect(within(form).getByText("Cleared")).toBeVisible();
    expect(within(form).queryByText("[missing]")).toBeNull();
    expect(screen.queryByText("Cost")).toBeNull();
    expect(screen.queryByText("Requirements")).toBeNull();
  });

  it("distinguishes unavailable actor collections from known empty without substituting zero", () => {
    const presentation = actorPresentation();
    if (presentation.body.kind !== "creature") throw new Error("fixture family");
    presentation.body.value.armor_class = {
      state: "invalid",
      value: null,
      adjustment: null,
    };
    presentation.body.value.activities = unavailableFact("invalid");
    render(<RecordPresentation presentation={presentation} onReference={vi.fn()} />);
    expect(screen.getAllByText("[invalid]")).toHaveLength(2);
    expect(screen.queryByRole("heading", { name: "Strikes" })).toBeNull();
  });
  it("retains physical and table reference facts and exact owned navigation", () => {
    const callback = vi.fn();
    const table = detailFixture("tables:wonder", "Rod of Wonder").presentation;
    const child = activityFixture("Result 1–10", "tables:wonder").navigation;
    const { rerender } = render(
      <RecordPresentation
        presentation={{
          ...table,
          body: {
            kind: "roll_table",
            value: {
              formula: knownFact("1d100"),
              results: knownFact([
                {
                  range: knownFact([1, 10]),
                  link: {
                    title: "Result 1–10",
                    family: "TableResult",
                    usage: "range 1–10",
                    action_cost: unavailableFact("not_applicable"),
                    navigation: child,
                  },
                },
              ]),
            },
          },
        }}
        onReference={callback}
      />,
    );
    expect(screen.getByText("1d100")).toBeVisible();
    fireEvent.click(screen.getByRole("link", { name: "Result 1–10" }));
    expect(callback).toHaveBeenCalledWith(
      "tables:wonder",
      expect.any(HTMLElement),
      child,
    );
    rerender(
      <RecordPresentation
        presentation={{
          ...table,
          body: {
            kind: "table_result",
            value: {
              range: knownFact("1–10"),
              weight: numberFact(0),
              result_type: knownFact("Text"),
            },
          },
        }}
        onReference={callback}
      />,
    );
    expect(screen.getByText("1–10")).toBeVisible();
    expect(screen.getByText("0")).toBeVisible();
    rerender(
      <RecordPresentation
        presentation={{
          ...table,
          body: {
            kind: "physical_reference",
            value: {
              usage: knownFact("held in one hand"),
              bulk: numberFact(0.1),
              price: knownFact({
                pp: { ...unavailableFact<number>("missing"), adjustment: null },
                gp: numberFact(10),
                sp: { ...unavailableFact<number>("missing"), adjustment: null },
                cp: { ...unavailableFact<number>("missing"), adjustment: null },
              }),
              price_per: numberFact(1),
            },
          },
        }}
        onReference={callback}
      />,
    );
    expect(screen.getByText("held in one hand")).toBeVisible();
    expect(screen.getByText("0.1")).toBeVisible();
    expect(screen.getByText("10 gp")).toBeVisible();
  });
  it("shows fixed-rank authored changes as reference data", () => {
    const spell = spellFixture();
    const missing = <T,>() => unavailableFact<T>();
    const changes: SpellChangeView = {
      cast: missing(),
      range: knownFact("60 feet"),
      target: missing(),
      area: missing(),
      defense: knownFact({
        statistic: missing(),
        basic: knownFact(false),
        passive: missing(),
      }),
      duration: missing(),
      sustained: missing(),
      traits: missing(),
      traditions: missing(),
      damage: missing(),
      heightening: missing(),
    };
    spell.heightening = knownFact({
      kind: "fixed",
      levels: knownFact([{ rank: 5, changes: knownFact(changes) }]),
    });
    render(
      <RecordPresentation
        presentation={{
          classification: "Reference",
          identity: summaryFixture("spells:mantis", "Mantis's Grasp"),
          content: [],
          owned: [],
          body: { kind: "spell", value: spell },
        }}
        onReference={vi.fn()}
      />,
    );
    const change = screen.getByRole("heading", {
      name: "Heightened (rank 5)",
    }).parentElement!;
    expect(within(change).getByText("60 feet")).toBeVisible();
    expect(within(change).getByText("No")).toBeVisible();
    expect(within(change).queryByText("[missing]")).toBeNull();
    expect(screen.getByText("500 feet")).toBeVisible();
    expect(screen.queryByRole("combobox")).toBeNull();
  });
});

describe("Rust-owned optional spell presentation", () => {
  it("omits only facts the projection explicitly marks absent", () => {
    const spell = {
      ...spellFixture(),
      requirements: null,
      cost: null,
      casting_entry: null,
    };
    render(
      <RecordPresentation
        presentation={{
          classification: "Reference",
          identity: summaryFixture("spell:test", "Test spell"),
          body: { kind: "spell", value: spell },
          content: [],
          owned: [],
        }}
        onReference={vi.fn()}
      />,
    );
    expect(screen.queryByText("Requirements")).toBeNull();
    expect(screen.queryByText("Cost")).toBeNull();
    expect(screen.queryByText("Casting entry")).toBeNull();
    expect(screen.getByText("Cast")).toBeVisible();
  });

  it("retains optional facts when their parent is unavailable and retains known zero", () => {
    const spell = {
      ...spellFixture(),
      requirements: unavailableFact<string>("invalid"),
      cost: knownFact("0"),
      casting_entry: unavailableFact<string>("missing"),
    };
    render(
      <RecordPresentation
        presentation={{
          classification: "Reference",
          identity: summaryFixture("spell:test", "Test spell"),
          body: { kind: "spell", value: spell },
          content: [],
          owned: [],
        }}
        onReference={vi.fn()}
      />,
    );
    const requirements = screen.getByText("Requirements").parentElement!;
    expect(within(requirements).getByText("[invalid]")).toBeVisible();
    expect(screen.queryByText("Casting entry")).toBeNull();
    expect(spell.casting_entry).toEqual(unavailableFact<string>("missing"));
    const cost = screen.getByText("Cost").parentElement!;
    expect(within(cost).getByText("0")).toBeVisible();
  });
});

describe("optional authored reference facts", () => {
  it("omits an explicitly absent baseline area while preserving a patch clear", () => {
    const spell = { ...spellFixture(), area: null };
    const { rerender } = render(<SpellReference spell={spell} />);
    expect(screen.queryByText("Area")).toBeNull();
    const absent = <T,>() => unavailableFact<T>("missing");
    rerender(
      <SpellChanges
        changes={{
          cast: absent(),
          range: absent(),
          target: absent(),
          area: unavailableFact("null"),
          defense: absent(),
          duration: absent(),
          sustained: absent(),
          traits: absent(),
          traditions: absent(),
          damage: absent(),
          heightening: absent(),
        }}
      />,
    );
    expect(screen.getByText("Area").parentElement).toHaveTextContent("Cleared");
  });

  it.each(["missing", "null", "invalid"] as const)(
    "retains a supplied unavailable baseline area (%s)",
    (state) => {
      render(
        <SpellReference spell={{ ...spellFixture(), area: unavailableFact(state) }} />,
      );
      expect(screen.getByText("Area").parentElement).toHaveTextContent(`[${state}]`);
    },
  );

  it.each(["missing", "null", "invalid", "value"] as const)(
    "omits only missing price units (%s) and preserves known zero price",
    (state) => {
      render(
        <RecordPresentation
          presentation={{
            classification: "Reference",
            identity: summaryFixture("equipment:staff", "Staff"),
            content: [],
            owned: [],
            body: {
              kind: "physical_reference",
              value: {
                usage: knownFact("held in one hand"),
                price: knownFact({
                  pp: { ...unavailableFact<number>("missing"), adjustment: null },
                  gp: { ...unavailableFact<number>("missing"), adjustment: null },
                  sp: { ...unavailableFact<number>("missing"), adjustment: null },
                  cp: numberFact(0),
                }),
                price_per:
                  state === "value"
                    ? numberFact(0)
                    : { state, value: null, adjustment: null },
                bulk: numberFact(0),
              },
            },
          }}
          onReference={vi.fn()}
        />,
      );
      expect(screen.getByText("Price").parentElement).toHaveTextContent("0 cp");
      if (state === "missing") expect(screen.queryByText("Price per")).toBeNull();
      else
        expect(screen.getByText("Price per").parentElement).toHaveTextContent(
          state === "value" ? "0" : `[${state}]`,
        );
    },
  );

  it.each(["missing", "null", "invalid"] as const)(
    "omits absent damage category (%s) without hiding required damage facts",
    (state) => {
      const { container } = render(
        <DamageComponents
          damage={knownFact([
            {
              id: "generated-id",
              formula: unavailableFact("missing"),
              damage_type: unavailableFact("null"),
              kinds: knownFact([]),
              category: unavailableFact(state),
              materials: knownFact([]),
              apply_modifier: knownFact(false),
            },
          ])}
        />,
      );
      expect(container).toHaveTextContent("[missing] [null]");
      if (state === "invalid")
        expect(container).toHaveTextContent("category [invalid]");
      else expect(container).not.toHaveTextContent("category");
    },
  );

  it("uses ordered damage captions instead of authored IDs for multiple components", () => {
    const component = (id: string) => ({
      id,
      formula: knownFact("1d6"),
      damage_type: knownFact("fire"),
      kinds: knownFact([]),
      category: unavailableFact<string>("null"),
      materials: knownFact([]),
      apply_modifier: knownFact(false),
    });
    const { container } = render(
      <DamageComponents
        damage={knownFact([component("p36zyj2o94q5m2av6t6a"), component("0")])}
      />,
    );
    expect(container).toHaveTextContent("Damage component 1:");
    expect(container).toHaveTextContent("Damage component 2:");
    expect(container).not.toHaveTextContent("p36zyj2o94q5m2av6t6a");
    expect(container).not.toHaveTextContent("Damage 0");
  });

  it("gives single and multiple heightening increments meaningful ordered captions", () => {
    const spell = spellFixture();
    spell.heightening = knownFact({
      kind: "interval",
      interval: numberFact(1),
      area: { state: "not_applicable", value: null, adjustment: null },
      damage: knownFact([{ id: "0", formula: knownFact("2d6") }]),
    });
    const { rerender } = render(<SpellReference spell={spell} />);
    expect(screen.getByText("Damage increase").parentElement).toHaveTextContent("2d6");
    if (!spell.heightening.value || spell.heightening.value.kind !== "interval")
      throw new Error("fixture heightening");
    spell.heightening.value.damage = knownFact([
      { id: "0", formula: knownFact("2d6") },
      { id: "generated-id", formula: knownFact("1d4") },
    ]);
    rerender(<SpellReference spell={spell} />);
    expect(screen.getByText("Damage increase 1").parentElement).toHaveTextContent(
      "2d6",
    );
    expect(screen.getByText("Damage increase 2").parentElement).toHaveTextContent(
      "1d4",
    );
    expect(screen.queryByText("Damage 0")).toBeNull();
  });
});
