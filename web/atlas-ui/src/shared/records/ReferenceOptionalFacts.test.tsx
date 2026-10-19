import { render, screen } from "@testing-library/react";
import { ContentFields } from "./RecordPresentation";
import { SpellChanges, SpellReference } from "./SpellReference";
import type {
  PreparedFieldBodyView,
  SpellPresentationView,
} from "../../generated/atlas";
import {
  knownFact,
  numberFact,
  spellFixture,
  unavailableFact,
} from "../../test/fixtures";

describe("optional reference content", () => {
  it.each(["missing", "null", "invalid"] as const)(
    "retains unavailable heightening/forms parents (%s) while omitting source-embedded inapplicable structures",
    (state) => {
      const spell: SpellPresentationView = {
        ...spellFixture(),
        heightening: unavailableFact(state),
        forms: unavailableFact<never[]>(state),
      };
      const { rerender } = render(<SpellReference spell={spell} />);
      expect(screen.getByText("Heightening").parentElement).toHaveTextContent(
        `[${state}]`,
      );
      expect(screen.getByText("Spell forms").parentElement).toHaveTextContent(
        `[${state}]`,
      );
      rerender(
        <SpellReference
          spell={{
            ...spell,
            heightening: unavailableFact("not_applicable"),
            forms: unavailableFact("not_applicable"),
          }}
        />,
      );
      expect(screen.queryByText("Heightening")).toBeNull();
      expect(screen.queryByText("Spell forms")).toBeNull();
    },
  );
  it("retains a spell-form defense clear independently of the baseline", () => {
    const absent = <T,>() => unavailableFact<T>("missing");
    render(
      <SpellChanges
        changes={{
          cast: absent(),
          range: absent(),
          target: absent(),
          area: absent(),
          defense: unavailableFact("null"),
          duration: absent(),
          sustained: absent(),
          traits: absent(),
          traditions: absent(),
          damage: absent(),
          heightening: absent(),
        }}
      />,
    );
    expect(screen.getByText("Defense").parentElement).toHaveTextContent("Cleared");
    expect(screen.queryByText("Save statistic")).toBeNull();
  });
  it("omits explicit no-defense and raw casting-entry IDs from baseline web reading", () => {
    const spell = {
      ...spellFixture(),
      defense: null,
      casting_entry: knownFact("local-foundry-entry"),
    };
    const { container } = render(<SpellReference spell={spell} />);
    expect(screen.queryByText("Defense")).toBeNull();
    expect(screen.queryByText("Casting entry")).toBeNull();
    expect(container).not.toHaveTextContent("local-foundry-entry");
    expect(spell.casting_entry.value).toBe("local-foundry-entry");
  });

  it.each(["missing", "null", "invalid"] as const)(
    "retains supplied unavailable baseline defense (%s)",
    (state) => {
      render(
        <SpellReference
          spell={{ ...spellFixture(), defense: unavailableFact(state) }}
        />,
      );
      expect(screen.getByText("Defense").parentElement).toHaveTextContent(`[${state}]`);
    },
  );
  it.each<PreparedFieldBodyView>([
    { kind: "plain", text: " \n" },
    { kind: "html", html: "", controls: [] },
  ])("omits only known-empty Notes content ($kind)", (body) => {
    render(
      <ContentFields
        fields={[
          {
            locator: { record: "actors:ghoul", embedded_path: [], field: "notes" },
            role: "notes",
            source_fingerprint: null,
            body,
          },
        ]}
        onReference={vi.fn()}
      />,
    );
    expect(screen.queryByRole("heading", { name: "Notes" })).toBeNull();
  });

  it.each(["missing", "null", "invalid"] as const)(
    "preserves unavailable Notes (%s)",
    (state) => {
      render(
        <ContentFields
          fields={[
            {
              locator: { record: "actors:ghoul", embedded_path: [], field: "notes" },
              role: "notes",
              source_fingerprint: null,
              body: { kind: "unavailable", state },
            },
          ]}
          onReference={vi.fn()}
        />,
      );
      expect(screen.getByRole("heading", { name: "Notes" })).toBeVisible();
    },
  );

  it("preserves nonempty Notes and uses authored or ordered spell-form captions", () => {
    render(
      <ContentFields
        fields={[
          {
            locator: { record: "actors:ghoul", embedded_path: [], field: "notes" },
            role: "notes",
            source_fingerprint: null,
            body: { kind: "plain", text: "void healing" },
          },
        ]}
        onReference={vi.fn()}
      />,
    );
    expect(screen.getByText("void healing")).toBeVisible();
    const spell = spellFixture();
    spell.forms = knownFact([
      {
        id: "37gy7l19tik74o4s",
        name: knownFact("Heal (vs. Living)"),
        sort: numberFact(1),
        changes: unavailableFact("invalid"),
      },
      {
        id: "raw-generated-id",
        name: unavailableFact("missing"),
        sort: numberFact(2),
        changes: unavailableFact("invalid"),
      },
      {
        id: "invalid-name-id",
        name: unavailableFact("invalid"),
        sort: numberFact(3),
        changes: unavailableFact("invalid"),
      },
    ]);
    render(<SpellReference spell={spell} />);
    expect(screen.getByRole("heading", { name: "Heal (vs. Living)" })).toBeVisible();
    expect(screen.getByRole("heading", { name: "Spell form 2" })).toBeVisible();
    expect(screen.getByRole("heading", { name: "Spell form 3" })).toBeVisible();
    expect(screen.getByText("Form name").parentElement).toHaveTextContent("[invalid]");
    expect(screen.queryByText("raw-generated-id")).toBeNull();
  });

  it("omits only known blank damage increments, retaining zero and unavailable formulas", () => {
    const spell = spellFixture();
    spell.heightening = knownFact({
      kind: "interval",
      interval: numberFact(1),
      area: { state: "not_applicable", value: null, adjustment: null },
      damage: knownFact([
        { id: "empty", formula: knownFact("") },
        { id: "whitespace", formula: knownFact(" \t") },
        { id: "zero", formula: knownFact("0") },
        { id: "unknown", formula: unavailableFact("missing") },
        { id: "invalid", formula: unavailableFact("invalid") },
      ]),
    });
    render(<SpellReference spell={spell} />);
    expect(screen.getByText("Damage increase 1").parentElement).toHaveTextContent("0");
    expect(screen.getByText("Damage increase 2").parentElement).toHaveTextContent(
      "[missing]",
    );
    expect(screen.getByText("Damage increase 3").parentElement).toHaveTextContent(
      "[invalid]",
    );
    expect(screen.queryByText("Damage increase 4")).toBeNull();
  });
});
