import { Tag } from "antd";
import { Pencil } from "lucide-react";
import { useState } from "react";
import type React from "react";

type MockCreature = {
  ac: number;
  description: string;
  hp: number;
  key: string;
  level: string;
  name: string;
  pack: string;
  perception: string;
  saves: string;
  skills: string;
  speed: string;
  traits: string[];
};

const creatures: MockCreature[] = [
  {
    ac: 28,
    description:
      "A fiendish dream-haunter that bargains with souls, curses sleepers, and becomes especially dangerous when supported by coven magic.",
    hp: 170,
    key: "pathfinder-bestiary:WQy7HBUcgDLsfVJd",
    level: "Creature 9",
    name: "Night Hag",
    pack: "Bestiary 1",
    perception: "+18",
    saves: "Fort +19, Ref +17, Will +18",
    skills:
      "Arcana +18, Deception +18, Diplomacy +18, Intimidation +14, Occultism +20, Religion +20",
    speed: "25 ft",
    traits: ["evil", "fiend", "hag", "humanoid", "unholy"],
  },
  {
    ac: 28,
    description:
      "An incorporeal undead spirit that drains life and slips through walls, forcing opponents to solve positioning and resistance problems.",
    hp: 115,
    key: "season-of-ghosts-bestiary:wraith",
    level: "Creature 10",
    name: "Wraith",
    pack: "Season of Ghosts",
    perception: "+19",
    saves: "Fort +16, Ref +21, Will +20",
    skills: "Acrobatics +21, Intimidation +20, Stealth +23",
    speed: "fly 40 ft",
    traits: ["incorporeal", "spirit", "undead"],
  },
  {
    ac: 22,
    description:
      "An armored undead commander with martial discipline, heavy defenses, and battlefield pressure from both strikes and intimidation.",
    hp: 68,
    key: "pfs-season-1-bestiary:graveknight",
    level: "Creature 7",
    name: "Graveknight Captain",
    pack: "PFS Season 1 Bestiary",
    perception: "+15",
    saves: "Fort +18, Ref +13, Will +15",
    skills: "Athletics +19, Intimidation +17, Warfare Lore +15",
    speed: "20 ft",
    traits: ["evil", "undead"],
  },
];

const featured = creatures[0];

type MockVariant = "weak" | "normal" | "elite";

type RuntimeMockState = {
  ac: number;
  attack: string;
  conditions: string[];
  damage: string;
  hp: number;
  maxHp: number;
  perception: string;
  ref: string;
  fort: string;
  tempHp: number;
  variant: MockVariant;
  will: string;
};

const variantLabels: Record<MockVariant, string> = {
  weak: "Weak",
  normal: "Normal",
  elite: "Elite",
};

const variantStats: Record<
  MockVariant,
  Pick<
    RuntimeMockState,
    "ac" | "attack" | "damage" | "fort" | "hp" | "maxHp" | "perception" | "ref" | "will"
  >
> = {
  weak: {
    ac: 26,
    attack: "+18",
    damage: "2d8 + 4 piercing + 1d6 spirit",
    fort: "+17",
    hp: 150,
    maxHp: 150,
    perception: "+16",
    ref: "+15",
    will: "+16",
  },
  normal: {
    ac: 28,
    attack: "+20",
    damage: "2d8 + 8 piercing + 1d6 spirit",
    fort: "+19",
    hp: 126,
    maxHp: 170,
    perception: "+18",
    ref: "+17",
    will: "+18",
  },
  elite: {
    ac: 30,
    attack: "+22",
    damage: "2d8 + 12 piercing + 1d6 spirit",
    fort: "+21",
    hp: 126,
    maxHp: 190,
    perception: "+20",
    ref: "+19",
    will: "+20",
  },
};

const conditionOptions = [
  "Frightened 1",
  "Off-Guard",
  "Sickened 1",
  "Slowed 1",
  "Quickened",
];

const participantSides = ["pc", "ally", "enemy", "neutral", "hazard"] as const;

function useRuntimeMockState(): RuntimeMockState & {
  addCondition: (condition: string) => void;
  applyDamage: (amount: number) => void;
  applyHeal: (amount: number) => void;
  removeCondition: (condition: string) => void;
  setDirectHp: (hp: number) => void;
  setTempHp: (hp: number) => void;
  setVariant: (variant: MockVariant) => void;
} {
  const [variant, setVariantState] = useState<MockVariant>("elite");
  const [hpByVariant, setHpByVariant] = useState<Record<MockVariant, number>>({
    weak: variantStats.weak.hp,
    normal: variantStats.normal.hp,
    elite: variantStats.elite.hp,
  });
  const [tempHp, setTempHpState] = useState(12);
  const [conditions, setConditions] = useState(["Frightened 1", "Off-Guard"]);
  const stats = variantStats[variant];
  const hp = Math.min(hpByVariant[variant], stats.maxHp);

  function updateHp(next: (current: number, maxHp: number) => number) {
    setHpByVariant((current) => ({
      ...current,
      [variant]: clamp(
        next(Math.min(current[variant], stats.maxHp), stats.maxHp),
        0,
        stats.maxHp,
      ),
    }));
  }

  return {
    ...stats,
    conditions,
    hp,
    tempHp,
    variant,
    addCondition(condition) {
      setConditions((current) =>
        current.includes(condition) ? current : [...current, condition],
      );
    },
    applyDamage(amount) {
      const damage = Math.max(0, amount);
      const tempAbsorbed = Math.min(tempHp, damage);
      setTempHpState((current) => Math.max(0, current - tempAbsorbed));
      updateHp((current) => current - (damage - tempAbsorbed));
    },
    applyHeal(amount) {
      updateHp((current) => current + Math.max(0, amount));
    },
    removeCondition(condition) {
      setConditions((current) => current.filter((item) => item !== condition));
    },
    setDirectHp(nextHp) {
      updateHp(() => nextHp);
    },
    setTempHp(nextHp) {
      setTempHpState(Math.max(0, nextHp));
    },
    setVariant(nextVariant) {
      setVariantState(nextVariant);
      setHpByVariant((current) => ({
        ...current,
        [nextVariant]: Math.min(current[nextVariant], variantStats[nextVariant].maxHp),
      }));
    },
  };
}

type RuntimeMockController = ReturnType<typeof useRuntimeMockState>;

function clamp(value: number, min: number, max: number) {
  return Math.max(min, Math.min(max, value));
}

