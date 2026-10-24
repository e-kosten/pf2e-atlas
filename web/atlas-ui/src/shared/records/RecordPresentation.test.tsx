import { fireEvent, render, screen, within, waitFor } from "@testing-library/react";
import { RecordPresentation } from "./RecordPresentation";
import { ActorActivities, ActorFacts, DamageComponents } from "./ActorReference";
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
    embedded: [],
  };
}

it.each([false, true])(
  "links only activities with additional reference details (encounter=%s)",
  (encounter) => {
    const actor = actorFixture();
    actor.activities = knownFact([
      { ...activityFixture("Complete summary"), has_reference_details: false },
      { ...activityFixture("Additional details"), has_reference_details: true },
    ]);
    const onReference = vi.fn();
    render(
      <ActorActivities actor={actor} onReference={onReference} encounter={encounter} />,
    );
    expect(screen.getByText("Complete summary")).toBeVisible();
    expect(screen.queryByRole("link", { name: "Complete summary" })).toBeNull();
    fireEvent.click(screen.getByRole("link", { name: "Additional details" }));
    expect(onReference).toHaveBeenCalledTimes(1);
  },
);

it("labels journal children as Pages even when the product kind is lore", () => {
  const presentation = detailFixture("journals:example", "Journal").presentation;
  const activity = activityFixture("A journal page", "journals:example");
  presentation.identity.kind = "lore";
  presentation.embedded = [
    {
      title: activity.title,
      family: "JournalEntryPage",
      usage: null,
      action_cost: unavailableFact("not_applicable"),
      navigation: {
        ...activity.navigation,
        embedded_path: [
          {
            collection: "/pages",
            identity: { Stable: "page-id" },
          },
        ],
      },
    },
  ];
  render(<RecordPresentation presentation={presentation} onReference={vi.fn()} />);
  expect(screen.getByRole("heading", { name: "Pages" })).toBeVisible();
  expect(screen.getByRole("link", { name: "A journal page" })).toBeVisible();
  expect(screen.queryByText("Owned records")).toBeNull();
});

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
    expect(onReference).toHaveBeenCalledWith(
      "equipment:catalog-staff",
      original,
      relationship.target,
    );
    onReference.mockClear();
    fireEvent.click(original, { ctrlKey: true });
    expect(onReference).not.toHaveBeenCalled();
    rerender(view(false));
    expect(screen.queryByRole("link", { name: "View original" })).toBeNull();
  });

  it("shows selected activity traits once in its identity and retains traits on embedded activities", () => {
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

  it("groups embedded spells by their exact casting entry and retains authored DC and attack", () => {
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
        embedded_path: [
          { collection: "items", identity: { SnapshotLocal: { index: 1 } } },
        ],
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
          locator: { record: "hazards:pit", embedded_path: [], field: role },
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
            embedded: [],
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
          ? "fire —; cold 0"
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
    expect(container).toHaveTextContent("1d6 fire; kinds [invalid]");
    expect(container).not.toHaveTextContent("materials");
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
            embedded: [],
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
          embedded: [],
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
    expect(within(form).queryByText("—")).toBeNull();
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
  it("retains physical and table reference facts and exact embedded navigation", () => {
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
              roll_note: "Reference rolls may repeat; results are not consumed.",
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
              details: null,
              quantity: numberFact(1),
              material: unavailableFact("not_applicable"),
              size: unavailableFact("not_applicable"),
              usage: knownFact("held in one hand"),
              bulk: numberFact(0.1),
              price: knownFact({
                pp: { ...unavailableFact<number>("missing"), adjustment: null },
                gp: numberFact(10),
                sp: { ...unavailableFact<number>("missing"), adjustment: null },
                cp: { ...unavailableFact<number>("missing"), adjustment: null },
              }),
              consumable: null,
              price_per: numberFact(1),
            },
          },
        }}
        onReference={callback}
      />,
    );
    expect(screen.getByText("held in one hand")).toBeVisible();
    expect(screen.getByText("L")).toBeVisible();
    expect(screen.getByText("10 gp")).toBeVisible();
  });
  it("shows fixed-rank authored changes as reference data", async () => {
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
          embedded: [],
          body: { kind: "spell", value: spell },
        }}
        onReference={vi.fn()}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "Heightening details" }));
    const change = screen.getByRole("heading", {
      name: "Rank 5",
    }).parentElement!;
    await waitFor(() => expect(within(change).getByText("60 feet")).toBeVisible());
    expect(within(change).getByText("No")).toBeVisible();
    expect(within(change).queryByText("—")).toBeNull();
    expect(screen.getByText("500 feet")).toBeVisible();
    expect(screen.queryByRole("combobox")).toBeNull();
  });
});

