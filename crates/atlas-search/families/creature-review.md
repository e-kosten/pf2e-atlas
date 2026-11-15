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

**Cosmic Amoeba and Hound Echo.** Stranded on Yesterday’s Tide uses
different blocks by encounter and party tier. Foundry stores typed Weak
adjustments on otherwise partly unadjusted stats. Verify the effective
levels, defenses, strike dice, and ability DCs against encounters B–D before
adding scoped overrides.

- Cosmic Amoeba (`pfs-season-5-bestiary:Mmsmoh7tpsnLZWrI`)
- Weak Cosmic Amoeba (`pfs-season-5-bestiary:c93xz68jVyaTiAI5`)
- Hound Echo (`pfs-season-5-bestiary:JGtNz1mLf5rsvJWC`)
- Weak Hound Echo (`pfs-season-5-bestiary:JKduEdHCeBVIYcdd`)
- Cacophonous Hound Echo (`pfs-season-5-bestiary:vvZktgY1C8Eabs3w`)
- Weak Cacophonous Hound Echo (`pfs-season-5-bestiary:Hysy3I3yiYsUZHxY`)

**Congealed Magic.** Lost and Forgotten supplies ordinary and Concentrated
blocks with typed Elite counterparts. Check the encounter’s scaling conditions
and effective stats before declaring the party uses and overrides.

- Congealed Magic (`pfs-season-6-bestiary:gjeL6QJBoniPAm4M`)
- Elite Congealed Magic (`pfs-season-6-bestiary:XMtgZP2i1lttMf15`)
- Concentrated Congealed Magic (`pfs-season-6-bestiary:agYSQuaxGkFQVBus`)
- Elite Concentrated Congealed Magic (`pfs-season-6-bestiary:tiPPgdHWXNWE7LKF`)

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

### Same-publication actor pairs

#### Pathfinder #186: Ghost King's Rage

The Foundry skeleton has 40 fewer HP and adds Flamboyant Reposition. Verify
the Celebrants encounter's printed instructions and whether the skeleton uses
the Rival Corpsekiller as its base.

- Rival Corpsekiller (`blood-lords-bestiary:1Hc1BLc9LIXLuBdg`) ↔ Skeleton Rival Corpsekiller (`blood-lords-bestiary:S1rfj73uV1ZUBCJ3`)

#### Pathfinder Adventure Path: Hell's Destiny

Vanda and the Champion share the Viper’s Fang item. Vanda is a named unique
actor
with a separate action that restates the item's activation and venom. Verify
whether the adventure uses the Champion as her underlying stat block.

- Vanda Spitelash (`hells-destiny-bestiary:1y6sYotLFK9UxFAo`) ↔ Champion of Asmodeus (`hells-destiny-bestiary:NZOqqzi4ThvJfpm8`)

#### Pathfinder Society Quest #15: In the Footsteps of Horror

In *In the Footsteps of Horror*, both Pixies have authored Elite stats and the
Weak Shadow Draugr retains 35 HP. Verify the published encounter and scaling
sidebars for the exact uses before declaring adjustment overrides.

- Arisen Shadow Pixie (`pfs-season-5-bestiary:8Zj3XDNa07fjkWAq`) ↔ Elite Arisen Shadow Pixie (`pfs-season-5-bestiary:ytb75tCpHFXxryFZ`)
- Elite Arisen Umbral Pixie (`pfs-season-5-bestiary:ep7eL4vrN4F6YCdd`) ↔ Arisen Umbral Pixie (`pfs-season-5-bestiary:umxjQH7AV65Eftfc`)
- Shadow Draugr (`pfs-season-5-bestiary:EgAoF7pXBsHxthA0`) ↔ Weak Shadow Draugr (`pfs-season-5-bestiary:qqBaCxN4Epxe1rnF`)

#### Pathfinder Society Quest #17: Escorting a Mirage

In *Escorting a Mirage*, the Angry squad and the Weak or Elite actors differ
by two creature levels. Verify the printed blocks and the conditions for
choosing each before treating them as scoped adjustments.

