# Creature family review notes

Use these notes with the [creature family review procedure](../../../docs/development/source-research.md#creature-family-review-during-a-source-upgrade). The [family TOMLs](creatures/) record accepted relationships and their sources. Entries here record reviewed exceptions and open questions.

The entries below were reviewed against Foundry PF2e revision
`28d9d156f252d084371d994dc359509b49eadf18`. The accepted families and remaining
comparisons in the later same-publication pass were checked against
`52dfeee703b887237f5e454747bcae45b136f9af`. Actor names are exact Foundry titles.

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

**Anatomical Model.** The [family](creatures/anatomical-model.toml) presents
the three pinned actors by party levels. Laboratory A11 on p. 22 of *The
Seven Secrets of Dacilane Academy* prints the 3–4 Elite model at creature
level 5. Foundry reports level 4 for both the ordinary and Elite 3–4
blocks. The printed level 3 Elite for party levels 1–2 has no pinned
actor. Recheck when the source changes.
[Published scenario](https://www.scribd.com/document/704766987/PF2-S02-09-the-Seven-Secrets-of-Dacilane-Academy)

**Alystair Caskwater.** The [family](creatures/alystair-caskwater.toml) uses
both party tiers from encounter A8 of *In the Footsteps of Horror*. Its lower
Vengeful block (`pfs-season-5-bestiary:nMMRUST4ow0kcbTy`) still needs comparison
with the printed stat block. The lower substitution depends on Challenge
Points and party size, so the family shows the party tier for that block.
[Published quest](https://www.studocu.com/pt-br/document/universidade-federal-do-estado-do-rio-de-janeiro/lingua-portuguesa-na-educacao/pf2-quest-15-in-the-footsteps-of-horror-adventure-guide/149792260)

**The Warden.** The [family](creatures/the-warden.toml) uses the tiers in
the Foundry actor names. Verify which conditions in *The Winter Queen's
Dollhouse* select the Upset blocks before adding encounter guidance.

### Source identity and publication

**Boggard Swampseer.** The [family](creatures/boggard-swampseer.toml) includes
the Monster Core actor and the three *Lost Legend* actors. The publication,
pack paths, and printed encounter A identify scenario 1-20. The Foundry
titles say 1-24. Pages 21 and 26 document the staff loadouts. The printed
Elite block reports creature level 4, while both Foundry Elite actors
store creature level 3 with a typed Elite adjustment. Recheck the source
titles and effective Elite statistics when the pin changes.
[Published scenario](https://www.scribd.com/document/655349423/PF2-S01-20-The-Lost-Legend)

**Phistophilus.** The pinned *Things Go to Hell* actor uses Unique rarity
and the *Monster Core* actor uses Common. In the two Foundry stat blocks,
the creature level, numerical stats, spell list, action text, and strike
statistics match. Verify the published encounter for any scenario-specific
changes before declaring a canonical link.

- Phistophilus (`one-shot-bestiary:3QyqHIuAXE3YVLUh`) in *Things Go to Hell*
- Phistophilus (`pathfinder-monster-core:3QyqHIuAXE3YVLUh`) in *Monster Core*

### Marcien's Shadow Doubles

The [Marcien Blakros family](creatures/marcien-blakros.toml) groups the
summoner and his copies by party tier. Encounter A3 gives the copies the
Summoned trait. The pinned 9–10 Shadow Double
(`pfs-season-3-bestiary:xVmDwoBxjKyCihYz`) omits that trait. Recheck if
the pinned Foundry source changes.

### Published adjustments and scaling

**Cosmic Amoeba.** The [family](creatures/cosmic-amoeba.toml) presents the
two pinned records. The reader and summary show the Weak actor's stored
creature level 1, while the encounter applies its typed Weak adjustment.
The adventure prints the Weak block at level −1 and also has an Elite block
without a pinned actor. Recheck effective damage and DCs before declaring an
override.

**Hound Echo.** The [family](creatures/hound-echo.toml) presents the
four records from *Stranded on Yesterday's Tide*. Foundry stores the Weak
blocks as typed adjustments on creature levels 2 and 4. The reader and
summary currently show those source levels, while the encounter applies Weak
and yields the printed levels 1 and 3. The encounter's scaling can use
ordinary and Weak blocks together.
[Published scenario](https://www.scribd.com/document/820438906/PF2-S05-17-Stranded-on-Yesterday-s-Tide)

**Congealed Magic.** The [family](creatures/congealed-magic.toml) presents
ordinary and Concentrated forms with typed Elite counterparts. Summaries
show stored creature levels 3 and 5. Verify exact encounter scaling and
effective statistics before declaring party uses or overrides.

**Student Exchange.** [Commodore Sticky Fingers](creatures/commodore-sticky-fingers.toml),
[Sewer Ooze](creatures/sewer-ooze.toml), and [Sprite](creatures/sprite.toml)
show their quest blocks individually. The ordinary Sprite has 11 HP and
different attack and Stealth bonuses from the 7 HP Monster Core Sprite.
The quest Sewer Oozes are authored Weak and Elite blocks with different
stats from the Monster Core block. Verify encounter scaling before defining
an override.

**Jinkin.** The [Jinkin family](creatures/jinkin.toml) presents the quest's
ordinary and Lesser blocks. The Lesser Jinkin
(`pfs-season-6-bestiary:D7XibY38wx9AjopU`) has Thievery +7 in Foundry and +5
on p. 16. The balcony scaling instructions add the Lesser block in specific
bands. Recheck Thievery if the pinned source changes.

**Friends in Need.** [Gluttonous Geode](creatures/gluttonous-geode.toml)
and [Wood Scamp](creatures/wood-scamp.toml) show their quest blocks
individually. Compared with Rage of Elements, the ordinary quest Geode omits
Petran. The ordinary quest Wood Scamp omits the climb Speed and regeneration
text and changes its language and traits. Verify encounter scaling before
defining an override.

**Animated Flame.** The [family](creatures/animated-flame.toml) presents
the ordinary, Weak, and Elite blocks separately. The Elite actor has
distinct task-related abilities. Check the published use before declaring
party ranges or an override.

**Animate Dream.** The [family](creatures/animate-dream.toml) includes the
Monster Core 2, Gatewalkers, Lesser, Faded, and Elite records. The two
unqualified blocks have different resistance exceptions. Verify the
conditions that select the Faded and Elite blocks before declaring
party uses or overrides.

## Discovery and grouping gaps

**Missing token names.** The [Spellskein](creatures/spellskein.toml),
[Wizard Sponge](creatures/wizard-sponge.toml), and
[Hesla](creatures/hesla-the-broken-ember.toml) families include
same-publication actors the token audit missed. Use them to check future
discovery passes.

**Grouping by ability across party tiers.** [Fleshforge Construct](creatures/fleshforge-construct.toml)
groups by party level and names the ability in each title. Each of the six
abilities appears on a Dreg and a Prototype. Review whether to offer an
ability grouping across both forms. The axes model requires two dimensions
and unique coordinates. Adding Form alongside Ability would add a grouping
with the same members as Party levels.

### Same-publication actor pairs

The [Champion of Asmodeus](creatures/champion-of-asmodeus.toml) and
[Rival Corpsekiller](creatures/rival-corpsekiller.toml) relationships rely on
the pinned Foundry actors. Their exact roles in the printed encounters remain
unverified.

#### Pathfinder Society Quest #15: In the Footsteps of Horror

The [Arisen Pixies family](creatures/arisen-pixies.toml) uses the reviewed
party levels 1–2 for the Shadow pair. The two Umbral blocks share the
signature ability and strike kit. Their exact party ranges still need
verification.

The [Shadow Draugr family](creatures/shadow-draugr.toml) retains both
published blocks as selectable variants. Foundry gives both actors a
greataxe, and the printed blocks use greatswords. Recheck the weapon if the
pinned source changes.
[Published quest](https://www.studocu.com/pt-br/document/universidade-federal-do-estado-do-rio-de-janeiro/lingua-portuguesa-na-educacao/pf2-quest-15-in-the-footsteps-of-horror-adventure-guide/149792260)

#### Pathfinder Society Quest #17: Escorting a Mirage

The [Mercenary Squad](creatures/mercenary-squad.toml),
[Foolhardy Mercenary](creatures/foolhardy-mercenary.toml),
[Trained Vulkariki Eidolon](creatures/trained-vulkariki-eidolon.toml), and
[Div Worshipper](creatures/div-worshipper.toml) families use the pinned
Foundry stat blocks. The standard-to-Weak or standard-to-Elite pairs differ
by two creature levels. The families show the authored Weak and Elite blocks
as selectable records. Verify the encounter's scaling instructions before
declaring a scoped adjustment.

#### Pathfinder Society Quest #21: Infernal Infiltration

The [Silver Devil Hooligan family](creatures/silver-devil-hooligan.toml)
presents both authored blocks.

*Infernal Infiltration* prints the Elite Silver Devil Hooligan at level 1
(p. 14), while Foundry reports level 2. The ordinary block's ranged dagger
bonus is +9 in print and +6 in Foundry. The upper-tier Hideout replaces the
ordinary block with Elite at 19–22 and 33+ Challenge Points. Recheck both
blocks if the pinned source changes.
[Published quest](https://www.scribd.com/document/1039010466/21-Infernal-Infiltration)

- Elite Silver Devil Hooligan (`pfs-season-6-bestiary:MjckZs5NKL35d8QL`) ↔ Silver Devil Hooligan (`pfs-season-6-bestiary:XRecLPsSrHHFwWHI`)

#### Pathfinder Society Scenario #1-18: Lodge of the Living God

The [Glass River Midge family](creatures/glass-river-midge.toml) presents
the standard and authored Elite blocks. Verify the published tier and scaling
conditions before defining an override.

#### Pathfinder Society Scenario #4-15: In Glorious Battle

The [Smokesight Wights family](creatures/smokesight-wights.toml) groups the
Hunter, Stalker, Prowler, Captain, Deadeye, and Commander by role and by
standard or Elite stat block. The scaling tables determine which blocks to use
in each encounter. The [Combustive Zombie family](creatures/combustive-zombie.toml)
presents its standard and Elite blocks separately.

Recheck these pairs when their pinned Foundry records change.

- The ordinary Combustive Zombie has 125 HP in Foundry and 175 in print
  (pp. 31 and 36).
- Elite Deadeye uses its base AC and saves in Foundry (AC 27, Fort +16,
  Ref +20, Will +17). The printed Elite has AC 29 and saves +18, +22, +19
  (p. 34).
- Elite Commander has AC 33 in Foundry and AC 34 before raising its shield in
  print (p. 39).

[Published scenario](https://www.scribd.com/document/669037731/Pathfinder-Society-Season-4-Year-of-Boundless-Wonder-Scenario-15)

- Elite Combustive Zombie (`pfs-season-4-bestiary:2N9DhnUedzp3FHhM`) ↔ Combustive Zombie (`pfs-season-4-bestiary:vA5VwmGb7xZ0WQdM`)
- Elite Smokesight Deadeye Wight (`pfs-season-4-bestiary:4G6VvfIsceXfEZBB`) ↔ Smokesight Deadeye Wight (`pfs-season-4-bestiary:4r7haYeUSxjW3wtR`)
- Elite Smokesight Wight Commander (`pfs-season-4-bestiary:KV2LmRMc9ho9mBPy`) ↔ Smokesight Wight Commander (`pfs-season-4-bestiary:Q6BTF30qgInYi2BA`)

#### Pathfinder Society Scenario #4-99: Blessings of the Forest

The [Animated Dagger family](creatures/animated-dagger.toml) presents both
published blocks as selectable variants. Cathedral Event 4 uses the Elite
block at party levels 1–2 and 3–4 under different Challenge Point bands. Revisit
an override if Atlas can represent those encounter conditions accurately.
[Published scenario](https://www.scribd.com/document/911522567/PF2-S04-99-Blessings-of-the-Forest)

#### Pathfinder Society Scenario #5-03: Heidmarch Heist

The [Krosovahn Mendesil family](creatures/krosovahn-mendesil.toml) shows
both reviewed party ranges and the source creature levels.

*Heidmarch Heist* (pp. 17, 22, 25) uses creature levels 5 and 7 at party
levels 5–6 and 7–8. Both Foundry records report level 5, including the upper
block with evasion and 111 HP. Recheck if the pinned Foundry source changes.
[Published scenario](https://www.scribd.com/document/870508711/PZOPFS0503E)

- Krosovahn Mendesil (`pfs-season-5-bestiary:H5lFXWXFl7vKrugU`) ↔ Krosovahn Mendesil (7-8) (`pfs-season-5-bestiary:TWsApxv2Bx0n7M3i`)

#### Pathfinder Society Scenario #6-09: The Power of Legends

The [Penumbra family](creatures/penumbra.toml) uses the 11–12 range from
the stronger Foundry actor's title and infers 9–10 for the unsuffixed
block from the adventure's full 9–12 range. Check the inferred assignment
against the printed encounter when available.
[Paizo scenario listing](https://store.paizo.com/pfs-scenario-6-09/)

#### Pathfinder Society Scenario #7-03: A Foot in the Door

The [Timonithaea family](creatures/timonithaea.toml) presents both blocks
under the friendly name. The level 9 Foundry title says 7–9, outside the
published scenario's party levels 5–8. Verify both assignments before
declaring scenario uses.
[Paizo scenario listing](https://store.paizo.com/pathfinder-society-scenario-7-03-a-foot-in-the-door/)

- Timonithaea (7-9) (`pfs-season-7-bestiary:Idl8zxFPu9dyRA5h`) ↔ Timonithaea (`pfs-season-7-bestiary:UG5iXBqsLxRifjSW`)
