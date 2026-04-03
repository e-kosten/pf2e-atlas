import { describe, expect, it } from "vitest";

import { deriveRecordTags } from "../../src/tags/index.js";

describe("derived tag rules: spell", () => {
  it("derives spell infiltration, scouting, navigation, and mobility tags", () => {
    expect(deriveRecordTags({
      name: "Illusory Disguise",
      category: "spell",
      subcategory: null,
      descriptionText: "You create an illusion that disguises the target and helps them pass as someone else.",
      traits: ["illusion"],
    })).toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));

    expect(deriveRecordTags({
      name: "Humanoid Form",
      category: "spell",
      subcategory: null,
      descriptionText: "You transform your appearance to that of a Small or Medium humanoid.",
      traits: ["polymorph"],
    })).toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));

    expect(deriveRecordTags({
      name: "Mimic Undead",
      category: "spell",
      subcategory: null,
      descriptionText: "You take death and wrap it about you like a cloak. Senses such as lifesense detect you as undead unless a creature succeeds at its check.",
      traits: ["concentrate", "manipulate"],
    })).toContain("disguise");

    expect(deriveRecordTags({
      name: "Mimic Undead",
      category: "spell",
      subcategory: null,
      descriptionText: "You take death and wrap it about you like a cloak. Senses such as lifesense detect you as undead unless a creature succeeds at its check.",
      traits: ["concentrate", "manipulate"],
    })).not.toContain("social_infiltration");

    expect(deriveRecordTags({
      name: "Face in the Crowd",
      category: "spell",
      subcategory: null,
      descriptionText: "While in a crowd of roughly similar creatures, your appearance becomes bland and nondescript. This counts as setting up a disguise for Impersonate and helps you go incognito among the crowd.",
      traits: ["focus", "visual"],
    })).toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));

    expect(deriveRecordTags({
      name: "Befitting Attire",
      category: "spell",
      subcategory: null,
      descriptionText: "You cloak the targets in an illusion, shaping their clothing and worn items into ones suitable for a particular occasion. This doesn't change identifying details of the targets' appearances other than their clothes.",
      traits: ["illusion", "visual"],
    })).toContain("social_infiltration");

    expect(deriveRecordTags({
      name: "Befitting Attire",
      category: "spell",
      subcategory: null,
      descriptionText: "You cloak the targets in an illusion, shaping their clothing and worn items into ones suitable for a particular occasion. This doesn't change identifying details of the targets' appearances other than their clothes.",
      traits: ["illusion", "visual"],
    })).not.toContain("disguise");

    expect(deriveRecordTags({
      name: "Inscrutable Mask",
      category: "spell",
      subcategory: null,
      descriptionText: "The mask grants you a +1 status bonus to Deception checks to Lie or Feint in areas of dim light or darkness.",
      traits: ["illusion", "shadow"],
    })).toContain("social_infiltration");

    expect(deriveRecordTags({
      name: "Inscrutable Mask",
      category: "spell",
      subcategory: null,
      descriptionText: "The mask grants you a +1 status bonus to Deception checks to Lie or Feint in areas of dim light or darkness.",
      traits: ["illusion", "shadow"],
    })).not.toContain("disguise");

    expect(deriveRecordTags({
      name: "Glimpse the Truth",
      category: "spell",
      subcategory: null,
      descriptionText: "Divine insight lets you see things as they are, unveiled by attempts to magically cloak the truth. If the check succeeds against an Illusory Disguise spell, you see the creature's true form.",
      traits: ["focus", "revelation"],
    })).not.toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));

    expect(deriveRecordTags({
      name: "Web of Influence",
      category: "spell",
      subcategory: null,
      descriptionText: "You learn the location of the nearest creature to whom the target is connected in a magical manner, such as all the targets of an Illusory Disguise spell.",
      traits: ["detection"],
    })).not.toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));

    expect(deriveRecordTags({
      name: "Portrait of the Artist",
      category: "spell",
      subcategory: null,
      descriptionText: "You change your appearance to look like a well-known artist and appear to mimic their skill.",
      traits: ["illusion", "visual"],
    })).toContain("disguise");

    expect(deriveRecordTags({
      name: "Shadow Double",
      category: "spell",
      subcategory: null,
      descriptionText: "You create an illusory duplicate of the target creature by drawing shadowy material from the Netherworld and sculpting it into a semi-solid form.",
      traits: ["illusion"],
    })).not.toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));

    expect(deriveRecordTags({
      name: "Focus Burst",
      category: "spell",
      subcategory: null,
      descriptionText: "You disrupt a creature as it tries to refocus.",
      traits: ["focus"],
    })).not.toEqual(expect.arrayContaining(["disguise", "social_infiltration"]));

    expect(deriveRecordTags({
      name: "Scouting Eye",
      category: "spell",
      subcategory: null,
      descriptionText: "You create an invisible, floating eye at a location you can see within 500 feet. It sees in all directions with your normal visual senses and continuously transmits what it sees.",
      traits: ["divination"],
    })).toContain("scouting");

    expect(deriveRecordTags({
      name: "Web of Eyes",
      category: "spell",
      subcategory: null,
      descriptionText: "You place an invisible scrying sensor on each target just above their eyes. Each sensor looks where that target looks, and you can see what each target sees.",
      traits: ["concentrate", "manipulate", "scrying"],
    })).toContain("scouting");

    expect(deriveRecordTags({
      name: "Proliferating Eyes",
      category: "spell",
      subcategory: null,
      descriptionText: "You implant an invisible, magical eye sensor on the target's body. The eye has sight and vision, and you can perceive through the magical eye sensor from afar.",
      traits: ["concentrate", "manipulate", "scrying"],
    })).toContain("scouting");

    expect(deriveRecordTags({
      name: "Painted Scout",
      category: "spell",
      subcategory: null,
      descriptionText: "You press your hand to the stone, causing hand-drawn scouts to spread out from your fingers. As long as you Sustain the Spell, you can see, hear, and smell through the scouts.",
      traits: ["concentrate", "manipulate", "scrying"],
    })).toContain("scouting");

    expect(deriveRecordTags({
      name: "Know the Way",
      category: "spell",
      subcategory: null,
      descriptionText: "In your mind's eye, you magically reorient yourself. You immediately know which direction is north, and you can choose a location you were at within the last 24 hours and learn what direction it lies.",
      traits: ["cantrip", "concentrate", "detection", "manipulate"],
    })).toContain("navigation");

    expect(deriveRecordTags({
      name: "Wanderer's Guide",
      category: "spell",
      subcategory: null,
      descriptionText: "You call upon the beyond to guide your route. When you Cast this Spell, choose a destination; you receive an inspired route to that destination, allowing you and allies who travel overland with you to reduce the movement penalty from difficult terrain by half for the duration.",
      traits: ["concentrate", "manipulate"],
    })).toEqual(expect.arrayContaining(["navigation", "mobility"]));

    expect(deriveRecordTags({
      name: "Spiritual Transport",
      category: "spell",
      subcategory: null,
      descriptionText: "You call upon the spiritual energies surrounding you to teleport you to an unoccupied space within range that you can see. Any items you're wearing and holding come with you. Heightened (7th) You don't need to be able to see your destination, as long as you have been there in the past and know its relative direction from you.",
      traits: ["concentrate", "manipulate", "teleportation"],
    })).toEqual(expect.arrayContaining(["navigation", "mobility"]));

    expect(deriveRecordTags({
      name: "Agile Feet",
      category: "spell",
      subcategory: null,
      descriptionText: "The blessings of your god make your feet faster and your movements more fluid. You gain a +5-foot status bonus to your Speed and ignore difficult terrain.",
      traits: ["focus"],
    })).toContain("mobility");

    expect(deriveRecordTags({
      name: "Gecko Grip",
      category: "spell",
      subcategory: null,
      descriptionText: "Tiny clinging hairs sprout across the creature's hands and feet, offering purchase on nearly any surface. The target gains a climb Speed equal to its Speed.",
      traits: ["concentrate", "manipulate"],
    })).toContain("mobility");

    expect(deriveRecordTags({
      name: "Jump",
      category: "spell",
      subcategory: null,
      descriptionText: "Your legs surge with strength, ready to leap high and far. You jump 30 feet in any direction without touching the ground.",
      traits: ["move"],
    })).toContain("mobility");

    expect(deriveRecordTags({
      name: "Dragon Wings",
      category: "spell",
      subcategory: null,
      descriptionText: "Leathery wings sprout from your back, giving you a fly Speed of 60 feet or your Speed, whichever is faster.",
      traits: ["focus", "morph"],
    })).toContain("mobility");

    expect(deriveRecordTags({
      name: "Unfettered Movement",
      category: "spell",
      subcategory: null,
      descriptionText: "You repel effects that would hinder a creature or slow its movement. While under this spell's effect, the target ignores effects that would give them a circumstance penalty to Speed.",
      traits: ["concentrate", "manipulate"],
    })).toContain("mobility");

    expect(deriveRecordTags({
      name: "Leaden Steps",
      category: "spell",
      subcategory: null,
      descriptionText: "You partially transform a foe's feet into unwieldy slabs of metal, slowing their steps. The target attempts a Fortitude saving throw.",
      traits: ["manipulate"],
    })).not.toContain("mobility");

    expect(deriveRecordTags({
      name: "Aerial Form",
      category: "spell",
      subcategory: null,
      descriptionText: "You harness your mastery of primal forces to reshape your body into a Medium flying animal battle form. While in this battle form, you gain the following statistics and abilities.",
      traits: ["polymorph"],
    })).not.toContain("mobility");

    expect(deriveRecordTags({
      name: "Blind Eye",
      category: "spell",
      subcategory: null,
      descriptionText: "You enchant a single object, preventing it from being used for magical observation. The item can't be used to cast scrying spells.",
      traits: ["abjuration"],
    })).not.toContain("scouting");
  });

  it("derives spell transformation tags and promotes the family tag", () => {
    expect(deriveRecordTags({
      name: "Metamorphosis",
      category: "spell",
      subcategory: null,
      descriptionText: "You transform your body into a monstrous new shape that gains new senses and attacks.",
      traits: ["polymorph", "transmutation"],
    })).toContain("transformation");

    expect(deriveRecordTags({
      name: "Aerial Form",
      category: "spell",
      subcategory: null,
      descriptionText: "You harness your mastery of primal forces to reshape your body into a Medium flying animal battle form. While in this battle form, you gain the following statistics and abilities.",
      traits: ["polymorph"],
    })).toEqual(expect.arrayContaining(["transformation", "battle_form"]));

    expect(deriveRecordTags({
      name: "Tiger Form",
      category: "spell",
      subcategory: null,
      descriptionText: "You assume a tiger battle form. While in this battle form, you gain the following statistics and abilities.",
      traits: ["polymorph"],
    })).toEqual(expect.arrayContaining(["transformation", "battle_form"]));
    expect(deriveRecordTags({
      name: "Tiger Form",
      category: "spell",
      subcategory: null,
      descriptionText: "You assume a tiger battle form. While in this battle form, you gain the following statistics and abilities.",
      traits: ["polymorph"],
    })).not.toContain("animal_form");

    expect(deriveRecordTags({
      name: "Animal Form",
      category: "spell",
      subcategory: null,
      descriptionText: "You transform yourself into a Medium animal battle form. While in this battle form, you gain the following statistics and abilities.",
      traits: ["polymorph"],
    })).toEqual(expect.arrayContaining(["transformation", "battle_form", "animal_form"]));

    expect(deriveRecordTags({
      name: "Elemental Form",
      category: "spell",
      subcategory: null,
      descriptionText: "You transform into a Medium elemental battle form. While in this battle form, you gain the following statistics and abilities.",
      traits: ["polymorph"],
    })).toEqual(expect.arrayContaining(["transformation", "battle_form", "elemental_form"]));

    expect(deriveRecordTags({
      name: "Humanoid Form",
      category: "spell",
      subcategory: null,
      descriptionText: "You transform your appearance to that of a Small or Medium humanoid.",
      traits: ["polymorph"],
    })).toContain("transformation");

    expect(deriveRecordTags({
      name: "Illusory Disguise",
      category: "spell",
      subcategory: null,
      descriptionText: "You create an illusion that disguises the target and helps them pass as someone else.",
      traits: ["illusion"],
    })).not.toContain("transformation");

    expect(deriveRecordTags({
      name: "Enlarge",
      category: "spell",
      subcategory: null,
      descriptionText: "You increase the target's size and grant a bonus to melee damage.",
      traits: ["transmutation"],
    })).not.toContain("transformation");

    expect(deriveRecordTags({
      name: "Summon Construct",
      category: "spell",
      subcategory: null,
      descriptionText: "You summon a construct that appears in an unoccupied space within range.",
      traits: ["conjuration", "summon"],
    })).not.toContain("transformation");

    expect(deriveRecordTags({
      name: "Animated Armor",
      category: "spell",
      subcategory: null,
      descriptionText: "You animate a suit of armor until it can move on its own.",
      traits: ["transmutation"],
    })).not.toContain("transformation");
  });

  it("derives spell security, communication, and countermagic tags", () => {
    expect(deriveRecordTags({
      name: "Alarm",
      category: "spell",
      subcategory: null,
      descriptionText: "You ward an area to alert you when creatures enter without your permission. Whenever a creature enters the spell's area without speaking the password, alarm sends your choice of a mental alert or an audible alarm.",
      traits: ["concentrate", "manipulate"],
    })).toContain("alarm");

    expect(deriveRecordTags({
      name: "Ravenous Portal",
      category: "spell",
      subcategory: null,
      descriptionText: "You place a ward upon the door that triggers when a creature attempts to open, unlock, or destroy the door. When that happens, the door transforms into a hostile mimic.",
      traits: ["concentrate", "manipulate"],
    })).not.toContain("alarm");

    expect(deriveRecordTags({
      name: "Dream Message",
      category: "spell",
      subcategory: null,
      descriptionText: "You send a message to your target's dream. The message is one-way, up to 1 minute of speech. If the target is asleep, they receive the message instantly.",
      traits: ["concentrate", "manipulate", "mental"],
    })).toContain("message_delivery");

    expect(deriveRecordTags({
      name: "Telepathic Demand",
      category: "spell",
      subcategory: null,
      descriptionText: "You send the target a message of 25 words or fewer, and it can respond immediately with its own message of 25 words or fewer. Your message is insidious and has the effect of a Suggestion spell.",
      traits: ["concentrate", "incapacitation", "linguistic", "manipulate", "mental"],
    })).not.toContain("message_delivery");

    expect(deriveRecordTags({
      name: "Sky Signs",
      category: "spell",
      subcategory: null,
      descriptionText: "You emblazon a message across the sky itself, using clouds or auroras to coordinate distant allies.",
      traits: ["air", "illusion", "visual"],
    })).toContain("signaling");

    expect(deriveRecordTags({
      name: "Dispel Magic",
      category: "spell",
      subcategory: null,
      descriptionText: "You unravel the magic behind one spell or magical effect. Attempt to counteract the target spell.",
      traits: ["concentrate", "manipulate"],
    })).toContain("countermagic");
  });

  it("derives spell support and warding tags", () => {
    expect(deriveRecordTags({
      name: "Healing Hymn",
      category: "spell",
      subcategory: null,
      descriptionText: "The hymn restores hit points and grants fast healing to the target.",
      traits: ["healing"],
    })).toContain("healing_support");

    expect(deriveRecordTags({
      name: "Merciful Renewal",
      category: "spell",
      subcategory: null,
      descriptionText: "You heal the target and let them recover hit points more quickly.",
      traits: [],
    })).toContain("healing_support");

    expect(deriveRecordTags({
      name: "Searing Burst",
      category: "spell",
      subcategory: null,
      descriptionText: "A blast of fire scorches foes in the area.",
      traits: ["fire"],
    })).not.toContain("healing_support");

    expect(deriveRecordTags({
      name: "Delay Affliction",
      category: "spell",
      subcategory: null,
      descriptionText: "You delay the affliction and remove a condition from the target.",
      traits: ["healing"],
    })).toContain("condition_support");

    expect(deriveRecordTags({
      name: "Cleansing Pulse",
      category: "spell",
      subcategory: null,
      descriptionText: "You counteract an affliction, remove a condition, and cure poison.",
      traits: [],
    })).toContain("condition_support");

    expect(deriveRecordTags({
      name: "Alarm",
      category: "spell",
      subcategory: null,
      descriptionText: "You ward an area to alert you when creatures enter without your permission. Whenever a creature enters the spell's area without speaking the password, alarm sends your choice of a mental alert or an audible alarm.",
      traits: ["concentrate", "manipulate"],
    })).not.toContain("protective_ward");

    expect(deriveRecordTags({
      name: "Sanctuary Circle",
      category: "spell",
      subcategory: null,
      descriptionText: "A warding circle protects the target and grants a bonus to AC while they stand within the sanctified space.",
      traits: ["abjuration"],
    })).toContain("protective_ward");

    expect(deriveRecordTags({
      name: "Defended by Spirits",
      category: "spell",
      subcategory: null,
      descriptionText: "Spirits defend the target and create a protective boundary around them.",
      traits: ["divine"],
    })).toContain("protective_ward");

    expect(deriveRecordTags({
      name: "Breath of Life",
      category: "spell",
      subcategory: null,
      descriptionText: "You stabilize the target and prevent it from dying.",
      traits: ["healing"],
    })).toContain("death_prevention");

    expect(deriveRecordTags({
      name: "Revival",
      category: "spell",
      subcategory: null,
      descriptionText: "Your magic returns the creature to life and brings it back from death.",
      traits: ["healing"],
    })).toContain("death_prevention");

    expect(deriveRecordTags({
      name: "Moonlit Veil",
      category: "spell",
      subcategory: null,
      descriptionText: "Soft moonlight cloaks the target in shimmering silver light.",
      traits: ["illusion"],
    })).not.toContain("death_prevention");

    expect(deriveRecordTags({
      name: "Energy Aegis",
      category: "spell",
      subcategory: null,
      descriptionText: "The target gains resistance 10 to fire and acid for 1 hour.",
      traits: ["abjuration"],
    })).toContain("resistance_support");

    expect(deriveRecordTags({
      name: "Storm Shell",
      category: "spell",
      subcategory: null,
      descriptionText: "The spell grants resistance to electricity and sonic damage.",
      traits: ["abjuration"],
    })).toContain("resistance_support");

    expect(deriveRecordTags({
      name: "Searing Blade",
      category: "spell",
      subcategory: null,
      descriptionText: "You surround a weapon in flames and deal fire damage on a hit.",
      traits: ["fire"],
    })).not.toContain("resistance_support");
  });
});
