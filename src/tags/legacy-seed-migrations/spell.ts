import type { DerivedTagLegacySeedMigrationCategory } from "../../types.js";

// Temporary migration bucket for legacy carried-over "seed" records that are still
// applied live but need manual review to become either true exemplars or assignments.
export const SPELL_DERIVED_TAG_LEGACY_SEED_MIGRATIONS = {
  category: "spell",
  tags: [
    {
      tag: "scouting",
      includeRecords: [
        {
          pack: "spells-srd",
          name: "Enhance Senses"
        },
        {
          pack: "spells-srd",
          name: "Grave Impressions"
        },
        {
          pack: "spells-srd",
          name: "Hunter's Vision"
        },
        {
          pack: "spells-srd",
          name: "Locate"
        }
      ]
    },
    {
      tag: "navigation",
      includeRecords: [
        {
          pack: "spells-srd",
          name: "Show the Way"
        },
        {
          pack: "spells-srd",
          name: "Return Beacon"
        },
        {
          pack: "spells-srd",
          name: "Nature's Pathway"
        },
        {
          pack: "spells-srd",
          name: "Fire's Pathway"
        },
        {
          pack: "spells-srd",
          name: "Interplanar Teleport"
        }
      ]
    },
    {
      tag: "mobility",
      includeRecords: [
        {
          pack: "spells-srd",
          name: "Air Walk"
        },
        {
          pack: "spells-srd",
          name: "Airlift"
        },
        {
          pack: "spells-srd",
          name: "Dive and Breach"
        },
        {
          pack: "spells-srd",
          name: "Friendly Push"
        },
        {
          pack: "spells-srd",
          name: "Levitate"
        },
        {
          pack: "spells-srd",
          name: "Migration"
        },
        {
          pack: "spells-srd",
          name: "Water Walk"
        },
        {
          pack: "spells-srd",
          name: "Nature's Pathway"
        },
        {
          pack: "spells-srd",
          name: "Fire's Pathway"
        }
      ]
    },
    {
      tag: "healing_support",
      includeRecords: [
        {
          pack: "spells-srd",
          name: "Field of Life"
        },
        {
          pack: "spells-srd",
          name: "Life-Giving Form"
        },
        {
          pack: "spells-srd",
          name: "Life Link"
        },
        {
          pack: "spells-srd",
          name: "Regenerate"
        },
        {
          pack: "spells-srd",
          name: "Soothing Mist"
        },
        {
          pack: "spells-srd",
          name: "Stabilize"
        },
        {
          pack: "spells-srd",
          name: "Soothe"
        },
        {
          pack: "spells-srd",
          name: "Spiritual Renewal"
        },
        {
          pack: "spells-srd",
          name: "Positive Attunement"
        },
        {
          pack: "spells-srd",
          name: "Vital Beacon"
        },
        {
          pack: "spells-srd",
          name: "Life Boost"
        },
        {
          pack: "spells-srd",
          name: "Halcyon Mists"
        }
      ]
    },
    {
      tag: "condition_support",
      includeRecords: [
        {
          pack: "spells-srd",
          name: "Clear Mind"
        },
        {
          pack: "spells-srd",
          name: "Restoration"
        },
        {
          pack: "spells-srd",
          name: "Cleansing Flames"
        },
        {
          pack: "spells-srd",
          name: "Delay Affliction"
        },
        {
          pack: "spells-srd",
          name: "Soothe"
        },
        {
          pack: "spells-srd",
          name: "Moment of Renewal"
        }
      ]
    },
    {
      tag: "affliction_cleanup",
      includeRecords: [
        {
          pack: "spells-srd",
          name: "Restoration"
        },
        {
          pack: "spells-srd",
          name: "Cleansing Flames"
        },
        {
          pack: "spells-srd",
          name: "Delay Affliction"
        },
        {
          pack: "spells-srd",
          name: "Moment of Renewal"
        }
      ]
    },
    {
      tag: "protective_ward",
      includeRecords: [
        {
          pack: "spells-srd",
          name: "Protective Wards"
        },
        {
          pack: "spells-srd",
          name: "Spellmaster's Ward"
        },
        {
          pack: "spells-srd",
          name: "Phoenix Ward"
        },
        {
          pack: "spells-srd",
          name: "Divine Aura"
        },
        {
          pack: "spells-srd",
          name: "Mystic Armor"
        },
        {
          pack: "spells-srd",
          name: "Protector's Sphere"
        },
        {
          pack: "spells-srd",
          name: "Shielded Arm"
        },
        {
          pack: "spells-srd",
          name: "Warding Aggression"
        },
        {
          pack: "spells-srd",
          name: "Arcane Countermeasure"
        }
      ]
    },
    {
      tag: "resistance_support",
      includeRecords: [
        {
          pack: "spells-srd",
          name: "Phoenix Ward"
        },
        {
          pack: "spells-srd",
          name: "Protector's Sphere"
        },
        {
          pack: "spells-srd",
          name: "Fire Shield"
        },
        {
          pack: "spells-srd",
          name: "Elemental Gift"
        },
        {
          pack: "spells-srd",
          name: "Life-Giving Form"
        },
        {
          pack: "spells-srd",
          name: "Aerial Form"
        },
        {
          pack: "spells-srd",
          name: "Cosmic Form"
        },
        {
          pack: "spells-srd",
          name: "Element Embodied"
        }
      ]
    },
    {
      tag: "persistent_damage",
      includeRecords: [
        {
          pack: "spells-srd",
          name: "Achaekek's Clutch"
        },
        {
          pack: "spells-srd",
          name: "Acid Arrow"
        },
        {
          pack: "spells-srd",
          name: "Acid Grip"
        },
        {
          pack: "spells-srd",
          name: "Acid Splash"
        },
        {
          pack: "spells-srd",
          name: "Advanced Scurvy"
        },
        {
          pack: "spells-srd",
          name: "Ancient Dust"
        },
        {
          pack: "spells-srd",
          name: "Armor of Thorn and Claw"
        },
        {
          pack: "spells-srd",
          name: "Beheading Buzz Saw"
        },
        {
          pack: "spells-srd",
          name: "Blazing Blade"
        },
        {
          pack: "spells-srd",
          name: "Blinding Foam"
        },
        {
          pack: "spells-srd",
          name: "Blister Bomb"
        },
        {
          pack: "spells-srd",
          name: "Blistering Invective"
        },
        {
          pack: "spells-srd",
          name: "Blood Feast"
        },
        {
          pack: "spells-srd",
          name: "Blood Vendetta"
        },
        {
          pack: "spells-srd",
          name: "Blood in the Water"
        },
        {
          pack: "spells-srd",
          name: "Bloodspray Curse"
        },
        {
          pack: "spells-srd",
          name: "Bone Flense"
        },
        {
          pack: "spells-srd",
          name: "Bone Spray"
        },
        {
          pack: "spells-srd",
          name: "Brine Dragon Bile"
        },
        {
          pack: "spells-srd",
          name: "Bursting Bloom"
        },
        {
          pack: "spells-srd",
          name: "Charged Javelin"
        },
        {
          pack: "spells-srd",
          name: "Cinder Swarm"
        },
        {
          pack: "spells-srd",
          name: "Combustion"
        },
        {
          pack: "spells-srd",
          name: "Cutting Insult"
        },
        {
          pack: "spells-srd",
          name: "Dehydrate"
        },
        {
          pack: "spells-srd",
          name: "Diadem of Divine Radiance"
        },
        {
          pack: "spells-srd",
          name: "Dimensional Excision"
        },
        {
          pack: "spells-srd",
          name: "Divine Immolation"
        },
        {
          pack: "spells-srd",
          name: "Earth's Bile"
        },
        {
          pack: "spells-srd",
          name: "Enervation"
        },
        {
          pack: "spells-srd",
          name: "Feral Shades"
        },
        {
          pack: "spells-srd",
          name: "Field of Razors"
        },
        {
          pack: "spells-srd",
          name: "Final Fate of the Locust Host"
        },
        {
          pack: "spells-srd",
          name: "Flame Dancer"
        },
        {
          pack: "spells-srd",
          name: "Flense"
        },
        {
          pack: "spells-srd",
          name: "Flourishing Flora"
        },
        {
          pack: "spells-srd",
          name: "Funeral Flames"
        },
        {
          pack: "spells-srd",
          name: "Fungal Infestation"
        },
        {
          pack: "spells-srd",
          name: "Glass Sand"
        },
        {
          pack: "spells-srd",
          name: "Gouging Claw"
        },
        {
          pack: "spells-srd",
          name: "Grim Tendrils"
        },
        {
          pack: "spells-srd",
          name: "Heat Metal"
        },
        {
          pack: "spells-srd",
          name: "Ignite Fireworks"
        },
        {
          pack: "spells-srd",
          name: "Ignition"
        },
        {
          pack: "spells-srd",
          name: "Implement of Destruction"
        },
        {
          pack: "spells-srd",
          name: "Live Wire"
        },
        {
          pack: "spells-srd",
          name: "Musical Shift"
        },
        {
          pack: "spells-srd",
          name: "Mutilate"
        },
        {
          pack: "spells-srd",
          name: "Noxious Metals"
        },
        {
          pack: "spells-srd",
          name: "Phantom Pain"
        },
        {
          pack: "spells-srd",
          name: "Puff of Poison"
        },
        {
          pack: "spells-srd",
          name: "Rainbow Fumarole"
        },
        {
          pack: "spells-srd",
          name: "Rusting Grasp"
        },
        {
          pack: "spells-srd",
          name: "Savor the Sting"
        },
        {
          pack: "spells-srd",
          name: "Sawtooth Terrain"
        },
        {
          pack: "spells-srd",
          name: "Scorching Blast"
        },
        {
          pack: "spells-srd",
          name: "Shocking Grasp"
        },
        {
          pack: "spells-srd",
          name: "Slashing Gust"
        }
      ]
    },
    {
      tag: "initiative_support",
      includeRecords: [
        {
          pack: "spells-srd",
          name: "Anticipate Peril"
        },
        {
          pack: "spells-srd",
          name: "Call to Arms"
        },
        {
          pack: "spells-srd",
          name: "Rewrite Possibility"
        },
        {
          pack: "spells-srd",
          name: "Song of Marching"
        },
        {
          pack: "spells-srd",
          name: "Zeal for Battle"
        },
        {
          pack: "spells-srd",
          name: "Know the Enemy"
        },
        {
          pack: "spells-srd",
          name: "Dull Ambition"
        },
        {
          pack: "spells-srd",
          name: "Perseis's Precautions"
        },
        {
          pack: "spells-srd",
          name: "Shock to the System"
        }
      ]
    },
    {
      tag: "eidolon_support",
      includeRecords: [
        {
          pack: "spells-srd",
          name: "Boost Eidolon"
        },
        {
          pack: "spells-srd",
          name: "Extend Boost"
        },
        {
          pack: "spells-srd",
          name: "Unfetter Eidolon"
        },
        {
          pack: "spells-srd",
          name: "Summoner's Precaution"
        },
        {
          pack: "spells-srd",
          name: "Summoner's Visage"
        },
        {
          pack: "spells-srd",
          name: "Protect Companion"
        },
        {
          pack: "spells-srd",
          name: "Reinforce Eidolon"
        },
        {
          pack: "spells-srd",
          name: "Evolution Surge"
        },
        {
          pack: "spells-srd",
          name: "Lifelink Surge"
        },
        {
          pack: "spells-srd",
          name: "Domora's Defense"
        }
      ]
    },
    {
      tag: "countermagic",
      includeRecords: [
        {
          pack: "spells-srd",
          name: "Antimagic Field"
        },
        {
          pack: "spells-srd",
          name: "Veil of Privacy"
        },
        {
          pack: "spells-srd",
          name: "Hidden Mind"
        },
        {
          pack: "spells-srd",
          name: "Blind Eye"
        },
        {
          pack: "spells-srd",
          name: "Detect Scrying"
        },
        {
          pack: "spells-srd",
          name: "False Vision"
        },
        {
          pack: "spells-srd",
          name: "Arcane Countermeasure"
        }
      ]
    }
  ]
} satisfies DerivedTagLegacySeedMigrationCategory;
