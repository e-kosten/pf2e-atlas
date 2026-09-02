import { RightOutlined } from "@ant-design/icons";
import { Collapse, Tag } from "antd";
import type { CollapseProps } from "antd";
import type {
  CreatureSurfaceDomainUnavailableView,
  CreatureSurfaceUnavailableCauseView,
  CreatureSurfaceUnavailableDomainsView,
  CreatureSurfaceUnavailableFieldView,
} from "../../generated/atlas";

export function DataAvailabilityDisclosure({
  unavailable,
}: {
  unavailable: CreatureSurfaceUnavailableDomainsView | undefined;
}) {
  const domains: Array<{
    key: string;
    label: string;
    value: CreatureSurfaceDomainUnavailableView | undefined;
  }> = [
    {
      key: "classification",
      label: "Classification",
      value: unavailable?.classification,
    },
    { key: "initiative", label: "Initiative", value: unavailable?.initiative },
    { key: "vitals", label: "Vitals", value: unavailable?.vitals },
    { key: "defenses", label: "Defenses", value: unavailable?.defenses },
    { key: "saves", label: "Saves", value: unavailable?.saves },
    { key: "awareness", label: "Awareness", value: unavailable?.awareness },
    { key: "abilities", label: "Abilities", value: unavailable?.abilities },
    { key: "skills", label: "Skills", value: unavailable?.skills },
    { key: "movement", label: "Movement", value: unavailable?.movement },
    { key: "resources", label: "Resources", value: unavailable?.resources },
    { key: "equipment", label: "Equipment", value: unavailable?.equipment },
    { key: "lore", label: "Lore", value: unavailable?.lore },
    { key: "spellcasting", label: "Spellcasting", value: unavailable?.spellcasting },
    { key: "activities", label: "Actions & Abilities", value: unavailable?.activities },
    { key: "relationships", label: "Relationships", value: unavailable?.relationships },
  ];
  const populated = domains.filter((domain) => domain.value?.causes.length);
  if (!populated.length) return null;
  return (
    <Collapse
      className="record-surface__secondary record-surface__availability"
      expandIcon={disclosureExpandIcon}
      ghost
      items={[
        {
          key: "data-availability",
          label: "Data availability",
          children: (
            <div className="creature-sheet__availability-domains">
              {populated.map((domain) => (
                <section key={domain.key}>
                  <h4>{domain.label}</h4>
                  <div className="creature-sheet__availability-list">
                    {domain.value!.causes.map((cause, index) => (
                      <UnavailableCause
                        cause={cause}
                        key={`${cause.field}:${cause.component_id ?? index}`}
                      />
                    ))}
                  </div>
                </section>
              ))}
            </div>
          ),
        },
      ]}
      size="small"
    />
  );
}

function UnavailableCause({ cause }: { cause: CreatureSurfaceUnavailableCauseView }) {
  const unmodeled = cause.unmodeled_skill;
  return (
    <article className="creature-sheet__availability-cause">
      <div className="creature-sheet__availability-heading">
        <strong>
          {unmodeled ? "Unmodeled skill entry" : unavailableFieldLabel(cause.field)}
        </strong>
        <Tag>{unavailableStateLabel(cause.state)}</Tag>
      </div>
      {unmodeled ? (
        <dl className="creature-sheet__authored-value">
          <dt>Value</dt>
          <dd>
            <code>{unmodeled.authored_key}</code>
          </dd>
        </dl>
      ) : null}
      <p>{cause.message}</p>
    </article>
  );
}

function unavailableStateLabel(state: CreatureSurfaceUnavailableCauseView["state"]) {
  switch (state) {
    case "missing":
      return "Missing";
    case "null":
      return "Null";
    case "unsupported":
      return "Unsupported";
  }
}

function unavailableFieldLabel(field: CreatureSurfaceUnavailableFieldView) {
  switch (field) {
    case "size":
      return "Size";
    case "adjustment":
      return "Adjustment";
    case "initiative_statistic":
      return "Initiative statistic";
    case "defenses":
      return "Defenses";
    case "hit_points":
      return "Hit points";
    case "armor_class":
      return "Armor Class";
    case "shield_armor_class_bonus":
      return "Shield AC bonus";
    case "shield_broken_threshold":
      return "Shield broken threshold";
    case "shield_hardness":
      return "Shield hardness";
    case "shield_maximum_hit_points":
      return "Shield maximum HP";
    case "saves":
      return "Saves";
    case "immunities":
      return "Immunities";
    case "resistances":
      return "Resistances";
    case "weaknesses":
      return "Weaknesses";
    case "iwr_amount":
      return "Defense amount";
    case "iwr_exceptions":
      return "Defense exceptions";
    case "iwr_double_vs":
      return "Doubled defense";
    case "perception":
      return "Perception";
    case "senses":
      return "Senses";
    case "sense_acuity":
      return "Sense acuity";
    case "languages":
      return "Languages";
    case "skills":
      return "Skills";
    case "skill_modifier":
      return "Skill modifier";
    case "skill_variant_modifier":
      return "Skill variant modifier";
    case "skill_variant_predicate":
      return "Skill variant condition";
    case "unmodeled_skill":
      return "Unmodeled skill entry";
    case "legacy_abilities":
      return "Ability modifiers";
    case "movement":
      return "Movement";
    case "movement_mode":
      return "Movement mode";
    case "movement_speed":
      return "Movement speed";
    case "resources":
      return "Resources";
    case "resource_maximum":
      return "Resource maximum";
    case "embedded_entities":
      return "Embedded capabilities";
    case "activity_traits":
      return "Activity traits";
    case "activity_action_cost":
      return "Action cost";
    case "action_frequency_maximum":
      return "Action frequency maximum";
    case "action_frequency_period":
      return "Action frequency period";
    case "action_uses_maximum":
      return "Action uses maximum";
    case "activity_roll":
      return "Activity roll";
    case "activity_damage":
      return "Activity damage";
    case "activity_content":
      return "Activity content";
    case "damage_formula":
      return "Damage formula";
    case "damage_type":
      return "Damage type";
    case "spell_preparation":
      return "Spell preparation";
    case "spell_tradition":
      return "Spell tradition";
    case "spell_attack":
      return "Spell attack";
    case "spell_difficulty_class":
      return "Spell DC";
    case "spell_traits":
      return "Spell traits";
    case "spell_rank":
      return "Spell rank";
    case "spell_uses_maximum":
      return "Spell uses maximum";
    case "spell_slot_maximum":
      return "Spell slot maximum";
    case "ritual_difficulty_class":
      return "Ritual DC";
    case "spell_content":
      return "Spell content";
    case "equipment":
      return "Equipment";
    case "equipment_uses_maximum":
      return "Equipment uses maximum";
    case "lore":
      return "Lore";
    case "lore_modifier":
      return "Lore modifier";
    case "relationships":
      return "Relationships";
  }
}

const disclosureExpandIcon: NonNullable<CollapseProps["expandIcon"]> = ({
  isActive,
}) => <RightOutlined aria-hidden="true" rotate={isActive ? 90 : 0} />;
