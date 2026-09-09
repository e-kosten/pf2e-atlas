import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { recordDetailFixture } from "../../test/recordFixtures";
import { RecordDetailPane } from "./RecordDetailPane";
import { fireEvent, render, screen, within } from "@testing-library/react";
import type {
  ConsumableFactView,
  ConsumableDefinitionView,
  ConsumableOccurrenceView,
  ConsumableSurfaceView,
  RecordSurfaceView,
} from "../../generated/atlas";
import { ConsumableOccurrences } from "./ConsumableRecordSurface";
import { RecordSurface } from "./RecordSurface";

const missing = { state: "missing" } as const;
const known = <T,>(value: T): ConsumableFactView<T> => ({ state: "known", value });

describe("ConsumableRecordSurface", () => {
  it.each(["pointer", "keyboard"])(
    "opens occurrence and ordinary disclosures without a motion start style via %s",
    (method) => {
      const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
      const detail = { surface: standaloneSurface() };
      const renderPane = (current: typeof detail) => (
        <QueryClientProvider client={client}>
          <RecordDetailPane detail={current} loading={false} onReference={vi.fn()} />
        </QueryClientProvider>
      );
      const { rerender } = render(renderPane(detail));
      const activate = (label: string) => {
        const header = screen.getByText(label).closest(".ant-collapse-header")!;
        if (method === "keyboard") {
          (header as HTMLElement).focus();
          fireEvent.keyDown(header, { key: "Enter", keyCode: 13 });
        } else fireEvent.click(header);
        expect(header).toHaveAttribute("aria-expanded", "true");
        const content = header
          .closest(".ant-collapse-item")!
          .querySelector(".ant-collapse-content") as HTMLElement;
        expect(content).not.toHaveClass("ant-motion-collapse-enter-start");
        expect(content.style.height).not.toBe("0px");
        expect(content.style.opacity).not.toBe("0");
        return content;
      };
      expect(
        within(activate("Source & provenance")).getByText(
          "packs/equipment/arboreal-wand.json",
        ),
      ).toBeVisible();
      const parent = recordDetailFixture();
      if (parent.surface.presentation.presentation_type !== "creature")
        throw new Error("creature fixture");
      parent.surface.presentation.body.consumables = [
        occurrence({
          occurrence_id: "disclosure-dose",
          name: "Disclosure dose",
          target: { state: "parent_owned", reason: "no_locator" },
          definition: consumableDefinition(),
        }),
      ];
      rerender(renderPane(parent));
      expect(
        within(activate("Disclosure dose")).getByText("Held In One Hand"),
      ).toBeVisible();
      // Real height and visible-text evidence is separately required by the Safari probe.
    },
  );

  it("renders standalone definition and source state without inferring missing facts", () => {
    const onReference = vi.fn();
    render(<RecordSurface onReference={onReference} surface={standaloneSurface()} />);

    expect(screen.getByRole("heading", { name: "Arboreal Wand" })).toBeVisible();
    expect(screen.getAllByText("Consumable")).toHaveLength(2);
    const details = screen.getByRole("region", { name: "Consumable details" });
    expect(within(details).getByText("Held In One Hand")).toBeVisible();
    expect(within(details).getByText("Bulk")).toBeVisible();
    expect(within(details).getByText("0.1")).toBeVisible();
    expect(screen.getByText("Quantity")).toBeVisible();
    expect(screen.getByText("2")).toBeVisible();
    expect(within(details).getByText("1 gp · per 1")).toBeVisible();
    expect(screen.queryByText("Container")).not.toBeInTheDocument();

    fireEvent.click(screen.getByText("Source & provenance"));
    expect(screen.getByText("packs/equipment/arboreal-wand.json")).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: "Open embedded spell" }));
    expect(onReference).toHaveBeenCalledWith("equipment:wand", {
      parent_record_key: "equipment:wand",
      child_id: "embedded-heal",
      target_record_key: "spells-srd:heal",
    });
  });

  it("keeps resolved and parent-owned occurrences distinct and navigable", () => {
    const onReference = vi.fn();
    const resolved = occurrence({
      occurrence_id: "resolved-dose",
      name: "Canonical Dose",
      target: {
        state: "resolved",
        record_key: "equipment:canonical-dose",
        mismatch_fields: ["definition.maximum_uses"],
      },
    });
    const local = occurrence({
      occurrence_id: "local-dose",
      name: "Local Dose",
      target: { state: "parent_owned", reason: "no_locator" },
      definition: consumableDefinition(),
    });

    render(
      <ConsumableOccurrences
        occurrences={[resolved, local]}
        onReference={onReference}
      />,
    );

    expect(screen.getByRole("heading", { name: "Consumables" })).toBeVisible();
    fireEvent.click(screen.getByText("Canonical Dose"));
    expect(screen.getByText("Local source differences retained")).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Open canonical consumable" }));
    expect(onReference).toHaveBeenCalledWith("equipment:canonical-dose");

    fireEvent.click(screen.getByText("Local Dose"));
    expect(screen.getByText("Embedded definition")).toBeVisible();
    const localOccurrence = screen
      .getByText("Local Dose")
      .closest(".ant-collapse-item");
    if (!(localOccurrence instanceof HTMLElement))
      throw new Error("local occurrence panel");
    expect(within(localOccurrence).getAllByText("Quantity")).toHaveLength(1);
    expect(screen.getAllByText("Embedded definition")).toHaveLength(1);
  });

  it("retains exact reused and local mismatch child locators", () => {
    const onReference = vi.fn();
    const reused = { parent_record_key: "equipment:target", child_id: "target-child" };
    const mismatch = {
      parent_record_key: "actors:owner",
      occurrence_id: "local-dose",
      child_id: "local-child",
    };
    render(
      <ConsumableOccurrences
        onReference={onReference}
        occurrences={[
          occurrence({
            occurrence_id: "reused-dose",
            name: "Reused spell",
            target: {
              state: "resolved",
              record_key: "equipment:target",
              mismatch_fields: [],
            },
            spell_child: reused,
          }),
          occurrence({
            occurrence_id: "local-dose",
            name: "Retained spell",
            target: {
              state: "resolved",
              record_key: "equipment:target",
              mismatch_fields: ["spell"],
            },
            spell_child: mismatch,
          }),
        ]}
      />,
    );
    fireEvent.click(screen.getByText("Reused spell"));
    fireEvent.click(screen.getByRole("button", { name: "Open embedded spell" }));
    expect(onReference).toHaveBeenLastCalledWith("equipment:target", reused);
    fireEvent.click(screen.getByText("Reused spell"));
    fireEvent.click(screen.getByText("Retained spell"));
    fireEvent.click(screen.getByRole("button", { name: "Open embedded spell" }));
    expect(onReference).toHaveBeenLastCalledWith("actors:owner", mismatch);
  });

  it("renders the exact structured damage summary without evaluating its formula", () => {
    const surface = standaloneSurface();
    if (surface.presentation?.presentation_type !== "consumable") {
      throw new Error("consumable fixture");
    }
    surface.presentation.body.damage = known({
      formula: known("4d6"),
      category: known("damage"),
      damage_type: known("cold"),
    });

    render(<RecordSurface onReference={() => undefined} surface={surface} />);

    const details = screen.getByRole("region", { name: "Consumable details" });
    expect(within(details).getByText("4d6")).toBeVisible();
    expect(within(details).getByText("Cold")).toBeVisible();
    expect(within(details).getByText("Damage")).toBeVisible();
  });

  it("uses the typed effect kind as the formula label without repeating it", () => {
    const surface = standaloneSurface();
    if (surface.presentation?.presentation_type !== "consumable") {
      throw new Error("consumable fixture");
    }
    surface.presentation.body.damage = known({
      formula: known("2d8+5"),
      category: known("healing"),
      damage_type: known("vitality"),
    });

    render(<RecordSurface onReference={() => undefined} surface={surface} />);

    const details = screen.getByRole("region", { name: "Consumable details" });
    expect(within(details).getByText("Healing")).toBeVisible();
    expect(within(details).getByText("2d8+5")).toBeVisible();
    expect(within(details).queryByText("Kind")).not.toBeInTheDocument();
  });

  it("omits unavailable optional state and exposes no item-use controls", () => {
    const surface = standaloneSurface();
    if (surface.presentation?.presentation_type !== "consumable") {
      throw new Error("consumable fixture");
    }
    surface.presentation.body.maximum_uses = {
      state: "unsupported",
      value: { reason: "source_field_drift" },
    };
    surface.presentation.body.auto_destroy = { state: "null" };
    surface.presentation.body.source_state = {
      quantity: { state: "unsupported", value: { reason: "source_field_drift" } },
      current_uses: { state: "null" },
      current_hp: missing,
      container_id: missing,
      equipped: { state: "unsupported", value: { reason: "source_field_drift" } },
    };

    render(<RecordSurface onReference={() => undefined} surface={surface} />);

    expect(screen.queryByText("Maximum uses")).not.toBeInTheDocument();
    expect(screen.queryByText("Uses remaining")).not.toBeInTheDocument();
    expect(screen.queryByText("Auto-destroy")).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /use|consume|roll|equip/i }),
    ).not.toBeInTheDocument();
  });

  it("keeps repeated authored occurrences and their references distinct", () => {
    const onReference = vi.fn();
    const first = occurrence({
      occurrence_id: "dose-1",
      authored_order: 3,
      name: "Dragon Dose",
      target: { state: "parent_owned", reason: "no_locator" },
      definition: consumableDefinition(),
      content: [occurrenceContent("dose-1-description", "spells-srd:heal")],
    });
    const second = occurrence({
      occurrence_id: "dose-2",
      authored_order: 4,
      name: "Dragon Dose",
      target: { state: "parent_owned", reason: "target_missing" },
      definition: consumableDefinition(),
      content: [occurrenceContent("dose-2-description", "spells-srd:harm")],
    });

    render(
      <ConsumableOccurrences occurrences={[first, second]} onReference={onReference} />,
    );

    const labels = screen.getAllByText("Dragon Dose");
    expect(labels).toHaveLength(2);
    fireEvent.click(labels[0]!);
    fireEvent.click(screen.getByRole("link", { name: "Heal" }));
    fireEvent.click(labels[1]!);
    fireEvent.click(screen.getByRole("link", { name: "Harm" }));
    expect(onReference.mock.calls).toEqual([["spells-srd:heal"], ["spells-srd:harm"]]);
  });
});

