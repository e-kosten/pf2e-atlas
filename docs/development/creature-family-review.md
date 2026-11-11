# Creature family review

This developer log records dispositions for the pinned creature audit queue.
Atlas loads family declarations from the catalog TOMLs. The log preserves why
a candidate was kept separate or deferred and what would reopen it.

The [creature audit](developer-commands.md#review-creature-families) on Foundry
`28d9d156f252d084371d994dc359509b49eadf18` returned 186 same-publication shared-token
groups. Four were covered by the pilot. The first batch reviewed 23 of the
remaining 182, and the second reviewed another 51. The original queue has 108
candidates still untouched. Two pilot groups appear in the first batch because
their publications were reviewed together.

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

## Second batch

| Publication | Shared token | Exact candidate keys | Disposition |
| --- | --- | --- | --- |
| Pathfinder #220: Crypt of Runes | Animate Nightmare | `revenge-of-the-runelords-bestiary:3yq9O31OreSgxhux`, `revenge-of-the-runelords-bestiary:9m1zbd4EyWSYHOB3`, `revenge-of-the-runelords-bestiary:AJ5x96JQzEL1kxPc`, `revenge-of-the-runelords-bestiary:MVvscUip8Qu7AgWl` | [animate-nightmare](../../crates/atlas-search/families/creatures/animate-nightmare.toml) |
| Pathfinder Adventure Path: Bastion of Blasphemies | Servitor Phantom | `bastion-of-blasphemies-bestiary:7KhhBPqRFYMDZmtm`, `bastion-of-blasphemies-bestiary:GT0G3ELbvhtfCw1A`, `bastion-of-blasphemies-bestiary:Mw0cHFd5a4RGEqPn`, `bastion-of-blasphemies-bestiary:buABPmUgUvhWxVaS` | [servitor-phantom](../../crates/atlas-search/families/creatures/servitor-phantom.toml) |
| Pathfinder Adventure Path: Bastion of Blasphemies | Velicarn | `bastion-of-blasphemies-bestiary:pRlJ0cepzWAyJeSJ`, `bastion-of-blasphemies-bestiary:weLjvcmjY81S9CV9` | [velicarn](../../crates/atlas-search/families/creatures/velicarn.toml) |
| Pathfinder Book of the Dead | Tormented | `book-of-the-dead-bestiary:0PrrwvV1936eSCQy`, `book-of-the-dead-bestiary:AXp5EzgP7p8FrYpN`, `book-of-the-dead-bestiary:VN2Vz1dxA9ti66bC`, `book-of-the-dead-bestiary:dcLHA7tihCF2Mraj`, `book-of-the-dead-bestiary:f2dAjBXK55w6rnsh`, `book-of-the-dead-bestiary:hjBcN7ulP6FSK7ie` | [tormented](../../crates/atlas-search/families/creatures/tormented.toml) |
| Pathfinder Monster Core | Shadow | `pathfinder-monster-core:clzHAfegb9Rn6DOe`, `season-of-ghosts-bestiary:IdbrdikgvRguKYNE` | Deferred. Monster Core’s Shadow Spawn and the Season of Ghosts pack’s Familiar Shadow share a publication field and token label. The generic Monster Core Shadow is outside the token group. Establish how each role relates to the base before adding a family. |
| Pathfinder Monster Core | Soulbound Doll | `pathfinder-monster-core:0PNjXE892kU89LyV`, `pathfinder-monster-core:59eY2OOFIuuj4f72`, `pathfinder-monster-core:5NnKgWCNbEzbSEN8`, `pathfinder-monster-core:6WgkWAumJDId0vuJ`, `pathfinder-monster-core:bdarrRaLjNmlYvNa`, `pathfinder-monster-core:cPRdGSSOmIIvAZVy`, `pathfinder-monster-core:meVRbBCY7LwIZq3t`, `pathfinder-monster-core:rATBpWa62XUPnGZb`, `pathfinder-monster-core:uaKlWSKuyZmt88ol`, `pathfinder-monster-core:v3jXIOHyQqXvImbc`, `pathfinder-monster-core:w2jByvjeZbAclGWf` | [soulbound-doll](../../crates/atlas-search/families/creatures/soulbound-doll.toml) |
| Pathfinder Society Quest #15: In the Footsteps of Horror | Alystair Caskwater | `pfs-season-5-bestiary:4JnqGDbDbDziIVgI`, `pfs-season-5-bestiary:L55H4HofGvCM02au` | Deferred. The ordinary and Vengeful records belong to one draugr across two tiers. Review the level 1–2 encounter scaling table before declaring the combined family. |
| Pathfinder Society Quest #15: In the Footsteps of Horror | Vengeful Alystair Caskwater | `pfs-season-5-bestiary:SJ39wba6n3d5bvIh`, `pfs-season-5-bestiary:nMMRUST4ow0kcbTy` | Deferred. The level 3–4 encounter substitutes this stat block at 28 or more Challenge Points. Verify the level 1–2 threshold before modeling both as scenario uses. |
| Pathfinder Society Quest #16: The Winter Queen's Dollhouse | The Upset Warden | `pfs-season-5-bestiary:8IJilQMumSAPhsqP`, `pfs-season-5-bestiary:Z9ofpt7xRr9o47Ya` | Deferred. Each Upset block is one creature level above the ordinary one. Check the quest for the substitution rule and whether it is an authored adjustment. |
| Pathfinder Society Quest #16: The Winter Queen's Dollhouse | The Warden | `pfs-season-5-bestiary:1kqDiAAQGH26eAC7`, `pfs-season-5-bestiary:7Ox34TV7Py1pZr9o` | Deferred. The ordinary and Upset blocks share tier-specific spell lists. Check the quest for the condition that selects each block before declaring their relationship. |
| Pathfinder Society Quest #2: Unforgiving Fire | Bandit Disciple | `pfs-season-1-bestiary:S50loNsuTigldGll`, `pfs-season-1-bestiary:jJd7Sa8DB7uBOn93` | [bandit-disciple](../../crates/atlas-search/families/creatures/bandit-disciple.toml) |
| Pathfinder Society Quest #2: Unforgiving Fire | Meleeka Sanvara | `pfs-season-1-bestiary:1MXO3xJS50lENkHL`, `pfs-season-1-bestiary:4MXoQZUrmGeuySrp` | [meleeka-sanvara](../../crates/atlas-search/families/creatures/meleeka-sanvara.toml) |
| Pathfinder Society Quest #5: The Dragon who Stole Evoking Day | Ahrkinos | `pfs-season-1-bestiary:js2sA6P939oTuLuI`, `pfs-season-1-bestiary:vV6a488IlsKsHpL8` | [ahrkinos](../../crates/atlas-search/families/creatures/ahrkinos.toml) |
| Pathfinder Society Quest #5: The Dragon who Stole Evoking Day | Muraxi | `pfs-season-1-bestiary:7VCfX1QpRb2T7324`, `pfs-season-1-bestiary:vgn39Yta3lgeRP1s` | [muraxi](../../crates/atlas-search/families/creatures/muraxi.toml) |
| Pathfinder Society Quest #7: A Curious Claim | Human Bandit | `pfs-season-1-bestiary:aWiBgV5B9sIO7UtN`, `pfs-season-1-bestiary:mWzScgUiWAGEb0RE` | [human-bandit](../../crates/atlas-search/families/creatures/human-bandit.toml) |
| Pathfinder Society Quest #7: A Curious Claim | Neidre Fliavazzana | `pfs-season-1-bestiary:BVgoWV3U3kHHAMg9`, `pfs-season-1-bestiary:zE9WuJXq3I4uZNNX` | [neidre-fliavazzana](../../crates/atlas-search/families/creatures/neidre-fliavazzana.toml) |
| Pathfinder Society Quest #8: Shadows of the Black Sovereign | Technic Zombie | `pfs-season-1-bestiary:7467YYEHyLkJzXvU`, `pfs-season-1-bestiary:PFSkqGyg7eGaES1k` | [technic-zombie](../../crates/atlas-search/families/creatures/technic-zombie.toml) |
| Pathfinder Society Quest #8: Shadows of the Black Sovereign | Zatqualmish | `pfs-season-1-bestiary:UQyvGqfFcYva3fzc`, `pfs-season-1-bestiary:hevuaQkMCF9o5KZ2` | [zatqualmish](../../crates/atlas-search/families/creatures/zatqualmish.toml) |
| Pathfinder Society Scenario #1-01: The Absalom Initiation | Fleshforge Dreg | `pfs-season-1-bestiary:JEwMnYFfyxr5i6Dy`, `pfs-season-1-bestiary:PXXHRzLRLB4bJ7M5`, `pfs-season-1-bestiary:WlJ8fVMV7gXNQ9eR`, `pfs-season-1-bestiary:bDKozdvVEElVQywQ`, `pfs-season-1-bestiary:cuTgbvAxDEkT4XhZ`, `pfs-season-1-bestiary:t1QyIgwLORMhHNL4` | [fleshforge-construct](../../crates/atlas-search/families/creatures/fleshforge-construct.toml) |
| Pathfinder Society Scenario #1-01: The Absalom Initiation | Fleshforge Prototype | `pfs-season-1-bestiary:QZVLjVPLl0PNo2K3`, `pfs-season-1-bestiary:R8D2GrilngQIb1eI`, `pfs-season-1-bestiary:lzDOAywWh0mP4UeH`, `pfs-season-1-bestiary:m3YBXS3ool7PmRfX`, `pfs-season-1-bestiary:sN5jueRXgYI9DDGH`, `pfs-season-1-bestiary:toAsLLjdRwu1pT0b` | [fleshforge-construct](../../crates/atlas-search/families/creatures/fleshforge-construct.toml) |
| Pathfinder Society Scenario #1-02: The Mosquito Witch | Gwibble | `pfs-season-1-bestiary:oQmjMw2MTQDSGRs7`, `pfs-season-1-bestiary:xFeeszDfh4RISbib` | [gwibble](../../crates/atlas-search/families/creatures/gwibble.toml) |
| Pathfinder Society Scenario #1-03: Escaping the Grave | Ralthiss | `pfs-season-1-bestiary:Tnq5pjDMMSmPkBtB`, `pfs-season-1-bestiary:awImEAjEPe1LOY8m` | [ralthiss](../../crates/atlas-search/families/creatures/ralthiss.toml) |
| Pathfinder Society Scenario #1-09: Star-Crossed Voyages | Paravaax | `pfs-season-1-bestiary:1eeMzFoSPuQAAsiY`, `pfs-season-1-bestiary:dtXiSNcy7bM4UITo` | [paravaax](../../crates/atlas-search/families/creatures/paravaax.toml) |
| Pathfinder Society Scenario #1-11: Flames of Rebellion | Ghost Of Diggen Thrune | `pfs-season-1-bestiary:5hR3mQw2sEy3Fy8g`, `pfs-season-1-bestiary:kpYq0piOlafgYSdo` | Deferred. Both blocks have the same friendly title, creature level, and HP. The 3–4 block has Deception +13, compared with +10 for 1–2. Revisit alongside the other same-level Search cases. |
| Pathfinder Society Scenario #1-12: The Burden of Envy | Barralbus | `pfs-season-1-bestiary:8q3pSr4OLkcIDMIH`, `pfs-season-1-bestiary:KR3t3QnVESdhkYr2` | [barralbus](../../crates/atlas-search/families/creatures/barralbus.toml) |
| Pathfinder Society Scenario #1-13: Devil at the Crossroads | The Ascendant | `pfs-season-1-bestiary:bM27sjs9yA3rX4DA`, `pfs-season-1-bestiary:kDEiL11fbXWHQrdH` | [the-ascendant](../../crates/atlas-search/families/creatures/the-ascendant.toml) |
| Pathfinder Society Scenario #1-14: Lions of Katapesh | Cinder Wolf | `pfs-season-1-bestiary:Hjl0cYnMyrcsbKlY`, `pfs-season-1-bestiary:mIz8CjpB8uxf6M9f` | Deferred. Both blocks are creature level 1 and would have the same friendly title. The 3–4 block adds Smoke Vision. Revisit alongside the other same-level Search cases. |
| Pathfinder Society Scenario #1-14: Lions of Katapesh | Keff The Lion | `pfs-season-1-bestiary:Nh3mKGv0uy1tovpL`, `pfs-season-1-bestiary:yn7x4Me1YrPuXFZf` | [keff-the-lion](../../crates/atlas-search/families/creatures/keff-the-lion.toml) |
| Pathfinder Society Scenario #1-14: Lions of Katapesh | Kip The Druid | `pfs-season-1-bestiary:PtU1netfc7U8MaR4`, `pfs-season-1-bestiary:QUimZFYGY12QjoC4` | [kip-the-druid](../../crates/atlas-search/families/creatures/kip-the-druid.toml) |
| Pathfinder Society Scenario #1-15: The Blooming Catastrophe | Farmer Drystan's Scarecrow | `pfs-season-1-bestiary:1JVuJXe27PLyUVwf`, `pfs-season-1-bestiary:iozA5orydjkBmbHx` | [farmer-drystans-scarecrow](../../crates/atlas-search/families/creatures/farmer-drystans-scarecrow.toml) |
| Pathfinder Society Scenario #1-15: The Blooming Catastrophe | Hogweed Leshy | `pfs-season-1-bestiary:FG0Cc949yZ8opW8I`, `pfs-season-1-bestiary:RzLRzsHVjaPBKCkh` | [hogweed-leshy](../../crates/atlas-search/families/creatures/hogweed-leshy.toml) |
| Pathfinder Society Scenario #1-15: The Blooming Catastrophe | Leshy Crafter | `pfs-season-1-bestiary:pxDyzHzkEQTz3R1K`, `pfs-season-1-bestiary:whbIBJJxlv5oVm6c` | [leshy-crafter](../../crates/atlas-search/families/creatures/leshy-crafter.toml) |
| Pathfinder Society Scenario #1-15: The Blooming Catastrophe | Plum Leshy | `pfs-season-1-bestiary:fK69SijSGN3bOgSE`, `pfs-season-1-bestiary:qumj2VUZ6aTnjjIU` | [plum-leshy](../../crates/atlas-search/families/creatures/plum-leshy.toml) |
| Pathfinder Society Scenario #1-16: The Perennial Crown Part 1, Opal of Bhopan | Lelzeshin | `pfs-season-1-bestiary:QtDH32oEVWpFmlCi`, `pfs-season-1-bestiary:b7VoFnPimmOwx8nX` | [lelzeshin](../../crates/atlas-search/families/creatures/lelzeshin.toml) |
| Pathfinder Society Scenario #1-16: The Perennial Crown Part 1, Opal of Bhopan | Mirrored Lelzeshin | `pfs-season-1-bestiary:X6xVydw9XznvvFC2`, `pfs-season-1-bestiary:yAZBjwCVDGbdT4Sb` | [mirrored-lelzeshin](../../crates/atlas-search/families/creatures/mirrored-lelzeshin.toml) |
| Pathfinder Society Scenario #1-17: The Perennial Crown Part 2, The Thorned Monarch | Manifestation of Qxal | `pfs-season-1-bestiary:lvWJic1WYdyJATa0`, `pfs-season-1-bestiary:rOYazaPBX73nLCJE` | [manifestation-of-qxal](../../crates/atlas-search/families/creatures/manifestation-of-qxal.toml) |
| Pathfinder Society Scenario #1-17: The Perennial Crown Part 2, The Thorned Monarch | Thorned Cocoon | `pfs-season-1-bestiary:AFQU2tJl09ykBkwY`, `pfs-season-1-bestiary:uZWktkWAmzadBTh1` | [thorned-cocoon](../../crates/atlas-search/families/creatures/thorned-cocoon.toml) |
| Pathfinder Society Scenario #1-17: The Perennial Crown Part 2, The Thorned Monarch | Webhekiz | `pfs-season-1-bestiary:3MSKMI7ehLTC1bqw`, `pfs-season-1-bestiary:f9NDRa2grZrqnTIs` | [webhekiz](../../crates/atlas-search/families/creatures/webhekiz.toml) |
| Pathfinder Society Scenario #1-20: The Lost Legend | Elite Boggard Swampseer | `pfs-season-1-bestiary:1LM0AUYkZcsbtETs`, `pfs-season-1-bestiary:D2Bc4GbmiuW29310` | Deferred. The actor titles say PFS 1-24 while both the pack path and publication point to The Lost Legend (1-20). The adventure describes an animal staff loadout. Resolve the title discrepancy and its relation to the standard Boggard Swampseer before declaring the family. |
| Pathfinder Society Scenario #1-21: Mistress of the Maze | Mr. Chitters | `pfs-season-1-bestiary:6dL5bhGVdRyID20x`, `pfs-season-1-bestiary:RUXPx9VPJCmuD1V8` | [mr-chitters](../../crates/atlas-search/families/creatures/mr-chitters.toml) |
| Pathfinder Society Scenario #1-21: Mistress of the Maze | Nevashi | `pfs-season-1-bestiary:1Yb78i4YajKrsens`, `pfs-season-1-bestiary:5tDBVocD4ScJHKQz` | [nevashi](../../crates/atlas-search/families/creatures/nevashi.toml) |
| Pathfinder Society Scenario #1-22: Doom of Cassomir | Ahksiva | `pfs-season-1-bestiary:86JAsGyUAfSviibC`, `pfs-season-1-bestiary:pPpc1IevAeLYHZsh` | [ahksiva](../../crates/atlas-search/families/creatures/ahksiva.toml) |
| Pathfinder Society Scenario #1-22: Doom of Cassomir | Groetus's Chosen | `pfs-season-1-bestiary:8hdL1UOQZ1WPb6v7`, `pfs-season-1-bestiary:ksClnBrkXAhoq4Di` | [groetuss-chosen](../../crates/atlas-search/families/creatures/groetuss-chosen.toml) |
| Pathfinder Society Scenario #1-22: Doom of Cassomir | Little Zura | `pfs-season-1-bestiary:Gxy0zuO7aqC9NBv0`, `pfs-season-1-bestiary:sLt5RvozduZ2soA5` | [little-zura](../../crates/atlas-search/families/creatures/little-zura.toml) |
| Pathfinder Society Scenario #1-22: Doom of Cassomir | Ptiro Valner | `pfs-season-1-bestiary:8axdoIyHw2WK9anh`, `pfs-season-1-bestiary:tBY1ayiQyI5tyME6` | [ptiro-valner](../../crates/atlas-search/families/creatures/ptiro-valner.toml) |
| Pathfinder Society Scenario #1-22: Doom of Cassomir | Reast Mycer | `pfs-season-1-bestiary:YtR25uDQSkKQMoaW`, `pfs-season-1-bestiary:hNAN6bqWxFKjxR4Q` | [reast-mycer](../../crates/atlas-search/families/creatures/reast-mycer.toml) |
| Pathfinder Society Scenario #1-24: Lightning Strikes, Stars Fall | Blue Streak Ambusher | `pfs-season-1-bestiary:Uhb6f4ettpS7K2Jl`, `pfs-season-1-bestiary:zAOgSgAqMLCrIvTl` | [blue-streak-ambusher](../../crates/atlas-search/families/creatures/blue-streak-ambusher.toml) |
| Pathfinder Society Scenario #1-24: Lightning Strikes, Stars Fall | Blue Streak Mage | `pfs-season-1-bestiary:mUWCqhhcmEHsSjKo`, `pfs-season-1-bestiary:oHbuqmMgS8PutzMZ` | [blue-streak-mage](../../crates/atlas-search/families/creatures/blue-streak-mage.toml) |
| Pathfinder Society Scenario #1-24: Lightning Strikes, Stars Fall | Synthetic Khismar | `pfs-season-1-bestiary:QYTRwhdkSWZKRtlj`, `pfs-season-1-bestiary:gB7km6bV9dCXUQjZ` | [synthetic-khismar](../../crates/atlas-search/families/creatures/synthetic-khismar.toml) |
| Pathfinder Society Scenario #1-25: Grim Symphony | Doctor Velshun | `pfs-season-1-bestiary:F7gjEHGO2c956FEy`, `pfs-season-1-bestiary:MOPGJAQUS4Aop7Mm` | [doctor-velshun](../../crates/atlas-search/families/creatures/doctor-velshun.toml) |
| Pathfinder Society Scenario #1-25: Grim Symphony | Iloise | `pfs-season-1-bestiary:LMyYMgdcJqQPgGiY`, `pfs-season-1-bestiary:zM01pTDcCS3F6z2V` | [iloise](../../crates/atlas-search/families/creatures/iloise.toml) |

The two Fleshforge audit groups point to the same family. The scenario’s area A2
and its subtier appendices identify Dreg and Prototype as the two party-tier
blocks for the encounter and give both the same six ability choices.

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
  before publishing friendly titles for either pair. Cinder Wolf and Ghost of
  Diggen Thrune provide two more examples.
- **Authored adjustment conflicts.** Compare the three Anatomical Model blocks
  with the adventure’s Laboratory encounter and scaling instructions. Establish
  the effective level, stats, and conditions before adding a scoped override.
- **Summoned copies.** Revisit Marcien’s doubles
  (`pfs-season-3-bestiary:PqawYWEK0iqvcBZj`,
  `pfs-season-3-bestiary:xVmDwoBxjKyCihYz`) and decide whether they belong
  in their own family and how to represent their link to the summoner. The
  separate Lelzeshin and Mirrored Lelzeshin families offer a second case for
  reviewing a link between a creature and its copy.

- **Abilities repeated across party tiers.** Fleshforge’s six abilities each have
  a Dreg and Prototype stat block. The family groups by party level and preserves
  the ability in each title. Grouping by ability as well would require allowing
  multiple records at the same coordinate or declaring the creature form as a
  separate axis. Review that presentation before adding either.
- **Authored scaling variants.** Confirm both subtiers’ substitution conditions
  for Alystair Caskwater and the Warden before adding them. The actor names
  alone do not establish an Elite override.
- **Pack and publication discrepancies.** Review the Familiar Shadow and the
  Elite Boggard Swampseer records against their original encounters before using
  their publication fields or parenthetical scenario numbers as family evidence.

Record newly discovered groups by signal so the baseline count remains comparable.
Measure additional candidates and false matches before widening the review queue.
