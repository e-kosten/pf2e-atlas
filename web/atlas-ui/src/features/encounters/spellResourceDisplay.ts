import type { EncounterSpellResourceView } from "../../generated/atlas";

// Generated i64 bindings use bigint, while JSON transports these bounded counts
// as numbers. Normalize at this presentation boundary before comparing them.
export function spellResourceExhausted(resource: EncounterSpellResourceView) {
  return Number(resource.remaining) <= 0;
}
export function spellResourceCount(resource: EncounterSpellResourceView) {
  const remaining = Number(resource.remaining);
  const capacity = Number(resource.capacity);
  return `${remaining}/${capacity}`;
}
