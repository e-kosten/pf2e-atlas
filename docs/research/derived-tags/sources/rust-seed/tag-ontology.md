# Historical snapshot: tag-ontology.md

Captured: 2026-10-19. Archival evidence only; not implementation authority.
Original location: `.worktrees/tag-corpus-loader/docs/architecture/tag-ontology.md`.

The fenced payload preserves the original UTF-8 bytes. Historical links and
instructions inside it describe their original checkout, not current navigation
or authorization. See [the source map](../../sources-and-cleanup.md) from nested
snapshots, or locate `sources-and-cleanup.md` at the research directory root.

````markdown
# Tag Ontology

This document describes the desired PF2e Atlas tag ontology at a product-design level. It is the durable reference for tag groups, subgroups, tag ids, and naming conventions.

The runtime ontology is implemented by validated registry files under `data/tags/registry/` and validated catalog files under `data/tags/catalog/`. Registry YAML is the machine-readable source for group and subgroup presentation metadata. Catalog YAML is the machine-readable source for tag ids, labels, applicability, and assignment guidance. This document defines the intended taxonomy those files should implement.

## Ontology Principles

- Prefer clear domain names over names joined by `And` to cover loosely related concepts.
- Keep presentation groups about one durable retrieval domain. Subgroups should refine that domain, not create a second taxonomy axis.
- Applicability should usually be broad. A tag can be applicable to a kind of record without every record of that kind deserving the tag.
- Use `foundry_record_types` only when the Foundry subtype is part of the concept. Do not add subtype refinements merely because the first examples were equipment.
- Prefer cross-cutting tags when the user question crosses record kinds. A GM asking "what can help the party survive fire?" should be able to find spells, equipment, feats, and character options through the same concept.
- Avoid composite tags whose only purpose is to bundle other tags. Add a composite only when users would naturally search for that concept directly.

## Presentation Groups

### Setting

Where a record strongly belongs or is useful in encounter, exploration, or story prep.

Subgroups:

- `Habitat`: aquatic, arctic, desert, forest, mountain, underground, swamp, sky, urban, wasteland.
- `Planar`: First World, Shadow Plane, Astral Plane, elemental planes, upper planes, lower planes.
- `Regional`: Darklands, gothic horror lands, magic-blighted wastelands, demon-scarred frontiers.
- `Site`: tomb, temple, dungeon, battlefield, bridge or chokepoint, settlement, wilderness ruin.
- `Culture`: factions, ancestries, societies, religions, organizations, civic identities.
- `Institution`: schools, churches, guilds, armies, courts, governments, formal orders.
- `Era`: ancient empire, mythic past, post-war landscape, pre-cataclysm remnant, ruined age.

### EncounterRole

How creatures or encounter pieces function in a scene.

Subgroups:

- `Combat`: ambusher, artillery, brute, commander, controller, defender, skirmisher, support.
- `Scene`: pack hunter, patrol member, guardian retinue, cult member, criminal cell, court entourage.
- `Social`: authority figure, informant, criminal, religious figure, merchant, scholar, guard.
- `ThreatVector`: poison threat, disease vector, possession threat, regeneration threat, spawn creator, infiltration threat.

### ProblemResponse

Records retrieved because they help solve a concrete obstacle or question.

Subgroups:

- `Countermeasure`: counteract magic, condition removal, curse removal, poison treatment, disease treatment, source neutralization.
- `Access`: lock bypass, trap bypass, barrier bypass, mechanism manipulation.
- `Investigation`: source detection, magic detection, truth revelation, diagnosis.
- `Communication`: language translation, message delivery, telepathic communication, signaling.
- `Restoration`: repair, restore, recover, undo damage, replenish resources, make whole again.
- `RiskMitigation`: prevention, contingency, warning, avoidance, harm reduction before a problem fully lands.

Tags:

