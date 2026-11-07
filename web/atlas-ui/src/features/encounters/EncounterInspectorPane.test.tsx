import { fireEvent, screen, within } from "@testing-library/react";
import type { ComponentProps, ReactNode } from "react";
import { EncounterInspectorPane } from "./EncounterInspectorPane";
import { encounterParticipant, renderEncounter } from "./encounterTestFixtures";
import {
  actorFixture,
  detailFixture,
  knownFact,
  numberFact,
  spellFixture,
} from "../../test/fixtures";
import type { EmbeddedReadingContext } from "../../shared/records/EmbeddedAbilityReading";
import { rootNavigation } from "../../shared/records/useReferenceReading";
import type { EncounterAbilityStateView } from "../../generated/atlas";

vi.mock("./EncounterActorReading", () => ({
  EncounterActorReading: ({
    reading,
    actorEncounter,
  }: {
    reading?: EmbeddedReadingContext;
    actorEncounter?: { conditions: ReactNode; durability: ReactNode };
  }) => {
    const controls = reading?.renderAbilityControls?.({
      record_key: reading.parent.record_key,
      embedded_path: [
        { collection: "/items", identity: { Stable: "ability000000000" } },
      ],
      field: null,
      passage: null,
      source_fingerprint: "source",
    });
    return (
      <>
        {actorEncounter?.durability}
        {actorEncounter?.conditions}
        <section aria-label="Inline ability">{controls?.activation}</section>
      </>
    );
  },
}));

vi.mock("../../api/atlasApi", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../api/atlasApi")>()),
  getEncounterConditionLifecycleActions: vi
    .fn()
    .mockResolvedValue({ encounter_revision: 1n, actions: [] }),
  getEncounterHpLifecycle: vi.fn().mockResolvedValue([]),
}));
vi.mock("./EncounterParticipantSettings", () => ({
  EncounterParticipantSettings: () => null,
}));
vi.mock("./EncounterConditionControls", () => ({
  EncounterConditionControls: () => null,
}));
vi.mock("../../shared/records/RecordReferenceDrawer", () => ({
  RecordReferenceDrawer: ({ actions }: { actions: ReactNode }) => <>{actions}</>,
}));
vi.mock("./EncounterSpellControls", () => ({
  encounterCastingAnnotations: () => undefined,
  EncounterSpellControls: ({
    disabled,
    onAction,
  }: {
    disabled: boolean;
    onAction: () => void;
  }) => (
    <button disabled={disabled} onClick={onAction}>
      Cast
    </button>
  ),
}));

it("keeps retained spell content read-only after a contextual refresh fails and enables it only after a successful retry", () => {
  const onSpellAction = vi.fn();
  const spell = detailFixture("actors:mage");
  spell.presentation.body = { kind: "spell", value: spellFixture() };
  const props = {
    disabled: false,
    encounterKey: "encounter",
    encounterRevision: 1n,
    participants: [],
    participant: encounterParticipant({
      participant_key: "one",
      participant_kind: "creature",
    }),
    preview: {
      destination: { kind: "record", navigation: rootNavigation("actors:mage") },
      detail: spell,
      loading: false,
    } satisfies Partial<ComponentProps<typeof EncounterInspectorPane>["preview"]>,
    onSpellAction,
  } as unknown as ComponentProps<typeof EncounterInspectorPane>;
  const { rerender } = renderEncounter(
    <EncounterInspectorPane
      {...props}
      preview={{ ...props.preview, error: new Error("Refresh failed") }}
    />,
  );
  expect(screen.getByRole("button", { name: "Cast" })).toBeDisabled();
  fireEvent.click(screen.getByRole("button", { name: "Cast" }));
  expect(onSpellAction).not.toHaveBeenCalled();
  rerender(
    <EncounterInspectorPane {...props} preview={{ ...props.preview, loading: true }} />,
  );
  expect(screen.getByRole("button", { name: "Cast" })).toBeDisabled();
  rerender(<EncounterInspectorPane {...props} />);
  expect(screen.getByRole("button", { name: "Cast" })).toBeEnabled();
  rerender(
    <EncounterInspectorPane
      {...props}
      preview={{ ...props.preview, detail: detailFixture("actors:mage") }}
    />,
  );
  expect(screen.queryByRole("button", { name: "Cast" })).toBeNull();
});

