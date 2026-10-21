import { render } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import type { EncounterParticipantView } from "../../generated/atlas";

export function encounterParticipant(
  overrides: Partial<EncounterParticipantView> = {},
): EncounterParticipantView {
  return {
    participant_key: "actor",
    display_name: "Actor",
    participant_kind: "pc",
    revision: 1n,
    hp_origin: "explicit",
    variant_origin: "explicit",
    participant_variant: "normal",
    status: "active",
    position: 1n,
    side: "pc",
    initiative_order: 1n,
    temporary_hp: 0n,
    defeated: false,
    hazard_disabled: false,
    cast_unavailable_reason: null,
    hidden: false,
    note_hint: null,
    track_dying: true,
    mortality: "alive",
    base_death_threshold: 4n,
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
