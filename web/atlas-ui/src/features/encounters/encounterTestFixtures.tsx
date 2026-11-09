import { render } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import type { EncounterParticipantView } from "../../generated/atlas";

export function encounterParticipant(
  overrides: Partial<EncounterParticipantView> = {},
): EncounterParticipantView {
  return {
    effective_max_hp: {
      state: overrides.max_hp === undefined ? "missing" : "value",
      value: overrides.max_hp === undefined ? null : overrides.max_hp,
      adjustment: null,
    },
    manual_trackers: [],
    knockout_options: [],
    participant_key: "actor",
    display_name: "Actor",
    participant_kind: "pc",
    revision: 1,
    hp_origin: "explicit",
    variant_origin: "explicit",
    participant_variant: "normal",
    status: "active",
    position: 1,
    side: "pc",
    initiative_order: 1,
    temporary_hp: 0,
    defeated: false,
    hazard_disabled: false,
    cast_unavailable_reason: null,
    hidden: false,
    note_hint: null,
    track_dying: true,
    mortality: "alive",
    base_death_threshold: 4,
    persistent_damage_groups: [],
    conditions: [],
    ...overrides,
  };
}

export function renderEncounter(ui: ReactNode) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(ui, {
    wrapper: ({ children }) => (
      <QueryClientProvider client={client}>{children}</QueryClientProvider>
    ),
  });
}