export function PresentationMocksView() {
  return (
    <main className="surface-mocks">
      <header className="surface-mocks__header">
        <div>
          <p className="eyebrow">Temporary design mock</p>
          <h2>Creature Search Result Layouts</h2>
          <p>
            Static options for deciding how compact record surfaces should appear in
            search before wiring the backend model.
          </p>
        </div>
      </header>

      <section className="surface-mock">
        <MockHeading
          label="Option A"
          title="Table-compatible dense rows"
          note="Closest to the current search table. Good for scanning many rows, weaker for grouped creature facts."
        />
        <div className="surface-table-mock" role="table" aria-label="Dense rows mock">
          <div className="surface-table-mock__head" role="row">
            <span>Record</span>
            <span>Description</span>
            <span>Stats</span>
            <span>Movement</span>
          </div>
          {creatures.map((creature) => (
            <div className="surface-table-mock__row" key={creature.key} role="row">
              <CreatureTitle creature={creature} />
              <p className="surface-search-description">{creature.description}</p>
              <span className="surface-inline-stats">
                <CompactStat label="AC" value={String(creature.ac)} />
                <CompactStat label="HP" value={String(creature.hp)} />
                <CompactStat label="Saves" value={creature.saves} />
              </span>
              <CompactStat label="Speed" value={creature.speed} />
            </div>
          ))}
        </div>
      </section>

      <section className="surface-mock">
        <MockHeading
          label="Option B"
          title="Rich list rows"
          note="Recommended baseline. Keeps keyboard-friendly rows while giving stats enough space to group naturally."
        />
        <div className="surface-list-mock">
          {creatures.map((creature) => (
            <article className="surface-result-row" key={creature.key}>
              <div className="surface-result-row__identity">
                <CreatureTitle creature={creature} />
                <p className="surface-search-description">{creature.description}</p>
                <TraitRow traits={creature.traits} />
              </div>
              <div className="surface-result-row__facts">
                <CompactStat label="AC" value={String(creature.ac)} />
                <CompactStat label="HP" value={String(creature.hp)} />
                <CompactStat label="Saves" value={creature.saves} />
                <CompactStat label="Perception" value={creature.perception} />
                <CompactStat label="Speed" value={creature.speed} />
              </div>
              <div className="surface-result-row__meta">
                <VariantMiniControl />
              </div>
            </article>
          ))}
        </div>
      </section>

      <section className="surface-mock">
        <MockHeading
          label="Option C"
          title="Stat strip rows"
          note="Very compact. Strong for AC/HP/saves/speed, but long skill or activity snippets have nowhere natural to live."
        />
        <div className="surface-strip-mock">
          {creatures.map((creature) => (
            <article className="surface-strip-row" key={creature.key}>
              <div className="surface-strip-row__identity">
                <CreatureTitle creature={creature} />
                <p className="surface-search-description">{creature.description}</p>
              </div>
              <div className="surface-strip-row__stats">
                <CompactStat label="AC" value={String(creature.ac)} />
                <CompactStat label="HP" value={String(creature.hp)} />
                <CompactStat label="Fort/Ref/Will" value={creature.saves} />
                <CompactStat label="Speed" value={creature.speed} />
              </div>
            </article>
          ))}
        </div>
      </section>

      <section className="surface-mock">
        <MockHeading
          label="Option D"
          title="Split row with expandable detail area"
          note="Best if search should preview richer creature facts, but it is visually heavier and may reduce result density."
        />
        <div className="surface-split-mock">
          {creatures.map((creature) => (
            <article className="surface-split-row" key={creature.key}>
              <div className="surface-split-row__summary">
                <CreatureTitle creature={creature} />
                <p className="surface-search-description">{creature.description}</p>
                <TraitRow traits={creature.traits} />
              </div>
              <div className="surface-split-row__detail">
                <CompactStat
                  label="Defenses"
                  value={`AC ${creature.ac}; ${creature.saves}`}
                />
                <CompactStat
                  label="Senses"
                  value={`Perception ${creature.perception}`}
                />
                <CompactStat label="Skills" value={creature.skills} />
                <CompactStat label="Movement" value={creature.speed} />
              </div>
            </article>
          ))}
        </div>
      </section>

      <section className="surface-mock surface-mock--wide">
        <MockHeading
          label="Surface Option E"
          title="Static creature sheet"
          note="A detail-style surface where source facts lead and rich prose follows. This is closer to what record detail would render for creatures."
        />
        <CreatureSurfaceFrame creature={featured} mode="static" />
      </section>

      <section className="surface-mock surface-mock--wide">
        <MockHeading
          label="Surface Option F"
          title="Encounter-adjusted creature sheet"
          note="Same section vocabulary as the static surface, but HP, conditions, runtime actions, movement, and adjusted stats are injected in-place."
        />
        <CreatureSurfaceFrame creature={featured} mode="encounter" />
      </section>

      <section className="surface-mock surface-mock--wide">
        <MockHeading
          label="Surface Option G"
          title="Preview-sized surface"
          note="A condensed version for popovers and side previews. It keeps the same sections, but drops controls and long prose."
        />
        <div className="surface-preview-layout">
          <CreaturePreviewSurface creature={featured} />
          <CreaturePreviewSurface creature={creatures[1]} />
        </div>
      </section>

      <section className="surface-mock surface-mock--wide">
        <MockHeading
          label="Surface Option H"
          title="Dense stat-block surface"
          note="AoN-inspired: compact labeled lines and section rules. Adjustments appear as inline annotations instead of separate fact boxes."
        />
        <CreatureStatBlockSurface creature={featured} mode="static" />
      </section>

      <section className="surface-mock surface-mock--wide">
        <MockHeading
          label="Surface Option I"
          title="Dense encounter stat block"
          note="Same dense stat-block shape, with runtime values highlighted in-line and controls grouped only where mutation happens."
        />
        <CreatureStatBlockSurface creature={featured} mode="encounter" />
      </section>

      <section className="surface-mock surface-mock--wide">
        <MockHeading
          label="Surface Option J"
          title="Hybrid sheet with dense lines"
          note="Uses sheet sections for width, but presents most facts as compact stat lines inside each section instead of individual subfact cards."
        />
        <CreatureHybridSurface creature={featured} mode="static" variantControl />
      </section>

      <section className="surface-mock surface-mock--wide">
        <MockHeading
          label="Surface Option K"
          title="Hybrid encounter sheet"
          note="Runtime controls stay attached to vitals and conditions, while defenses, skills, movement, and activities use denser in-section lines."
        />
        <CreatureHybridSurface creature={featured} mode="encounter" />
      </section>

      <section className="surface-mock surface-mock--wide">
        <MockHeading
          label="Surface Option O1"
          title="Structured search surface"
          note="The same record composition without participant-only runtime controls. Search can preview record facts and optional variants, but never shows encounter notes or mutation UI."
        />
        <CreatureStructuredSurface creature={featured} profile="search" />
      </section>

      <section className="surface-mock surface-mock--wide">
        <MockHeading
          label="Surface Option O2"
          title="Structured encounter participant surface"
          note="The leading participant-level candidate: the structured surface plus the currently supported participant edit affordances, with notes hidden behind an edit button."
        />
        <CreatureStructuredSurface creature={featured} profile="encounter" />
      </section>

      <section className="surface-mock surface-mock--wide">
        <MockHeading
          label="Surface Option L"
          title="Side-rail reference sheet"
          note="A quieter character-sheet layout: compact summary rail on the left, denser stat-block sections on the right."
        />
        <CreatureRailSurface creature={featured} />
      </section>

      <section className="surface-mock surface-mock--wide">
        <MockHeading
          label="Surface Option M"
          title="Pathbuilder-style dashboard"
          note="Dashboard panels with compact lists: top identity and key stats, then grouped saves, skills, movement, and actions."
        />
        <CreaturePathbuilderSurface creature={featured} mode="static" />
      </section>

      <section className="surface-mock surface-mock--wide">
        <MockHeading
          label="Surface Option M2"
          title="Runtime dashboard"
          note="Same dashboard structure with the earlier HP bar, temp HP, adjusted values, and condition controls restored."
        />
        <CreaturePathbuilderSurface creature={featured} mode="encounter" />
      </section>

      <section className="surface-mock surface-mock--wide">
        <MockHeading
          label="Surface Option N"
          title="D&D Beyond-style top stats"
          note="Prominent top stat tiles for the values used constantly, with tab-like detail sections below."
        />
        <CreatureBeyondSurface creature={featured} mode="static" />
      </section>

      <section className="surface-mock surface-mock--wide">
        <MockHeading
          label="Surface Option N2"
          title="Runtime top-stats sheet"
          note="Prominent runtime vitals with HP bar and adjusted top stats, while detail sections keep a denser sheet treatment."
        />
        <CreatureBeyondSurface creature={featured} mode="encounter" />
      </section>
    </main>
  );
}

