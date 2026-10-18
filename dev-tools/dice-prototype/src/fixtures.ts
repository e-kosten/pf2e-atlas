// Small source-backed fixtures, not a product source reader. See README provenance.
export type Actor = {
  id: string;
  name: string;
  perception: number;
  stealth: number;
  stealthNote: string;
};
export const actors: Actor[] = [
  {
    id: "warrior-1",
    name: "Kobold Warrior 1",
    perception: 3,
    stealth: 5,
    stealthNote: "Listed skill",
  },
  {
    id: "warrior-2",
    name: "Kobold Warrior 2",
    perception: 3,
    stealth: 5,
    stealthNote: "Listed skill",
  },
  {
    id: "scout",
    name: "Kobold Scout",
    perception: 8,
    stealth: 7,
    stealthNote: "Listed skill",
  },
  {
    id: "ogre",
    name: "Ogre Warrior",
    perception: 5,
    stealth: -1,
    stealthNote: "Unlisted skill · Dexterity −1",
  },
];
export const terrain = [
  { low: 1, high: 3, text: "Plains" },
  { low: 4, high: 5, text: "Desert" },
  { low: 6, high: 7, text: "Aquatic (lake, sea, or ocean)" },
  { low: 8, high: 9, text: "Mountain" },
  { low: 10, high: 11, text: "Forest" },
  { low: 12, high: 12, text: "Swamp" },
  { low: 13, high: 13, text: "Arctic" },
  { low: 14, high: 20, text: "Match the previous hex" },
];
export const signed = (n: number) => (n < 0 ? `−${Math.abs(n)}` : `+${n}`);
export const formula = (n: number) => `1d20${n < 0 ? n : `+${n}`}`;