- Angry Mercenary Squad (`pfs-season-5-bestiary:WoITmnqltzfUlvry`) ↔ Mercenary Squad (`pfs-season-5-bestiary:l4fXYWBhDNM5FAyT`)
- Elite Foolhardy Mercenary (`pfs-season-5-bestiary:M1POW9BD03uSF6pA`) ↔ Foolhardy Mercenary (`pfs-season-5-bestiary:RPhiMYpPORMIIDJi`)
- Trained Vulkariki Eidolon (`pfs-season-5-bestiary:Hvrz7fywW7hmpYcJ`) ↔ Elite Trained Vulkariki Eidolon (`pfs-season-5-bestiary:vK1DxX5eTu1QKhbm`)
- Weak "Enlightened" Div Worshipper (`pfs-season-5-bestiary:jHtXyQDDZLdytpqk`) ↔ "Enlightened" Div Worshipper (`pfs-season-5-bestiary:u74d5wQNBJLVHB7m`)
- Weak Div Worshipper (`pfs-season-5-bestiary:LBzI86ngk7wYf7p6`) ↔ Div Worshipper (`pfs-season-5-bestiary:qvdMGFjaJe08JQW5`)

#### Pathfinder Society Quest #21: Infernal Infiltration

The Silver Devil Hooligan and its level 2 Elite form carry authored stats.
Verify *Infernal Infiltration*'s encounter and scaling sidebar for the exact
override scope.

- Elite Silver Devil Hooligan (`pfs-season-6-bestiary:MjckZs5NKL35d8QL`) ↔ Silver Devil Hooligan (`pfs-season-6-bestiary:XRecLPsSrHHFwWHI`)

#### Pathfinder Society Scenario #1-18: Lodge of the Living God

The Elite Midge is two creature levels higher. Verify the published *Lodge of
the Living God* encounter and identify the party tiers and scaling conditions
for the two blocks.

- Glass River Midge (`pfs-season-1-bestiary:I3jGuCG3qL5ua7yE`) ↔ Elite Glass River Midge (`pfs-season-1-bestiary:xu1dm0KnEHx5vySR`)

#### Pathfinder Society Scenario #4-15: In Glorious Battle