function CreatureSurfaceFrame({
  creature,
  mode,
}: {
  creature: MockCreature;
  mode: "static" | "encounter";
}) {
  const encounter = mode === "encounter";
  return (
    <article className="surface-detail">
      <header className="surface-detail__hero">
        <div className="surface-detail__identity">
          <p className="eyebrow">{encounter ? "Encounter participant" : "Creature"}</p>
          <h3>{encounter ? "Elite Night Hag" : creature.name}</h3>
          <p>
            {creature.level} · {creature.key}
          </p>
          <TraitRow traits={creature.traits} />
        </div>
        {encounter ? (
          <div className="surface-variant-control">
            <span>Variant</span>
            <strong>Elite</strong>
          </div>
        ) : (
          <VariantMiniControl />
        )}
      </header>

      <div className="surface-detail__grid">
        <section className="surface-section surface-section--vitals">
          <SectionTitle title="Vitals" />
          {encounter ? (
            <>
              <div className="surface-hp-bar" aria-hidden="true">
                <span style={{ width: "55%" }} />
                <em style={{ width: "11%" }} />
              </div>
              <div className="surface-value-grid">
                <SurfaceValue label="HP" value="62 / 113" base="95 max" adjusted />
                <SurfaceValue label="Temp" value="12" />
                <SurfaceValue label="Initiative" value="28" />
              </div>
              <div className="surface-control-row">
                <button type="button">Damage</button>
                <button type="button">Heal</button>
                <button type="button">Set HP</button>
              </div>
            </>
          ) : (
            <div className="surface-value-grid">
              <SurfaceValue label="HP" value={String(creature.hp)} />
              <SurfaceValue label="Level" value="9" />
              <SurfaceValue label="Perception" value={creature.perception} />
            </div>
          )}
        </section>

        <section className="surface-section">
          <SectionTitle title="Defenses" />
          <div className="surface-value-grid">
            <SurfaceValue
              label="AC"
              value={encounter ? "27" : String(creature.ac)}
              base={encounter ? String(creature.ac) : undefined}
              adjusted={encounter}
            />
            <SurfaceValue
              label="Fort"
              value={encounter ? "+19" : "+17"}
              base={encounter ? "+17" : undefined}
              adjusted={encounter}
            />
            <SurfaceValue
              label="Ref"
              value={encounter ? "+20" : "+18"}
              base={encounter ? "+18" : undefined}
              adjusted={encounter}
            />
            <SurfaceValue
              label="Will"
              value={encounter ? "+23" : "+21"}
              base={encounter ? "+21" : undefined}
              adjusted={encounter}
            />
          </div>
        </section>

        <section className="surface-section">
          <SectionTitle title="Skills" />
          <div className="surface-value-list">
            <SurfaceValue
              label="Arcana"
              value={encounter ? "+22" : "+20"}
              base={encounter ? "+20" : undefined}
              adjusted={encounter}
            />
            <SurfaceValue
              label="Deception"
              value={encounter ? "+23" : "+21"}
              base={encounter ? "+21" : undefined}
              adjusted={encounter}
            />
            <SurfaceValue
              label="Occultism"
              value={encounter ? "+22" : "+20"}
              base={encounter ? "+20" : undefined}
              adjusted={encounter}
            />
          </div>
        </section>

        <section className="surface-section">
          <SectionTitle title="Movement" />
          <div className="surface-value-grid">
            <SurfaceValue label="Speed" value={creature.speed} />
            {encounter && <SurfaceValue label="Actions" value="4" base="3" adjusted />}
            {encounter && <SurfaceValue label="Reactions" value="1" />}
          </div>
          {encounter && (
            <p className="surface-note">Quickened grants 1 restricted action.</p>
          )}
        </section>

        {encounter && (
          <section className="surface-section surface-section--conditions">
            <SectionTitle title="Conditions" />
            <div className="surface-condition-row">
              <strong>Frightened 1</strong>
              <span>status penalty to checks and DCs</span>
              <button type="button">Details</button>
            </div>
            <div className="surface-condition-row">
              <strong>Off-Guard</strong>
              <span>-2 circumstance penalty to AC</span>
              <button type="button">Details</button>
            </div>
            <button className="surface-add-button" type="button">
              Add condition
            </button>
          </section>
        )}

        <section className="surface-section surface-section--wide">
          <SectionTitle title="Activities" />
          <div className="surface-activity-list">
            <ActivityMock
              title="Jaws"
              meta="melee Strike"
              primary={encounter ? "+22" : "+20"}
              base={encounter ? "+20" : undefined}
              damage={encounter ? "2d8 + 10 piercing" : "2d8 + 8 piercing"}
            />
            <ActivityMock
              title="Dream Haunting"
              meta="occult, curse, mental"
              primary="DC 28 Will"
              damage="nightmare effect"
            />
          </div>
        </section>

        <section className="surface-section surface-section--wide">
          <SectionTitle title="Description" />
          {encounter ? (
            <details className="surface-collapsed-prose">
              <summary>Description</summary>
              <p>
                Night hags haunt dreams, trade in corrupted souls, and use coven magic
                to twist bargains into lasting curses. Runtime views keep this available
                without putting it above the operational facts.
              </p>
            </details>
          ) : (
            <p className="surface-prose">
              Night hags haunt dreams, trade in corrupted souls, and use coven magic to
              twist bargains into lasting curses. This area represents rich source prose
              that remains available without duplicating structured combat facts above.
            </p>
          )}
        </section>
      </div>
    </article>
  );
}

function CreaturePreviewSurface({ creature }: { creature: MockCreature }) {
  return (
    <article className="surface-preview">
      <CreatureTitle creature={creature} />
      <TraitRow traits={creature.traits.slice(0, 3)} />
      <p className="surface-search-description">{creature.description}</p>
      <div className="surface-value-grid">
        <SurfaceValue label="AC" value={String(creature.ac)} />
        <SurfaceValue label="HP" value={String(creature.hp)} />
        <SurfaceValue label="Perception" value={creature.perception} />
        <SurfaceValue label="Speed" value={creature.speed} />
      </div>
      <p className="surface-prose">
        Condensed source prose and links would appear here without full activity detail.
      </p>
    </article>
  );
}

function CreatureStatBlockSurface({
  creature,
  mode,
}: {
  creature: MockCreature;
  mode: "static" | "encounter";
}) {
  const encounter = mode === "encounter";
  return (
    <article className="surface-statblock">
      <header className="surface-statblock__header">
        <div>
          <h3>{encounter ? "Elite Night Hag" : creature.name}</h3>
          <p>
            {creature.level} · {creature.key}
          </p>
        </div>
        <TraitRow traits={creature.traits} />
      </header>

      {encounter && (
        <section className="surface-statblock__runtime">
          <div className="surface-statblock-hp">
            <strong>HP 62 / 113</strong>
            <span className="surface-statblock-hp__track">
              <span style={{ width: "55%" }} />
              <em style={{ width: "11%" }} />
            </span>
            <small>base max 95 · temp 12</small>
          </div>
          <div className="surface-statblock-actions">
            <button type="button">Damage</button>
            <button type="button">Heal</button>
            <button type="button">Add condition</button>
          </div>
        </section>
      )}

      <section className="surface-statblock__body">
        <StatLine
          label="Perception"
          value={
            encounter ? (
              <>
                <AdjustedText value="+20" base="+18" />; darkvision
              </>
            ) : (
              "+18; darkvision"
            )
          }
        />
        <StatLine label="Languages" value="Aklo, Common, Jotun" />
        <StatLine
          label="Skills"
          value={
            encounter ? (
              <>
                Arcana <AdjustedText value="+22" base="+20" />, Deception{" "}
                <AdjustedText value="+23" base="+21" />, Occultism{" "}
                <AdjustedText value="+22" base="+20" />
              </>
            ) : (
              "Arcana +20, Deception +21, Occultism +20"
            )
          }
        />
        <AbilitySpread />
      </section>

      <Rule />

      <section className="surface-statblock__body">
        <StatLine
          label="AC"
          value={encounter ? <AdjustedText value="27" base="25" /> : "25"}
        />
        <StatLine
          label="Saves"
          value={
            encounter ? (
              <>
                Fort <AdjustedText value="+19" base="+17" />, Ref{" "}
                <AdjustedText value="+20" base="+18" />, Will{" "}
                <AdjustedText value="+23" base="+21" />
              </>
            ) : (
              creature.saves
            )
          }
        />
        <StatLine label="HP" value={encounter ? "62 / 113 (12 temp)" : "95"} />
        {encounter && (
          <StatLine
            label="Conditions"
            value={
              <>
                <ConditionPill value="Frightened 1" />{" "}
                <ConditionPill value="Off-Guard" />
              </>
            }
          />
        )}
      </section>

      <Rule />

      <section className="surface-statblock__body">
        <StatLine
          label="Speed"
          value={
            encounter ? (
              <>
                25 ft; Actions <AdjustedText value="4" base="3" />{" "}
                <small>(quickened, restricted)</small>
              </>
            ) : (
              "25 ft"
            )
          }
        />
        <ActionLine
          title="Melee"
          value={
            <>
              jaws <AdjustedText value="+22" base={encounter ? "+20" : undefined} />{" "}
              (magical), Damage{" "}
              <AdjustedText
                value={encounter ? "2d8 + 10 piercing" : "2d8 + 8 piercing"}
                base={encounter ? "2d8 + 8 piercing" : undefined}
              />
            </>
          }
        />
        <ActionLine
          title="Dream Haunting"
          value="occult, curse, mental; DC 28 Will; nightmare effect"
        />
      </section>

      <Rule />

      <section className="surface-statblock__body">
        {encounter ? (
          <details className="surface-statblock__details">
            <summary>Description</summary>
            <p>
              <strong>Coven</strong> A night hag gains additional spells and rituals
              when working with a coven. This is available in runtime, but collapsed
              behind the operational stat block.
            </p>
          </details>
        ) : (
          <p className="surface-statblock__prose">
            <strong>Coven</strong> A night hag gains additional spells and rituals when
            working with a coven. This mock keeps prose in the stat block flow instead
            of isolating it inside a generic card.
          </p>
        )}
      </section>
    </article>
  );
}

