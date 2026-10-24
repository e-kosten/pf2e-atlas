import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { ShieldReferenceView, WeaponReferenceView } from "../../generated/atlas";
import { knownFact, numberFact, unavailableFact } from "../../test/fixtures";
import { PhysicalDetails } from "./PhysicalDetails";

describe("authored equipment facts", () => {
  it("keeps base damage separate from runes and preserves range and damage qualifiers", () => {
    const weapon: WeaponReferenceView = {
      category: knownFact("martial"),
      group: knownFact("bow"),
      damage: knownFact({
        dice: numberFact(1),
        die: knownFact("d8"),
        damage_type: knownFact("piercing"),
        modifier: { ...unavailableFact<number>("missing"), adjustment: null },
        persistent: knownFact({
          number: numberFact(2),
          faces: { ...unavailableFact<number>("null"), adjustment: null },
          damage_type: knownFact("fire"),
        }),
      }),
      range_increment: numberFact(100),
      maximum_range: { ...unavailableFact<number>("missing"), adjustment: null },
      reload: knownFact("0"),
      splash_damage: knownFact("1"),
      ammunition: unavailableFact("missing"),
      runes: knownFact({
        potency: numberFact(2),
        striking: knownFact("Greater striking"),
        property: knownFact([]),
      }),
      melee_usage: unavailableFact("missing"),
    };
    render(<PhysicalDetails details={{ kind: "weapon", value: weapon }} />);
    expect(screen.getByText("Base damage").parentElement).toHaveTextContent(
      "Base damage 1d8 piercing",
    );
    expect(screen.queryByText("3d8", { exact: false })).toBeNull();
    expect(screen.getByText("Striking rune").parentElement).toHaveTextContent(
      "Greater striking",
    );
    expect(screen.getByText("Range increment").parentElement).toHaveTextContent(
      "100 feet",
    );
    expect(screen.queryByText("Maximum range")).toBeNull();
    expect(screen.getByText("Reload").parentElement).toHaveTextContent("0");
    expect(screen.getByText("Persistent damage").parentElement).toHaveTextContent(
      "Persistent damage 2 fire",
    );
    expect(screen.getByText("Splash damage").parentElement).toHaveTextContent("1");
  });

  it("omits duplicate shield durability only when the service confirms an authored table", () => {
    const shield: ShieldReferenceView = {
      ac_bonus: numberFact(2),
      speed_penalty: numberFact(0),
      hardness: numberFact(5),
      hit_points: numberFact(20),
      reinforcing: numberFact(0),
      durability_in_table: true,
    };
    const { rerender } = render(
      <PhysicalDetails details={{ kind: "shield", value: shield }} />,
    );
    expect(screen.getByText("AC bonus").parentElement).toHaveTextContent("+2");
    expect(screen.queryByText("Hardness")).toBeNull();
    expect(screen.queryByText("HP")).toBeNull();
    shield.durability_in_table = false;
    shield.hardness = numberFact(0);
    rerender(<PhysicalDetails details={{ kind: "shield", value: shield }} />);
    expect(screen.getByText("Hardness").parentElement).toHaveTextContent("0");
    expect(screen.getByText("HP").parentElement).toHaveTextContent("20");
    expect(screen.queryByText("BT")).toBeNull();
  });

  it("keeps container zero and unavailable capacity distinct, and omits an empty ammunition group", () => {
    const { container, rerender } = render(
      <PhysicalDetails
        details={{
          kind: "container",
          value: {
            capacity: numberFact(0),
            ignored_bulk: numberFact(0),
            held_or_stowed_bulk: numberFact(0.1),
            stowing: knownFact(false),
          },
        }}
      />,
    );
    expect(screen.getByText("Capacity").parentElement).toHaveTextContent("0 Bulk");
    expect(screen.getByText("Ignored Bulk").parentElement).toHaveTextContent("0");
    expect(screen.getByText("Held or stowed Bulk").parentElement).toHaveTextContent(
      "L",
    );
    expect(screen.getByText("Stowing").parentElement).toHaveTextContent("No");
    rerender(
      <PhysicalDetails
        details={{
          kind: "ammunition",
          value: { maximum_uses: numberFact(1), source_uses: numberFact(1) },
        }}
      />,
    );
    expect(container).toBeEmptyDOMElement();
    rerender(
      <PhysicalDetails
        details={{
          kind: "container",
          value: {
            capacity: { ...unavailableFact<number>("invalid"), adjustment: null },
            ignored_bulk: { ...unavailableFact<number>("null"), adjustment: null },
            held_or_stowed_bulk: {
              ...unavailableFact<number>("missing"),
              adjustment: null,
            },
            stowing: unavailableFact("missing"),
          },
        }}
      />,
    );
    expect(screen.getByText("Capacity").parentElement).toHaveTextContent("[invalid]");
    expect(screen.getByText("Ignored Bulk").parentElement).toHaveTextContent("[null]");
    expect(container).not.toHaveTextContent("0 Bulk");
  });
});