- `problem.counteract_magic`: suppress, dispel, counteract, or remove active magical effects.
- `problem.condition_removal`: remove, suppress, delay, or recover from harmful conditions and ongoing afflictions.
- `problem.curse_removal`: identify, suppress, remove, reverse, or resolve active curses.
- `problem.poison_treatment`: prevent, delay, neutralize, treat, or recover from poison.
- `problem.disease_treatment`: prevent, treat, counteract, suppress, or recover from disease.
- `problem.fear_recovery`: remove, suppress, prevent, or recover from fear, panic, dread, or fear-driven impairment.
- `problem.contamination_cleanup`: purify, cleanse, neutralize, or contain contamination, taint, pollution, or corrupting residue.
- `problem.bleed_treatment`: stop, reduce, suppress, or recover from bleed and similar ongoing blood-loss effects.
- `problem.petrification_reversal`: reverse, suppress, counteract, delay, or recover from petrification.
- `problem.paralysis_recovery`: remove, suppress, delay, mitigate, or recover from paralysis.
- `problem.possession_expulsion`: expel, suppress, sever, counteract, or recover from possession or hostile presence.
- `problem.parasite_removal`: remove, expel, extract, suppress, or recover from parasites, implants, invasive organisms, or attached creatures.
- `problem.lock_bypass`: open, unseal, circumvent, or defeat locks, sealed containers, secured doors, or restraints.
- `problem.trap_bypass`: disarm, suppress, safely trigger, or bypass traps and warded access points.
- `problem.barrier_bypass`: circumvent walls, force barriers, sealed passages, blocked routes, or other non-lock obstacles.
- `problem.barrier_removal`: break, destroy, demolish, open, or remove physical or magical access barriers.
- `problem.mechanism_manipulation`: operate, alter, jam, disarm, control, or safely manipulate mechanisms and control interfaces.
- `problem.restraint_escape`: escape, loosen, open, slip, or break bonds, manacles, grapples, immobilization, or confinement.
- `problem.source_detection`: identify the hidden source, anchor, origin point, contamination engine, or cause behind an ongoing danger.
- `problem.diagnosis`: determine what kind of problem, affliction, malfunction, hazard, or abnormal state is present.
- `problem.magic_detection`: detect, identify, locate, read, or reveal magical auras, effects, items, or active magical sources.
- `problem.truth_revelation`: reveal lies, false appearances, hidden facts, deceptive claims, or concealed truths.
- `problem.invisibility_detection`: reveal, locate, sense, or defeat hidden presence from invisibility or similar unseen states.
- `problem.source_neutralization`: disable, sever, destroy, purify, cleanse, or neutralize a hidden source or anchor.
- `problem.language_translation`: bridge spoken or written language barriers.
- `problem.message_delivery`: send, carry, project, store, or relay messages when ordinary conversation is unavailable.
- `problem.telepathic_communication`: enable mind-to-mind communication, silent coordination, or private mental exchange.
- `problem.signaling`: create signals, markers, warnings, calls, beacons, or simple coordination cues.
- `problem.repair`: fix, restore, mend, rebuild, or return damaged objects, equipment, structures, or mechanisms to usable function.
- `problem.emergency_restoration`: quickly restore function, consciousness, control, or survivability after incapacitation, loss of control, or sudden failure.
- `problem.alarm`: warn of intrusion, danger, triggering events, tampering, or approaching threats.
- `problem.hazard_mitigation`: shield, ward, insulate, anchor, or reduce danger from hazards and hazardous environments.
- `problem.scrying_protection`: block, misdirect, reveal, or reduce remote magical observation and divination surveillance.
- `problem.fall_protection`: prevent, slow, cushion, mitigate, or recover from dangerous falls and forced drops.
- `problem.death_prevention`: prevent death, delay fatal consequences, or preserve life at the edge of death.

### Exploration

Travel, scouting, survival, infiltration, and field activity outside the `CharacterBuild` lens.

Subgroups:

- `Traversal`: flight, aquatic movement, climbing, terrain traversal, teleportation, route access.
- `Reconnaissance`: scouting, tracking, remote observation, extended senses.
- `Survival`: environmental adaptation, sustenance, hostile-climate endurance, exposure protection.
- `Shelter`: field shelter, protected rest, campsite refuge, temporary safe haven.
- `Logistics`: carrying capacity, supplies, light, tools, mounts, vehicles, camp operations.

Tags:

- `exploration.flight`: flying movement, sustained aerial travel, or practical airborne maneuvering.
- `exploration.aquatic_travel`: swimming, underwater breathing, water-surface travel, or aquatic operations.
- `exploration.environmental_adaptation`: endurance against hostile climates, pressure, smoke, altitude, vacuum, or other expedition extremes.
- `exploration.field_shelter`: shelter, refuge, protected rest, or field habitation support.
- `exploration.scouting`: distant observation, extended senses, forward inspection, or remote scene information.
- `exploration.tracking`: locating and following a specific creature, object, destination, or supernatural trail.
- `exploration.stealth`: moving quietly, avoiding notice, suppressing presence, or supporting covert approach.
- `exploration.anti_tracking`: hiding trails, obscuring passage, defeating pursuit, or preventing tracking.
- `exploration.navigation`: determining direction, following routes, avoiding becoming lost, or maintaining course.
- `exploration.teleportation`: moving creatures or objects across distance or around terrain without ordinary travel.
- `exploration.sustenance`: providing food, water, breathing support, rest support, or other expedition basics.