function CreatureHybridSurface({
  creature,
  mode,
  variantControl = false,
}: {
  creature: MockCreature;
  mode: "static" | "encounter";
  variantControl?: boolean;
}) {
  const encounter = mode === "encounter";
  return (
    <article className="surface-hybrid">
      <header className="surface-hybrid__header">
        <div>
          <p className="eyebrow">{encounter ? "Encounter participant" : "Creature"}</p>
          <h3>{encounter ? "Elite Night Hag" : creature.name}</h3>
          <p>
            {creature.level} · {creature.key}
          </p>
          <TraitRow traits={creature.traits} />
        </div>
        {encounter ? (
          <div className="surface-variant-control">
            <span>Variant</span>
            <strong>Elite</strong>
          </div>
        ) : variantControl ? (
          <VariantMiniControl />
        ) : null}
      </header>

      <div className="surface-hybrid__grid">
        <section className="surface-hybrid-section surface-hybrid-section--vitals">
          <SectionTitle title="Vitals" />
          {encounter ? (
            <>
              <div className="surface-hp-bar" aria-hidden="true">
                <span style={{ width: "55%" }} />
                <em style={{ width: "11%" }} />
              </div>
              <p className="surface-statline">
                <strong>HP</strong> <AdjustedText value="62 / 113" base="95 max" />{" "}
                <span className="surface-muted-text">12 temp</span>
              </p>
              <div className="surface-control-row">
                <button type="button">Damage</button>
                <button type="button">Heal</button>
                <button type="button">Set HP</button>
              </div>
            </>
          ) : (
            <>
              <StatLine label="HP" value={String(creature.hp)} />
              <StatLine
                label="Perception"
                value={`${creature.perception}; darkvision`}
              />
            </>
          )}
        </section>

        <section className="surface-hybrid-section">
          <SectionTitle title="Defenses" />
          <StatLine
            label="AC"
            value={encounter ? <AdjustedText value="27" base="25" /> : "25"}
          />
          <StatLine
            label="Saves"
            value={
              encounter ? (
                <>
                  Fort <AdjustedText value="+19" base="+17" />, Ref{" "}
                  <AdjustedText value="+20" base="+18" />, Will{" "}
                  <AdjustedText value="+23" base="+21" />
                </>
              ) : (
                creature.saves
              )
            }
          />
        </section>

        <section className="surface-hybrid-section">
          <SectionTitle title="Skills" />
          <StatLine
            label="Skills"
            value={
              encounter ? (
                <>
                  Arcana <AdjustedText value="+22" base="+20" />, Deception{" "}
                  <AdjustedText value="+23" base="+21" />, Occultism{" "}
                  <AdjustedText value="+22" base="+20" />
                </>
              ) : (
                creature.skills
              )
            }
          />
          <AbilitySpread />
        </section>

        <section className="surface-hybrid-section">
          <SectionTitle title="Movement" />
          <StatLine label="Speed" value={creature.speed} />
          {encounter && (
            <StatLine
              label="Actions"
              value={
                <>
                  <AdjustedText value="4" base="3" />{" "}
                  <span className="surface-muted-text">quickened, restricted</span>
                </>
              }
            />
          )}
        </section>

        {encounter && (
          <section className="surface-hybrid-section surface-hybrid-section--conditions">
            <SectionTitle title="Conditions" />
            <p className="surface-statline">
              <ConditionPill value="Frightened 1" /> <ConditionPill value="Off-Guard" />
            </p>
            <p className="surface-muted-text">
              Frightened applies to checks and DCs. Off-Guard applies to AC.
            </p>
            <button className="surface-add-button" type="button">
              Add condition
            </button>
          </section>
        )}

        <section className="surface-hybrid-section surface-hybrid-section--wide">
          <SectionTitle title="Activities" />
          <ActionLine
            title="Melee"
            value={
              <>
                jaws{" "}
                <AdjustedText
                  value={encounter ? "+22" : "+20"}
                  base={encounter ? "+20" : undefined}
                />{" "}
                (magical), Damage{" "}
                <AdjustedText
                  value={encounter ? "2d8 + 10 piercing" : "2d8 + 8 piercing"}
                  base={encounter ? "2d8 + 8 piercing" : undefined}
                />
              </>
            }
          />
          <ActionLine
            title="Dream Haunting"
            value="occult, curse, mental; DC 28 Will; nightmare effect"
          />
        </section>

        <section className="surface-hybrid-section surface-hybrid-section--wide">
          <SectionTitle title="Description" />
          {encounter ? (
            <details className="surface-statblock__details">
              <summary>Description</summary>
              <p>{creature.description}</p>
            </details>
          ) : (
            <p className="surface-prose">{creature.description}</p>
          )}
        </section>
      </div>
    </article>
  );
}

