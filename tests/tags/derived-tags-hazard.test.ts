import { describe, expect, it } from "vitest";

import { deriveRecordTags } from "../../src/tags/runtime.js";

describe("derived tag rules: hazard", () => {
  it("derives hazard mechanism tags", () => {
    expect(
      deriveRecordTags({
        name: "Kharnas's Lesser Glyph",
        category: "hazard",
        subcategory: "trap",
        descriptionText:
          "A concealed glyph triggers when a creature crosses the threshold, releasing a blast of force.",
        traits: ["magical", "trap"],
      }),
    ).toContain("ward_trigger");

    expect(
      deriveRecordTags({
        name: "Mask Summoning Rune",
        category: "hazard",
        subcategory: "trap",
        descriptionText: "When touched, the rune activates and calls forth the mask guardian.",
        traits: ["magical", "trap"],
      }),
    ).toContain("ward_trigger");

    expect(
      deriveRecordTags({
        name: "Hidden Ward Sigil",
        category: "hazard",
        subcategory: "trap",
        descriptionText: "An etched rune lies hidden in the stone and flares when disturbed.",
        traits: ["magical", "trap"],
      }),
    ).toContain("ward_trigger");

    expect(
      deriveRecordTags({
        name: "Pressure Plate Launcher",
        category: "hazard",
        subcategory: "trap",
        descriptionText: "Stepping on the pressure plate depresses the floor panel and triggers a volley of darts.",
        traits: ["mechanical", "trap"],
      }),
    ).toContain("pressure_trigger");

    expect(
      deriveRecordTags({
        name: "Tripwire Spear Rack",
        category: "hazard",
        subcategory: "trap",
        descriptionText: "A taut wire across the corridor trips intruders and triggers a spring-loaded spear rack.",
        traits: ["mechanical", "trap"],
      }),
    ).toContain("tripwire_trigger");

    expect(
      deriveRecordTags({
        name: "Warding Bell",
        category: "hazard",
        subcategory: null,
        descriptionText: "A protective ward bell watches over the shrine without any trigger rune or glyph.",
        traits: ["magical"],
      }),
    ).not.toContain("ward_trigger");

    expect(
      deriveRecordTags({
        name: "Poisoned Secret Door Trap",
        category: "hazard",
        subcategory: "trap",
        descriptionText: "The secret door slams shut and locks the doorway, trapping the triggering creature inside.",
        traits: ["mechanical", "trap"],
      }),
    ).toContain("threshold_lockdown");

    expect(
      deriveRecordTags({
        name: "Electrified Gate",
        category: "hazard",
        subcategory: "trap",
        descriptionText: "The gate seals the entrance and bars the passage while shocking intruders.",
        traits: ["mechanical", "trap"],
      }),
    ).toContain("threshold_lockdown");

    expect(
      deriveRecordTags({
        name: "Clockwork Trap",
        category: "hazard",
        subcategory: "trap",
        descriptionText: "The gears grind loudly as the mechanism locks into place.",
        traits: ["mechanical", "trap"],
      }),
    ).not.toContain("threshold_lockdown");

    expect(
      deriveRecordTags({
        name: "Button Mash",
        category: "hazard",
        subcategory: "trap",
        descriptionText: "Each button press on the hidden panel activates a different blast.",
        traits: ["mechanical", "trap"],
      }),
    ).toContain("control_interface");

    expect(
      deriveRecordTags({
        name: "Blast Tumbler",
        category: "hazard",
        subcategory: "trap",
        descriptionText: "A tumbler inside the console must be set before the hazard can be operated.",
        traits: ["mechanical", "trap"],
      }),
    ).toContain("control_interface");

    expect(
      deriveRecordTags({
        name: "Clockwork Valve",
        category: "hazard",
        subcategory: "trap",
        descriptionText: "The gears grind and the mechanism locks into place without any visible controls.",
        traits: ["mechanical", "trap"],
      }),
    ).not.toContain("control_interface");

    expect(
      deriveRecordTags({
        name: "Hallowed Wheel",
        category: "hazard",
        subcategory: "trap",
        descriptionText:
          "An ornate wheel is mounted on a pole and controlled by a lever that can be triggered manually or by a nearby sensor.",
        traits: ["magical", "mechanical", "trap"],
      }),
    ).toContain("control_interface");

    expect(
      deriveRecordTags({
        name: "Keystone Trap",
        category: "hazard",
        subcategory: "trap",
        descriptionText: "A lever on the stone block slides the vault into view, exposing the hidden trap.",
        traits: ["mechanical", "trap"],
      }),
    ).toContain("control_interface");

    expect(
      deriveRecordTags({
        name: "Skewering Hall",
        category: "hazard",
        subcategory: "trap",
        descriptionText:
          "A dozen spears hidden within sockets in the walls thrust into the hall when the pressure plate is triggered.",
        traits: ["mechanical", "trap"],
      }),
    ).not.toContain("control_interface");

    expect(
      deriveRecordTags({
        name: "Air Rift",
        category: "hazard",
        subcategory: null,
        descriptionText: "A violent rift opens in the air, tearing a hole in reality and spilling out planar wind.",
        traits: ["magical"],
      }),
    ).toContain("planar_breach");

    expect(
      deriveRecordTags({
        name: "Nightmare Portal",
        category: "hazard",
        subcategory: "trap",
        descriptionText: "An unstable portal opens into a nightmare realm through a tear in the fabric of reality.",
        traits: ["magical", "trap"],
      }),
    ).toContain("planar_breach");

    expect(
      deriveRecordTags({
        name: "Portal Ward",
        category: "hazard",
        subcategory: null,
        descriptionText: "A protective ward prevents portal travel and keeps the chamber sealed.",
        traits: ["magical"],
      }),
    ).not.toContain("planar_breach");
  });

  it("derives hazard function, impact, and environmental tags", () => {
    expect(
      deriveRecordTags({
        name: "Alarm Ward",
        category: "hazard",
        subcategory: null,
        descriptionText:
          "A silent ward flares when a creature crosses the threshold, raising the alarm and alerting nearby guards.",
        traits: ["magical"],
      }),
    ).toContain("alarm");

    expect(
      deriveRecordTags({
        name: "Snaring Glyph",
        category: "hazard",
        subcategory: null,
        descriptionText:
          "A glowing sigil lashes out with force bands to bind intruders in place until they Escape. The creature becomes Restrained.",
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
      }),
    ).toContain("restraint_capture");

    expect(
      deriveRecordTags({
        name: "Locking Door",
        category: "hazard",
        subcategory: "haunt",
        descriptionText: "A door slams shut and locks.",
        traits: ["haunt"],
      }),
    ).toContain("barrier_lockdown");

    expect(
      deriveRecordTags({
        name: "Falling Portcullis Trap",
        category: "hazard",
        subcategory: "trap",
        descriptionText: "An iron portcullis drops from the ceiling when a pressure plate is triggered.",
        traits: ["mechanical", "trap"],
      }),
    ).toContain("barrier_lockdown");

    expect(
      deriveRecordTags({
        name: "Slamming Gate",
        category: "hazard",
        subcategory: "trap",
        descriptionText:
          "A pressure-sensitive floor panel causes a spiked door to slam down into place from the ceiling, skewering anyone caught underneath.",
        traits: ["mechanical", "trap"],
      }),
    ).toContain("barrier_lockdown");

    expect(
      deriveRecordTags({
        name: "Battering Door",
        category: "hazard",
        subcategory: "trap",
        descriptionText: "Animated door slams into trespassers.",
        traits: ["trap"],
      }),
    ).not.toContain("barrier_lockdown");

    expect(
      deriveRecordTags({
        name: "Cave-in",
        category: "hazard",
        subcategory: null,
        descriptionText: "The tunnel collapses, filling the passage with stone.",
        traits: ["environmental"],
      }),
    ).toContain("barrier_lockdown");

    expect(
      deriveRecordTags({
        name: "Consuming Cabinet",
        category: "hazard",
        subcategory: "trap",
        descriptionText:
          "Animated cabinet imprisons intruders. The cabinet has two compartments; each can hold a single Medium creature.",
        traits: ["mechanical", "trap"],
      }),
    ).toContain("barrier_lockdown");

    expect(
      deriveRecordTags({
        name: "Slamming Door",
        category: "hazard",
        subcategory: "trap",
        descriptionText:
          "Pressure-sensitive panels in the floor connect to a stone slab hidden in a hallway's ceiling.",
        traits: ["mechanical", "trap"],
      }),
    ).toContain("barrier_lockdown");

    expect(
      deriveRecordTags({
        name: "Crushing Gate Trap",
        category: "hazard",
        subcategory: "trap",
        descriptionText:
          "A hidden stone rod causes two immense stone doors to fall forward from their gate, crushing anything beneath them.",
        traits: ["trap"],
      }),
    ).toContain("barrier_lockdown");

    expect(
      deriveRecordTags({
        name: "False Door Trap",
        category: "hazard",
        subcategory: "trap",
        descriptionText: "A hidden needle delivers a magical poison to anyone trying to open the door.",
        traits: ["magical", "mechanical"],
      }),
    ).not.toContain("barrier_lockdown");

    expect(
      deriveRecordTags({
        name: "Explosive Barrels",
        category: "hazard",
        subcategory: null,
        descriptionText: "Wooden barrels marked with an oil-drop symbol catch fire and explode.",
        traits: [],
      }),
    ).toContain("fire_hazard");

    expect(
      deriveRecordTags({
        name: "Acid Mist",
        category: "hazard",
        subcategory: "trap",
        descriptionText: "A cloud of caustic acid sprays across the chamber and corrodes exposed gear.",
        traits: ["mechanical", "trap"],
      }),
    ).toContain("acid_hazard");

    expect(
      deriveRecordTags({
        name: "Gas Trap",
        category: "hazard",
        subcategory: "trap",
        descriptionText:
          "A spring slams and locks the room's door before four hidden gas vents begin pumping poison gas into the chamber.",
        traits: ["mechanical", "trap"],
      }),
    ).toContain("poison_hazard");

    expect(
      deriveRecordTags({
        name: "Gas Trap",
        category: "hazard",
        subcategory: "trap",
        descriptionText:
          "A spring slams and locks the room's door before four hidden gas vents begin pumping poison gas into the chamber.",
        traits: ["mechanical", "trap"],
      }),
    ).not.toContain("acid_hazard");

    expect(
      deriveRecordTags({
        name: "Thin Ice",
        category: "hazard",
        subcategory: null,
        descriptionText: "The freezing floor gives way into a sheet of thin ice above frigid water.",
        traits: ["environmental"],
      }),
    ).toContain("cold_hazard");

    expect(
      deriveRecordTags({
        name: "Crumbling Archway",
        category: "hazard",
        subcategory: null,
        descriptionText: "The archway collapses when the stone supports are struck.",
        traits: ["environmental"],
      }),
    ).not.toContain("cold_hazard");

    expect(
      deriveRecordTags({
        name: "Electric Latch Rune",
        category: "hazard",
        subcategory: "trap",
        descriptionText: "Lightning crackles through the rune and a shock surges across the metal latch.",
        traits: ["magical", "trap"],
      }),
    ).toContain("electric_hazard");

    expect(
      deriveRecordTags({
        name: "Clockwork Trap",
        category: "hazard",
        subcategory: "trap",
        descriptionText: "The gears grind loudly as the mechanism locks into place.",
        traits: ["mechanical", "trap"],
      }),
    ).not.toContain("electric_hazard");

    expect(
      deriveRecordTags({
        name: "Buzzing Latch Rune",
        category: "hazard",
        subcategory: "trap",
        descriptionText: "A shrieking resonance and deafening sound burst from the rune, rattling the chamber.",
        traits: ["magical", "trap"],
      }),
    ).toContain("sound_hazard");

    expect(
      deriveRecordTags({
        name: "Alarm Ward",
        category: "hazard",
        subcategory: null,
        descriptionText:
          "A silent ward flares when a creature crosses the threshold, raising the alarm and alerting nearby guards.",
        traits: ["magical"],
      }),
    ).toContain("alarm");

    expect(
      deriveRecordTags({
        name: "Alarm Ward",
        category: "hazard",
        subcategory: null,
        descriptionText:
          "A silent ward flares when a creature crosses the threshold, raising the alarm and alerting nearby guards.",
        traits: ["magical"],
      }),
    ).not.toContain("sound_hazard");

    expect(
      deriveRecordTags({
        name: "Ballista Defense",
        category: "hazard",
        subcategory: null,
        descriptionText: "A ballista armed with a massive bolt fires on a creature approaching the palace.",
        traits: [],
      }),
    ).not.toContain("fire_hazard");

    expect(
      deriveRecordTags({
        name: "Spitting Daffodil",
        category: "hazard",
        subcategory: null,
        descriptionText: "Daffodils spit acidic nectar at non-plant creatures in the area.",
        traits: [],
      }),
    ).not.toContain("poison_hazard");

    expect(
      deriveRecordTags({
        name: "Smoke-Filled Hallway",
        category: "hazard",
        subcategory: null,
        descriptionText: "Dense choking smoke fills the hallway, making it difficult to see and breathe.",
        traits: ["environmental"],
      }),
    ).toContain("respiratory_hazard");

    expect(
      deriveRecordTags({
        name: "Sudden Geysers",
        category: "hazard",
        subcategory: null,
        descriptionText: "Superheated water erupts upward in sudden geysers and floods the chamber floor.",
        traits: ["environmental"],
      }),
    ).toContain("water_hazard");

    expect(
      deriveRecordTags({
        name: "Ash Web",
        category: "hazard",
        subcategory: null,
        descriptionText:
          "The creature becomes immobilized in the sticky web and remains held fast until it tears free.",
        traits: ["environmental"],
      }),
    ).toContain("restraint_capture");

    expect(
      deriveRecordTags({
        name: "Drowning Pit",
        category: "hazard",
        subcategory: "trap",
        descriptionText:
          "A trapdoor covers a 10-foot-square pit that's 30 feet deep and has 5 feet of water at the bottom.",
        traits: ["mechanical", "trap"],
      }),
    ).toContain("pitfall");

    expect(
      deriveRecordTags({
        name: "Pitiless Hall",
        category: "hazard",
        subcategory: null,
        descriptionText: "A merciless illusion torments intruders with visions of an unforgiving tyrant.",
        traits: ["magical"],
      }),
    ).not.toContain("pitfall");

    expect(
      deriveRecordTags({
        name: "Collapsing Bridge",
        category: "hazard",
        subcategory: "trap",
        descriptionText:
          "Metal supports twist and shear off the bridge, causing stretches of the structure to collapse.",
        traits: ["mechanical", "trap"],
      }),
    ).toContain("collapse_hazard");

    expect(
      deriveRecordTags({
        name: "Benefactor's End",
        category: "hazard",
        subcategory: "haunt",
        descriptionText:
          "A spectral assailant drives a stake into a dying vampire's heart; the vampire collapses and releases a surge of void energy.",
        traits: ["haunt", "magical"],
      }),
    ).not.toContain("collapse_hazard");

    expect(
      deriveRecordTags({
        name: "Rushing Wind",
        category: "hazard",
        subcategory: null,
        descriptionText: "A raging wind sucks creatures in the area toward the aiudara.",
        traits: ["environmental"],
      }),
    ).toContain("forced_movement");

    expect(
      deriveRecordTags({
        name: "Mental Scream Trap",
        category: "hazard",
        subcategory: "trap",
        descriptionText: "A psychic scream disorients creatures in the area and leaves them overwhelmed with fear.",
        traits: ["magical"],
      }),
    ).toContain("mental_impairment");

    expect(
      deriveRecordTags({
        name: "Mental Assault",
        category: "hazard",
        subcategory: "trap",
        descriptionText:
          "Magic sigils hidden in the grain of the wooden door trigger a magic trap that damages the mind of anyone attempting to pick the door's lock.",
        traits: ["magical", "mechanical", "trap"],
      }),
    ).toContain("mental_impairment");

    expect(
      deriveRecordTags({
        name: "Images of Failure",
        category: "hazard",
        subcategory: "trap",
        descriptionText:
          "Psychically enhanced illusions flood the minds of creatures in the hallway with memories of their past failures.",
        traits: ["magical", "trap"],
      }),
    ).toContain("mental_impairment");

    expect(
      deriveRecordTags({
        name: "Phantasmagoric Fog Trap",
        category: "hazard",
        subcategory: "trap",
        descriptionText:
          "Cloying violet mist billows from the undergrowth, filling the heads of any creatures inside with maddening visions of dark moonless nights and twisted malignant trees.",
        traits: ["magical", "trap"],
      }),
    ).toContain("mental_impairment");

    expect(
      deriveRecordTags({
        name: "Punish Defiance",
        category: "hazard",
        subcategory: "trap",
        descriptionText:
          "A magical mural threatens uncooperative inmates, causing mental trauma to those who defy its commands.",
        traits: ["magical", "trap"],
      }),
    ).toContain("mental_impairment");

    expect(
      deriveRecordTags({
        name: "Hampering Web",
        category: "hazard",
        subcategory: null,
        descriptionText:
          "Semitransparent sheets of webbing span the entryway, ready to capture small insects or hamper larger creatures that pass through.",
        traits: [],
      }),
    ).toContain("mobility_impairment");

    expect(
      deriveRecordTags({
        name: "Hampering Web",
        category: "hazard",
        subcategory: null,
        descriptionText:
          "Semitransparent sheets of webbing span the entryway, ready to capture small insects or hamper larger creatures that pass through.",
        traits: [],
      }),
    ).not.toContain("forced_movement");

    expect(
      deriveRecordTags({
        name: "Paralysis Trap",
        category: "hazard",
        subcategory: null,
        descriptionText:
          "Magical symbols carved in the doorframe magically ward this door; anyone attempting to pick the lock is Paralyzed and wracked with pain.",
        traits: ["magical"],
      }),
    ).toContain("mobility_impairment");

    expect(
      deriveRecordTags({
        name: "Psychic Wave",
        category: "hazard",
        subcategory: "trap",
        descriptionText:
          "A wave of psychic energy ripples through the room, violently prying into the minds of creatures in the area and alerting Kemnebi to the presence of intruders.",
        traits: ["magical"],
      }),
    ).not.toContain("mental_impairment");
  });

  it("derives hazard spawned attacker, navigation disruption, and overhead strike tags", () => {
    expect(
      deriveRecordTags({
        name: "Shadow Guards",
        category: "hazard",
        subcategory: null,
        descriptionText: "Shadowy caricatures peel themselves from the floor and attack everyone in the room.",
        traits: ["haunt", "magical"],
      }),
    ).toContain("spawned_attackers");

    expect(
      deriveRecordTags({
        name: "Confounding Portal",
        category: "hazard",
        subcategory: null,
        descriptionText:
          "Tiny runes carved around a doorframe confound creatures into circling the room or hallway they attempted to exit.",
        traits: ["magical"],
      }),
    ).toContain("navigation_disruption");

    expect(
      deriveRecordTags({
        name: "Rockfall Ceiling",
        category: "hazard",
        subcategory: "trap",
        descriptionText:
          "A load of rocks, held up by a rope pulley, is dropped on the cavern's lower level from the ceiling above.",
        traits: ["mechanical", "trap"],
      }),
    ).toContain("overhead_strike");

    expect(
      deriveRecordTags({
        name: "Murder Chandelier",
        category: "hazard",
        subcategory: "trap",
        descriptionText: "The chandelier overhead crashes down from above when the support chain is cut.",
        traits: ["mechanical", "trap"],
      }),
    ).toContain("overhead_strike");

    expect(
      deriveRecordTags({
        name: "Clockwork Ballista",
        category: "hazard",
        subcategory: "trap",
        descriptionText: "A fixed ballista turret fires bolts down the hallway whenever the mechanism is triggered.",
        traits: ["mechanical", "trap"],
      }),
    ).toContain("projectile_emitter");

    expect(
      deriveRecordTags({
        name: "Hall of Mirrors",
        category: "hazard",
        subcategory: null,
        descriptionText:
          "Among the many mirrored reflections lurk trapped souls that cause wounds to appear on reflected creatures.",
        traits: ["haunt", "magical"],
      }),
    ).not.toContain("spawned_attackers");

    expect(
      deriveRecordTags({
        name: "Guardian Mural",
        category: "hazard",
        subcategory: "trap",
        descriptionText:
          "One of the carvings on the mural suddenly animates and clambers out of the painting to become a real creature in the middle of the chamber.",
        traits: ["magical", "trap"],
      }),
    ).toContain("spawned_attackers");

    expect(
      deriveRecordTags({
        name: "Spear Launcher",
        category: "hazard",
        subcategory: "trap",
        descriptionText:
          "An old heavy crossbow is hidden in a pile of trash, loaded with a wooden spear, and connected to the rope holding the door.",
        traits: ["mechanical", "trap"],
      }),
    ).toContain("projectile_emitter");

    expect(
      deriveRecordTags({
        name: "Dream-Poisoned Door",
        category: "hazard",
        subcategory: "trap",
        descriptionText:
          "Eerie shapes manifest within the doorway as it's opened, conjuring a group of four animate dreams into being.",
        traits: ["magical", "trap"],
      }),
    ).toContain("spawned_attackers");

    expect(
      deriveRecordTags({
        name: "Distortion Circle",
        category: "hazard",
        subcategory: "trap",
        descriptionText:
          "A softly flickering circle of runes pulses before a raw red wound in reality splits open to disgorge demons and lash out with tendrils of acidic blood.",
        traits: ["magical", "trap"],
      }),
    ).toContain("spawned_attackers");

    expect(
      deriveRecordTags({
        name: "Angazhan's Rake Trap",
        category: "hazard",
        subcategory: "trap",
        descriptionText: "Six metal talons concealed in the walls swing out and rake across the room.",
        traits: ["mechanical", "trap"],
      }),
    ).toContain("projectile_emitter");

    expect(
      deriveRecordTags({
        name: "Distortion Mirror",
        category: "hazard",
        subcategory: "trap",
        descriptionText:
          "Fun-house mirrors distort a viewer's reflection, painfully reshaping their body to match what appears in the reflected images.",
        traits: ["magical", "trap"],
      }),
    ).toContain("illusion_assault");

    expect(
      deriveRecordTags({
        name: "Confounding Portal",
        category: "hazard",
        subcategory: null,
        descriptionText:
          "Tiny runes carved around a doorframe confound creatures into circling the room or hallway they attempted to exit.",
        traits: ["magical"],
      }),
    ).not.toContain("illusion_assault");

    expect(
      deriveRecordTags({
        name: "Wrath of the Destroyer",
        category: "hazard",
        subcategory: "trap",
        descriptionText:
          "Heavy doors carved with a dragon's image manifest a vision of its head emerging from the doorway to strike at a foe.",
        traits: ["magical", "trap"],
      }),
    ).not.toContain("spawned_attackers");
  });

  it("derives haunt manifestation and consequence tags", () => {
    expect(
      deriveRecordTags({
        name: "Ashes of Despair",
        category: "hazard",
        subcategory: "haunt",
        descriptionText:
          "Swirling ashes fill the room and form into the semblance of a massive predatory bird that swoops upon those in the chamber, draining their life force away.",
        traits: ["haunt"],
      }),
    ).toContain("life_drain_hazard");

    expect(
      deriveRecordTags({
        name: "Blood of Belcorra",
        category: "hazard",
        subcategory: "haunt",
        descriptionText:
          "A bloody image of Belcorra arises, emits a soul-draining light, then inhales blood from living creatures in the room.",
        traits: ["haunt"],
      }),
    ).toContain("life_drain_hazard");

    expect(
      deriveRecordTags({
        name: "Blood-Soaked Soil",
        category: "hazard",
        subcategory: "haunt",
        descriptionText:
          "The blood of those who died in the area bubbles up from the earth, soaking the soil and turning it into a bloody morass.",
        traits: ["haunt"],
      }),
    ).not.toContain("life_drain_hazard");

    expect(
      deriveRecordTags({
        name: "Benefactor's End",
        category: "hazard",
        subcategory: "haunt",
        descriptionText:
          "A spectral assailant drives a stake into a dying vampire's heart; the vampire collapses and releases a surge of void energy.",
        traits: ["haunt"],
      }),
    ).toContain("phantom_assailants");

    expect(
      deriveRecordTags({
        name: "Army of Mist",
        category: "hazard",
        subcategory: "haunt",
        descriptionText:
          "The wind intensifies and the seeping fog rushes forward in a stampede of ghostly orcs screaming wordless battle cries, weapons raised to slaughter their foes.",
        traits: ["environmental", "haunt", "occult"],
      }),
    ).toContain("phantom_assailants");

    expect(
      deriveRecordTags({
        name: "Phantom Soldiers",
        category: "hazard",
        subcategory: "haunt",
        descriptionText: "Spirits of long-dead soldiers appear in the mist and attack intruders.",
        traits: ["haunt"],
      }),
    ).toContain("phantom_assailants");

    expect(
      deriveRecordTags({
        name: "Stonescale Spirits",
        category: "hazard",
        subcategory: "haunt",
        descriptionText: "A half-dozen ghostly kobolds rise from the rubble in a howling vortex.",
        traits: ["haunt"],
      }),
    ).toContain("phantom_assailants");

    expect(
      deriveRecordTags({
        name: "Haunting Presence",
        category: "hazard",
        subcategory: "haunt",
        descriptionText:
          "Ghostly sounds or glimpses of apparitions move to startle or distract those who walk the streets.",
        traits: ["haunt"],
      }),
    ).not.toContain("phantom_assailants");

    expect(
      deriveRecordTags({
        name: "Spirit Window",
        category: "hazard",
        subcategory: "haunt",
        descriptionText: "Spirits trapped inside a haunted window harm those who touch the window.",
        traits: ["haunt"],
      }),
    ).not.toContain("phantom_assailants");

    expect(
      deriveRecordTags({
        name: "Broken Rebus Attack",
        category: "hazard",
        subcategory: "haunt",
        descriptionText:
          "Ghostly attackers knock the tables about, send dishes flying, and pull fleeing creatures into the room.",
        traits: ["haunt", "magical"],
      }),
    ).toContain("battlefield_disruption");

    expect(
      deriveRecordTags({
        name: "Battle Illusion",
        category: "hazard",
        subcategory: "haunt",
        descriptionText:
          "The scene of the arid Cinderlands melts away as the water of the river refills and green grass spreads across the land. The sounds of battle can be heard everywhere and all allies have vanished, murderous orcs from the One Eye Hold taking their place.",
        traits: ["haunt", "magical", "occult"],
      }),
    ).toContain("battlefield_disruption");
  });

  it("derives haunt lure and compulsion tags", () => {
    expect(
      deriveRecordTags({
        name: "Damurdiel's Vengeance",
        category: "hazard",
        subcategory: "haunt",
        descriptionText:
          "An elven woman wearing a robe stands in the water. She beckons anyone who enters the room to join her in the pool.",
        traits: ["haunt"],
      }),
    ).toContain("lure_compulsion");

    expect(
      deriveRecordTags({
        name: "Dance Of Death",
        category: "hazard",
        subcategory: "haunt",
        descriptionText: "An eerie orchestra compels all who hear it to dance until they collapse from exhaustion.",
        traits: ["haunt"],
      }),
    ).toContain("lure_compulsion");

    expect(
      deriveRecordTags({
        name: "Echoes of Betrayal",
        category: "hazard",
        subcategory: "haunt",
        descriptionText:
          "Malevolent spirits led by a ghostly image of Ludika rise and begin a deadly brawl. These spirits attempt to overwhelm living creatures, forcing them to join the battle.",
        traits: ["haunt"],
      }),
    ).toContain("lure_compulsion");

    expect(
      deriveRecordTags({
        name: "False Floor",
        category: "hazard",
        subcategory: "trap",
        descriptionText:
          "The floor in this chamber is an illusion, which conceals a 40-foot drop to the true floor below. Additionally, the area beneath the floor is magically silenced-no sound is audible within nor does sound leave this space.",
        traits: ["illusion", "magical", "trap"],
      }),
    ).not.toContain("lure_compulsion");
  });
});