function standaloneSurface(): RecordSurfaceView {
  return {
    metadata: {
      record_key: "equipment:arboreal-wand",
      title: "Arboreal Wand",
      kind: "equipment",
      kind_label: "Consumable",
      level: 4,
      rarity: "common",
      traits: ["consumable", "magical", "wand"],
      source: {
        publication_title: "Pathfinder Player Core",
        pack_label: "Equipment",
        document_type: "Item",
        record_type: "consumable",
        source_path: "packs/equipment/arboreal-wand.json",
      },
    },
    profile: "record_detail",
    presentation: { presentation_type: "consumable", body: consumableBody() },
  };
}

function consumableBody(): ConsumableSurfaceView {
  return {
    ...consumableDefinition(),
    source_state: {
      quantity: known("2"),
      current_uses: known("1"),
      current_hp: known("1"),
      container_id: { state: "null" },
      equipped: missing,
    },
    spell_child: {
      parent_record_key: "equipment:wand",
      child_id: "embedded-heal",
      target_record_key: "spells-srd:heal",
    },
    content: [],
  };
}

function consumableDefinition(): ConsumableDefinitionView {
  return {
    slug: known("arboreal-wand"),
    level: known("4"),
    category: known("wand"),
    rarity: known("common"),
    traits: known(["consumable", "magical", "wand"]),
    other_tags: known([]),
    usage: known("held-in-one-hand"),
    base_item: missing,
    bulk: known("0.1"),
    size: missing,
    stack_group: missing,
    material: missing,
    price: known({
      denominations: known([{ denomination: "gp", amount: "1" }]),
      per: known("1"),
    }),
    maximum_uses: known("1"),
    auto_destroy: known(false),
    maximum_hp: known("1"),
    hardness: known("0"),
    publication: missing,
    damage: missing,
  };
}

function occurrence(
  values: Pick<ConsumableOccurrenceView, "occurrence_id" | "name" | "target"> &
    Partial<ConsumableOccurrenceView>,
): ConsumableOccurrenceView {
  return {
    authored_order: 0,
    identity_stability: "stable_source_identity",
    source_state: consumableBody().source_state,
    ...values,
  };
}

function occurrenceContent(contentKey: string, recordKey: string) {
  const label = recordKey.endsWith(":heal") ? "Heal" : "Harm";
  return {
    content_key: contentKey,
    role: "primary_description" as const,
    authored_order: 0,
    blocks: [
      {
        block_type: "paragraph" as const,
        spans: [
          { span_type: "text" as const, text: "See " },
          {
            span_type: "reference" as const,
            label,
            record_key: recordKey,
            embedded: false,
          },
        ],
      },
    ],
    content_hash: `${contentKey}-hash`,
    visibility: "public" as const,
    provenance: {
      source_record_key: "bestiary:fixture",
      relative_source_path: `items/${contentKey}/system/description/value`,
      field_family: "consumable_description",
    },
  };
}