### CombatEffect

What an effect does inside an encounter or rules interaction.

Subgroups:

- `BattlefieldControl`: barrier creation, area denial, mobility denial, action denial.
- `ForcedPosition`: forced movement, pull, push, drag, reposition.
- `SensoryControl`: line-of-sight control, concealment, silencing, scent denial, other sensory interference.
- `Damage`: direct damage, area damage, persistent damage, death burst, damage-over-time pressure.
- `Debilitation`: fear, slowed, stupefied, drained, sickened, action loss, other harmful combat debuffs.
- `Protection`: resistance, immunity, mitigation, warding when the focus is the effect rather than a build option.

Tags:

- `combat.control.barrier_creation`: create walls, domes, cages, force barriers, or other blocking structures.
- `combat.control.forced_movement`: push, pull, drag, teleport, slide, or reposition a target against its will.
- `combat.control.mobility_denial`: slow, ground, pin, immobilize, restrain, entangle, or prevent normal repositioning.
- `combat.control.line_of_sight`: block vision, create concealment, obscure sight lines, or deny clear observation.
- `combat.control.silencing`: suppress speech, sound production, verbal casting, audible signals, or voice-dependent actions.
- `combat.damage.burst`: deliver concentrated short-window damage.
- `combat.damage.persistent`: apply recurring damage, bleed, burning, poison ticks, or other ongoing harm.
- `combat.debilitation.fear_pressure`: force fear, panic, dread, morale collapse, or fear-driven action loss.
- `combat.debilitation.confusion`: disrupt agency, targeting, action choice, or coherent behavior.
- `combat.protection.resistance`: grant resistance, mitigation, immunity-like protection, or damage-type shielding.

### Theme

Genre, moral, relationship, visual, object, or creature motifs that help a GM find material for a scene or story.

Subgroups:

- `Genre`: gothic horror, folk horror, cosmic horror, sword-and-sorcery, science fantasy.
- `MoralConflict`: redemption, sacrifice, justice, taboo, hubris, corruption-as-choice.
- `Relationship`: betrayal, loyalty, family, rivalry, patronage, forbidden bond.
- `VisualSymbol`: shadow, flame, bone, mirror, blood, crystal, fungal, clockwork.
- `CreatureArchetype`: parasite, swarm, undead court, fairy trickster, artificial servant.
- `ObjectSymbol`: clockwork, cursed object, living weapon, ritual focus, relic.

### SceneChallenge

Scene structures, obstacles, threats, and pressure patterns that shape how a scene plays.

Subgroups:

- `ResolutionPattern`: layered resolution, observation first, timing window, hidden source, puzzle-like interaction.
- `SpatialConstraint`: threshold lockdown, chokepoint, forced crossing, containment, blocked route.
- `EscalationPressure`: reinforcement, pursuit, endurance pressure, resource pressure, countdown, spreading threat.
- `EnvironmentalThreat`: flooding chamber, collapsing structure, battlefield hazard, wilderness hazard, fire, smoke, choking terrain.
- `SocialObstacle`: negotiation barrier, deception web, faction standoff, public pressure, legal or civic obstacle.
- `Mystery`: clue chain, hidden culprit, false lead, reveal, investigation scene.
- `HauntPattern`: appeasement, haunt source, spiritual repetition, unresolved death, or other haunt-specific mechanical pattern.

Tags:

