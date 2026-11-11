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

## Deferred decisions

### Same-level scenario versions in Search

These pairs differ by party tier and mechanics. Using the same friendly title,
creature level, and publication would make their compact Search cards difficult
to distinguish. Reopen when reviewing how Search can distinguish them.

| Foundry actors | Evidence |
| --- | --- |
| Animal Companion (Wolf) (1-2) (`pfs-season-1-bestiary:czYt4Yp6ZeGnzyV6`) and Animal Companion (Wolf) (3-4) (`pfs-season-1-bestiary:9KauvSo8zxsaBWRc`) | Both are level 0 in *Bandits of Immenwood* and have different HP. |
| Sparkles (9-10) (`pfs-season-3-bestiary:8CEKF7wq9qZzmveH`) and Sparkles (11-12) (`pfs-season-3-bestiary:SkV5zYJgu7hdpZE0`) | Both are level 9 in *Cavern of Sundered Songs* and have different HP. |
| Ghost Of Diggen Thrune (1-2) (`pfs-season-1-bestiary:kpYq0piOlafgYSdo`) and Ghost Of Diggen Thrune (3-4) (`pfs-season-1-bestiary:5hR3mQw2sEy3Fy8g`) | Both are level 4 with 30 HP in *Flames of Rebellion*. Deception changes from +10 to +13. |
| Cinder Wolf (1-2) (`pfs-season-1-bestiary:mIz8CjpB8uxf6M9f`) and Cinder Wolf (3-4) (`pfs-season-1-bestiary:Hjl0cYnMyrcsbKlY`) | Both are level 1 in *Lions of Katapesh*. The 3–4 block adds Smoke Vision. |

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

## Discovery and grouping gaps

**Missing token names.** The [Spellskein](creatures/spellskein.toml) and
[Wizard Sponge](creatures/wizard-sponge.toml) families include same-publication
base actors the token audit missed. Use them and the Hesla pair in *The Overthrow
of Ambition* to check a same-publication discovery pass. Verify the adventure
before assigning a tier to the unsuffixed Hesla.

- Hesla, The Broken Ember (`pfs-season-6-bestiary:EWutX9eW7tYLpors`)
- Hesla, The Broken Ember (11-12) (`pfs-season-6-bestiary:k6VFoe5R52PbVZJB`)

**Cross-publication variants.** The *Hands of the Devil* encounter actors change
the Barbazu's equipment. Compare them with the Bestiary base before declaring the
family and deciding how it should appear. Use the result to test cross-publication
discovery.

- Torture Chamber Barbazu (`abomination-vaults-bestiary:4bznEiwsJvInwZwA`)
- Drill Field Barbazu (`abomination-vaults-bestiary:A4MusxxoLxwMVZua`)
- Hellforge Barbazu (`abomination-vaults-bestiary:vX4SBXIICKfrM4pF`)
- Barbazu (`pathfinder-bestiary:FIoRPHaHdYUPVKdT`)

**Grouping by ability across party tiers.** [Fleshforge Construct](creatures/fleshforge-construct.toml)
groups by party level and names the ability in each title. Each of the six
abilities appears on a Dreg and a Prototype. Review whether to offer an
ability grouping across both forms.