function CreatureStructuredSurface({
  creature,
  profile,
}: {
  creature: MockCreature;
  profile: "search" | "encounter";
}) {
  const runtime = useRuntimeMockState();
  const encounter = profile === "encounter";
  const [participantName, setParticipantName] = useState("Night Hag 1");
  const [initiative, setInitiative] = useState("28");
  const [side, setSide] = useState<(typeof participantSides)[number]>("enemy");
  const [defeated, setDefeated] = useState(false);
  const [noteOpen, setNoteOpen] = useState(false);
  const [participantNote, setParticipantNote] = useState(
    "Soul gem is hidden in the reliquary.",
  );
  const displayName =
    profile === "search"
      ? creature.name
      : runtime.variant === "normal"
        ? participantName
        : `${variantLabels[runtime.variant]} ${participantName}`;
  const acValue = encounter ? String(runtime.ac) : String(creature.ac);
  const acBase = encounter ? String(creature.ac) : undefined;
  const attackValue = encounter ? runtime.attack : "+20";
  const attackBase = encounter ? "+20" : undefined;
  const damageValue = encounter ? runtime.damage : "2d8 + 8 piercing + 1d6 spirit";
  const damageBase = encounter ? "2d8 + 8 piercing + 1d6 spirit" : undefined;
  const variantAdjustment = variantAdjustmentLabel(runtime.variant);
  const variantDamageAdjustment = variantDamageAdjustmentLabel(runtime.variant);
  const variantHpAdjustment = variantHpAdjustmentLabel(runtime.variant);

  return (
    <article
      className={[
        "surface-structured",
        encounter ? "surface-structured--encounter" : "surface-structured--search",
      ].join(" ")}
    >
      <header className="surface-structured__header">
        <div className="surface-structured__identity">
          <p className="eyebrow">
            {encounter ? "Encounter participant" : "Search result surface"}
          </p>
          <h3>{displayName}</h3>
          <p>
            {creature.level} · {creature.key}
          </p>
          <TraitRow traits={creature.traits} />
        </div>
        <div className="surface-structured__header-actions">
          {encounter && (
            <button
              className="surface-icon-button"
              type="button"
              aria-label={noteOpen ? "Hide participant note" : "Edit participant note"}
              onClick={() => setNoteOpen((open) => !open)}
            >
              <Pencil size={15} />
            </button>
          )}
          <VariantMiniControl
            value={encounter ? runtime.variant : "normal"}
            onChange={encounter ? runtime.setVariant : undefined}
          />
        </div>
      </header>

      {encounter && (
        <>
          <div className="surface-structured__participant">
            <label>
              <span>Name</span>
              <input
                value={participantName}
                onChange={(event) => setParticipantName(event.target.value)}
              />
            </label>
            <label>
              <span>Initiative</span>
              <input
                inputMode="numeric"
                value={initiative}
                onChange={(event) => setInitiative(event.target.value)}
              />
            </label>
            <label>
              <span>Side</span>
              <select
                value={side}
                onChange={(event) =>
                  setSide(event.target.value as (typeof participantSides)[number])
                }
              >
                {participantSides.map((sideOption) => (
                  <option key={sideOption} value={sideOption}>
                    {sideOption}
                  </option>
                ))}
              </select>
            </label>
            <button
              className={
                defeated
                  ? "surface-toggle-button surface-toggle-button--on"
                  : "surface-toggle-button"
              }
              type="button"
              onClick={() => setDefeated((value) => !value)}
            >
              {defeated ? "Defeated" : "Active"}
            </button>
          </div>
          {noteOpen && (
            <section className="surface-structured-note">
              <label>
                <span>Participant note</span>
                <textarea
                  value={participantNote}
                  onChange={(event) => setParticipantNote(event.target.value)}
                />
              </label>
            </section>
          )}
          <div className="surface-structured__runtime">
            <section className="surface-structured-card surface-structured-card--vitals">
              <SectionTitle title="Vitals" />
              <RuntimeHpBar
                hp={runtime.hp}
                maxHp={runtime.maxHp}
                tempHp={runtime.tempHp}
              />
              <p className="surface-statline">
                <strong>HP</strong>{" "}
                <AdjustedText
                  value={`${runtime.hp} / ${runtime.maxHp}`}
                  base={`${creature.hp} max`}
                  adjustments={variantHpAdjustment ? [variantHpAdjustment] : undefined}
                  label="HP"
                />{" "}
                <span className="surface-muted-text">{runtime.tempHp} temp</span>
              </p>
              <RuntimeHpControls runtime={runtime} />
            </section>

            <section className="surface-structured-card surface-structured-card--conditions">
              <SectionTitle title="Conditions" />
              <ConditionMock runtime={runtime} detailed />
              <p className="surface-note">
                Frightened changes checks and DCs. Off-Guard is reflected in AC.
              </p>
            </section>
          </div>
        </>
      )}

      <div className="surface-structured__grid">
        <div className="surface-structured__column surface-structured__column--facts">
          <section className="surface-structured-card">
            <SectionTitle title="Defenses" />
            <CompactFactRows
              rows={[
                [
                  "AC",
                  <AdjustedText
                    key="ac"
                    value={acValue}
                    base={acBase}
                    adjustments={variantAdjustment ? [variantAdjustment] : undefined}
                    label="AC"
                  />,
                ],
                [
                  "Saves",
                  encounter ? (
                    <>
                      Fort{" "}
                      <AdjustedText
                        value={runtime.fort}
                        base="+19"
                        adjustments={
                          variantAdjustment ? [variantAdjustment] : undefined
                        }
                        label="Fortitude"
                      />
                      , Ref{" "}
                      <AdjustedText
                        value={runtime.ref}
                        base="+17"
                        adjustments={
                          variantAdjustment ? [variantAdjustment] : undefined
                        }
                        label="Reflex"
                      />
                      , Will{" "}
                      <AdjustedText
                        value={runtime.will}
                        base="+18"
                        adjustments={
                          variantAdjustment ? [variantAdjustment] : undefined
                        }
                        label="Will"
                      />
                    </>
                  ) : (
                    creature.saves
                  ),
                ],
                ["Immunities", "sleep"],
                ["Resistances", "mental 10"],
                ["Weaknesses", "cold iron 10"],
                [
                  "HP",
                  encounter ? (
                    <AdjustedText
                      value={`${runtime.maxHp} max`}
                      base={`${creature.hp} max`}
                      adjustments={
                        variantHpAdjustment ? [variantHpAdjustment] : undefined
                      }
                      label="Max HP"
                    />
                  ) : (
                    String(creature.hp)
                  ),
                ],
              ]}
            />
          </section>

          <section className="surface-structured-card">
            <SectionTitle title="Senses & Movement" />
            <CompactFactRows
              rows={[
                [
                  "Perception",
                  <AdjustedText
                    key="perception"
                    value={encounter ? runtime.perception : creature.perception}
                    base={encounter ? creature.perception : undefined}
                    adjustments={variantAdjustment ? [variantAdjustment] : undefined}
                    label="Perception"
                  />,
                ],
                ["Senses", "darkvision"],
                ["Speed", "land 25 ft"],
                [
                  "Actions",
                  encounter ? (
                    <>
                      3 actions, 1 reaction{" "}
                      <span className="surface-muted-text">no action condition</span>
                    </>
                  ) : (
                    "3 actions, 1 reaction"
                  ),
                ],
              ]}
            />
          </section>

          <section className="surface-structured-card surface-structured-card--skills">
            <SectionTitle title="Skills" />
            <CompactFactRows
              columns={2}
              rows={[
                [
                  "Arcana",
                  <AdjustedText
                    key="arcana"
                    value={
                      !encounter
                        ? "+18"
                        : runtime.variant === "elite"
                          ? "+20"
                          : runtime.variant === "weak"
                            ? "+16"
                            : "+18"
                    }
                    base={
                      !encounter || runtime.variant === "normal" ? undefined : "+18"
                    }
                    adjustments={variantAdjustment ? [variantAdjustment] : undefined}
                    label="Arcana"
                  />,
                ],
                [
                  "Deception",
                  <AdjustedText
                    key="deception"
                    value={
                      !encounter
                        ? "+18"
                        : runtime.variant === "elite"
                          ? "+20"
                          : runtime.variant === "weak"
                            ? "+16"
                            : "+18"
                    }
                    base={
                      !encounter || runtime.variant === "normal" ? undefined : "+18"
                    }
                    adjustments={variantAdjustment ? [variantAdjustment] : undefined}
                    label="Deception"
                  />,
                ],
                [
                  "Diplomacy",
                  <AdjustedText
                    key="diplomacy"
                    value={
                      !encounter
                        ? "+18"
                        : runtime.variant === "elite"
                          ? "+20"
                          : runtime.variant === "weak"
                            ? "+16"
                            : "+18"
                    }
                    base={
                      !encounter || runtime.variant === "normal" ? undefined : "+18"
                    }
                    adjustments={variantAdjustment ? [variantAdjustment] : undefined}
                    label="Diplomacy"
                  />,
                ],
                [
                  "Intimidation",
                  <AdjustedText
                    key="intimidation"
                    value={
                      !encounter
                        ? "+14"
                        : runtime.variant === "elite"
                          ? "+16"
                          : runtime.variant === "weak"
                            ? "+12"
                            : "+14"
                    }
                    base={
                      !encounter || runtime.variant === "normal" ? undefined : "+14"
                    }
                    adjustments={variantAdjustment ? [variantAdjustment] : undefined}
                    label="Intimidation"
                  />,
                ],
                [
                  "Occultism",
                  <AdjustedText
                    key="occultism"
                    value={
                      !encounter
                        ? "+20"
                        : runtime.variant === "elite"
                          ? "+22"
                          : runtime.variant === "weak"
                            ? "+18"
                            : "+20"
                    }
                    base={
                      !encounter || runtime.variant === "normal" ? undefined : "+20"
                    }
                    adjustments={variantAdjustment ? [variantAdjustment] : undefined}
                    label="Occultism"
                  />,
                ],
                [
                  "Religion",
                  <AdjustedText
                    key="religion"
                    value={
                      !encounter
                        ? "+20"
                        : runtime.variant === "elite"
                          ? "+22"
                          : runtime.variant === "weak"
                            ? "+18"
                            : "+20"
                    }
                    base={
                      !encounter || runtime.variant === "normal" ? undefined : "+20"
                    }
                    adjustments={variantAdjustment ? [variantAdjustment] : undefined}
                    label="Religion"
                  />,
                ],
              ]}
            />
          </section>
        </div>

        <div className="surface-structured__column surface-structured__column--content">
          <section className="surface-structured-card surface-structured-card--activities">
            <SectionTitle title="Activities" />
            <StructuredActivity
              title="Jaws"
              meta="melee, magical, unarmed, unholy"
              facts={[
                [
                  "Attack",
                  <AdjustedText
                    key="jaws-attack"
                    value={attackValue}
                    base={attackBase}
                    adjustments={variantAdjustment ? [variantAdjustment] : undefined}
                    label="Jaws attack"
                  />,
                ],
                [
                  "Damage",
                  <AdjustedText
                    key="jaws-damage"
                    value={damageValue}
                    base={damageBase}
                    adjustments={
                      variantDamageAdjustment ? [variantDamageAdjustment] : undefined
                    }
                    label="Jaws damage"
                  />,
                ],
                ["Effect", "abyssal plague"],
              ]}
            />
            <StructuredActivity
              title="Claw"
              meta="melee, agile, magical, unarmed, unholy"
              facts={[
                [
                  "Attack",
                  <AdjustedText
                    key="claw-attack"
                    value={attackValue}
                    base={attackBase}
                    adjustments={variantAdjustment ? [variantAdjustment] : undefined}
                    label="Claw attack"
                  />,
                ],
                [
                  "Damage",
                  <AdjustedText
                    key="claw-damage"
                    value={
                      !encounter
                        ? "2d10 + 8 slashing + 1d6 spirit"
                        : runtime.variant === "elite"
                          ? "2d10 + 12 slashing + 1d6 spirit"
                          : runtime.variant === "weak"
                            ? "2d10 + 4 slashing + 1d6 spirit"
                            : "2d10 + 8 slashing + 1d6 spirit"
                    }
                    base={encounter ? "2d10 + 8 slashing + 1d6 spirit" : undefined}
                    adjustments={
                      variantDamageAdjustment ? [variantDamageAdjustment] : undefined
                    }
                    label="Claw damage"
                  />,
                ],
              ]}
            />
            <StructuredActivity
              title="Dream Haunting"
              meta="occult, curse, mental"
              facts={[
                [
                  "Use",
                  "ethereal and hovering over a sleeping chaotic or evil creature",
                ],
                [
                  "Effect",
                  "casts Nightmare on the victim and exposes it to abyssal plague",
                ],
                [
                  "Note",
                  "Only an ethereal being can confront the night hag and stop the haunting.",
                ],
              ]}
            />
            <StructuredActivity
              title="Innate Occult Spells"
              meta="DC 28, attack +20, spell ambush can penalize target defenses"
              facts={[
                ["Constant", "detect alignment, detect magic 3rd"],
                [
                  "At will",
                  "dream message, invisibility, magic missile, ray of enfeeblement, sleep",
                ],
                ["Heartstone", "bind soul, ethereal jaunt 9th, shadow blast 2/day"],
                [
                  encounter ? "Adjusted attack" : "Spell attack",
                  <AdjustedText
                    key="spell-attack"
                    value={attackValue}
                    base={attackBase}
                    adjustments={variantAdjustment ? [variantAdjustment] : undefined}
                    label="Spell attack"
                  />,
                ],
              ]}
            />
            <StructuredActivity
              title="Coven"
              meta="mental, occult"
              facts={[
                [
                  "Spells",
                  "adds dominate, nightmare, scrying, and spellwrack to her coven's spells",
                ],
                [
                  "Model gap",
                  "This wants a linked rule popover plus a spellcasting-entry grouping, not loose prose.",
                ],
              ]}
            />
            <StructuredActivity
              title="Spell Ambush"
              meta="target-state interaction"
              facts={[
                ["Trigger", "target is Off-Guard to the night hag"],
                [
                  "Effect",
                  "-2 circumstance penalty to checks and DCs to defend against her spells",
                ],
              ]}
            />
          </section>

          <section className="surface-structured-card surface-structured-card--description">
            <SectionTitle title="Description" />
            {encounter ? (
              <details className="surface-statblock__details">
                <summary>Description</summary>
                <p>{creature.description}</p>
              </details>
            ) : (
              <p className="surface-prose">{creature.description}</p>
            )}
          </section>
        </div>
      </div>
    </article>
  );
}