it("puts spell operating facts before prose without losing authored headings or unavailable facts", () => {
  const detail = detailFixture("spells:allfood", "Allfood");
  detail.presentation.body = {
    kind: "spell",
    value: {
      ...spellFixture(),
      target: knownFact("one object"),
      requirements: unavailableFact("invalid"),
    },
  };
  const locator = {
    record: detail.record.record_key,
    embedded_path: [],
    field: "/system/description/value",
  };
  detail.presentation.content = [
    {
      locator,
      role: "description",
      source_fingerprint: null,
      body: {
        kind: "html",
        inclusions: [],
        html: "<p>You transform one object.</p><h4>Heightened (4th)</h4><p>The food has a longer duration.</p>",
        controls: [],
      },
    },
    {
      locator: { ...locator, field: "/system/notes" },
      role: "notes",
      source_fingerprint: null,
      body: { kind: "plain", text: "A separate supplied note." },
    },
  ];
  render(
    <RecordPresentation presentation={detail.presentation} onReference={vi.fn()} />,
  );
  const description = screen.getByText("You transform one object.");
  expect(
    screen.getByText("one object").compareDocumentPosition(description) &
      Node.DOCUMENT_POSITION_FOLLOWING,
  ).toBeTruthy();
  expect(screen.getByText("Requirements").parentElement).toHaveTextContent("invalid");
  expect(screen.queryByRole("heading", { name: "Description" })).toBeNull();
  expect(screen.getByRole("heading", { name: "Heightened (4th)" })).toBeVisible();
  expect(screen.getByText("The food has a longer duration.")).toBeVisible();
  expect(screen.getByRole("heading", { name: "Notes" })).toBeVisible();
  expect(screen.getByText("A separate supplied note.")).toBeVisible();
});