The *In Glorious Battle* appendices present Elite replacements in encounter
scaling tables. For example, encounter A (p. 31) replaces Combustive Zombies
with Elites at 23+ Challenge Points, and encounter B uses different wight
roles by party tier. Map each role and exact replacement across encounters
before declaring overrides.
[Published scenario](https://www.scribd.com/document/669037731/Pathfinder-Society-Season-4-Year-of-Boundless-Wonder-Scenario-15)

- Elite Combustive Zombie (`pfs-season-4-bestiary:2N9DhnUedzp3FHhM`) ↔ Combustive Zombie (`pfs-season-4-bestiary:vA5VwmGb7xZ0WQdM`)
- Elite Smokesight Deadeye Wight (`pfs-season-4-bestiary:4G6VvfIsceXfEZBB`) ↔ Smokesight Deadeye Wight (`pfs-season-4-bestiary:4r7haYeUSxjW3wtR`)
- Elite Smokesight Hunter Wight (`pfs-season-4-bestiary:UDodAGDnGkGwYsMi`) ↔ Smokesight Hunter Wight (`pfs-season-4-bestiary:pomHkkLGJxTvpAyA`)
- Elite Smokesight Prowler Wight (`pfs-season-4-bestiary:9fBLqeIyvH232sFp`) ↔ Smokesight Prowler Wight (`pfs-season-4-bestiary:SO8aRCEXdCRJ8TIG`)
- Elite Smokesight Stalker Wight (`pfs-season-4-bestiary:YjTKd7VZwCFaTWPB`) ↔ Smokesight Stalker Wight (`pfs-season-4-bestiary:eC1Y25pgLNSpAoki`)
- Elite Smokesight Wight Captain (`pfs-season-4-bestiary:BRo5HCigb4YTjJys`) ↔ Smokesight Wight Captain (`pfs-season-4-bestiary:qqNfvVoquLzXsg9j`)
- Elite Smokesight Wight Commander (`pfs-season-4-bestiary:KV2LmRMc9ho9mBPy`) ↔ Smokesight Wight Commander (`pfs-season-4-bestiary:Q6BTF30qgInYi2BA`)

#### Pathfinder Society Scenario #4-16: Dacilane Academy's First Great Prank War

The unsuffixed Pappy is level 3 and the 5–6 title is level 5. Check the
*Dacilane Academy's First Great Prank War* encounter appendix for the
party-level assignments.

- Pappy Draighean (`pfs-season-4-bestiary:1G589gnuWGIOdrou`) ↔ Pappy Draighean (5-6) (`pfs-season-4-bestiary:ACDuWRp4JELgqmo8`)

#### Pathfinder Society Scenario #4-99: Blessings of the Forest

Cathedral Event 4 in *Blessings of the Forest* (pp. 45, 57) prints a level −1
base dagger and a level 0 Elite dagger. The latter appears in conditional
Challenge Point bands, including 20–32 at party levels 3–4. Model bounded
conditions before declaring an override for the Foundry actor titled `Animated
Dagger (3-4)`.
[Published scenario](https://www.scribd.com/document/911522567/PF2-S04-99-Blessings-of-the-Forest)

- Animated Dagger (3-4) (`pfs-season-4-bestiary:16Ye7Wd2pXiUiGJo`) ↔ Animated Dagger (`pfs-season-4-bestiary:548JZ6YeXsfYxVe5`)

#### Pathfinder Society Scenario #5-03: Heidmarch Heist

*Heidmarch Heist* (pp. 17, 22, 25) uses creature levels 5 and 7 at party
levels 5–6 and 7–8. Both Foundry records report level 5, including the upper
block with evasion and 111 HP. Recheck after a source correction so Atlas can
show the correct creature level.
[Published scenario](https://www.scribd.com/document/870508711/PZOPFS0503E)

- Krosovahn Mendesil (`pfs-season-5-bestiary:H5lFXWXFl7vKrugU`) ↔ Krosovahn Mendesil (7-8) (`pfs-season-5-bestiary:TWsApxv2Bx0n7M3i`)

#### Pathfinder Society Scenario #5-10: The Crocodile's Smile

The unsuffixed *Crocodile's Smile* actors have lower creature levels than the
3–4 actors. The higher-tier Zibini also gains two named kicks. Verify the
encounter appendix's party-level assignments.

- Little Brother Zibini (`pfs-season-5-bestiary:NoatZIUfJETSq20A`) ↔ Little Brother Zibini (3-4) (`pfs-season-5-bestiary:epmml7kA0akrIGPG`)
- Tisbah (3-4) (`pfs-season-5-bestiary:V5gA4pv3ewFxfY6L`) ↔ Tisbah (`pfs-season-5-bestiary:kLL35GP9ZczW1tKV`)

#### Pathfinder Society Scenario #5-14: Poisonous Council

The 5–6 Sronwa gains Surprise Attack and is two creature levels higher. Verify
*Poisonous Council*'s encounter for the unsuffixed actor's party range.

- Sronwa (`pfs-season-5-bestiary:R1S3vlOlNEEqW8kO`) ↔ Sronwa (5-6) (`pfs-season-5-bestiary:RghG76jeaTeQCT6j`)

#### Pathfinder Society Scenario #6-04: An Enkindled Carnival

The unsuffixed Puddles Phantom is level 4 and the 5–6 title is level 6. Verify
*An Enkindled Carnival*'s encounter for the lower block's party range.

- Puddles Phantom (`pfs-season-6-bestiary:5Txfy0wV529fleUw`) ↔ Puddles Phantom (5-6) (`pfs-season-6-bestiary:ooCLBFbCLCQWhPqD`)

#### Pathfinder Society Scenario #6-09: The Power of Legends

Both Penumbra actors report creature level 11. The 11–12 block has 210 HP
versus 170 HP. Compare both with the published *Power of Legends* blocks
before assigning scenario uses.

- Penumbra (`pfs-season-6-bestiary:Mg8gzA54MY5DMHZk`) ↔ Penumbra (11-12) (`pfs-season-6-bestiary:cyOOXZliztAjS9YM`)

#### Pathfinder Society Scenario #6-12: The Burning of Greensteeples

Both Maralictors have a 5–6 actor two creature levels higher than the
unsuffixed one. Verify *The Burning of Greensteeples* encounter appendix for
both party ranges.

- Maralictor Marius Salutio (`pfs-season-6-bestiary:Tx53uPtEt55FwP1U`) ↔ Maralictor Marius Salutio (5-6) (`pfs-season-6-bestiary:kSpMlrZrl3VxOOwm`)
- Maralictor Petra Fidena (`pfs-season-6-bestiary:08AsM66FZV6yII9l`) ↔ Maralictor Petra Fidena (5-6) (`pfs-season-6-bestiary:9ypN5dFE6uYKeoLQ`)

#### Pathfinder Society Scenario #6-19: What Walks Again

The 13–14 Husk of Deskari is creature level 17. The unsuffixed actor is level
14. Verify the assignments in *What Walks Again*.

- The Husk of Deskari (13-14) (`pfs-season-6-bestiary:7H1fqxjhk0PzUMy4`) ↔ The Husk of Deskari (`pfs-season-6-bestiary:cJMsjCIJQB2iafmk`)

#### Pathfinder Society Scenario #6-20: The Overthrow of Ambition

The 11–12 Harbinger is creature level 9 and the unsuffixed actor is level 7.
Verify the party uses in *The Overthrow of Ambition*.

- Harbinger of Ingalor (11-12) (`pfs-season-6-bestiary:RgAi5UNl5yxAk9y2`) ↔ Harbinger of Ingalor (`pfs-season-6-bestiary:qbAQmR4WUnNF3g4Z`)

#### Pathfinder Society Scenario #7-01: Intro to the Year of Battle's Spark: Enough is Enough

The three 3–4 actors have higher creature levels. The Imp Minion has 1 HP in
both records. Verify both encounter tiers in *Enough is Enough*, including
whether the imp stat block is merely rescaled.

- Bartolo Posandi (3-4) (`pfs-season-7-bestiary:lGBzkcTH3i49Bg7e`) ↔ Bartolo Posandi (`pfs-season-7-bestiary:sjMxdb8gEpK7ejBQ`)
- Imp Minion (`pfs-season-7-bestiary:BeaQYUmT1DrnFxpR`) ↔ Imp Minion (3-4) (`pfs-season-7-bestiary:xaoetADwlJqF8Hy0`)
- Olinda Posandi (`pfs-season-7-bestiary:GD8dO41GvkdPQTT9`) ↔ Olinda Posandi (3-4) (`pfs-season-7-bestiary:McQbKbTiy38rEEna`)

#### Pathfinder Society Scenario #7-03: A Foot in the Door

The level 9 Timonithaea title says 7–9. Verify the actual upper party range in
*A Foot in the Door* before displaying it.

- Timonithaea (7-9) (`pfs-season-7-bestiary:Idl8zxFPu9dyRA5h`) ↔ Timonithaea (`pfs-season-7-bestiary:UG5iXBqsLxRifjSW`)

#### Pathfinder Society Scenario #7-05: Battle of the Thorns

The unsuffixed Kizrech is level 9 and the 11–12 title is level 11. Verify the
lower range in *Battle of the Thorns*.

- Kizrech (`pfs-season-7-bestiary:W2puJUeo61CLMMxQ`) ↔ Kizrech (11-12) (`pfs-season-7-bestiary:xyT4zCZjAbnAgDLV`)

#### Pathfinder Society Scenario #7-08: The Haunted Corridor

The unsuffixed Lyari is level 1 and the 3–4 title is level 3. Verify the lower
range in *The Haunted Corridor*.

- Lyari Zinfeld (`pfs-season-7-bestiary:SnoWbBMVQdoAKJA9`) ↔ Lyari Zinfeld (3-4) (`pfs-season-7-bestiary:vngxFO3AQdnBivra`)