function CompactFactRows({
  columns = 1,
  rows,
}: {
  columns?: 1 | 2;
  rows: Array<[string, React.ReactNode]>;
}) {
  return (
    <dl
      className={
        columns === 2
          ? "surface-compact-facts surface-compact-facts--two"
          : "surface-compact-facts"
      }
    >
      {rows.map(([label, value]) => (
        <div key={label}>
          <dt>{label}</dt>
          <dd>{value}</dd>
        </div>
      ))}
    </dl>
  );
}

function StructuredActivity({
  facts,
  meta,
  title,
}: {
  facts: Array<[string, React.ReactNode]>;
  meta: string;
  title: string;
}) {
  return (
    <article className="surface-structured-activity">
      <header>
        <strong>{title}</strong>
        <span>{meta}</span>
      </header>
      <CompactFactRows rows={facts} />
    </article>
  );
}

function CreatureRailSurface({ creature }: { creature: MockCreature }) {
  return (
    <article className="surface-rail">
      <aside className="surface-rail__summary">
        <CreatureTitle creature={creature} />
        <VariantMiniControl />
        <TraitRow traits={creature.traits} />
        <div className="surface-rail__facts">
          <CompactStat label="HP" value={String(creature.hp)} />
          <CompactStat label="AC" value={String(creature.ac)} />
          <CompactStat label="Speed" value={creature.speed} />
        </div>
      </aside>
      <div className="surface-rail__body">
        <section>
          <SectionTitle title="Description" />
          <p className="surface-prose">{creature.description}</p>
        </section>
        <Rule />
        <section>
          <SectionTitle title="Defenses" />
          <StatLine label="AC" value="25" />
          <StatLine label="Saves" value={creature.saves} />
        </section>
        <Rule />
        <section>
          <SectionTitle title="Skills & Movement" />
          <StatLine label="Skills" value={creature.skills} />
          <StatLine label="Speed" value={creature.speed} />
          <AbilitySpread />
        </section>
        <Rule />
        <section>
          <SectionTitle title="Activities" />
          <ActionLine
            title="Melee"
            value="jaws +20 (magical), Damage 2d8 + 8 piercing"
          />
          <ActionLine
            title="Dream Haunting"
            value="occult, curse, mental; DC 28 Will; nightmare effect"
          />
        </section>
      </div>
    </article>
  );
}

function CreaturePathbuilderSurface({
  creature,
  mode,
}: {
  creature: MockCreature;
  mode: "static" | "encounter";
}) {
  const encounter = mode === "encounter";
  const runtime = useRuntimeMockState();
  return (
    <article className="surface-pathbuilder">
      <header className="surface-pathbuilder__identity">
        <div>
          <p className="eyebrow">{encounter ? "Encounter participant" : "Creature"}</p>
          <h3>
            {encounter && runtime.variant !== "normal"
              ? `${variantLabels[runtime.variant]} ${creature.name}`
              : creature.name}
          </h3>
          <p>{creature.level}</p>
        </div>
        {encounter ? (
          <VariantMiniControl value={runtime.variant} onChange={runtime.setVariant} />
        ) : (
          <VariantMiniControl />
        )}
      </header>
      <div className="surface-pathbuilder__main">
        <section className="surface-pb-card surface-pb-card--span">
          <TraitRow traits={creature.traits} />
          <p className="surface-search-description">{creature.description}</p>
        </section>
        {encounter && (
          <section className="surface-pb-card surface-pb-card--runtime surface-pb-card--span">
            <span>Runtime HP</span>
            <RuntimeHpBar
              hp={runtime.hp}
              maxHp={runtime.maxHp}
              tempHp={runtime.tempHp}
            />
            <p className="surface-statline">
              <strong>HP</strong>{" "}
              <AdjustedText
                value={`${runtime.hp} / ${runtime.maxHp}`}
                base={`${creature.hp} max`}
              />{" "}
              <span className="surface-muted-text">{runtime.tempHp} temp</span>
            </p>
            <RuntimeHpControls runtime={runtime} />
          </section>
        )}
        <section className="surface-pb-card surface-pb-card--ac">
          <span>AC</span>
          {encounter ? (
            <AdjustedText value={String(runtime.ac)} base={String(creature.ac)} />
          ) : (
            <strong>{creature.ac}</strong>
          )}
        </section>
        <section className="surface-pb-card">
          <span>HP</span>
          <strong>
            {encounter ? `${runtime.hp} / ${runtime.maxHp}` : String(creature.hp)}
          </strong>
          <small>{encounter ? `${runtime.tempHp} temp` : "No shield"}</small>
        </section>
        <section className="surface-pb-card">
          <span>Speed</span>
          <strong>{creature.speed}</strong>
        </section>
        <section className="surface-pb-card">
          <SectionTitle title="Saves" />
          <DenseList
            rows={
              encounter
                ? [
                    ["Fort", `${runtime.fort} base +17`],
                    ["Ref", `${runtime.ref} base +18`],
                    ["Will", `${runtime.will} base +21`],
                  ]
                : [
                    ["Fort", "+17"],
                    ["Ref", "+18"],
                    ["Will", "+21"],
                  ]
            }
          />
        </section>
        <section className="surface-pb-card">
          <SectionTitle title="Checks" />
          <DenseList
            rows={
              encounter
                ? [
                    ["Perception", `${runtime.perception} base +18`],
                    [
                      "Arcana",
                      runtime.variant === "normal"
                        ? "+20"
                        : `${runtime.variant === "elite" ? "+22" : "+18"} base +20`,
                    ],
                    [
                      "Deception",
                      runtime.variant === "normal"
                        ? "+21"
                        : `${runtime.variant === "elite" ? "+23" : "+19"} base +21`,
                    ],
                    [
                      "Occultism",
                      runtime.variant === "normal"
                        ? "+20"
                        : `${runtime.variant === "elite" ? "+22" : "+18"} base +20`,
                    ],
                  ]
                : [
                    ["Perception", creature.perception],
                    ["Arcana", "+20"],
                    ["Deception", "+21"],
                    ["Occultism", "+20"],
                  ]
            }
          />
        </section>
        {encounter && (
          <section className="surface-pb-card surface-pb-card--span">
            <SectionTitle title="Conditions" />
            <ConditionMock runtime={runtime} />
          </section>
        )}
        <section className="surface-pb-card surface-pb-card--span">
          <SectionTitle title="Actions" />
          <ActionLine
            title="Jaws"
            value={
              encounter ? (
                <>
                  melee <AdjustedText value={runtime.attack} base="+20" />,{" "}
                  <AdjustedText value={runtime.damage} base="2d8 + 8 piercing" />
                </>
              ) : (
                "melee +20, 2d8 + 8 piercing"
              )
            }
          />
          <ActionLine title="Dream Haunting" value="DC 28 Will; nightmare effect" />
        </section>
      </div>
    </article>
  );
}