it("keeps the field label when the principal description is unavailable", () => {
  const detail = detailFixture("actions:example", "Example");
  detail.presentation.content = [
    {
      locator: {
        record: detail.record.record_key,
        embedded_path: [],
        field: "/system/description/value",
      },
      role: "description",
      source_fingerprint: null,
      body: { kind: "unavailable", state: "invalid" },
    },
  ];
  render(
    <RecordPresentation presentation={detail.presentation} onReference={vi.fn()} />,
  );
  expect(screen.getByRole("heading", { name: "Description" })).toBeVisible();
  expect(screen.getByRole("alert")).toHaveTextContent("Content unavailable: invalid");
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
          embedded: [],
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
          embedded: [],
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
      expect(screen.getByText("Area").parentElement).toHaveTextContent(
        state === "missing" ? "—" : `[${state}]`,
      );
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
            embedded: [],
            body: {
              kind: "physical_reference",
              value: {
                details: null,
                quantity: numberFact(1),
                material: unavailableFact("not_applicable"),
                size: unavailableFact("not_applicable"),
                usage: knownFact("held in one hand"),
                price: knownFact({
                  pp: { ...unavailableFact<number>("missing"), adjustment: null },
                  gp: { ...unavailableFact<number>("missing"), adjustment: null },
                  sp: { ...unavailableFact<number>("missing"), adjustment: null },
                  cp: numberFact(0),
                }),
                consumable: null,
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
      expect(container).toHaveTextContent("— [null]");
      if (state === "invalid")
        expect(container).toHaveTextContent("category [invalid]");
      else expect(container).not.toHaveTextContent("category");
    },
  );

  it("keeps distinct damage rows without mechanical numbering or authored IDs", () => {
    const component = (id: string, formula: string, damageType: string) => ({
      id,
      formula: knownFact(formula),
      damage_type: knownFact(damageType),
      kinds: knownFact([]),
      category: unavailableFact<string>("null"),
      materials: knownFact([]),
      apply_modifier: knownFact(false),
    });
    const { container } = render(
      <DamageComponents
        damage={knownFact([
          component("p36zyj2o94q5m2av6t6a", "1d6", "fire"),
          component("0", "2d8", "cold"),
        ])}
      />,
    );
    expect(
      [...container.querySelectorAll(".record-damage p")].map((row) => row.textContent),
    ).toEqual(["1d6 fire", "2d8 cold"]);
    expect(container).not.toHaveTextContent("Damage component");
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
    fireEvent.click(screen.getByRole("button", { name: "Heightening details" }));
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

it("keeps authored qualifiers beside their numbers and general save notes scoped to the group", () => {
  const presentation = actorPresentation();
  if (presentation.body.kind !== "creature") throw new Error("creature");
  const actor = presentation.body.value;
  actor.defense_notes.armor_class = knownFact("21 while broken");
  actor.defense_notes.hit_points = knownFact("void healing");
  actor.defense_notes.all_saves = knownFact("+2 status vs. magic");
  actor.defense_notes.fortitude = knownFact("against disease");
  actor.hardness = numberFact(0);
  actor.ritual_difficulty_class = numberFact(23);
  const activity = activityFixture("Focus casting");
  activity.focus_capacity = numberFact(3);
  activity.frequency = knownFact({
    maximum: numberFact(1),
    interval: knownFact("minute"),
  });
  actor.activities = knownFact([activity]);
  const { rerender } = render(
    <RecordPresentation presentation={presentation} onReference={vi.fn()} />,
  );
  expect(
    screen.getByText("AC", { selector: "strong" }).parentElement,
  ).toHaveTextContent("AC 16 21 while broken");
  expect(
    screen.getByText("HP", { selector: "strong" }).parentElement,
  ).toHaveTextContent("HP 20 void healing");
  expect(
    screen.getByText("Fortitude", { selector: "strong" }).parentElement,
  ).toHaveTextContent("Fortitude +7 against disease");
  expect(screen.queryByText("All saves")).toBeNull();
  expect(screen.getByText("+2 status vs. magic").parentElement).toHaveTextContent(
    "Fortitude +7 against disease",
  );
  expect(screen.queryByText("AC notes")).toBeNull();
  expect(screen.queryByText("HP notes")).toBeNull();
  for (const text of [
    "+2 status vs. magic",
    "Focus capacity",
    "Ritual DC",
    "1 per minute",
    "Hardness",
  ]) {
    expect(screen.getByText(text)).toBeVisible();
  }
  rerender(<ActorFacts actor={actor} encounter hideHitPoints />);
  expect(
    screen.getByText("HP", { selector: "strong" }).parentElement,
  ).toHaveTextContent("HP void healing");
  expect(
    screen.getByText("HP", { selector: "strong" }).parentElement,
  ).not.toHaveTextContent("20");
});

it("opens verified self effects through the shared preview and leaves unverified labels as text", () => {
  const actor = actorFixture();
  const activity = activityFixture("Ward");
  activity.has_reference_details = false;
  const target = {
    ...activity.navigation,
    record_key: "effects:ward",
    embedded_path: [],
    source_fingerprint: null,
  };
  activity.self_effect = knownFact({
    name: knownFact("Ward effect"),
    authored_target: knownFact("Compendium.pf2e.effects.Item.ward"),
    navigation: target,
  });
  activity.strike_effects = knownFact(["unresolved-poison"]);
  actor.activities = knownFact([activity]);
  const onReference = vi.fn();
  const { rerender } = render(
    <ActorActivities actor={actor} onReference={onReference} />,
  );
  fireEvent.click(screen.getByRole("link", { name: "Preview Ward effect" }));
  expect(onReference).toHaveBeenCalledWith("effects:ward", expect.anything(), target);
  expect(screen.getByText("unresolved-poison")).toBeVisible();
  activity.self_effect.value!.navigation = null;
  rerender(<ActorActivities actor={actor} onReference={onReference} />);
  expect(screen.queryByRole("link", { name: "Preview Ward effect" })).toBeNull();
  expect(screen.getByText(/Ward effect.*reference unavailable/)).toBeVisible();
});

it("presents consumable spell semantics, exact navigation, package prices and source counts", () => {
  const navigation = {
    record_key: "equipment:wand",
    embedded_path: [{ collection: "/system/spell", identity: { Stable: "spell" } }],
    field: null,
    passage: null,
    source_fingerprint: null,
    spell_rank: 2,
  };
  const consumable = {
    category: knownFact("wand"),
    damage: unavailableFact<import("../../generated/atlas").ConsumableDamageView>(),
    spell: knownFact({
      title: "Heal",
      family: "spell",
      navigation,
      usage: null,
      action_cost: unavailableFact<import("../../generated/atlas").ActionCostView>(),
    }),
    spell_presented_inline: false,
    maximum_uses: numberFact(500),
    source_uses: numberFact(420),
  };
  const presentation: RecordPresentationView = {
    classification: "Item",
    identity: summaryFixture("equipment:wand", "Wand"),
    content: [
      {
        locator: {
          record: "equipment:wand",
          embedded_path: [],
          field: "/system/description/value",
        },
        source_fingerprint: null,
        role: "description",
        body: { kind: "plain", text: "The exact wand activation and frequency." },
      },
    ],
    embedded: [],
    body: {
      kind: "physical_reference",
      value: {
        details: null,
        quantity: numberFact(10),
        material: unavailableFact("not_applicable"),
        size: unavailableFact("not_applicable"),
        consumable,
        usage: knownFact("Held in one hand"),
        bulk: numberFact(0.1),
        price_per: numberFact(10),
        price: knownFact({
          pp: numberFact(0),
          gp: numberFact(1),
          sp: numberFact(0),
          cp: numberFact(0),
        }),
      },
    },
  };
  const onReference = vi.fn();
  const { rerender } = render(
    <RecordPresentation presentation={presentation} onReference={onReference} />,
  );
  expect(
    screen
      .getByText("Price per 10")
      .compareDocumentPosition(
        screen.getByText("The exact wand activation and frequency."),
      ) & Node.DOCUMENT_POSITION_FOLLOWING,
  ).toBeTruthy();
  expect(screen.getByText("Uses").parentElement).toHaveTextContent("420 / 500");
  expect(screen.getByText("Quantity").parentElement).toHaveTextContent("10");
  expect(screen.getByText("Price per 10").parentElement?.textContent).toBe(
    "Price per 10 1 gp",
  );
  expect(screen.getByText("Spell").parentElement).toHaveTextContent("Heal · Rank 2");
  fireEvent.click(screen.getByRole("link", { name: "Heal" }));
  expect(onReference).toHaveBeenCalledWith(
    "equipment:wand",
    expect.any(HTMLElement),
    navigation,
  );
  expect(screen.queryByText("Contents")).toBeNull();
  expect(screen.getByText("Category").parentElement).toHaveTextContent("wand");
  presentation.identity.traits = [
    { kind: "trait", value: "wand", label: "Wand", trait_key: "trait:wand" },
  ];
  consumable.spell_presented_inline = true;
  consumable.source_uses = numberFact(1);
  consumable.maximum_uses = numberFact(1);
  if (presentation.body.kind === "physical_reference")
    presentation.body.value.quantity = numberFact(1);
  rerender(
    <RecordPresentation presentation={presentation} onReference={onReference} />,
  );
  expect(screen.queryByText("Spell")).toBeNull();
  expect(screen.queryByText("Uses")).toBeNull();
  expect(screen.queryByText("Quantity")).toBeNull();
  expect(screen.queryByText("Category")).toBeNull();
  expect(screen.getByRole("link", { name: "Wand" })).toBeVisible();
});