it("labels multiple inline options and keeps a single option beside its existing ability heading", () => {
  const presentation = detailFixture("actors:abcdefghijklmnop").presentation;
  presentation.body = { kind: "creature", value: actorFixture() };
  const abilities: EncounterAbilityStateView[] = [
    "Black",
    "Gold",
    "Green",
    "White",
  ].map((color, ordinal) => ({
    label: `${color} Aeon Star used`,
    rule_ordinal: ordinal,
    navigation: {
      record_key: "actors:abcdefghijklmnop",
      embedded_path: [
        { collection: "/items", identity: { Stable: "ability000000000" } },
      ],
      field: null,
      passage: null,
      source_fingerprint: "source",
    },
    state: {
      kind: "option",
      active: false,
      always_active: false,
      selection: null,
      choices: [],
    },
  }));
  const participant = encounterParticipant({
    participant_kind: "creature",
    record_key: "actors:abcdefghijklmnop",
    presentation,
    ability_states: abilities,
  });
  const props = {
    disabled: false,
    encounterKey: "encounter",
    encounterRevision: 1n,
    artifactGeneration: "test-generation",
    participants: [participant],
    participant,
    onAbilityState: vi.fn().mockResolvedValue(true),
    preview: { open: vi.fn(), destination: null },
  } as unknown as ComponentProps<typeof EncounterInspectorPane>;
  const { rerender } = renderEncounter(<EncounterInspectorPane {...props} />);
  const inline = within(screen.getByRole("region", { name: "Inline ability" }));
  const sidebar = within(screen.getByRole("region", { name: "Ability states" }));
  for (const ability of abilities) {
    expect(inline.getByText(ability.label)).toBeVisible();
    expect(
      inline.getByRole("switch", { name: `${ability.label} active` }),
    ).toBeVisible();
    expect(sidebar.getByRole("link", { name: ability.label })).toHaveAttribute(
      "href",
      expect.stringContaining("/records/actors:abcdefghijklmnop?at=1."),
    );
  }
  const mounted = { ...abilities[0], label: "Mounted" };
  rerender(
    <EncounterInspectorPane
      {...props}
      participant={{ ...participant, ability_states: [mounted] }}
    />,
  );
  expect(inline.queryByText("Mounted")).toBeNull();
  expect(inline.getByRole("switch", { name: "Mounted active" })).toBeVisible();
});

it("omits the encounter durability section when a hazard has no HP controls or defenses", () => {
  const presentation = detailFixture("hazards:natron").presentation;
  const hazard = {
    actor: actorFixture(),
    complex: knownFact(false),
    stealth: numberFact(20),
    detection_dc: numberFact(30),
    hardness: numberFact(0),
    broken_threshold: numberFact(0),
    defense_block: knownFact(false),
    show_saves: false,
  };
  const participant = encounterParticipant({
    participant_kind: "hazard",
    presentation: { ...presentation, body: { kind: "hazard", value: hazard } },
  });
  const props = {
    disabled: false,
    encounterKey: "encounter",
    encounterRevision: 1n,
    participants: [participant],
    participant,
    preview: { open: vi.fn(), destination: null },
  } as unknown as ComponentProps<typeof EncounterInspectorPane>;
  const { rerender } = renderEncounter(<EncounterInspectorPane {...props} />);
  expect(screen.queryByRole("region", { name: "HP and defenses" })).toBeNull();

  rerender(
    <EncounterInspectorPane
      {...props}
      participant={{
        ...participant,
        presentation: {
          ...presentation,
          body: { kind: "hazard", value: { ...hazard, show_saves: true } },
        },
      }}
    />,
  );
  const durability = screen.getByRole("region", { name: "HP and defenses" });
  expect(within(durability).getByText("Fortitude")).toBeVisible();
});