function CreatureBeyondSurface({
  creature,
  mode,
}: {
  creature: MockCreature;
  mode: "static" | "encounter";
}) {
  const encounter = mode === "encounter";
  const runtime = useRuntimeMockState();
  return (
    <article className="surface-beyond">
      <header className="surface-beyond__header">
        <div>
          <h3>
            {encounter && runtime.variant !== "normal"
              ? `${variantLabels[runtime.variant]} ${creature.name}`
              : creature.name}
          </h3>
          <p>
            {creature.level} · {creature.key}
          </p>
        </div>
        {encounter ? (
          <VariantMiniControl value={runtime.variant} onChange={runtime.setVariant} />
        ) : (
          <VariantMiniControl />
        )}
      </header>
      {encounter && (
        <section className="surface-beyond-runtime">
          <div>
            <strong>
              HP {runtime.hp} / {runtime.maxHp}
            </strong>
            <RuntimeHpBar
              hp={runtime.hp}
              maxHp={runtime.maxHp}
              tempHp={runtime.tempHp}
            />
            <small>
              base max {creature.hp} · {runtime.tempHp} temp
            </small>
          </div>
          <RuntimeHpControls runtime={runtime} compact />
        </section>
      )}
      <div className="surface-beyond__topstats">
        <HeroStat
          adjusted={encounter}
          label="AC"
          sublabel={encounter ? "base 25" : "armor class"}
          value={encounter ? String(runtime.ac) : String(creature.ac)}
        />
        <HeroStat
          adjusted={encounter}
          label="HP"
          sublabel={encounter ? `${runtime.tempHp} temp` : "hit points"}
          value={encounter ? `${runtime.hp} / ${runtime.maxHp}` : String(creature.hp)}
        />
        <HeroStat
          adjusted={encounter}
          label="Perception"
          sublabel={encounter ? "base +18" : undefined}
          value={encounter ? runtime.perception : creature.perception}
        />
        <HeroStat label="Speed" value={creature.speed} />
      </div>
      <nav className="surface-beyond__tabs" aria-label="Mock sections">
        <span aria-current="page">Stats</span>
        <span>Actions</span>
        <span>Description</span>
      </nav>
      <div className="surface-beyond__content">
        <section>
          <SectionTitle title="Abilities, Saves, Senses" />
          <AbilitySpread />
          <StatLine
            label="Saves"
            value={
              encounter ? (
                <>
                  Fort <AdjustedText value="+19" base="+17" />, Ref{" "}
                  <AdjustedText value={runtime.ref} base="+18" />, Will{" "}
                  <AdjustedText value={runtime.will} base="+21" />
                </>
              ) : (
                creature.saves
              )
            }
          />
          <StatLine
            label="Senses"
            value={`Perception ${creature.perception}; darkvision`}
          />
        </section>
        <section>
          <SectionTitle title="Skills" />
          <StatLine label="Trained" value={creature.skills} />
        </section>
        <section>
          <SectionTitle title="Actions" />
          <ActionLine
            title="Jaws"
            value={
              encounter ? (
                <>
                  melee <AdjustedText value={runtime.attack} base="+20" />,{" "}
                  <AdjustedText value={runtime.damage} base="2d8 + 8 piercing" />
                </>
              ) : (
                "melee +20, 2d8 + 8 piercing"
              )
            }
          />
          <ActionLine title="Dream Haunting" value="DC 28 Will; nightmare effect" />
        </section>
        {encounter && (
          <section>
            <SectionTitle title="Conditions" />
            <ConditionMock runtime={runtime} />
          </section>
        )}
      </div>
    </article>
  );
}

function DenseList({ rows }: { rows: Array<[string, string]> }) {
  return (
    <dl className="surface-dense-list">
      {rows.map(([label, value]) => (
        <div key={label}>
          <dt>{label}</dt>
          <dd>{value}</dd>
        </div>
      ))}
    </dl>
  );
}

function HeroStat({
  adjusted = false,
  label,
  sublabel,
  value,
}: {
  adjusted?: boolean;
  label: string;
  sublabel?: string;
  value: string;
}) {
  return (
    <div
      className={
        adjusted ? "surface-hero-stat surface-hero-stat--adjusted" : "surface-hero-stat"
      }
    >
      <span>{label}</span>
      <strong>{value}</strong>
      {sublabel && <small>{sublabel}</small>}
    </div>
  );
}

function RuntimeHpBar({
  hp,
  maxHp,
  tempHp,
}: {
  hp: number;
  maxHp: number;
  tempHp: number;
}) {
  const mainWidth = maxHp > 0 ? Math.round((hp / (maxHp + tempHp)) * 100) : 0;
  const tempWidth =
    maxHp + tempHp > 0 ? Math.round((tempHp / (maxHp + tempHp)) * 100) : 0;

  return (
    <div className="surface-hp-bar" aria-hidden="true">
      <span style={{ width: `${mainWidth}%` }} />
      {tempHp > 0 && <em style={{ width: `${tempWidth}%` }} />}
    </div>
  );
}

function RuntimeHpControls({
  compact = false,
  runtime,
}: {
  compact?: boolean;
  runtime: RuntimeMockController;
}) {
  const [changeInput, setChangeInput] = useState("10");
  const [hpDraft, setHpDraft] = useState<string | null>(null);
  const [tempDraft, setTempDraft] = useState<string | null>(null);
  const hpInput = hpDraft ?? String(runtime.hp);
  const tempInput = tempDraft ?? String(runtime.tempHp);

  function parsed(value: string) {
    const next = Number.parseInt(value, 10);
    return Number.isFinite(next) ? next : 0;
  }

  function applySignedChange() {
    const amount = parsed(changeInput);
    if (amount < 0) {
      runtime.applyDamage(Math.abs(amount));
      return;
    }
    runtime.applyHeal(amount);
  }

  function setHp() {
    runtime.setDirectHp(parsed(hpInput));
    setHpDraft(null);
  }

  function setTemp() {
    runtime.setTempHp(parsed(tempInput));
    setTempDraft(null);
  }

  return (
    <div
      className={
        compact
          ? "surface-runtime-controls surface-runtime-controls--compact"
          : "surface-runtime-controls"
      }
    >
      <label>
        <span>HP change</span>
        <input
          value={changeInput}
          onChange={(event) => setChangeInput(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              applySignedChange();
            }
          }}
        />
      </label>
      <div className="surface-control-row">
        <button
          type="button"
          onClick={() => runtime.applyDamage(Math.abs(parsed(changeInput)))}
        >
          Damage
        </button>
        <button
          type="button"
          onClick={() => runtime.applyHeal(Math.abs(parsed(changeInput)))}
        >
          Heal
        </button>
      </div>
      <label>
        <span>HP</span>
        <input
          value={hpInput}
          onChange={(event) => setHpDraft(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              setHp();
            }
          }}
        />
      </label>
      <button type="button" onClick={setHp}>
        Set
      </button>
      <label>
        <span>Temp HP</span>
        <input
          value={tempInput}
          onChange={(event) => setTempDraft(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter") {
              setTemp();
            }
          }}
        />
      </label>
      <button type="button" onClick={setTemp}>
        Set
      </button>
    </div>
  );
}

