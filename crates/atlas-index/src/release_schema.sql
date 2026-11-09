-- Reference schema at the release checkpoint. Model-specific vec0 tables are created on publication.

CREATE TABLE ability_projection(
 id INTEGER PRIMARY KEY,
 root_record_id INTEGER UNIQUE REFERENCES records ON DELETE CASCADE,
 actor_item_id INTEGER UNIQUE REFERENCES actor_items ON DELETE CASCADE,
 action_type_state TEXT NOT NULL,
 action_type TEXT,
 action_count_state TEXT NOT NULL,
 action_count ANY,
 category_state TEXT NOT NULL,
 category TEXT,
 CHECK(action_type_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((action_type_state='value' AND action_type IS NOT NULL) OR (action_type_state<>'value' AND action_type IS NULL)),
 CHECK(action_count_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((action_count_state='value' AND action_count IS NOT NULL AND typeof(action_count) IN ('integer','real')) OR (action_count_state<>'value' AND action_count IS NULL)),
 CHECK(category_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((category_state='value' AND category IS NOT NULL) OR (category_state<>'value' AND category IS NULL)),
 CHECK((root_record_id IS NOT NULL) <> (actor_item_id IS NOT NULL))
) STRICT;

CREATE TABLE actor_items(
 id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES actor_projection ON DELETE CASCADE,
 original_index INTEGER NOT NULL CHECK(original_index>=0),
 owner_selector_json TEXT NOT NULL CHECK(json_valid(owner_selector_json)),
 authored_id_state TEXT NOT NULL,
 authored_id TEXT,
 source_type_state TEXT NOT NULL,
 source_type TEXT,
 traits_state TEXT NOT NULL,
 rarity_state TEXT NOT NULL,
 rarity TEXT,
 level_state TEXT NOT NULL,
 level ANY,
 size_state TEXT NOT NULL,
 size TEXT,
 publication_title_state TEXT NOT NULL,
 publication_title TEXT,
 publication_remaster_state TEXT NOT NULL,
 publication_remaster INTEGER,
 UNIQUE(record_id,original_index),
 CHECK(authored_id_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((authored_id_state='value' AND authored_id IS NOT NULL) OR (authored_id_state<>'value' AND authored_id IS NULL)),
 CHECK(source_type_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((source_type_state='value' AND source_type IS NOT NULL) OR (source_type_state<>'value' AND source_type IS NULL)),
 CHECK(traits_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK(rarity_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((rarity_state='value' AND rarity IS NOT NULL) OR (rarity_state<>'value' AND rarity IS NULL)),
 CHECK(level_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((level_state='value' AND level IS NOT NULL AND typeof(level) IN ('integer','real')) OR (level_state<>'value' AND level IS NULL)),
 CHECK(size_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((size_state='value' AND size IS NOT NULL) OR (size_state<>'value' AND size IS NULL)),
 CHECK(publication_title_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((publication_title_state='value' AND publication_title IS NOT NULL) OR (publication_title_state<>'value' AND publication_title IS NULL)),
 CHECK(publication_remaster_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((publication_remaster_state='value' AND publication_remaster IS NOT NULL AND publication_remaster IN (0,1)) OR (publication_remaster_state<>'value' AND publication_remaster IS NULL))
) STRICT;

CREATE TABLE actor_iwr_entries(
 entry_id INTEGER PRIMARY KEY,record_id INTEGER NOT NULL REFERENCES actor_projection ON DELETE CASCADE,
 kind TEXT NOT NULL CHECK(kind IN ('immunity','weakness','resistance')),
 original_index INTEGER NOT NULL CHECK(original_index>=0),
 type_state TEXT NOT NULL,type TEXT,
 value_state TEXT NOT NULL,value ANY,
 UNIQUE(record_id,kind,original_index),
 CHECK(type_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((type_state='value' AND type IS NOT NULL)
   OR (type_state<>'value' AND type IS NULL)),
 CHECK(value_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((value_state='value' AND value IS NOT NULL AND typeof(value) IN ('integer','real'))
   OR (value_state<>'value' AND value IS NULL)),
 CHECK((kind='immunity' AND value_state='not_applicable')
   OR (kind<>'immunity' AND value_state<>'not_applicable'))
) STRICT;

CREATE TABLE actor_languages(record_id INTEGER NOT NULL REFERENCES actor_projection ON DELETE CASCADE,value TEXT NOT NULL,PRIMARY KEY(record_id,value)) WITHOUT ROWID,STRICT;

CREATE TABLE actor_projection(
 record_id INTEGER PRIMARY KEY REFERENCES records ON DELETE CASCADE,
 armor_class_state TEXT NOT NULL,
 armor_class ANY,
 hp_maximum_state TEXT NOT NULL,
 hp_maximum ANY,
 hardness_state TEXT NOT NULL,
 hardness ANY,
 complexity_state TEXT NOT NULL,
 complexity INTEGER,
 items_state TEXT NOT NULL,
 fortitude_state TEXT NOT NULL,
 fortitude ANY,
 reflex_state TEXT NOT NULL,
 reflex ANY,
 will_state TEXT NOT NULL,
 will ANY,
 immunities_state TEXT NOT NULL,
 weaknesses_state TEXT NOT NULL,
 resistances_state TEXT NOT NULL,
 perception_state TEXT NOT NULL,
 perception ANY,
 land_speed_state TEXT NOT NULL,
 land_speed ANY,
 languages_state TEXT NOT NULL,
 speeds_state TEXT NOT NULL,
 senses_state TEXT NOT NULL, stealth_state TEXT NOT NULL DEFAULT 'missing' CHECK(stealth_state IN ('value','missing','null','invalid','not_applicable')), stealth ANY CHECK((stealth_state='value' AND stealth IS NOT NULL AND typeof(stealth) IN ('integer','real')) OR (stealth_state<>'value' AND stealth IS NULL)), ability_str_state TEXT NOT NULL DEFAULT 'missing' CHECK(ability_str_state IN ('value','missing','null','invalid','not_applicable')), ability_str ANY CHECK((ability_str_state='value' AND ability_str IS NOT NULL AND typeof(ability_str) IN ('integer','real')) OR (ability_str_state<>'value' AND ability_str IS NULL)), ability_dex_state TEXT NOT NULL DEFAULT 'missing' CHECK(ability_dex_state IN ('value','missing','null','invalid','not_applicable')), ability_dex ANY CHECK((ability_dex_state='value' AND ability_dex IS NOT NULL AND typeof(ability_dex) IN ('integer','real')) OR (ability_dex_state<>'value' AND ability_dex IS NULL)), ability_con_state TEXT NOT NULL DEFAULT 'missing' CHECK(ability_con_state IN ('value','missing','null','invalid','not_applicable')), ability_con ANY CHECK((ability_con_state='value' AND ability_con IS NOT NULL AND typeof(ability_con) IN ('integer','real')) OR (ability_con_state<>'value' AND ability_con IS NULL)), ability_int_state TEXT NOT NULL DEFAULT 'missing' CHECK(ability_int_state IN ('value','missing','null','invalid','not_applicable')), ability_int ANY CHECK((ability_int_state='value' AND ability_int IS NOT NULL AND typeof(ability_int) IN ('integer','real')) OR (ability_int_state<>'value' AND ability_int IS NULL)), ability_wis_state TEXT NOT NULL DEFAULT 'missing' CHECK(ability_wis_state IN ('value','missing','null','invalid','not_applicable')), ability_wis ANY CHECK((ability_wis_state='value' AND ability_wis IS NOT NULL AND typeof(ability_wis) IN ('integer','real')) OR (ability_wis_state<>'value' AND ability_wis IS NULL)), ability_cha_state TEXT NOT NULL DEFAULT 'missing' CHECK(ability_cha_state IN ('value','missing','null','invalid','not_applicable')), ability_cha ANY CHECK((ability_cha_state='value' AND ability_cha IS NOT NULL AND typeof(ability_cha) IN ('integer','real')) OR (ability_cha_state<>'value' AND ability_cha IS NULL)), skill_acrobatics_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_acrobatics_modifier_state IN ('value','missing','null','invalid','not_applicable')), skill_acrobatics_modifier ANY CHECK((skill_acrobatics_modifier_state='value' AND skill_acrobatics_modifier IS NOT NULL AND typeof(skill_acrobatics_modifier) IN ('integer','real')) OR (skill_acrobatics_modifier_state<>'value' AND skill_acrobatics_modifier IS NULL)), skill_acrobatics_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_acrobatics_rank_state IN ('value','missing','null','invalid','not_applicable')), skill_acrobatics_rank ANY CHECK((skill_acrobatics_rank_state='value' AND skill_acrobatics_rank IS NOT NULL AND typeof(skill_acrobatics_rank) IN ('integer','real')) OR (skill_acrobatics_rank_state<>'value' AND skill_acrobatics_rank IS NULL)), skill_arcana_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_arcana_modifier_state IN ('value','missing','null','invalid','not_applicable')), skill_arcana_modifier ANY CHECK((skill_arcana_modifier_state='value' AND skill_arcana_modifier IS NOT NULL AND typeof(skill_arcana_modifier) IN ('integer','real')) OR (skill_arcana_modifier_state<>'value' AND skill_arcana_modifier IS NULL)), skill_arcana_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_arcana_rank_state IN ('value','missing','null','invalid','not_applicable')), skill_arcana_rank ANY CHECK((skill_arcana_rank_state='value' AND skill_arcana_rank IS NOT NULL AND typeof(skill_arcana_rank) IN ('integer','real')) OR (skill_arcana_rank_state<>'value' AND skill_arcana_rank IS NULL)), skill_athletics_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_athletics_modifier_state IN ('value','missing','null','invalid','not_applicable')), skill_athletics_modifier ANY CHECK((skill_athletics_modifier_state='value' AND skill_athletics_modifier IS NOT NULL AND typeof(skill_athletics_modifier) IN ('integer','real')) OR (skill_athletics_modifier_state<>'value' AND skill_athletics_modifier IS NULL)), skill_athletics_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_athletics_rank_state IN ('value','missing','null','invalid','not_applicable')), skill_athletics_rank ANY CHECK((skill_athletics_rank_state='value' AND skill_athletics_rank IS NOT NULL AND typeof(skill_athletics_rank) IN ('integer','real')) OR (skill_athletics_rank_state<>'value' AND skill_athletics_rank IS NULL)), skill_computers_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_computers_modifier_state IN ('value','missing','null','invalid','not_applicable')), skill_computers_modifier ANY CHECK((skill_computers_modifier_state='value' AND skill_computers_modifier IS NOT NULL AND typeof(skill_computers_modifier) IN ('integer','real')) OR (skill_computers_modifier_state<>'value' AND skill_computers_modifier IS NULL)), skill_computers_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_computers_rank_state IN ('value','missing','null','invalid','not_applicable')), skill_computers_rank ANY CHECK((skill_computers_rank_state='value' AND skill_computers_rank IS NOT NULL AND typeof(skill_computers_rank) IN ('integer','real')) OR (skill_computers_rank_state<>'value' AND skill_computers_rank IS NULL)), skill_crafting_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_crafting_modifier_state IN ('value','missing','null','invalid','not_applicable')), skill_crafting_modifier ANY CHECK((skill_crafting_modifier_state='value' AND skill_crafting_modifier IS NOT NULL AND typeof(skill_crafting_modifier) IN ('integer','real')) OR (skill_crafting_modifier_state<>'value' AND skill_crafting_modifier IS NULL)), skill_crafting_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_crafting_rank_state IN ('value','missing','null','invalid','not_applicable')), skill_crafting_rank ANY CHECK((skill_crafting_rank_state='value' AND skill_crafting_rank IS NOT NULL AND typeof(skill_crafting_rank) IN ('integer','real')) OR (skill_crafting_rank_state<>'value' AND skill_crafting_rank IS NULL)), skill_deception_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_deception_modifier_state IN ('value','missing','null','invalid','not_applicable')), skill_deception_modifier ANY CHECK((skill_deception_modifier_state='value' AND skill_deception_modifier IS NOT NULL AND typeof(skill_deception_modifier) IN ('integer','real')) OR (skill_deception_modifier_state<>'value' AND skill_deception_modifier IS NULL)), skill_deception_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_deception_rank_state IN ('value','missing','null','invalid','not_applicable')), skill_deception_rank ANY CHECK((skill_deception_rank_state='value' AND skill_deception_rank IS NOT NULL AND typeof(skill_deception_rank) IN ('integer','real')) OR (skill_deception_rank_state<>'value' AND skill_deception_rank IS NULL)), skill_diplomacy_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_diplomacy_modifier_state IN ('value','missing','null','invalid','not_applicable')), skill_diplomacy_modifier ANY CHECK((skill_diplomacy_modifier_state='value' AND skill_diplomacy_modifier IS NOT NULL AND typeof(skill_diplomacy_modifier) IN ('integer','real')) OR (skill_diplomacy_modifier_state<>'value' AND skill_diplomacy_modifier IS NULL)), skill_diplomacy_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_diplomacy_rank_state IN ('value','missing','null','invalid','not_applicable')), skill_diplomacy_rank ANY CHECK((skill_diplomacy_rank_state='value' AND skill_diplomacy_rank IS NOT NULL AND typeof(skill_diplomacy_rank) IN ('integer','real')) OR (skill_diplomacy_rank_state<>'value' AND skill_diplomacy_rank IS NULL)), skill_intimidation_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_intimidation_modifier_state IN ('value','missing','null','invalid','not_applicable')), skill_intimidation_modifier ANY CHECK((skill_intimidation_modifier_state='value' AND skill_intimidation_modifier IS NOT NULL AND typeof(skill_intimidation_modifier) IN ('integer','real')) OR (skill_intimidation_modifier_state<>'value' AND skill_intimidation_modifier IS NULL)), skill_intimidation_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_intimidation_rank_state IN ('value','missing','null','invalid','not_applicable')), skill_intimidation_rank ANY CHECK((skill_intimidation_rank_state='value' AND skill_intimidation_rank IS NOT NULL AND typeof(skill_intimidation_rank) IN ('integer','real')) OR (skill_intimidation_rank_state<>'value' AND skill_intimidation_rank IS NULL)), skill_medicine_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_medicine_modifier_state IN ('value','missing','null','invalid','not_applicable')), skill_medicine_modifier ANY CHECK((skill_medicine_modifier_state='value' AND skill_medicine_modifier IS NOT NULL AND typeof(skill_medicine_modifier) IN ('integer','real')) OR (skill_medicine_modifier_state<>'value' AND skill_medicine_modifier IS NULL)), skill_medicine_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_medicine_rank_state IN ('value','missing','null','invalid','not_applicable')), skill_medicine_rank ANY CHECK((skill_medicine_rank_state='value' AND skill_medicine_rank IS NOT NULL AND typeof(skill_medicine_rank) IN ('integer','real')) OR (skill_medicine_rank_state<>'value' AND skill_medicine_rank IS NULL)), skill_nature_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_nature_modifier_state IN ('value','missing','null','invalid','not_applicable')), skill_nature_modifier ANY CHECK((skill_nature_modifier_state='value' AND skill_nature_modifier IS NOT NULL AND typeof(skill_nature_modifier) IN ('integer','real')) OR (skill_nature_modifier_state<>'value' AND skill_nature_modifier IS NULL)), skill_nature_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_nature_rank_state IN ('value','missing','null','invalid','not_applicable')), skill_nature_rank ANY CHECK((skill_nature_rank_state='value' AND skill_nature_rank IS NOT NULL AND typeof(skill_nature_rank) IN ('integer','real')) OR (skill_nature_rank_state<>'value' AND skill_nature_rank IS NULL)), skill_occultism_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_occultism_modifier_state IN ('value','missing','null','invalid','not_applicable')), skill_occultism_modifier ANY CHECK((skill_occultism_modifier_state='value' AND skill_occultism_modifier IS NOT NULL AND typeof(skill_occultism_modifier) IN ('integer','real')) OR (skill_occultism_modifier_state<>'value' AND skill_occultism_modifier IS NULL)), skill_occultism_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_occultism_rank_state IN ('value','missing','null','invalid','not_applicable')), skill_occultism_rank ANY CHECK((skill_occultism_rank_state='value' AND skill_occultism_rank IS NOT NULL AND typeof(skill_occultism_rank) IN ('integer','real')) OR (skill_occultism_rank_state<>'value' AND skill_occultism_rank IS NULL)), skill_performance_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_performance_modifier_state IN ('value','missing','null','invalid','not_applicable')), skill_performance_modifier ANY CHECK((skill_performance_modifier_state='value' AND skill_performance_modifier IS NOT NULL AND typeof(skill_performance_modifier) IN ('integer','real')) OR (skill_performance_modifier_state<>'value' AND skill_performance_modifier IS NULL)), skill_performance_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_performance_rank_state IN ('value','missing','null','invalid','not_applicable')), skill_performance_rank ANY CHECK((skill_performance_rank_state='value' AND skill_performance_rank IS NOT NULL AND typeof(skill_performance_rank) IN ('integer','real')) OR (skill_performance_rank_state<>'value' AND skill_performance_rank IS NULL)), skill_piloting_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_piloting_modifier_state IN ('value','missing','null','invalid','not_applicable')), skill_piloting_modifier ANY CHECK((skill_piloting_modifier_state='value' AND skill_piloting_modifier IS NOT NULL AND typeof(skill_piloting_modifier) IN ('integer','real')) OR (skill_piloting_modifier_state<>'value' AND skill_piloting_modifier IS NULL)), skill_piloting_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_piloting_rank_state IN ('value','missing','null','invalid','not_applicable')), skill_piloting_rank ANY CHECK((skill_piloting_rank_state='value' AND skill_piloting_rank IS NOT NULL AND typeof(skill_piloting_rank) IN ('integer','real')) OR (skill_piloting_rank_state<>'value' AND skill_piloting_rank IS NULL)), skill_religion_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_religion_modifier_state IN ('value','missing','null','invalid','not_applicable')), skill_religion_modifier ANY CHECK((skill_religion_modifier_state='value' AND skill_religion_modifier IS NOT NULL AND typeof(skill_religion_modifier) IN ('integer','real')) OR (skill_religion_modifier_state<>'value' AND skill_religion_modifier IS NULL)), skill_religion_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_religion_rank_state IN ('value','missing','null','invalid','not_applicable')), skill_religion_rank ANY CHECK((skill_religion_rank_state='value' AND skill_religion_rank IS NOT NULL AND typeof(skill_religion_rank) IN ('integer','real')) OR (skill_religion_rank_state<>'value' AND skill_religion_rank IS NULL)), skill_society_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_society_modifier_state IN ('value','missing','null','invalid','not_applicable')), skill_society_modifier ANY CHECK((skill_society_modifier_state='value' AND skill_society_modifier IS NOT NULL AND typeof(skill_society_modifier) IN ('integer','real')) OR (skill_society_modifier_state<>'value' AND skill_society_modifier IS NULL)), skill_society_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_society_rank_state IN ('value','missing','null','invalid','not_applicable')), skill_society_rank ANY CHECK((skill_society_rank_state='value' AND skill_society_rank IS NOT NULL AND typeof(skill_society_rank) IN ('integer','real')) OR (skill_society_rank_state<>'value' AND skill_society_rank IS NULL)), skill_stealth_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_stealth_modifier_state IN ('value','missing','null','invalid','not_applicable')), skill_stealth_modifier ANY CHECK((skill_stealth_modifier_state='value' AND skill_stealth_modifier IS NOT NULL AND typeof(skill_stealth_modifier) IN ('integer','real')) OR (skill_stealth_modifier_state<>'value' AND skill_stealth_modifier IS NULL)), skill_stealth_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_stealth_rank_state IN ('value','missing','null','invalid','not_applicable')), skill_stealth_rank ANY CHECK((skill_stealth_rank_state='value' AND skill_stealth_rank IS NOT NULL AND typeof(skill_stealth_rank) IN ('integer','real')) OR (skill_stealth_rank_state<>'value' AND skill_stealth_rank IS NULL)), skill_survival_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_survival_modifier_state IN ('value','missing','null','invalid','not_applicable')), skill_survival_modifier ANY CHECK((skill_survival_modifier_state='value' AND skill_survival_modifier IS NOT NULL AND typeof(skill_survival_modifier) IN ('integer','real')) OR (skill_survival_modifier_state<>'value' AND skill_survival_modifier IS NULL)), skill_survival_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_survival_rank_state IN ('value','missing','null','invalid','not_applicable')), skill_survival_rank ANY CHECK((skill_survival_rank_state='value' AND skill_survival_rank IS NOT NULL AND typeof(skill_survival_rank) IN ('integer','real')) OR (skill_survival_rank_state<>'value' AND skill_survival_rank IS NULL)), skill_thievery_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_thievery_modifier_state IN ('value','missing','null','invalid','not_applicable')), skill_thievery_modifier ANY CHECK((skill_thievery_modifier_state='value' AND skill_thievery_modifier IS NOT NULL AND typeof(skill_thievery_modifier) IN ('integer','real')) OR (skill_thievery_modifier_state<>'value' AND skill_thievery_modifier IS NULL)), skill_thievery_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_thievery_rank_state IN ('value','missing','null','invalid','not_applicable')), skill_thievery_rank ANY CHECK((skill_thievery_rank_state='value' AND skill_thievery_rank IS NOT NULL AND typeof(skill_thievery_rank) IN ('integer','real')) OR (skill_thievery_rank_state<>'value' AND skill_thievery_rank IS NULL)),
 CHECK(armor_class_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((armor_class_state='value' AND armor_class IS NOT NULL AND typeof(armor_class) IN ('integer','real')) OR (armor_class_state<>'value' AND armor_class IS NULL)),
 CHECK(hp_maximum_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((hp_maximum_state='value' AND hp_maximum IS NOT NULL AND typeof(hp_maximum) IN ('integer','real')) OR (hp_maximum_state<>'value' AND hp_maximum IS NULL)),
 CHECK(hardness_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((hardness_state='value' AND hardness IS NOT NULL AND typeof(hardness) IN ('integer','real')) OR (hardness_state<>'value' AND hardness IS NULL)),
 CHECK(complexity_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((complexity_state='value' AND complexity IS NOT NULL AND complexity IN (0,1)) OR (complexity_state<>'value' AND complexity IS NULL)),
 CHECK(fortitude_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((fortitude_state='value' AND fortitude IS NOT NULL AND typeof(fortitude) IN ('integer','real')) OR (fortitude_state<>'value' AND fortitude IS NULL)),
 CHECK(reflex_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((reflex_state='value' AND reflex IS NOT NULL AND typeof(reflex) IN ('integer','real')) OR (reflex_state<>'value' AND reflex IS NULL)),
 CHECK(will_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((will_state='value' AND will IS NOT NULL AND typeof(will) IN ('integer','real')) OR (will_state<>'value' AND will IS NULL)),
 CHECK(immunities_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK(weaknesses_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK(resistances_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK(items_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK(perception_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((perception_state='value' AND perception IS NOT NULL AND typeof(perception) IN ('integer','real')) OR (perception_state<>'value' AND perception IS NULL)),
 CHECK(land_speed_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((land_speed_state='value' AND land_speed IS NOT NULL AND typeof(land_speed) IN ('integer','real')) OR (land_speed_state<>'value' AND land_speed IS NULL)),
 CHECK(languages_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK(speeds_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK(senses_state IN ('value','missing','null','invalid','not_applicable'))
) STRICT;

CREATE TABLE actor_senses(
 id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES actor_projection ON DELETE CASCADE,
 original_index INTEGER NOT NULL CHECK(original_index>=0),
 type_state TEXT NOT NULL,
 type TEXT, acuity_state TEXT NOT NULL DEFAULT 'missing' CHECK(acuity_state IN ('value','missing','null','invalid','not_applicable')), acuity TEXT CHECK((acuity_state='value' AND acuity IS NOT NULL) OR (acuity_state<>'value' AND acuity IS NULL)), range_state TEXT NOT NULL DEFAULT 'missing' CHECK(range_state IN ('value','missing','null','invalid','not_applicable')), range ANY CHECK((range_state='value' AND range IS NOT NULL AND typeof(range) IN ('integer','real')) OR (range_state<>'value' AND range IS NULL)),
 CHECK(type_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((type_state='value' AND type IS NOT NULL) OR (type_state<>'value' AND type IS NULL)),
 UNIQUE(record_id,original_index)
) STRICT;

CREATE TABLE actor_speeds(
 id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES actor_projection ON DELETE CASCADE,
 original_index INTEGER NOT NULL CHECK(original_index>=0),
 type_state TEXT NOT NULL,
 type TEXT,
 value_state TEXT NOT NULL,
 value ANY,
 CHECK(type_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((type_state='value' AND type IS NOT NULL) OR (type_state<>'value' AND type IS NULL)),
 CHECK(value_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((value_state='value' AND value IS NOT NULL AND typeof(value) IN ('integer','real')) OR (value_state<>'value' AND value IS NULL)),
 UNIQUE(record_id,original_index)
) STRICT;

CREATE TABLE armor_projection(
 physical_id INTEGER PRIMARY KEY REFERENCES physical_projection ON DELETE CASCADE,
 category_state TEXT NOT NULL,
 category TEXT,
 ac_bonus_state TEXT NOT NULL,
 ac_bonus ANY,
 dex_cap_state TEXT NOT NULL,
 dex_cap ANY, strength_state TEXT NOT NULL DEFAULT 'missing' CHECK(strength_state IN ('value','missing','null','invalid','not_applicable')), strength ANY CHECK((strength_state='value' AND strength IS NOT NULL AND typeof(strength) IN ('integer','real')) OR (strength_state<>'value' AND strength IS NULL)), check_penalty_state TEXT NOT NULL DEFAULT 'missing' CHECK(check_penalty_state IN ('value','missing','null','invalid','not_applicable')), check_penalty ANY CHECK((check_penalty_state='value' AND check_penalty IS NOT NULL AND typeof(check_penalty) IN ('integer','real')) OR (check_penalty_state<>'value' AND check_penalty IS NULL)), speed_penalty_state TEXT NOT NULL DEFAULT 'missing' CHECK(speed_penalty_state IN ('value','missing','null','invalid','not_applicable')), speed_penalty ANY CHECK((speed_penalty_state='value' AND speed_penalty IS NOT NULL AND typeof(speed_penalty) IN ('integer','real')) OR (speed_penalty_state<>'value' AND speed_penalty IS NULL)), armor_group_state TEXT NOT NULL DEFAULT 'missing' CHECK(armor_group_state IN ('value','missing','null','invalid','not_applicable')), armor_group TEXT CHECK((armor_group_state='value' AND armor_group IS NOT NULL) OR (armor_group_state<>'value' AND armor_group IS NULL)),
 CHECK(category_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((category_state='value' AND category IS NOT NULL) OR (category_state<>'value' AND category IS NULL)),
 CHECK(ac_bonus_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((ac_bonus_state='value' AND ac_bonus IS NOT NULL AND typeof(ac_bonus) IN ('integer','real')) OR (ac_bonus_state<>'value' AND ac_bonus IS NULL)),
 CHECK(dex_cap_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((dex_cap_state='value' AND dex_cap IS NOT NULL AND typeof(dex_cap) IN ('integer','real')) OR (dex_cap_state<>'value' AND dex_cap IS NULL))
) STRICT;

CREATE TABLE artifact_context(
 singleton INTEGER PRIMARY KEY CHECK(singleton=1),
 format_version INTEGER NOT NULL, context_hash TEXT NOT NULL,
 context_json TEXT NOT NULL CHECK(json_valid(context_json))
, query_catalog_sha256 BLOB
  CHECK(length(query_catalog_sha256)=32)) STRICT;

CREATE TABLE condition_projection(
 id INTEGER PRIMARY KEY,
 root_record_id INTEGER UNIQUE REFERENCES records ON DELETE CASCADE,
 actor_item_id INTEGER UNIQUE REFERENCES actor_items ON DELETE CASCADE,
 is_valued_state TEXT NOT NULL,
 is_valued INTEGER,
 CHECK(is_valued_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((is_valued_state='value' AND is_valued IS NOT NULL AND is_valued IN (0,1)) OR (is_valued_state<>'value' AND is_valued IS NULL)),
 CHECK((root_record_id IS NOT NULL) <> (actor_item_id IS NOT NULL))
) STRICT;

CREATE TABLE "core_lexical_units"(
 unit_id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES core_roots ON DELETE CASCADE,
 FOREIGN KEY(record_id,unit_id) REFERENCES lexical_units(record_id,unit_id) ON DELETE CASCADE
) STRICT;

CREATE TABLE "core_parts"(
 section_id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES core_roots ON DELETE CASCADE,
 FOREIGN KEY(section_id,record_id) REFERENCES selected_sections(section_id,record_id) ON DELETE CASCADE
) STRICT;

CREATE TABLE core_roots(
 record_id INTEGER PRIMARY KEY REFERENCES records ON DELETE CASCADE,
 family TEXT NOT NULL,
 modelled INTEGER NOT NULL CHECK(modelled IN (0,1)),
 fallback_reason TEXT CHECK(fallback_reason IS NULL OR fallback_reason IN ('unmodelled_kind','no_core_prose')),
 recipe TEXT NOT NULL
) STRICT;

CREATE TABLE "core_semantic_locations"(
 location_id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES core_roots ON DELETE CASCADE,
 FOREIGN KEY(record_id,location_id) REFERENCES semantic_locations(record_id,location_id) ON DELETE CASCADE
) STRICT;

CREATE TABLE deity_domains(deity_id INTEGER NOT NULL REFERENCES deity_projection ON DELETE CASCADE,kind TEXT NOT NULL CHECK(kind IN ('primary','alternate')),value TEXT NOT NULL,PRIMARY KEY(deity_id,kind,value)) WITHOUT ROWID,STRICT;

CREATE TABLE deity_fonts(deity_id INTEGER NOT NULL REFERENCES deity_projection ON DELETE CASCADE,value TEXT NOT NULL,PRIMARY KEY(deity_id,value)) WITHOUT ROWID,STRICT;

CREATE TABLE deity_projection(
 id INTEGER PRIMARY KEY,
 root_record_id INTEGER UNIQUE REFERENCES records ON DELETE CASCADE,
 actor_item_id INTEGER UNIQUE REFERENCES actor_items ON DELETE CASCADE,
 primary_domains_state TEXT NOT NULL,
 alternate_domains_state TEXT NOT NULL,
 fonts_state TEXT NOT NULL,
 CHECK(primary_domains_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK(alternate_domains_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK(fonts_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((root_record_id IS NOT NULL) <> (actor_item_id IS NOT NULL))
) STRICT;

CREATE TABLE developer_diagnostics(
 diagnostic_id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES records ON DELETE CASCADE,
 owners_json TEXT CHECK(owners_json IS NULL OR json_valid(owners_json)),
 field_path TEXT,
 stage TEXT NOT NULL CHECK(stage IN ('admission','preparation','resolution','projection')),
 details_json TEXT NOT NULL CHECK(json_valid(details_json))
) STRICT;

CREATE TABLE effect_projection(
 id INTEGER PRIMARY KEY,
 root_record_id INTEGER UNIQUE REFERENCES records ON DELETE CASCADE,
 actor_item_id INTEGER UNIQUE REFERENCES actor_items ON DELETE CASCADE,
 duration_unit_state TEXT NOT NULL,
 duration_unit TEXT,
 duration_value_state TEXT NOT NULL,
 duration_value ANY,
 CHECK(duration_unit_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((duration_unit_state='value' AND duration_unit IS NOT NULL) OR (duration_unit_state<>'value' AND duration_unit IS NULL)),
 CHECK(duration_value_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((duration_value_state='value' AND duration_value IS NOT NULL AND typeof(duration_value) IN ('integer','real')) OR (duration_value_state<>'value' AND duration_value IS NULL)),
 CHECK((root_record_id IS NOT NULL) <> (actor_item_id IS NOT NULL)),
 CHECK(duration_value_state<>'value' OR (duration_unit_state='value' AND duration_unit IN ('rounds','minutes','hours','days'))),
 CHECK(duration_unit_state<>'value' OR duration_unit NOT IN ('unlimited','encounter') OR duration_value_state='not_applicable')
) STRICT;

CREATE TABLE embedding_inputs(
 input_id INTEGER PRIMARY KEY,
 model_id INTEGER NOT NULL REFERENCES embedding_models,
 input_hash BLOB NOT NULL CHECK(length(input_hash)=32),
 token_count INTEGER NOT NULL CHECK(token_count>0),
 published_hash BLOB CHECK(length(published_hash)=32),
 UNIQUE(model_id,input_hash)
) STRICT;

CREATE TABLE embedding_models(
 model_id INTEGER PRIMARY KEY,
 identity_json TEXT NOT NULL UNIQUE CHECK(json_valid(identity_json))
, published_token_count_digest BLOB) STRICT;

CREATE TABLE heritage_projection(
 id INTEGER PRIMARY KEY,
 root_record_id INTEGER UNIQUE REFERENCES records ON DELETE CASCADE,
 actor_item_id INTEGER UNIQUE REFERENCES actor_items ON DELETE CASCADE,
 ancestry_uuid_state TEXT NOT NULL,
 ancestry_uuid TEXT,
 ancestry_slug_state TEXT NOT NULL,
 ancestry_slug TEXT,
 versatile_state TEXT NOT NULL,
 versatile INTEGER,
 CHECK(ancestry_uuid_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((ancestry_uuid_state='value' AND ancestry_uuid IS NOT NULL) OR (ancestry_uuid_state<>'value' AND ancestry_uuid IS NULL)),
 CHECK(ancestry_slug_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((ancestry_slug_state='value' AND ancestry_slug IS NOT NULL) OR (ancestry_slug_state<>'value' AND ancestry_slug IS NULL)),
 CHECK(versatile_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((versatile_state='value' AND versatile IS NOT NULL AND versatile IN (0,1)) OR (versatile_state<>'value' AND versatile IS NULL)),
 CHECK((root_record_id IS NOT NULL) <> (actor_item_id IS NOT NULL))
) STRICT;

CREATE VIRTUAL TABLE lexical_fts USING fts5(
 identity_terms,alias_terms,structured_terms,definition_terms,
 content='lexical_units',content_rowid='unit_id',
 tokenize='unicode61 remove_diacritics 2'
);

CREATE TABLE "lexical_units"(
 input_hash TEXT NOT NULL UNIQUE,
 unit_id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES records ON DELETE CASCADE,
 section_id INTEGER,
 owners_json TEXT CHECK(owners_json IS NULL OR (json_valid(owners_json) AND json_type(owners_json)='array')),
 unit_kind TEXT NOT NULL CHECK(unit_kind IN ('root_name','root_alias','owned_name','heading','definition_label')),
 identity_terms TEXT NOT NULL,
 alias_terms TEXT NOT NULL,
 structured_terms TEXT NOT NULL,
 definition_terms TEXT NOT NULL,
 FOREIGN KEY(section_id,record_id) REFERENCES selected_sections(section_id,record_id) DEFERRABLE INITIALLY DEFERRED,
 CHECK((unit_kind IN ('heading','definition_label') AND section_id IS NOT NULL AND owners_json IS NULL) OR (unit_kind IN ('root_name','root_alias','owned_name') AND section_id IS NULL AND owners_json IS NOT NULL))
) STRICT;

CREATE TABLE owned_ability_evidence(
    record_key TEXT NOT NULL REFERENCES records(key) ON DELETE CASCADE,
    owners_json TEXT NOT NULL CHECK(json_type(owners_json)='array'),
    help_json TEXT CHECK(help_json IS NULL OR json_type(help_json)='object'),
    has_declaration INTEGER NOT NULL CHECK(has_declaration IN (0,1)),
    action_keys_json TEXT NOT NULL CHECK(json_type(action_keys_json)='array'),
    PRIMARY KEY(record_key,owners_json)
) WITHOUT ROWID, STRICT;

CREATE TABLE packs(pack_id TEXT PRIMARY KEY,label TEXT NOT NULL) STRICT;

CREATE TABLE pending_embedding_values(
 input_id INTEGER PRIMARY KEY REFERENCES embedding_inputs ON DELETE CASCADE,
 vector BLOB NOT NULL,
 vector_hash BLOB NOT NULL CHECK(length(vector_hash)=32),
 cached_at INTEGER NOT NULL DEFAULT(unixepoch())
) STRICT;

CREATE TABLE physical_projection(
 id INTEGER PRIMARY KEY,
 root_record_id INTEGER UNIQUE REFERENCES records ON DELETE CASCADE,
 actor_item_id INTEGER UNIQUE REFERENCES actor_items ON DELETE CASCADE,
 price_per_item_cp_state TEXT NOT NULL,
 price_per_item_cp ANY,
 bulk_state TEXT NOT NULL,
 bulk ANY,
 usage_state TEXT NOT NULL,
 usage TEXT,
 consumable_category_state TEXT NOT NULL,
 consumable_category TEXT, consumable_damage_kind_state TEXT NOT NULL DEFAULT 'missing'
 CHECK(consumable_damage_kind_state IN ('value','missing','null','invalid','not_applicable')), consumable_damage_kind TEXT
 CHECK((consumable_damage_kind_state='value' AND consumable_damage_kind IS NOT NULL) OR (consumable_damage_kind_state<>'value' AND consumable_damage_kind IS NULL)), consumable_damage_type_state TEXT NOT NULL DEFAULT 'missing'
 CHECK(consumable_damage_type_state IN ('value','missing','null','invalid','not_applicable')), consumable_damage_type TEXT
 CHECK((consumable_damage_type_state='value' AND consumable_damage_type IS NOT NULL) OR (consumable_damage_type_state<>'value' AND consumable_damage_type IS NULL)), base_type_state TEXT NOT NULL DEFAULT 'missing' CHECK(base_type_state IN ('value','missing','null','invalid','not_applicable')), base_type TEXT CHECK((base_type_state='value' AND base_type IS NOT NULL) OR (base_type_state<>'value' AND base_type IS NULL)),
 CHECK(price_per_item_cp_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((price_per_item_cp_state='value' AND price_per_item_cp IS NOT NULL AND typeof(price_per_item_cp) IN ('integer','real')) OR (price_per_item_cp_state<>'value' AND price_per_item_cp IS NULL)),
 CHECK(bulk_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((bulk_state='value' AND bulk IS NOT NULL AND typeof(bulk) IN ('integer','real')) OR (bulk_state<>'value' AND bulk IS NULL)),
 CHECK(usage_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((usage_state='value' AND usage IS NOT NULL) OR (usage_state<>'value' AND usage IS NULL)),
 CHECK(consumable_category_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((consumable_category_state='value' AND consumable_category IS NOT NULL) OR (consumable_category_state<>'value' AND consumable_category IS NULL)),
 CHECK((root_record_id IS NOT NULL) <> (actor_item_id IS NOT NULL))
) STRICT;

CREATE TABLE prepared_content(
 published INTEGER NOT NULL CHECK(published IN (0,1)),
 metadata_json TEXT NOT NULL CHECK(json_valid(metadata_json)),
 content_id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES records ON DELETE CASCADE,
 owners_json TEXT NOT NULL CHECK(json_valid(owners_json)),
 field_path TEXT NOT NULL, role TEXT NOT NULL, visibility TEXT NOT NULL,
 authored_markup_sha256 TEXT NOT NULL CHECK(length(authored_markup_sha256)=64),
 preparation_context_hash TEXT NOT NULL CHECK(length(preparation_context_hash)=64),
 outcome TEXT NOT NULL CHECK(outcome IN
   ('prepared','empty','format_unavailable','unsupported_format','preparation_failed')),
 html_gzip BLOB,
 interactions_json TEXT NOT NULL CHECK(json_valid(interactions_json)
   AND json_type(interactions_json)='array'),
 UNIQUE(record_id,owners_json,field_path),
 CHECK((outcome='prepared' AND html_gzip IS NOT NULL) OR
   (outcome<>'prepared' AND html_gzip IS NULL)),
 CHECK(outcome='prepared' OR json_array_length(interactions_json)=0)
) STRICT;

CREATE TABLE projection_allocations(
 record_id INTEGER PRIMARY KEY REFERENCES record_identity(id),
 seed INTEGER NOT NULL CHECK(seed>=0),
 end_id INTEGER NOT NULL CHECK(end_id>=seed)) STRICT;

CREATE TABLE record_bodies(
 record_id INTEGER PRIMARY KEY REFERENCES records ON DELETE CASCADE,
 codec_version INTEGER NOT NULL, encoding TEXT NOT NULL CHECK(encoding='gzip6'),
 snapshot BLOB NOT NULL
) STRICT;

CREATE TABLE record_identity(
 id INTEGER PRIMARY KEY,
 key TEXT NOT NULL UNIQUE,
 last_known_name TEXT,
 availability TEXT NOT NULL CHECK(availability IN ('available','unavailable')),
 UNIQUE(id,key)
) STRICT;

CREATE TABLE record_remaster_context(
 record_id INTEGER NOT NULL REFERENCES records ON DELETE CASCADE,
 ordinal INTEGER NOT NULL,
 payload_json TEXT NOT NULL CHECK(json_valid(payload_json)),
 PRIMARY KEY(record_id,ordinal)
) WITHOUT ROWID,STRICT;

CREATE TABLE records(
 record_id INTEGER PRIMARY KEY,
 key TEXT NOT NULL UNIQUE,
 pack_id TEXT NOT NULL REFERENCES packs,
 document_kind TEXT NOT NULL,
 source_path TEXT NOT NULL,
 content_hash TEXT NOT NULL,
 name_state TEXT NOT NULL,
 name TEXT,
 name_lookup_key TEXT,
 source_type_state TEXT NOT NULL,
 source_type TEXT,
 record_kind_state TEXT NOT NULL,
 record_kind TEXT,
 rarity_state TEXT NOT NULL,
 rarity TEXT,
 publication_title_state TEXT NOT NULL,
 publication_title TEXT,
 publication_remaster_state TEXT NOT NULL,
 publication_remaster INTEGER,
 traits_state TEXT NOT NULL,
 level_state TEXT NOT NULL,
 level ANY,
 size_state TEXT NOT NULL,
 size TEXT,
 FOREIGN KEY(record_id,key) REFERENCES record_identity(id,key),
 CHECK(name_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((name_state='value' AND name IS NOT NULL) OR (name_state<>'value' AND name IS NULL)),
 CHECK((name_state='value' AND name_lookup_key IS NOT NULL)
   OR (name_state<>'value' AND name_lookup_key IS NULL)),
 CHECK(source_type_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((source_type_state='value' AND source_type IS NOT NULL) OR (source_type_state<>'value' AND source_type IS NULL)),
 CHECK(record_kind_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((record_kind_state='value' AND record_kind IS NOT NULL) OR (record_kind_state<>'value' AND record_kind IS NULL)),
 CHECK(rarity_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((rarity_state='value' AND rarity IS NOT NULL) OR (rarity_state<>'value' AND rarity IS NULL)),
 CHECK(publication_title_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((publication_title_state='value' AND publication_title IS NOT NULL) OR (publication_title_state<>'value' AND publication_title IS NULL)),
 CHECK(publication_remaster_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((publication_remaster_state='value' AND publication_remaster IS NOT NULL AND publication_remaster IN (0,1)) OR (publication_remaster_state<>'value' AND publication_remaster IS NULL)),
 CHECK(traits_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK(level_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((level_state='value' AND level IS NOT NULL AND typeof(level) IN ('integer','real')) OR (level_state<>'value' AND level IS NULL)),
 CHECK(size_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((size_state='value' AND size IS NOT NULL) OR (size_state<>'value' AND size IS NULL))
) STRICT;

CREATE TABLE reference_component_digests (
 record_id INTEGER PRIMARY KEY REFERENCES records(record_id) ON DELETE CASCADE,
 recipe TEXT NOT NULL CHECK(recipe='closed-reference-components/v1'),
 digests_json TEXT NOT NULL CHECK(json_valid(digests_json) AND json_type(digests_json)='object'
 AND json_type(digests_json,'$.snapshot')='text' AND length(json_extract(digests_json,'$.snapshot'))=64
 AND json_type(digests_json,'$.query')='text' AND length(json_extract(digests_json,'$.query'))=64
 AND json_type(digests_json,'$.content')='text' AND length(json_extract(digests_json,'$.content'))=64
 AND json_type(digests_json,'$.relationships')='text' AND length(json_extract(digests_json,'$.relationships'))=64
 AND json_type(digests_json,'$.sections')='text' AND length(json_extract(digests_json,'$.sections'))=64
 AND json_type(digests_json,'$.lexical')='text' AND length(json_extract(digests_json,'$.lexical'))=64
 AND json_type(digests_json,'$.trait_associations')='text' AND length(json_extract(digests_json,'$.trait_associations'))=64
 AND json_type(digests_json,'$.core')='text' AND length(json_extract(digests_json,'$.core'))=64)
) STRICT;

CREATE TABLE "reference_entries"(key TEXT PRIMARY KEY, definition_json TEXT NOT NULL CHECK(json_valid(definition_json)),definition_hash TEXT NOT NULL) STRICT;

CREATE TABLE reference_notices(
    id TEXT PRIMARY KEY,
    sha256 TEXT NOT NULL,
    text TEXT NOT NULL,
    source_url TEXT
) STRICT;

CREATE TABLE relationship_occurrences(
 occurrence_id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES records ON DELETE CASCADE,
 owners_json TEXT NOT NULL CHECK(json_valid(owners_json)),
 field_path TEXT NOT NULL, ordinal INTEGER NOT NULL CHECK(ordinal>=0),
 origin TEXT NOT NULL CHECK(origin IN ('structured','content')),
 kind TEXT NOT NULL, authored_target TEXT,
 occurrence_path TEXT, details_json TEXT CHECK(details_json IS NULL OR json_valid(details_json)),
 resolution TEXT NOT NULL CHECK(resolution IN
   ('resolved_record','resolved_owned','resolved_url','unverified_url','unresolved','blocked')),
 target_record_id INTEGER REFERENCES records DEFERRABLE INITIALLY DEFERRED,
 target_owners_json TEXT CHECK(target_owners_json IS NULL OR json_valid(target_owners_json)),
 target_url TEXT,
 CHECK((resolution IN ('resolved_record','resolved_owned') AND target_record_id IS NOT NULL)
   OR (resolution NOT IN ('resolved_record','resolved_owned') AND target_record_id IS NULL)),
 CHECK((resolution='resolved_owned' AND target_owners_json IS NOT NULL)
   OR (resolution<>'resolved_owned' AND target_owners_json IS NULL)),
 CHECK((resolution IN ('resolved_url','unverified_url') AND target_url IS NOT NULL)
   OR (resolution NOT IN ('resolved_url','unverified_url') AND target_url IS NULL))
) STRICT;

CREATE TABLE remaster_pairs(
 legacy_record_id INTEGER PRIMARY KEY REFERENCES records ON DELETE CASCADE,
 remaster_record_id INTEGER NOT NULL REFERENCES records ON DELETE CASCADE,
 evidence_json TEXT NOT NULL CHECK(json_valid(evidence_json)),
 CHECK(legacy_record_id<>remaster_record_id)
) STRICT;

CREATE TABLE requested_reference_config(id INTEGER PRIMARY KEY CHECK(id=1), config_json TEXT NOT NULL CHECK(json_valid(config_json))) STRICT;

CREATE TABLE "search_profiles"(
 profile_id TEXT PRIMARY KEY,
 model_id INTEGER NOT NULL UNIQUE REFERENCES embedding_models
) STRICT;

CREATE TABLE selected_sections(
 section_id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES records ON DELETE CASCADE,
 selector TEXT NOT NULL UNIQUE,
 owners_json TEXT NOT NULL CHECK(json_valid(owners_json) AND json_type(owners_json)='array'),
 field_path TEXT NOT NULL,
 section_ordinal INTEGER NOT NULL CHECK(section_ordinal>=0),
 body_hash TEXT NOT NULL CHECK(length(body_hash)=64),
 address_kind TEXT NOT NULL CHECK(address_kind IN ('html','plain')),
 prepared_html_sha256 TEXT CHECK(prepared_html_sha256 IS NULL OR length(prepared_html_sha256)=64),
 selection_version TEXT NOT NULL,
 section_label TEXT,
 byte_length INTEGER NOT NULL CHECK(byte_length>0),
 role TEXT NOT NULL,
 owner_label TEXT NOT NULL,
 recipe TEXT NOT NULL,
 UNIQUE(record_id,owners_json,field_path,section_ordinal),
 UNIQUE(section_id,record_id),
 CHECK((address_kind='html')=(prepared_html_sha256 IS NOT NULL))
) STRICT;

CREATE TABLE "semantic_locations"(
 location_id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES records DEFERRABLE INITIALLY DEFERRED,
 section_id INTEGER,
 chunk_ordinal INTEGER CHECK(chunk_ordinal IS NULL OR chunk_ordinal>=0),
 chunk_start INTEGER,
 chunk_end INTEGER,
 FOREIGN KEY(section_id,record_id) REFERENCES selected_sections(section_id,record_id) DEFERRABLE INITIALLY DEFERRED,
 UNIQUE(section_id,chunk_ordinal,chunk_start,chunk_end),
 CHECK((section_id IS NOT NULL AND chunk_ordinal IS NOT NULL AND chunk_start IS NOT NULL AND chunk_end IS NOT NULL AND chunk_start>=0 AND chunk_end>chunk_start) OR (section_id IS NULL AND chunk_start IS NULL AND chunk_end IS NULL AND (chunk_ordinal IS NULL OR chunk_ordinal=0)))
) STRICT;

CREATE TABLE "semantic_units"(
 unit_id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES records DEFERRABLE INITIALLY DEFERRED,
 profile_id TEXT NOT NULL REFERENCES search_profiles,
 location_id INTEGER NOT NULL,
 input_id INTEGER NOT NULL REFERENCES embedding_inputs,
 unit_kind TEXT NOT NULL CHECK(unit_kind IN ('identity','passage')),
 UNIQUE(location_id,profile_id),
 FOREIGN KEY(record_id,location_id) REFERENCES semantic_locations(record_id,location_id)
) STRICT;

CREATE TABLE shield_projection(
 physical_id INTEGER PRIMARY KEY REFERENCES physical_projection ON DELETE CASCADE,
 hardness_state TEXT NOT NULL,
 hardness ANY,
 hp_maximum_state TEXT NOT NULL,
 hp_maximum ANY, ac_bonus_state TEXT NOT NULL DEFAULT 'missing' CHECK(ac_bonus_state IN ('value','missing','null','invalid','not_applicable')), ac_bonus ANY CHECK((ac_bonus_state='value' AND ac_bonus IS NOT NULL AND typeof(ac_bonus) IN ('integer','real')) OR (ac_bonus_state<>'value' AND ac_bonus IS NULL)), broken_threshold_state TEXT NOT NULL DEFAULT 'missing' CHECK(broken_threshold_state IN ('value','missing','null','invalid','not_applicable')), broken_threshold ANY CHECK((broken_threshold_state='value' AND broken_threshold IS NOT NULL AND typeof(broken_threshold) IN ('integer','real')) OR (broken_threshold_state<>'value' AND broken_threshold IS NULL)),
 CHECK(hardness_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((hardness_state='value' AND hardness IS NOT NULL AND typeof(hardness) IN ('integer','real')) OR (hardness_state<>'value' AND hardness IS NULL)),
 CHECK(hp_maximum_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((hp_maximum_state='value' AND hp_maximum IS NOT NULL AND typeof(hp_maximum) IN ('integer','real')) OR (hp_maximum_state<>'value' AND hp_maximum IS NULL))
) STRICT;

CREATE TABLE spell_damage_entries(
 id INTEGER PRIMARY KEY,
 spell_id INTEGER NOT NULL REFERENCES spell_projection ON DELETE CASCADE,
 original_index INTEGER NOT NULL CHECK(original_index>=0),
 type_state TEXT NOT NULL,
 type TEXT,
 kinds_state TEXT NOT NULL,
 original_key TEXT NOT NULL,
 CHECK(type_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((type_state='value' AND type IS NOT NULL) OR (type_state<>'value' AND type IS NULL)),
 CHECK(kinds_state IN ('value','missing','null','invalid','not_applicable')),
 UNIQUE(spell_id,original_index),
 UNIQUE(spell_id,original_key)
) STRICT;

CREATE TABLE spell_damage_kinds(entry_id INTEGER NOT NULL REFERENCES spell_damage_entries ON DELETE CASCADE,value TEXT NOT NULL,PRIMARY KEY(entry_id,value)) WITHOUT ROWID,STRICT;

CREATE TABLE spell_projection(
 id INTEGER PRIMARY KEY,
 root_record_id INTEGER UNIQUE REFERENCES records ON DELETE CASCADE,
 actor_item_id INTEGER UNIQUE REFERENCES actor_items ON DELETE CASCADE,
 rank_state TEXT NOT NULL,
 rank ANY,
 traditions_state TEXT NOT NULL,
 focus_state TEXT NOT NULL,
 focus INTEGER,
 ritual_state TEXT NOT NULL,
 ritual INTEGER,
 casting_time_state TEXT NOT NULL,
 casting_time TEXT,
 casting_form_state TEXT NOT NULL,
 casting_form TEXT,
 save_state TEXT NOT NULL,
 save TEXT,
 basic_save_state TEXT NOT NULL,
 basic_save INTEGER,
 passive_defense_state TEXT NOT NULL,
 passive_defense TEXT,
 area_type_state TEXT NOT NULL,
 area_type TEXT,
 area_size_state TEXT NOT NULL,
 area_size ANY,
 duration_text_state TEXT NOT NULL,
 duration_text TEXT,
 sustained_state TEXT NOT NULL,
 sustained INTEGER,
 damage_state TEXT NOT NULL,
 CHECK(rank_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((rank_state='value' AND rank IS NOT NULL AND typeof(rank) IN ('integer','real')) OR (rank_state<>'value' AND rank IS NULL)),
 CHECK(traditions_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK(focus_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((focus_state='value' AND focus IS NOT NULL AND focus IN (0,1)) OR (focus_state<>'value' AND focus IS NULL)),
 CHECK(ritual_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((ritual_state='value' AND ritual IS NOT NULL AND ritual IN (0,1)) OR (ritual_state<>'value' AND ritual IS NULL)),
 CHECK(casting_time_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((casting_time_state='value' AND casting_time IS NOT NULL) OR (casting_time_state<>'value' AND casting_time IS NULL)),
 CHECK(casting_form_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((casting_form_state='value' AND casting_form IS NOT NULL) OR (casting_form_state<>'value' AND casting_form IS NULL)),
 CHECK(save_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((save_state='value' AND save IS NOT NULL) OR (save_state<>'value' AND save IS NULL)),
 CHECK(basic_save_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((basic_save_state='value' AND basic_save IS NOT NULL AND basic_save IN (0,1)) OR (basic_save_state<>'value' AND basic_save IS NULL)),
 CHECK(passive_defense_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((passive_defense_state='value' AND passive_defense IS NOT NULL) OR (passive_defense_state<>'value' AND passive_defense IS NULL)),
 CHECK(area_type_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((area_type_state='value' AND area_type IS NOT NULL) OR (area_type_state<>'value' AND area_type IS NULL)),
 CHECK(area_size_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((area_size_state='value' AND area_size IS NOT NULL AND typeof(area_size) IN ('integer','real')) OR (area_size_state<>'value' AND area_size IS NULL)),
 CHECK(duration_text_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((duration_text_state='value' AND duration_text IS NOT NULL) OR (duration_text_state<>'value' AND duration_text IS NULL)),
 CHECK(sustained_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((sustained_state='value' AND sustained IS NOT NULL AND sustained IN (0,1)) OR (sustained_state<>'value' AND sustained IS NULL)),
 CHECK(damage_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((root_record_id IS NOT NULL) <> (actor_item_id IS NOT NULL))
) STRICT;

CREATE TABLE spell_traditions(spell_id INTEGER NOT NULL REFERENCES spell_projection ON DELETE CASCADE,value TEXT NOT NULL,PRIMARY KEY(spell_id,value)) WITHOUT ROWID,STRICT;

CREATE TABLE trait_members(
    value TEXT NOT NULL,
    record_key TEXT NOT NULL REFERENCES records(key) ON DELETE CASCADE,
    owners_json TEXT NOT NULL CHECK(json_valid(owners_json) AND json_type(owners_json)='array'),
    lookup_member INTEGER NOT NULL CHECK(lookup_member IN (0,1)),
    source_label TEXT,
    CHECK(lookup_member=1 OR source_label IS NULL),
    PRIMARY KEY(value,record_key,owners_json)
) WITHOUT ROWID,STRICT;

CREATE TABLE trait_references(trait_key TEXT NOT NULL REFERENCES "reference_entries"(key) ON DELETE CASCADE,record_key TEXT NOT NULL REFERENCES records(key) DEFERRABLE INITIALLY DEFERRED,evidence TEXT NOT NULL,PRIMARY KEY(trait_key,record_key,evidence)) WITHOUT ROWID,STRICT;

CREATE TABLE verified_aliases(
 record_id INTEGER NOT NULL REFERENCES records ON DELETE CASCADE, alias TEXT NOT NULL,
 alias_lookup_key TEXT NOT NULL,
 evidence_json TEXT NOT NULL CHECK(json_valid(evidence_json)),
 PRIMARY KEY(record_id,alias)
) WITHOUT ROWID,STRICT;

CREATE TABLE weapon_projection(
 physical_id INTEGER PRIMARY KEY REFERENCES physical_projection ON DELETE CASCADE,
 category_state TEXT NOT NULL,
 category TEXT,
 weapon_group_state TEXT NOT NULL,
 weapon_group TEXT,
 damage_type_state TEXT NOT NULL,
 damage_type TEXT,
 range_state TEXT NOT NULL,
 range ANY,
 reload_state TEXT NOT NULL,
 reload TEXT, damage_dice_state TEXT NOT NULL DEFAULT 'missing' CHECK(damage_dice_state IN ('value','missing','null','invalid','not_applicable')), damage_dice ANY CHECK((damage_dice_state='value' AND damage_dice IS NOT NULL AND typeof(damage_dice) IN ('integer','real')) OR (damage_dice_state<>'value' AND damage_dice IS NULL)), damage_die_faces_state TEXT NOT NULL DEFAULT 'missing' CHECK(damage_die_faces_state IN ('value','missing','null','invalid','not_applicable')), damage_die_faces ANY CHECK((damage_die_faces_state='value' AND damage_die_faces IS NOT NULL AND typeof(damage_die_faces) IN ('integer','real')) OR (damage_die_faces_state<>'value' AND damage_die_faces IS NULL)),
 CHECK(category_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((category_state='value' AND category IS NOT NULL) OR (category_state<>'value' AND category IS NULL)),
 CHECK(weapon_group_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((weapon_group_state='value' AND weapon_group IS NOT NULL) OR (weapon_group_state<>'value' AND weapon_group IS NULL)),
 CHECK(damage_type_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((damage_type_state='value' AND damage_type IS NOT NULL) OR (damage_type_state<>'value' AND damage_type IS NULL)),
 CHECK(range_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((range_state='value' AND range IS NOT NULL AND typeof(range) IN ('integer','real')) OR (range_state<>'value' AND range IS NULL)),
 CHECK(reload_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((reload_state='value' AND reload IS NOT NULL) OR (reload_state<>'value' AND reload IS NULL))
) STRICT;

CREATE INDEX actor_ac ON actor_projection(armor_class,record_id) WHERE armor_class_state='value';

CREATE INDEX actor_fortitude ON actor_projection(fortitude,record_id) WHERE fortitude_state='value';

CREATE INDEX actor_hp ON actor_projection(hp_maximum,record_id) WHERE hp_maximum_state='value';

CREATE INDEX actor_items_level ON actor_items(level,record_id,id) WHERE level_state='value';

CREATE INDEX actor_iwr_type ON actor_iwr_entries(kind,type,record_id,entry_id) WHERE type_state='value';

CREATE INDEX actor_iwr_value ON actor_iwr_entries(kind,type,value,record_id,entry_id)
 WHERE type_state='value' AND value_state='value';

CREATE INDEX actor_land_speed ON actor_projection(land_speed) WHERE land_speed_state='value';

CREATE INDEX actor_languages_value ON actor_languages(value,record_id);

CREATE INDEX actor_perception ON actor_projection(perception) WHERE perception_state='value';

CREATE INDEX actor_reflex ON actor_projection(reflex,record_id) WHERE reflex_state='value';

CREATE INDEX actor_senses_type ON actor_senses(type,record_id) WHERE type_state='value';

CREATE INDEX actor_speeds_type_speed ON actor_speeds(type,value,record_id) WHERE type_state='value' AND value_state='value';

CREATE INDEX actor_will ON actor_projection(will,record_id) WHERE will_state='value';

CREATE INDEX core_lexical_units_root ON core_lexical_units(record_id,unit_id);

CREATE INDEX core_parts_root ON core_parts(record_id,section_id);

CREATE INDEX core_semantic_locations_root ON core_semantic_locations(record_id,location_id);

CREATE INDEX deity_domains_value ON deity_domains(kind,value,deity_id);

CREATE INDEX deity_fonts_value ON deity_fonts(value,deity_id);

CREATE INDEX developer_diagnostics_root ON developer_diagnostics(record_id,diagnostic_id);

CREATE UNIQUE INDEX lexical_units_root ON lexical_units(record_id,unit_id);

CREATE INDEX pending_embedding_oldest ON pending_embedding_values(cached_at,input_id);

CREATE INDEX physical_bulk ON physical_projection(bulk) WHERE bulk_state='value';

CREATE INDEX physical_price ON physical_projection(price_per_item_cp) WHERE price_per_item_cp_state='value';

CREATE INDEX records_level ON records(level,record_id) WHERE level_state='value';

CREATE INDEX records_name_lookup ON records(name_lookup_key,record_id)
 WHERE name_state='value';

CREATE INDEX records_variant_lookup ON records(pack_id,source_type,name_lookup_key,record_id);

CREATE INDEX relationship_incoming ON relationship_occurrences(target_record_id,record_id)
 WHERE target_record_id IS NOT NULL;

CREATE INDEX relationship_outgoing ON relationship_occurrences(record_id,origin,kind);

CREATE INDEX selected_sections_role ON selected_sections(role,record_id);

CREATE UNIQUE INDEX semantic_identity_location ON semantic_locations(record_id) WHERE section_id IS NULL AND chunk_ordinal=0;

CREATE UNIQUE INDEX semantic_identity_root ON semantic_units(record_id,profile_id) WHERE unit_kind='identity';

CREATE UNIQUE INDEX semantic_locations_root ON semantic_locations(record_id,location_id);

CREATE INDEX semantic_locations_section ON semantic_locations(section_id,location_id);

CREATE INDEX semantic_units_input ON semantic_units(input_id,unit_id);

CREATE INDEX semantic_units_root ON semantic_units(record_id,profile_id,unit_id);

CREATE INDEX spell_damage_kind ON spell_damage_kinds(value,entry_id);

CREATE INDEX spell_damage_type ON spell_damage_entries(type,spell_id,id) WHERE type_state='value';

CREATE INDEX spell_rank ON spell_projection(rank) WHERE rank_state='value';

CREATE INDEX spell_traditions_value ON spell_traditions(value,spell_id);

CREATE INDEX trait_members_owner ON trait_members(record_key,owners_json,value);

CREATE INDEX trait_members_root_value ON trait_members(value,record_key) WHERE owners_json='[]';

CREATE INDEX trait_references_record_key ON trait_references(record_key,trait_key,evidence);

CREATE TRIGGER lexical_delete AFTER DELETE ON lexical_units BEGIN INSERT INTO lexical_fts(lexical_fts,rowid,identity_terms,alias_terms,structured_terms,definition_terms) VALUES('delete',old.unit_id,old.identity_terms,old.alias_terms,old.structured_terms,old.definition_terms); END;

CREATE TRIGGER lexical_insert AFTER INSERT ON lexical_units BEGIN INSERT INTO lexical_fts(rowid,identity_terms,alias_terms,structured_terms,definition_terms) VALUES(new.unit_id,new.identity_terms,new.alias_terms,new.structured_terms,new.definition_terms); END;

CREATE TRIGGER lexical_update AFTER UPDATE ON lexical_units BEGIN INSERT INTO lexical_fts(lexical_fts,rowid,identity_terms,alias_terms,structured_terms,definition_terms) VALUES('delete',old.unit_id,old.identity_terms,old.alias_terms,old.structured_terms,old.definition_terms); INSERT INTO lexical_fts(rowid,identity_terms,alias_terms,structured_terms,definition_terms) VALUES(new.unit_id,new.identity_terms,new.alias_terms,new.structured_terms,new.definition_terms); END;