- `scene.resolution.layered_resolution`: multiple linked pieces must be solved together.
- `scene.resolution.observation_first`: careful watching, clue gathering, pattern reading, or interpretation is rewarded before safe action.
- `scene.resolution.timing_window`: the scene is best handled by acting during the right cycle, opening, lull, or timing pattern.
- `scene.resolution.hidden_source`: finding a hidden source, anchor, origin, or engine is part of resolving the scene.
- `scene.spatial.threshold_lockdown`: gates, bridges, chokepoints, stairwells, doors, or forced-passage bottlenecks are controlled.
- `scene.spatial.blocked_route`: route obstruction, forced detour, sealed passage, or obstructed travel is the central constraint.
- `scene.pressure.reinforcement`: the scene escalates by calling, creating, summoning, attracting, or enabling additional threats.
- `scene.pressure.spreading_threat`: danger expands through contagion, contamination, fire, overgrowth, collapse, flooding, or other spread.
- `scene.environment.collapse`: falling structures, cave-ins, floor failure, debris, or unstable architecture are the main environmental threat.

### CharacterBuild

Character-facing options and repeatable tactical roles for PCs, NPCs, enemies, companions, and loadouts. This group answers both "what can I give my players to help them do this?" and "what can I give their enemies to create this tactical experience?"

Subgroups:

- `Offense`: burst damage, sustained damage, area damage, precision damage, anti-caster pressure.
- `Defense`: personal defense, party defense, reactive defense, resistance, mitigation, emergency survival.
- `Support`: healing, condition removal, buffing, action economy, coordination, resource support.
- `Mobility`: personal mobility, party mobility, terrain traversal, escape, repositioning.
- `Utility`: stealth support, infiltration support, social support, skill support, scouting support.
- `ResourceSupport`: resource conservation, recovery, preparation, replenishment, or ways to stretch party resources.
- `Skill`: skill niche support, downtime niche support, social niche support, crafting niche support.
- `Summoning`: summon support, companion support, minion support, eidolon support, conjured battlefield role.

Tags:

- `build.defense.party`: protects allies, improves group defense, redirects harm, or helps the party survive together.
- `build.defense.personal`: improves the user's own AC, saves, resistance, mitigation, survivability, or personal escape.
- `build.defense.reactive`: triggers in response to attacks, failed saves, damage, being targeted, or incoming danger.
- `build.offense.burst`: produces strong short-window damage or decisive opening pressure.
- `build.offense.sustained`: produces repeatable damage over multiple rounds.
- `build.offense.area`: damages or pressures multiple targets or zones.
- `build.offense.precision`: rewards accuracy, setup, weaknesses, precision, or single-target exploitation.
- `build.support.healing`: restores Hit Points, stabilizes, or improves recovery.
- `build.support.condition_removal`: removes or mitigates harmful states.
- `build.support.buffing`: improves ally performance.
- `build.support.action_economy`: grants, saves, or improves actions.
- `build.mobility.personal`: improves the user's own movement or repositioning.
- `build.mobility.party`: improves shared movement, travel speed, or group traversal.
- `build.utility.stealth`: helps avoid notice, move quietly, disguise presence, or support covert approach.

### UsagePattern

Cross-record usage patterns that describe when and how an option, effect, creature, item, or hazard is used over time.

Subgroups:

- `OpeningPlay`: opener, prebuff, first-round tool, early positioning tool, encounter setup.
- `SustainedUse`: sustained stance, repeatable routine, maintenance action, attrition plan.
- `ResourceUse`: consumable burst, limited daily power, renewable power, charge-based use, expendable setup.
- `ReactiveUse`: counterplay, interrupt, punishment, emergency response, trigger-driven play.
- `SetupPayoff`: setup payoff, combo enabler, mark-and-trigger, delayed reward, preparation-dependent effect.

### StatusEffect

Conditions and afflictions as problems, progressions, mechanisms, and body-state changes.

Subgroups:

- `AfflictionProgression`: slow burn, staged escalation, contagious spread, relapse, hidden onset.
- `AfflictionMechanism`: disease, poison, curse, fear, paralysis, petrification, staged onset, contagious spread.
- `MentalState`: frightened, confused, controlled, fascinated, stupefied, emotion-based states.
- `Impairment`: slowed, clumsy, enfeebled, drained, sickened, blinded, deafened.
- `OngoingDamage`: persistent damage, bleed, poison ticking damage, recurring harm.
- `Transformation`: polymorph, mutation, body alteration, form change.
- `PhysiologyOverride`: polymorph, petrification, possession, paralysis, sleep, body control.

## Open Design Notes

- Some tags can reasonably appear in more than one user's mental model. The baseline remains one primary presentation path per tag; related tags can carry secondary navigation later.
- If a concept applies to spells, equipment, feats, and character options, prefer one `build.*` tag with broad applicability over parallel `spell.*` and `equipment.*` tags.
````
