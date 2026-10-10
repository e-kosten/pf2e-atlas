import * as R from "remeda";
import { RANGE_TRAITS } from "@item/base/data/values.ts";
import { sluggify } from "@util";
import { OtherArmorTag } from "@item/armor/types.ts";

const baseTraits = {
    fire: "PF2E.Fire",
    shared: "PF2E.Original",
};
const rangeTraits = R.mapToObj(RANGE_TRAITS, (trait) => [trait, `PF2E.Trait${sluggify(trait, { camel: "bactrian" })}`]);
const equipmentTraits = {
    ...baseTraits,
    shared: "PF2E.Override",
    "deadly-2d10": "PF2E.Deadly",
    undocumented: "PF2E.Undocumented",
    missing: "PF2E.MissingLabel",
};
const weaponTraits = {
    ...baseTraits,
    ...rangeTraits,
    "deadly-2d10": "PF2E.Deadly",
};
const actionTraits = R.pick(equipmentTraits, ["shared", "fire"]);
const effectTraits = R.omit(actionTraits, [...R.keys(R.pick(baseTraits, ["fire"]))]);
const otherArmorTags: Record<OtherArmorTag, string> = { fire: "PF2E.OtherFire", shoddy: "PF2E.Shoddy" };
const rangeDescriptions = R.mapToObj(RANGE_TRAITS, (trait) => [trait, "PF2E.RangeDescription"]);
const traitDescriptions = {
    fire: "PF2E.FireDescription",
    shared: "PF2E.SharedDescription",
    "deadly-2d10": "PF2E.DeadlyDescription",
    missing: "PF2E.MissingDescription",
    ...rangeDescriptions,
};
export { equipmentTraits, weaponTraits, actionTraits, effectTraits, otherArmorTags, traitDescriptions };
