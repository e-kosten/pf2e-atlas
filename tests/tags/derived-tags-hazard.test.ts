import { describe, expect, it } from "vitest";

import { deriveRecordTags } from "../../src/tags/index.js";

describe("derived tag rules: hazard", () => {
  it("derives hazard function, impact, and environmental tags", () => {
    expect(deriveRecordTags({
      name: "Alarm Ward",
      category: "hazard",
      subcategory: null,
      descriptionText: "A silent ward flares when a creature crosses the threshold, raising the alarm and alerting nearby guards.",
      traits: ["magical"],
    })).toContain("alarm");

    expect(deriveRecordTags({
      name: "Snaring Glyph",
      category: "hazard",
      subcategory: null,
      descriptionText: "A glowing sigil lashes out with force bands to bind intruders in place until they Escape. The creature becomes Restrained.",
      traits: ["magical"],
      references: [
        {
          recordKey: "actionspf2e:escape-1",
          packName: "actionspf2e",
          name: "Escape",
          category: "rule",
          subcategory: "action",
          traits: ["attack"],
        },
        {
          recordKey: "conditionitems:restrained-1",
          packName: "conditionitems",
          name: "Restrained",
          category: "rule",
          subcategory: "condition",
          traits: [],
        },
      ],
    })).toContain("restraint_capture");

    expect(deriveRecordTags({
      name: "Locking Door",
      category: "hazard",
      subcategory: "haunt",
      descriptionText: "A door slams shut and locks.",
      traits: ["haunt"],
    })).toContain("barrier_lockdown");

    expect(deriveRecordTags({
      name: "Falling Portcullis Trap",
      category: "hazard",
      subcategory: "trap",
      descriptionText: "An iron portcullis drops from the ceiling when a pressure plate is triggered.",
      traits: ["mechanical", "trap"],
    })).toContain("barrier_lockdown");

    expect(deriveRecordTags({
      name: "Slamming Gate",
      category: "hazard",
      subcategory: "trap",
      descriptionText: "A pressure-sensitive floor panel causes a spiked door to slam down into place from the ceiling, skewering anyone caught underneath.",
      traits: ["mechanical", "trap"],
    })).toContain("barrier_lockdown");

    expect(deriveRecordTags({
      name: "Battering Door",
      category: "hazard",
      subcategory: "trap",
      descriptionText: "Animated door slams into trespassers.",
      traits: ["trap"],
    })).not.toContain("barrier_lockdown");

    expect(deriveRecordTags({
      name: "Cave-in",
      category: "hazard",
      subcategory: null,
      descriptionText: "The tunnel collapses, filling the passage with stone.",
      traits: ["environmental"],
    })).toContain("barrier_lockdown");

    expect(deriveRecordTags({
      name: "Consuming Cabinet",
      category: "hazard",
      subcategory: "trap",
      descriptionText: "Animated cabinet imprisons intruders. The cabinet has two compartments; each can hold a single Medium creature.",
      traits: ["mechanical", "trap"],
    })).toContain("barrier_lockdown");

    expect(deriveRecordTags({
      name: "Explosive Barrels",
      category: "hazard",
      subcategory: null,
      descriptionText: "Wooden barrels marked with an oil-drop symbol catch fire and explode.",
      traits: [],
    })).toContain("fire_hazard");

    expect(deriveRecordTags({
      name: "Acid Mist",
      category: "hazard",
      subcategory: "trap",
      descriptionText: "A cloud of caustic acid sprays across the chamber and corrodes exposed gear.",
      traits: ["mechanical", "trap"],
    })).toContain("acid_hazard");

    expect(deriveRecordTags({
      name: "Gas Trap",
      category: "hazard",
      subcategory: "trap",
      descriptionText: "A spring slams and locks the room's door before four hidden gas vents begin pumping poison gas into the chamber.",
      traits: ["mechanical", "trap"],
    })).toContain("poison_hazard");

    expect(deriveRecordTags({
      name: "Gas Trap",
      category: "hazard",
      subcategory: "trap",
      descriptionText: "A spring slams and locks the room's door before four hidden gas vents begin pumping poison gas into the chamber.",
      traits: ["mechanical", "trap"],
    })).not.toContain("acid_hazard");

    expect(deriveRecordTags({
      name: "Thin Ice",
      category: "hazard",
      subcategory: null,
      descriptionText: "The freezing floor gives way into a sheet of thin ice above frigid water.",
      traits: ["environmental"],
    })).toContain("cold_hazard");

    expect(deriveRecordTags({
      name: "Crumbling Archway",
      category: "hazard",
      subcategory: null,
      descriptionText: "The archway collapses when the stone supports are struck.",
      traits: ["environmental"],
    })).not.toContain("cold_hazard");

    expect(deriveRecordTags({
      name: "Electric Latch Rune",
      category: "hazard",
      subcategory: "trap",
      descriptionText: "Lightning crackles through the rune and a shock surges across the metal latch.",
      traits: ["magical", "trap"],
    })).toContain("electric_hazard");

    expect(deriveRecordTags({
      name: "Clockwork Trap",
      category: "hazard",
      subcategory: "trap",
      descriptionText: "The gears grind loudly as the mechanism locks into place.",
      traits: ["mechanical", "trap"],
    })).not.toContain("electric_hazard");

    expect(deriveRecordTags({
      name: "Buzzing Latch Rune",
      category: "hazard",
      subcategory: "trap",
      descriptionText: "A shrieking resonance and deafening sound burst from the rune, rattling the chamber.",
      traits: ["magical", "trap"],
    })).toContain("sound_hazard");

    expect(deriveRecordTags({
      name: "Alarm Ward",
      category: "hazard",
      subcategory: null,
      descriptionText: "A silent ward flares when a creature crosses the threshold, raising the alarm and alerting nearby guards.",
      traits: ["magical"],
    })).toContain("alarm");

    expect(deriveRecordTags({
      name: "Alarm Ward",
      category: "hazard",
      subcategory: null,
      descriptionText: "A silent ward flares when a creature crosses the threshold, raising the alarm and alerting nearby guards.",
      traits: ["magical"],
    })).not.toContain("sound_hazard");

    expect(deriveRecordTags({
      name: "Ballista Defense",
      category: "hazard",
      subcategory: null,
      descriptionText: "A ballista armed with a massive bolt fires on a creature approaching the palace.",
      traits: [],
    })).not.toContain("fire_hazard");

    expect(deriveRecordTags({
      name: "Spitting Daffodil",
      category: "hazard",
      subcategory: null,
      descriptionText: "Daffodils spit acidic nectar at non-plant creatures in the area.",
      traits: [],
    })).not.toContain("poison_hazard");

    expect(deriveRecordTags({
      name: "Drowning Pit",
      category: "hazard",
      subcategory: "trap",
      descriptionText: "A trapdoor covers a 10-foot-square pit that's 30 feet deep and has 5 feet of water at the bottom.",
      traits: ["mechanical", "trap"],
    })).toContain("pitfall");

    expect(deriveRecordTags({
      name: "Pitiless Hall",
      category: "hazard",
      subcategory: null,
      descriptionText: "A merciless illusion torments intruders with visions of an unforgiving tyrant.",
      traits: ["magical"],
    })).not.toContain("pitfall");

    expect(deriveRecordTags({
      name: "Collapsing Bridge",
      category: "hazard",
      subcategory: "trap",
      descriptionText: "Metal supports twist and shear off the bridge, causing stretches of the structure to collapse.",
      traits: ["mechanical", "trap"],
    })).toContain("collapse_hazard");

    expect(deriveRecordTags({
      name: "Benefactor's End",
      category: "hazard",
      subcategory: "haunt",
      descriptionText: "A spectral assailant drives a stake into a dying vampire's heart; the vampire collapses and releases a surge of void energy.",
      traits: ["haunt", "magical"],
    })).not.toContain("collapse_hazard");

    expect(deriveRecordTags({
      name: "Rushing Wind",
      category: "hazard",
      subcategory: null,
      descriptionText: "A raging wind sucks creatures in the area toward the aiudara.",
      traits: ["environmental"],
    })).toContain("forced_movement");

    expect(deriveRecordTags({
      name: "Mental Scream Trap",
      category: "hazard",
      subcategory: "trap",
      descriptionText: "A psychic scream disorients creatures in the area and leaves them overwhelmed with fear.",
      traits: ["magical"],
    })).toContain("mental_impairment");

    expect(deriveRecordTags({
      name: "Mental Assault",
      category: "hazard",
      subcategory: "trap",
      descriptionText: "Magic sigils hidden in the grain of the wooden door trigger a magic trap that damages the mind of anyone attempting to pick the door's lock.",
      traits: ["magical", "mechanical", "trap"],
    })).toContain("mental_impairment");

    expect(deriveRecordTags({
      name: "Images of Failure",
      category: "hazard",
      subcategory: "trap",
      descriptionText: "Psychically enhanced illusions flood the minds of creatures in the hallway with memories of their past failures.",
      traits: ["magical", "trap"],
    })).toContain("mental_impairment");

    expect(deriveRecordTags({
      name: "Phantasmagoric Fog Trap",
      category: "hazard",
      subcategory: "trap",
      descriptionText: "Cloying violet mist billows from the undergrowth, filling the heads of any creatures inside with maddening visions of dark moonless nights and twisted malignant trees.",
      traits: ["magical", "trap"],
    })).toContain("mental_impairment");

    expect(deriveRecordTags({
      name: "Punish Defiance",
      category: "hazard",
      subcategory: "trap",
      descriptionText: "A magical mural threatens uncooperative inmates, causing mental trauma to those who defy its commands.",
      traits: ["magical", "trap"],
    })).toContain("mental_impairment");

    expect(deriveRecordTags({
      name: "Hampering Web",
      category: "hazard",
      subcategory: null,
      descriptionText: "Semitransparent sheets of webbing span the entryway, ready to capture small insects or hamper larger creatures that pass through.",
      traits: [],
    })).toContain("mobility_impairment");

    expect(deriveRecordTags({
      name: "Hampering Web",
      category: "hazard",
      subcategory: null,
      descriptionText: "Semitransparent sheets of webbing span the entryway, ready to capture small insects or hamper larger creatures that pass through.",
      traits: [],
    })).not.toContain("forced_movement");

    expect(deriveRecordTags({
      name: "Paralysis Trap",
      category: "hazard",
      subcategory: null,
      descriptionText: "Magical symbols carved in the doorframe magically ward this door; anyone attempting to pick the lock is Paralyzed and wracked with pain.",
      traits: ["magical"],
    })).toContain("mobility_impairment");

    expect(deriveRecordTags({
      name: "Psychic Wave",
      category: "hazard",
      subcategory: "trap",
      descriptionText: "A wave of psychic energy ripples through the room, violently prying into the minds of creatures in the area and alerting Kemnebi to the presence of intruders.",
      traits: ["magical"],
    })).not.toContain("mental_impairment");
  });
});
