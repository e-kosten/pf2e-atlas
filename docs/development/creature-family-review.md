# Creature family review

The [creature audit](developer-commands.md#review-creature-families) on Foundry
`28d9d156f252d084371d994dc359509b49eadf18` returned 186 same-publication shared-token
groups. Four were covered by the pilot. This batch records decisions for
23 of the remaining 182 candidates, leaving 159 untouched in that queue.
Two pilot groups appear below because their publications were reviewed together.
One candidate has both accepted and deferred members.

Regenerate the audit at the recorded source revision and match the exact actor
keys below. Accepted rows point to the family TOML for membership and evidence.
Reopen decisions when an actor is added, removed, or its relevant source evidence
changes. Compare overlapping groups directly before assigning any new members.

## First batch

| Publication | Shared token | Exact candidate keys | Disposition |
| --- | --- | --- | --- |
| Pathfinder #167: Ready? Fight! | Rivka | `fists-of-the-ruby-phoenix-bestiary:56U7JoNKbci67vRE`, `fists-of-the-ruby-phoenix-bestiary:EYr9GBleLFezHqBk`, `fists-of-the-ruby-phoenix-bestiary:MdkbAYfKgIvtTxBO`, `fists-of-the-ruby-phoenix-bestiary:k5UjbiYr87NVK7qp`, `fists-of-the-ruby-phoenix-bestiary:zpXQHbe037haBVmD` | [rivka](../../crates/atlas-search/families/creatures/rivka.toml) |
| Pathfinder #169: Kindled Magic | Spellskein | `strength-of-thousands-bestiary:7aqbEcMv3d1ZVBEz`, `strength-of-thousands-bestiary:9bv6UWHz66TRwP4H`, `strength-of-thousands-bestiary:ZBG4tyw04lsISbHL` | [spellskein](../../crates/atlas-search/families/creatures/spellskein.toml) |
| Pathfinder #180: The Smoking Gun | Corrupt Shieldmarshal | `outlaws-of-alkenstar-bestiary:CNFAfmVoVebnj7bY`, `outlaws-of-alkenstar-bestiary:RpGReUxsaT8wCz2u` | [corrupt-shieldmarshal](../../crates/atlas-search/families/creatures/corrupt-shieldmarshal.toml) |
| Pathfinder Bestiary 3 | Living Graffiti | `pathfinder-bestiary-3:IyxWVWAi9BjKPxop`, `pathfinder-bestiary-3:SIE8CcEblKI9VwqO`, `pathfinder-bestiary-3:XfHvnixZO9zVwZxC`, `pathfinder-bestiary-3:m0w5VfUTRvRcGfba` | [living-graffiti](../../crates/atlas-search/families/creatures/living-graffiti.toml) |
| Pathfinder Bestiary 3 | Living Rune | `pathfinder-bestiary-3:EMT6L7RarzkSiEOq`, `pathfinder-bestiary-3:WfCLrFwwWSA7KRUu`, `pathfinder-bestiary-3:pCDp2hdim12wonW5`, `pathfinder-bestiary-3:tvAlNMQluKDpfXMz` | [living-rune](../../crates/atlas-search/families/creatures/living-rune.toml) |
| Pathfinder Bestiary 3 | Wizard Sponge | `pathfinder-bestiary-3:Al5OHM0hbWcqIplK`, `pathfinder-bestiary-3:TeaF0WreNshQxbe8`, `pathfinder-bestiary-3:dKFzkdgTntTm8ydA`, `pathfinder-bestiary-3:rqNNvJO0XEVHWwbW`, `pathfinder-bestiary-3:sNeIVL8w7NPFtpK2` | [wizard-sponge](../../crates/atlas-search/families/creatures/wizard-sponge.toml) |
| Pathfinder #164: Hands of the Devil | Barbazu | `abomination-vaults-bestiary:4bznEiwsJvInwZwA`, `abomination-vaults-bestiary:A4MusxxoLxwMVZua`, `abomination-vaults-bestiary:vX4SBXIICKfrM4pF` | Deferred. The three Hands of the Devil encounters change the Barbazu’s equipment. Review the Pathfinder Bestiary base alongside the encounter blocks before deciding how the family should present them. |
| Pathfinder Game Night: The Dragon's Crown | NPC | `dawn-of-the-frogs:YPyw9jQlFnb5iHwq`, `dawn-of-the-frogs:rB5PGOKN1b8SukWs` | Keep separate. King Stankus is a goblin aberration and Poppy is an animal with a different stat block. Their shared token label is a placeholder. |
| Pathfinder Society Intro: Year of Boundless Wonder | Shard of Aslynn | `pfs-season-4-bestiary:JGbgYTBRSSf94v0U`, `pfs-season-4-bestiary:OOwfdaGQYDknAH4e` | [shard-of-aslynn](../../crates/atlas-search/families/creatures/shard-of-aslynn.toml) |
| Pathfinder Society Quest #14: The Swordlord's Challenge | Dura | `pfs-season-4-bestiary:RG0E58wEmyclbeJA`, `pfs-season-4-bestiary:iacML9PbFyo49TMe`, `pfs-season-4-bestiary:xiowQvReC58NPbD5`, `pfs-season-4-bestiary:yMIbNWGAZTLYUFVI` | [dura](../../crates/atlas-search/families/creatures/dura.toml) |
| Pathfinder Society Quest #14: The Swordlord's Challenge | Halran | `pfs-season-4-bestiary:hm3ZkeNOSM8b5KLo`, `pfs-season-4-bestiary:m68Zvfw17TYfoacL`, `pfs-season-4-bestiary:t4trkvNuN5cQudRR`, `pfs-season-4-bestiary:wvH5n8xxN8rYeNeC` | [halran](../../crates/atlas-search/families/creatures/halran.toml) |
| Pathfinder Society Quest #14: The Swordlord's Challenge | Rhenei | `pfs-season-4-bestiary:K9ClMcqPZ4BdZ7iU`, `pfs-season-4-bestiary:MGcRvpxJjShaEIoD`, `pfs-season-4-bestiary:gqQoahBEk37LqeTL`, `pfs-season-4-bestiary:p35bUOIZohXTurQM` | [rhenei](../../crates/atlas-search/families/creatures/rhenei.toml) |
| Pathfinder Society Quest #14: The Swordlord's Challenge | Thisnabel | `pfs-season-4-bestiary:3TEapF0mBKvW3Rqf`, `pfs-season-4-bestiary:TMiMJVvmOhZVQkVW`, `pfs-season-4-bestiary:gApYWrEZpYI0fOnk`, `pfs-season-4-bestiary:weilkjJuw0L1iFdZ` | [thisnabel](../../crates/atlas-search/families/creatures/thisnabel.toml) |
| Pathfinder Society Quest #9: Wayfinder Origins | Toxic Snapjaw | `pfs-season-1-bestiary:KcLEkekZZw9uI88I`, `pfs-season-1-bestiary:a15tV7snudRPQXHO` | [toxic-snapjaw](../../crates/atlas-search/families/creatures/toxic-snapjaw.toml) |
| Pathfinder Society Quest #9: Wayfinder Origins | Prefect | `pfs-season-1-bestiary:7PXPcv8x3Wnc730i`, `pfs-season-1-bestiary:GdYMRHVctjyvOD7D`, `pfs-season-1-bestiary:NQgsuMnXXABeebuS`, `pfs-season-1-bestiary:oKACWOSiBpxgMqa4` | Already covered by [prefect](../../crates/atlas-search/families/creatures/prefect.toml), including the Elite Fist record outside the shared-token group. |
| Pathfinder Society Scenario #1-04: Bandits of Immenwood | Animal Companion (Wolf) | `pfs-season-1-bestiary:9KauvSo8zxsaBWRc`, `pfs-season-1-bestiary:czYt4Yp6ZeGnzyV6` | Deferred for presentation review. Both Bandits of Immenwood records have creature level 0 and share the same friendly title, but have different HP and party tiers. With the current compact Search cards the tier is needed to distinguish them. |
| Pathfinder Society Scenario #1-04: Bandits of Immenwood | Halfling Druid | `pfs-season-1-bestiary:Jfm63bPimFxf3u1T`, `pfs-season-1-bestiary:eWlUE6xrNLGMcfMX` | [halfling-druid](../../crates/atlas-search/families/creatures/halfling-druid.toml) |
| Pathfinder Society Scenario #1-04: Bandits of Immenwood | Vengeant Thorn | `pfs-season-1-bestiary:FXwYc62zORdcfnYb`, `pfs-season-1-bestiary:NmkeNdV7SWYG7bYe` | [vengeant-thorn](../../crates/atlas-search/families/creatures/vengeant-thorn.toml) |
| Pathfinder Society Scenario #2-09: The Seven Secrets of Dacilane Academy | Anatomical Model | `pfs-season-2-bestiary:BXnmDgVkhD97fI1o`, `pfs-season-2-bestiary:ZshsztmoWjDZEqGT`, `pfs-season-2-bestiary:j7K9SefeHJ16qHMl` | Deferred pending source reconciliation. The Foundry Elite and ordinary 3–4 blocks both report creature level 4 and have no typed adjustment. The Elite block has 20 more HP. Verify the published level, scaling condition, and effective stats before declaring an override. |
| Pathfinder Society Scenario #2-09: The Seven Secrets of Dacilane Academy | The Scholar of Sorts | `pfs-season-2-bestiary:6kRWSFIjPusy4d58`, `pfs-season-2-bestiary:JxJ3vbV7piLxZrJ9` | [the-scholar-of-sorts](../../crates/atlas-search/families/creatures/the-scholar-of-sorts.toml) |
| Pathfinder Society Scenario #3-15: Cavern of Sundered Songs | Caliclotherax | `pfs-season-3-bestiary:AaCmgrEQ4sPakXca`, `pfs-season-3-bestiary:XyrToL9Wyl4MoAf1` | [caliclotherax](../../crates/atlas-search/families/creatures/caliclotherax.toml) |
| Pathfinder Society Scenario #3-15: Cavern of Sundered Songs | Eltha Embercall | `pfs-season-3-bestiary:3r3etd3bUOn58TH3`, `pfs-season-3-bestiary:pYR3Ovkd9eKQQ5qE`, `pfs-season-3-bestiary:xiBVzPUwh1QI2lIs` | Already covered by [eltha-embercall](../../crates/atlas-search/families/creatures/eltha-embercall.toml). |
| Pathfinder Society Scenario #3-15: Cavern of Sundered Songs | Sparkles | `pfs-season-3-bestiary:8CEKF7wq9qZzmveH`, `pfs-season-3-bestiary:SkV5zYJgu7hdpZE0` | Deferred for presentation review. Both Cavern of Sundered Songs records have creature level 9. They share the friendly title but differ by party tier and HP, so the current compact Search cards would be hard to tell apart. |
| Pathfinder Society Scenario #3-19: Mean Streets of Shadow Absalom | Arisept | `pfs-season-3-bestiary:LG2pZMbLXZKtcWZh`, `pfs-season-3-bestiary:gtcISL1zMhQ0mv5g` | [arisept](../../crates/atlas-search/families/creatures/arisept.toml) |
| Pathfinder Society Scenario #3-19: Mean Streets of Shadow Absalom | Marcien Blakros | `pfs-season-3-bestiary:8tDDIvcinSurN8BF`, `pfs-season-3-bestiary:PqawYWEK0iqvcBZj`, `pfs-season-3-bestiary:qDpKFbhWgSWEEA00`, `pfs-season-3-bestiary:xVmDwoBxjKyCihYz` | [marcien-blakros](../../crates/atlas-search/families/creatures/marcien-blakros.toml) covers Marcien. The two Shadow Double keys remain deferred. They represent summoned copies and need their own relationship decision. |

The Swordlord’s Challenge thresholds were reviewed against the published
appendices and the encounter A scaling sidebars on pp. 12 and 15. “Determined”
is an encounter-scaling qualifier with its own stat blocks. The exact actor
titles are preserved in each declaration.

## Gaps to revisit

- **Missing token names.** The Spellskein and Wizard Sponge declarations include
  same-publication base records found during source review. Use those records
  and the two Hesla actors from The Overthrow of Ambition
  (`pfs-season-6-bestiary:EWutX9eW7tYLpors`,
  `pfs-season-6-bestiary:k6VFoe5R52PbVZJB`) to measure a follow-up discovery pass.
  Check the adventure before assigning a tier to the unsuffixed Hesla.
- **Cross-publication variants.** Compare the Hands of the Devil Barbazu blocks
  with the Bestiary actor `pathfinder-bestiary:FIoRPHaHdYUPVKdT`. Record
  the evidence for the relationship and review how the base should appear.
- **Same-level scenario uses.** Compare the Wolf and Sparkles pairs in Search
  before publishing friendly titles for either pair.
- **Authored adjustment conflicts.** Compare the three Anatomical Model blocks
  with the adventure’s Laboratory encounter and scaling instructions. Establish
  the effective level, stats, and conditions before adding a scoped override.
- **Summoned copies.** Revisit Marcien’s doubles
  (`pfs-season-3-bestiary:PqawYWEK0iqvcBZj`,
  `pfs-season-3-bestiary:xVmDwoBxjKyCihYz`) and decide whether they belong
  in their own family and how to represent their link to the summoner.

Record newly discovered groups by signal so the baseline count remains comparable.
Measure additional candidates and false matches before widening the review queue.
