import type { EncounterSpellResourceView } from "../../generated/atlas";

export function spellResourceExhausted(resource: EncounterSpellResourceView) {
  return resource.remaining <= 0;
}
export function spellResourceCount(resource: EncounterSpellResourceView) {
  const remaining = resource.remaining;
  const capacity = resource.capacity;
  return `${remaining}/${capacity}`;
}