function ConditionMock({
  detailed = false,
  runtime,
}: {
  detailed?: boolean;
  runtime: RuntimeMockController;
}) {
  const [selected, setSelected] = useState(conditionOptions[2]);
  const [value, setValue] = useState("1");
  const [duration, setDuration] = useState("3");
  const [source, setSource] = useState("Kyra");
  const [note, setNote] = useState("Applied by spell effect.");
  const selectedHasValue = selected !== "Off-Guard" && selected !== "Quickened";

  return (
    <div className="surface-condition-mock">
      <div className="surface-condition-list">
        {runtime.conditions.map((condition) => (
          <div className="surface-condition-item" key={condition}>
            <ConditionPill
              value={condition}
              onRemove={() => runtime.removeCondition(condition)}
            />
            {detailed && condition === "Frightened 1" && (
              <span className="surface-condition-meta">3 rounds · Details</span>
            )}
          </div>
        ))}
      </div>
      <div className="surface-condition-add">
        <select value={selected} onChange={(event) => setSelected(event.target.value)}>
          {conditionOptions.map((condition) => (
            <option key={condition} value={condition}>
              {condition}
            </option>
          ))}
        </select>
        {detailed && selectedHasValue && (
          <input
            aria-label="Condition value"
            inputMode="numeric"
            value={value}
            onChange={(event) => setValue(event.target.value)}
          />
        )}
        <button type="button" onClick={() => runtime.addCondition(selected)}>
          Add condition
        </button>
      </div>
      {detailed && (
        <details className="surface-condition-details">
          <summary>Condition details</summary>
          <div>
            <label>
              <span>Duration</span>
              <input
                inputMode="numeric"
                value={duration}
                onChange={(event) => setDuration(event.target.value)}
              />
            </label>
            <label>
              <span>Source</span>
              <select
                value={source}
                onChange={(event) => setSource(event.target.value)}
              >
                {["Kyra", "Night Hag 1", "Valeros"].map((participant) => (
                  <option key={participant} value={participant}>
                    {participant}
                  </option>
                ))}
              </select>
            </label>
            <label className="surface-condition-details__note">
              <span>Note</span>
              <textarea
                value={note}
                onChange={(event) => setNote(event.target.value)}
              />
            </label>
          </div>
        </details>
      )}
    </div>
  );
}

function StatLine({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <p className="surface-statline">
      <strong>{label}</strong> {value}
    </p>
  );
}

function ActionLine({ title, value }: { title: string; value: React.ReactNode }) {
  return (
    <p className="surface-actionline">
      <strong>{title}</strong> {value}
    </p>
  );
}

function AbilitySpread() {
  return (
    <div className="surface-ability-line" aria-label="Ability modifiers">
      {[
        ["Str", "+4"],
        ["Dex", "+5"],
        ["Con", "+3"],
        ["Int", "+6"],
        ["Wis", "+5"],
        ["Cha", "+6"],
      ].map(([label, value]) => (
        <span key={label}>
          <strong>{label}</strong> {value}
        </span>
      ))}
    </div>
  );
}

function AdjustedText({
  adjustments = [],
  base,
  label,
  value,
}: {
  adjustments?: string[];
  base?: string;
  label?: string;
  value: string;
}) {
  const changed = base !== undefined && base !== value;
  if (!changed && adjustments.length === 0) {
    return <strong className="surface-inline-value">{value}</strong>;
  }
  const title = [
    label,
    base ? `Base ${base}` : undefined,
    ...adjustments,
    `Final ${value}`,
  ]
    .filter(Boolean)
    .join("\n");
  return (
    <button className="surface-adjusted-inline" type="button" title={title}>
      <strong>{value}</strong>
    </button>
  );
}

function variantAdjustmentLabel(variant: MockVariant): string | undefined {
  if (variant === "normal") {
    return undefined;
  }
  return `${variant === "elite" ? "+2" : "-2"} ${variantLabels[variant].toLowerCase()}`;
}

function variantDamageAdjustmentLabel(variant: MockVariant): string | undefined {
  if (variant === "normal") {
    return undefined;
  }
  return `${variant === "elite" ? "+4" : "-4"} ${variantLabels[variant].toLowerCase()}`;
}

function variantHpAdjustmentLabel(variant: MockVariant): string | undefined {
  if (variant === "normal") {
    return undefined;
  }
  return `${variant === "elite" ? "+20" : "-20"} ${variantLabels[variant].toLowerCase()}`;
}

function ConditionPill({ onRemove, value }: { onRemove?: () => void; value: string }) {
  return (
    <span className="surface-condition-pill">
      {value}
      {onRemove && (
        <button type="button" onClick={onRemove} aria-label={`Remove ${value}`}>
          x
        </button>
      )}
    </span>
  );
}

function Rule() {
  return <hr className="surface-statblock-rule" />;
}

function SectionTitle({ title }: { title: string }) {
  return (
    <header className="surface-section__title">
      <h4>{title}</h4>
    </header>
  );
}

function SurfaceValue({
  adjusted = false,
  base,
  label,
  value,
}: {
  adjusted?: boolean;
  base?: string;
  label: string;
  value: string;
}) {
  return (
    <div
      className={adjusted ? "surface-value surface-value--adjusted" : "surface-value"}
    >
      <span>{label}</span>
      <strong>{value}</strong>
      {base && <small>base {base}</small>}
    </div>
  );
}

function ActivityMock({
  base,
  damage,
  meta,
  primary,
  title,
}: {
  base?: string;
  damage: string;
  meta: string;
  primary: string;
  title: string;
}) {
  return (
    <div className="surface-activity">
      <div>
        <strong>{title}</strong>
        <span>{meta}</span>
      </div>
      <SurfaceValue
        label="Attack"
        value={primary}
        base={base}
        adjusted={base !== undefined}
      />
      <SurfaceValue label="Damage" value={damage} adjusted={base !== undefined} />
    </div>
  );
}

function MockHeading({
  label,
  note,
  title,
}: {
  label: string;
  note: string;
  title: string;
}) {
  return (
    <header className="surface-mock__heading">
      <div>
        <p className="eyebrow">{label}</p>
        <h3>{title}</h3>
      </div>
      <p>{note}</p>
    </header>
  );
}

function CreatureTitle({ creature }: { creature: MockCreature }) {
  return (
    <div className="surface-creature-title">
      <strong>{creature.name}</strong>
      <span>
        {creature.level} · {creature.key}
      </span>
    </div>
  );
}

function TraitRow({ traits }: { traits: string[] }) {
  return (
    <div className="surface-traits">
      {traits.map((trait) => (
        <Tag key={trait}>{trait}</Tag>
      ))}
    </div>
  );
}

function VariantMiniControl({
  onChange,
  value = "normal",
}: {
  onChange?: (variant: MockVariant) => void;
  value?: MockVariant;
}) {
  return (
    <div className="surface-variant-mini" aria-label="Variant preview">
      <span>Variant</span>
      <div>
        {(["weak", "normal", "elite"] as const).map((variant) => (
          <button
            className={variant === value ? "surface-variant-mini__active" : undefined}
            key={variant}
            type="button"
            onClick={() => onChange?.(variant)}
          >
            {variantLabels[variant]}
          </button>
        ))}
      </div>
    </div>
  );
}

function CompactStat({ label, value }: { label: string; value: string }) {
  return (
    <span className="surface-stat">
      <span>{label}</span>
      <strong>{value}</strong>
    </span>
  );
}
