# Creature family review notes

Use these notes with the [creature family review procedure](../../../docs/development/source-research.md#creature-family-review-during-a-source-upgrade). The [family TOMLs](creatures/) record accepted relationships and their sources. Entries here record reviewed exceptions and open questions.

The entries below were reviewed against Foundry PF2e revision
`28d9d156f252d084371d994dc359509b49eadf18`. Actor names are exact Foundry titles.

## Keep separate

### King Stankus and Poppy

*The Dragon's Crown* uses the placeholder token name `NPC` for unrelated
creatures. King Stankus is a goblin aberration and Poppy is an animal with a
different stat block. Revisit if the actor set changes or the source establishes
a relationship.

- King Stankus (`dawn-of-the-frogs:YPyw9jQlFnb5iHwq`)
- Poppy (`dawn-of-the-frogs:rB5PGOKN1b8SukWs`)

### The Dead God’s Hand NPC token

The four actors share the placeholder token name `NPC`. Their source
descriptions and stat blocks identify an apprentice armorer, an Azlanti
noble, an anadi torturer, and a hryngar cleric. Revisit if the actor set
changes or the source establishes a relationship.

- Blostin (`the-dead-gods-hand-bestiary:GEMV5dcsfSfExOkY`)
- Lord Pandelay (`the-dead-gods-hand-bestiary:O7dYMUmjENgExuFr`)
- Shulgurk (`the-dead-gods-hand-bestiary:oFK6kZAsaCe2Th41`)
- Zulgara (`the-dead-gods-hand-bestiary:zFTfx1Q0oJxk8Bno`)

## Deferred decisions

### Authored scaling

**Anatomical Model.** In *The Seven Secrets of Dacilane Academy*, Foundry's
ordinary and Elite 3–4 blocks both report creature level 4 and have no typed
adjustment. The Elite block has 20 more HP. Verify the published level, scaling
condition, and effective stats against the Laboratory encounter before declaring
an override.

- Anatomical Model (1-2) (`pfs-season-2-bestiary:BXnmDgVkhD97fI1o`)
- Anatomical Model (3-4) (`pfs-season-2-bestiary:j7K9SefeHJ16qHMl`)
- Elite Anatomical Model (3-4) (`pfs-season-2-bestiary:ZshsztmoWjDZEqGT`)

**Alystair Caskwater.** *In the Footsteps of Horror* supplies ordinary and
Vengeful blocks in both subtiers. At 3–4 the Vengeful block substitutes at 28 or
more Challenge Points. Verify the level 1–2 scaling table before declaring the
combined family and scenario uses.

- Alystair Caskwater (1-2) (`pfs-season-5-bestiary:L55H4HofGvCM02au`)
- Alystair Caskwater (3-4) (`pfs-season-5-bestiary:4JnqGDbDbDziIVgI`)
- Vengeful Alystair Caskwater (1-2) (`pfs-season-5-bestiary:nMMRUST4ow0kcbTy`)
- Vengeful Alystair Caskwater (3-4) (`pfs-season-5-bestiary:SJ39wba6n3d5bvIh`)

**The Warden.** The ordinary and Upset records from *The Winter Queen's
Dollhouse* share spell lists within each tier. Each Upset block is one creature
level higher. Confirm the published conditions for switching blocks and whether
the quest calls for an adjustment.

- The Warden (1-2) (`pfs-season-5-bestiary:7Ox34TV7Py1pZr9o`)
- The Warden (3-4) (`pfs-season-5-bestiary:1kqDiAAQGH26eAC7`)
- The Upset Warden (1-2) (`pfs-season-5-bestiary:8IJilQMumSAPhsqP`)
- The Upset Warden (3-4) (`pfs-season-5-bestiary:Z9ofpt7xRr9o47Ya`)

### Source identity and publication

**Shadow.** The shared token and Monster Core publication field combine a
Monster Core spawn with an actor from the Season of Ghosts pack. Compare their
roles with the generic Monster Core Shadow before assigning a family.

- Shadow Spawn (`pathfinder-monster-core:clzHAfegb9Rn6DOe`)
- Familiar Shadow (`season-of-ghosts-bestiary:IdbrdikgvRguKYNE`)
- Shadow (`pathfinder-monster-core:VotlWUsFKdOrHWF6`)

**Elite Boggard Swampseer.** The titles say PFS 1-24, while the publication and
pack paths point to *The Lost Legend* (1-20), which describes an animal staff
loadout. Resolve the title discrepancy and compare the published encounter
blocks with the standard Boggard Swampseer before declaring the family.

- Elite Boggard Swampseer (PFS 1-24, Staff) (`pfs-season-1-bestiary:1LM0AUYkZcsbtETs`)
- Elite Boggard Swampseer (PFS 1-24, Animal Staff) (`pfs-season-1-bestiary:D2Bc4GbmiuW29310`)

### Marcien's Shadow Doubles

The [Marcien Blakros family](creatures/marcien-blakros.toml) covers the original.
These are summoned copies. Decide whether the copies form their own family and
how to link it to the summoner. [Lelzeshin](creatures/lelzeshin.toml) and
[Mirrored Lelzeshin](creatures/mirrored-lelzeshin.toml) provide a second case.

- Marcien Blakros (7-8) (Shadow Double) (`pfs-season-3-bestiary:PqawYWEK0iqvcBZj`)
- Marcien Blakros (9-10) (Shadow Double) (`pfs-season-3-bestiary:xVmDwoBxjKyCihYz`)

### Published adjustments and scaling

**Commodore Sticky Fingers.** Student Exchange has an ordinary actor outside the token group and Elite versions of both the ordinary and Lively blocks. Check the published conditions for choosing Lively and Elite and compare the effective stats.

- Commodore Sticky Fingers (`pfs-season-5-bestiary:76sfB4T9gkJjAJOS`)
- Elite Commodore Sticky Fingers (`pfs-season-5-bestiary:FU2kzh3jmBopL5t9`)
- Elite Lively Commodore Sticky Fingers (`pfs-season-5-bestiary:WcLXNcwcA1k9QtbU`)
- Lively Commodore Sticky Fingers (`pfs-season-5-bestiary:g8vmUHZ62eNlvhDu`)

**Sewer Ooze.** Student Exchange has Weak and Elite Sewer Ooze blocks at creature levels −1 and 2. Check the printed encounter for the base stat block and the conditions for using each adjustment.

- Elite Sewer Ooze (PFS Q18) (`pfs-season-5-bestiary:vsWgbVO6zJIjiqAs`)
- Weak Sewer Ooze (PFS Q18) (`pfs-season-5-bestiary:dXSti6gNOpoEYoIF`)

**Sprite.** Student Exchange has Sprite and Elite Sprite at creature levels −1 and 1. Check the published base and effective stats and determine the scenario use before declaring an override.

- Elite Sprite (PFS Q18) (`pfs-season-5-bestiary:zbh0oprNBaxTbsqh`)
- Sprite (PFS Q18) (`pfs-season-5-bestiary:q9VY2Whn0rV7zJVK`)

**Jinkin.** The Dacilane Academy’s Show Must Go On supplies the Lesser Jinkin and a 1–2 Jinkin with shared Tinker and Sneak Attack. Confirm what selects the Lesser block before attaching party ranges or an adjustment.

- Jinkin (Quest 20) (1-2) (`pfs-season-6-bestiary:MU8oL7P1AX4P4kKB`)
- Lesser Jinkin (`pfs-season-6-bestiary:D7XibY38wx9AjopU`)

**Gluttonous Geode.** Friends in Need has a quest-specific Gluttonous Geode and a Weak Gluttonous Geode. Verify which encounter conditions call for the Weak block and compare it with the base stats.

- Gluttonous Geode (Quest 22) (`pfs-season-6-bestiary:WxH0CPkr7mrYCiI0`)
- Weak Gluttonous Geode (`pfs-season-6-bestiary:qLD4CzztZ0CMaRCj`)

**Wood Scamp.** Friends in Need has ordinary, Weak, and Elite Wood Scamp actors. Verify the encounter scaling and effective stats before declaring scoped overrides.

- Elite Wood Scamp (`pfs-season-6-bestiary:hcHtUbPVInjQ5TEb`)
- Weak Wood Scamp (`pfs-season-6-bestiary:qnvKhJ7BP2BWPkVo`)
- Wood Scamp (Quest 22) (`pfs-season-6-bestiary:AG5aG5TmtxAzIERA`)

**Animated Flame.** Tanuki Trouble has ordinary, Weak, and Elite blocks. The Weak and Elite actors have different task-related abilities. Check the published stat blocks and usage before treating either as an override.

- Animated Flame (`pfs-season-6-bestiary:88HAT2lHPAm8bWHC`)
- Elite Animated Flame (`pfs-season-6-bestiary:niboeMqpzuQ8yvOX`)
- Weak Animated Flame (`pfs-season-6-bestiary:xNKBknnZ5ev5lBes`)

**Animate Dream.** Prisoners of the Electric Castle has Faded and Elite Animate Dream blocks at creature levels 7 and 8. Verify the conditions selecting each block and compare their effective stats with the base before declaring an override.

- Elite Animate Dream (PFS 4-11) (`pfs-season-4-bestiary:eWJwFmEasIOSGQgu`)
- Faded Animate Dream (`pfs-season-4-bestiary:AkbAW7XeZQ67qW1R`)

**Clacking Skull Swarm.** In Glorious Battle has Weak and Elite Clacking Skull Swarms at creature levels 9 and 11. Identify the base and verify the scenario conditions for each adjustment.

- Elite Clacking Skull Swarm (`pfs-season-4-bestiary:H9z43UB0KXksVTZ6`)
- Weak Clacking Skull Swarm (`pfs-season-4-bestiary:56GciWoZDuiqjINk`)

### Encounter identity and variants

**Bosun.** Iron Ring Bosun and Vaggas the Bosun share Bosun’s Command, Pike and Strike, and the same level and HP. Check Foundation’s Price for evidence that Vaggas is a named example of this base or a distinct encounter role.

- Iron Ring Bosun (`pfs-season-3-bestiary:FmZ8bz1ItIzxuu1D`)
- Vaggas the Bosun (`pfs-season-3-bestiary:RZ8sWKkogng0IU8M`)

**Wasp Swarm.** Guardian’s Covenant supplies Arctic and Heretical forms and a block marked PFS 3-13. Check how the adventure uses that scenario copy and how it relates to the generic Wasp Swarm before choosing membership and titles.

- Wasp Swarm (Arctic) (`pfs-season-3-bestiary:xShj7KlFATzm397y`)
- Wasp Swarm (Heretical) (`pfs-season-3-bestiary:JZtLQye4QlfFjrYT`)
- Wasp Swarm (PFS 3-13) (`pfs-season-3-bestiary:dCljcVK6RAc0Y8Z7`)

## Discovery and grouping gaps

**Missing token names.** The [Spellskein](creatures/spellskein.toml) and
[Wizard Sponge](creatures/wizard-sponge.toml) families include same-publication
base actors the token audit missed. Use them and the Hesla pair in *The Overthrow
of Ambition* to check a same-publication discovery pass. Verify the adventure
before assigning a tier to the unsuffixed Hesla.

- Hesla, The Broken Ember (`pfs-season-6-bestiary:EWutX9eW7tYLpors`)
- Hesla, The Broken Ember (11-12) (`pfs-season-6-bestiary:k6VFoe5R52PbVZJB`)

**Grouping by ability across party tiers.** [Fleshforge Construct](creatures/fleshforge-construct.toml)
groups by party level and names the ability in each title. Each of the six
abilities appears on a Dreg and a Prototype. Review whether to offer an
ability grouping across both forms.

**Nathatel’s undead stat blocks.** [Nathatel](creatures/nathatel.toml)
covers all four encounter G tiers, including the unsuffixed 1–2 actor. The
adventure also supplies two stat blocks for undead Nathatel in a later
encounter. The unsuffixed undead actor has no token name. Review how the undead
form relates to the living actor alongside other missing-token cases.

- Undead Nathatel (`pfs-season-4-bestiary:C9gWLpxJr4QKxP64`)
- Undead Nathatel (7-8) (`pfs-season-4-bestiary:nr6qCJkCLNAZh5ED`)
