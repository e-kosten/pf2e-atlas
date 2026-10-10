-- Source-backed reference artifact schema. Changed artifacts require rebuilding.
-- Source bindings and normalization: filter-bindings.md and actor/item-filter-bindings.md.
-- Native vector DDL is instantiated from validated model dimensions; see bottom.
PRAGMA foreign_keys=ON;
CREATE TABLE packs(pack_id TEXT PRIMARY KEY,label TEXT NOT NULL) STRICT;

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

CREATE TABLE record_traits(record_id INTEGER NOT NULL REFERENCES records,value TEXT NOT NULL,PRIMARY KEY(record_id,value)) WITHOUT ROWID,STRICT;

CREATE TABLE actor_projection(
 record_id INTEGER PRIMARY KEY REFERENCES records,
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
 senses_state TEXT NOT NULL,
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



CREATE TABLE actor_items(
 id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES actor_projection,
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

CREATE TABLE actor_item_traits(item_id INTEGER NOT NULL REFERENCES actor_items,value TEXT NOT NULL,PRIMARY KEY(item_id,value)) WITHOUT ROWID,STRICT;


-- IWR is authored entry scope, not a flattened effective damage model.
CREATE TABLE actor_iwr_entries(
 entry_id INTEGER PRIMARY KEY,record_id INTEGER NOT NULL REFERENCES actor_projection,
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

CREATE TABLE actor_languages(record_id INTEGER NOT NULL REFERENCES actor_projection,value TEXT NOT NULL,PRIMARY KEY(record_id,value)) WITHOUT ROWID,STRICT;

CREATE TABLE actor_speeds(
 id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES actor_projection,
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

CREATE TABLE actor_senses(
 id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES actor_projection,
 original_index INTEGER NOT NULL CHECK(original_index>=0),
 type_state TEXT NOT NULL,
 type TEXT,
 CHECK(type_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((type_state='value' AND type IS NOT NULL) OR (type_state<>'value' AND type IS NULL)),
 UNIQUE(record_id,original_index)
) STRICT;

CREATE TABLE spell_projection(
 id INTEGER PRIMARY KEY,
 root_record_id INTEGER UNIQUE REFERENCES records,
 actor_item_id INTEGER UNIQUE REFERENCES actor_items,
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

CREATE TABLE spell_traditions(spell_id INTEGER NOT NULL REFERENCES spell_projection,value TEXT NOT NULL,PRIMARY KEY(spell_id,value)) WITHOUT ROWID,STRICT;

CREATE TABLE spell_damage_entries(
 id INTEGER PRIMARY KEY,
 spell_id INTEGER NOT NULL REFERENCES spell_projection,
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

CREATE TABLE spell_damage_kinds(entry_id INTEGER NOT NULL REFERENCES spell_damage_entries,value TEXT NOT NULL,PRIMARY KEY(entry_id,value)) WITHOUT ROWID,STRICT;

CREATE TABLE physical_projection(
 id INTEGER PRIMARY KEY,
 root_record_id INTEGER UNIQUE REFERENCES records,
 actor_item_id INTEGER UNIQUE REFERENCES actor_items,
 price_per_item_cp_state TEXT NOT NULL,
 price_per_item_cp ANY,
 bulk_state TEXT NOT NULL,
 bulk ANY,
 usage_state TEXT NOT NULL,
 usage TEXT,
 consumable_category_state TEXT NOT NULL,
 consumable_category TEXT,
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

CREATE TABLE weapon_projection(
 physical_id INTEGER PRIMARY KEY REFERENCES physical_projection,
 category_state TEXT NOT NULL,
 category TEXT,
 weapon_group_state TEXT NOT NULL,
 weapon_group TEXT,
 damage_type_state TEXT NOT NULL,
 damage_type TEXT,
 range_state TEXT NOT NULL,
 range ANY,
 reload_state TEXT NOT NULL,
 reload TEXT,
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

CREATE TABLE armor_projection(
 physical_id INTEGER PRIMARY KEY REFERENCES physical_projection,
 category_state TEXT NOT NULL,
 category TEXT,
 ac_bonus_state TEXT NOT NULL,
 ac_bonus ANY,
 dex_cap_state TEXT NOT NULL,
 dex_cap ANY,
 CHECK(category_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((category_state='value' AND category IS NOT NULL) OR (category_state<>'value' AND category IS NULL)),
 CHECK(ac_bonus_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((ac_bonus_state='value' AND ac_bonus IS NOT NULL AND typeof(ac_bonus) IN ('integer','real')) OR (ac_bonus_state<>'value' AND ac_bonus IS NULL)),
 CHECK(dex_cap_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((dex_cap_state='value' AND dex_cap IS NOT NULL AND typeof(dex_cap) IN ('integer','real')) OR (dex_cap_state<>'value' AND dex_cap IS NULL))
) STRICT;

CREATE TABLE shield_projection(
 physical_id INTEGER PRIMARY KEY REFERENCES physical_projection,
 hardness_state TEXT NOT NULL,
 hardness ANY,
 hp_maximum_state TEXT NOT NULL,
 hp_maximum ANY,
 CHECK(hardness_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((hardness_state='value' AND hardness IS NOT NULL AND typeof(hardness) IN ('integer','real')) OR (hardness_state<>'value' AND hardness IS NULL)),
 CHECK(hp_maximum_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((hp_maximum_state='value' AND hp_maximum IS NOT NULL AND typeof(hp_maximum) IN ('integer','real')) OR (hp_maximum_state<>'value' AND hp_maximum IS NULL))
) STRICT;

CREATE TABLE ability_projection(
 id INTEGER PRIMARY KEY,
 root_record_id INTEGER UNIQUE REFERENCES records,
 actor_item_id INTEGER UNIQUE REFERENCES actor_items,
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

CREATE TABLE heritage_projection(
 id INTEGER PRIMARY KEY,
 root_record_id INTEGER UNIQUE REFERENCES records,
 actor_item_id INTEGER UNIQUE REFERENCES actor_items,
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

CREATE TABLE effect_projection(
 id INTEGER PRIMARY KEY,
 root_record_id INTEGER UNIQUE REFERENCES records,
 actor_item_id INTEGER UNIQUE REFERENCES actor_items,
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

CREATE TABLE condition_projection(
 id INTEGER PRIMARY KEY,
 root_record_id INTEGER UNIQUE REFERENCES records,
 actor_item_id INTEGER UNIQUE REFERENCES actor_items,
 is_valued_state TEXT NOT NULL,
 is_valued INTEGER,
 CHECK(is_valued_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((is_valued_state='value' AND is_valued IS NOT NULL AND is_valued IN (0,1)) OR (is_valued_state<>'value' AND is_valued IS NULL)),
 CHECK((root_record_id IS NOT NULL) <> (actor_item_id IS NOT NULL))
) STRICT;

CREATE TABLE deity_projection(
 id INTEGER PRIMARY KEY,
 root_record_id INTEGER UNIQUE REFERENCES records,
 actor_item_id INTEGER UNIQUE REFERENCES actor_items,
 primary_domains_state TEXT NOT NULL,
 alternate_domains_state TEXT NOT NULL,
 fonts_state TEXT NOT NULL,
 CHECK(primary_domains_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK(alternate_domains_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK(fonts_state IN ('value','missing','null','invalid','not_applicable')),
 CHECK((root_record_id IS NOT NULL) <> (actor_item_id IS NOT NULL))
) STRICT;

CREATE TABLE deity_domains(deity_id INTEGER NOT NULL REFERENCES deity_projection,kind TEXT NOT NULL CHECK(kind IN ('primary','alternate')),value TEXT NOT NULL,PRIMARY KEY(deity_id,kind,value)) WITHOUT ROWID,STRICT;

CREATE TABLE deity_fonts(deity_id INTEGER NOT NULL REFERENCES deity_projection,value TEXT NOT NULL,PRIMARY KEY(deity_id,value)) WITHOUT ROWID,STRICT;

CREATE INDEX spell_rank ON spell_projection(rank) WHERE rank_state='value';

CREATE INDEX physical_price ON physical_projection(price_per_item_cp) WHERE price_per_item_cp_state='value';

CREATE INDEX physical_bulk ON physical_projection(bulk) WHERE bulk_state='value';

CREATE INDEX actor_perception ON actor_projection(perception) WHERE perception_state='value';

CREATE INDEX actor_land_speed ON actor_projection(land_speed) WHERE land_speed_state='value';

CREATE INDEX actor_items_level ON actor_items(level,record_id,id) WHERE level_state='value';

CREATE INDEX actor_languages_value ON actor_languages(value,record_id);

CREATE INDEX actor_speeds_type_speed ON actor_speeds(type,value,record_id) WHERE type_state='value' AND value_state='value';

CREATE INDEX actor_senses_type ON actor_senses(type,record_id) WHERE type_state='value';

CREATE INDEX spell_traditions_value ON spell_traditions(value,spell_id);

CREATE INDEX spell_damage_type ON spell_damage_entries(type,spell_id,id) WHERE type_state='value';

CREATE INDEX spell_damage_kind ON spell_damage_kinds(value,entry_id);

CREATE INDEX deity_domains_value ON deity_domains(kind,value,deity_id);

CREATE INDEX deity_fonts_value ON deity_fonts(value,deity_id);

-- Context is one checked typed document, not free-form feature flags.
CREATE TABLE artifact_context(
 singleton INTEGER PRIMARY KEY CHECK(singleton=1),
 format_version INTEGER NOT NULL, context_hash TEXT NOT NULL,
 context_json TEXT NOT NULL CHECK(json_valid(context_json))
) STRICT;

CREATE TABLE record_bodies(
 record_id INTEGER PRIMARY KEY REFERENCES records,
 codec_version INTEGER NOT NULL, encoding TEXT NOT NULL CHECK(encoding='gzip6'),
 snapshot BLOB NOT NULL
) STRICT;

CREATE TABLE prepared_content(
 content_id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES records,
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

CREATE TABLE developer_diagnostics(
 diagnostic_id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES records,
 owners_json TEXT CHECK(owners_json IS NULL OR json_valid(owners_json)),
 field_path TEXT,
 stage TEXT NOT NULL CHECK(stage IN ('admission','preparation','resolution','projection')),
 details_json TEXT NOT NULL CHECK(json_valid(details_json))
) STRICT;

CREATE TABLE relationship_occurrences(
 occurrence_id INTEGER PRIMARY KEY,
 record_id INTEGER NOT NULL REFERENCES records,
 owners_json TEXT NOT NULL CHECK(json_valid(owners_json)),
 field_path TEXT NOT NULL, ordinal INTEGER NOT NULL CHECK(ordinal>=0),
 origin TEXT NOT NULL CHECK(origin IN ('structured','content')),
 kind TEXT NOT NULL, authored_target TEXT,
 occurrence_path TEXT, details_json TEXT CHECK(details_json IS NULL OR json_valid(details_json)),
 resolution TEXT NOT NULL CHECK(resolution IN
   ('resolved_record','resolved_owned','resolved_url','unverified_url','unresolved','blocked')),
 target_record_id INTEGER REFERENCES records,
 target_owners_json TEXT CHECK(target_owners_json IS NULL OR json_valid(target_owners_json)),
 target_url TEXT,
 CHECK((resolution IN ('resolved_record','resolved_owned') AND target_record_id IS NOT NULL)
   OR (resolution NOT IN ('resolved_record','resolved_owned') AND target_record_id IS NULL)),
 CHECK((resolution='resolved_owned' AND target_owners_json IS NOT NULL)
   OR (resolution<>'resolved_owned' AND target_owners_json IS NULL)),
 CHECK((resolution IN ('resolved_url','unverified_url') AND target_url IS NOT NULL)
   OR (resolution NOT IN ('resolved_url','unverified_url') AND target_url IS NULL))
) STRICT;

-- Excluded content occurrences remain developer/source evidence, not graph edges.
CREATE TABLE verified_aliases(
 record_id INTEGER NOT NULL REFERENCES records, alias TEXT NOT NULL,
 alias_lookup_key TEXT NOT NULL,
 evidence_json TEXT NOT NULL CHECK(json_valid(evidence_json)),
 PRIMARY KEY(record_id,alias)
) WITHOUT ROWID,STRICT;

CREATE TABLE remaster_pairs(
 legacy_record_id INTEGER PRIMARY KEY REFERENCES records,
 remaster_record_id INTEGER NOT NULL REFERENCES records,
 evidence_json TEXT NOT NULL CHECK(json_valid(evidence_json)),
 CHECK(legacy_record_id<>remaster_record_id)
) STRICT;

-- Checked against executable Rust bindings; artifact data never supplies SQL.
CREATE TABLE query_field_catalog(
 field TEXT PRIMARY KEY, definition_json TEXT NOT NULL CHECK(json_valid(definition_json))
) WITHOUT ROWID,STRICT;

CREATE TABLE lexical_units(
 unit_id INTEGER PRIMARY KEY, record_id INTEGER NOT NULL REFERENCES records,
 owners_json TEXT NOT NULL CHECK(json_valid(owners_json)),
 field_path TEXT, section_json TEXT CHECK(section_json IS NULL OR json_valid(section_json)),
 unit_kind TEXT NOT NULL CHECK(unit_kind IN ('root_name','owned_name','heading','definition_label')),
 identity_terms TEXT NOT NULL, alias_terms TEXT NOT NULL,
 structured_terms TEXT NOT NULL, definition_terms TEXT NOT NULL
) STRICT;

CREATE VIRTUAL TABLE lexical_fts USING fts5(
 identity_terms,alias_terms,structured_terms,definition_terms,
 content='lexical_units',content_rowid='unit_id',
 tokenize='unicode61 remove_diacritics 2'
);

CREATE TABLE semantic_models(
 model_id INTEGER PRIMARY KEY, identity_json TEXT NOT NULL CHECK(json_valid(identity_json)),
 dimensions INTEGER NOT NULL CHECK(dimensions>0),
 unit_policy_version INTEGER NOT NULL
) STRICT;

CREATE TABLE semantic_units(
 unit_id INTEGER PRIMARY KEY, record_id INTEGER NOT NULL REFERENCES records,
 model_id INTEGER NOT NULL REFERENCES semantic_models,
 owners_json TEXT NOT NULL CHECK(json_valid(owners_json)),
 unit_kind TEXT NOT NULL CHECK(unit_kind IN ('identity','passage')),
 field_path TEXT,
 section_json TEXT NOT NULL CHECK(json_valid(section_json)),
 chunk_ordinal INTEGER NOT NULL CHECK(chunk_ordinal>=0),
 input_token_count INTEGER NOT NULL CHECK(input_token_count>0), input_hash TEXT NOT NULL,
 UNIQUE(record_id,model_id,owners_json,field_path,section_json,chunk_ordinal),
 CHECK((unit_kind='passage' AND field_path IS NOT NULL)
   OR (unit_kind='identity' AND field_path IS NULL))
) STRICT;

CREATE INDEX record_traits_value ON record_traits(value,record_id);
CREATE INDEX records_name_lookup ON records(name_lookup_key,record_id)
 WHERE name_state='value';
CREATE INDEX aliases_name_lookup ON verified_aliases(alias_lookup_key,record_id);
CREATE INDEX records_variant_lookup ON records(pack_id,source_type,name_lookup_key,record_id);
CREATE INDEX records_level ON records(level,record_id) WHERE level_state='value';
CREATE INDEX actor_ac ON actor_projection(armor_class,record_id) WHERE armor_class_state='value';
CREATE INDEX actor_hp ON actor_projection(hp_maximum,record_id) WHERE hp_maximum_state='value';
CREATE INDEX actor_fortitude ON actor_projection(fortitude,record_id) WHERE fortitude_state='value';
CREATE INDEX actor_reflex ON actor_projection(reflex,record_id) WHERE reflex_state='value';
CREATE INDEX actor_will ON actor_projection(will,record_id) WHERE will_state='value';
CREATE INDEX actor_iwr_type ON actor_iwr_entries(kind,type,record_id,entry_id) WHERE type_state='value';
CREATE INDEX actor_iwr_value ON actor_iwr_entries(kind,type,value,record_id,entry_id)
 WHERE type_state='value' AND value_state='value';
CREATE INDEX actor_item_traits_value ON actor_item_traits(value,item_id);
CREATE INDEX relationship_outgoing ON relationship_occurrences(record_id,origin,kind);
CREATE INDEX relationship_incoming ON relationship_occurrences(target_record_id,record_id)
 WHERE target_record_id IS NOT NULL;
CREATE INDEX lexical_units_root ON lexical_units(record_id,unit_id);
CREATE INDEX semantic_units_root ON semantic_units(record_id,model_id,unit_id);
CREATE INDEX semantic_units_input_hash ON semantic_units(input_hash,unit_id);
CREATE UNIQUE INDEX semantic_identity_root ON semantic_units(record_id,model_id)
 WHERE unit_kind='identity';
CREATE INDEX developer_diagnostics_root ON developer_diagnostics(record_id,diagnostic_id);

-- Production registers sqlite-vec and creates one model-bound table with validated
-- dimension substitution, e.g. CREATE VIRTUAL TABLE semantic_vectors USING
-- vec0(unit_id INTEGER PRIMARY KEY, embedding FLOAT[384] distance_metric=cosine);
-- Exactly one model is active per initial artifact. Writer/final validation checks
-- unit-ID coverage, dimensions, finiteness and normalization. Eligibility constrains
-- vector rowids before top-k; vectors repeat no filter metadata or content body.
-- Composite row/collection/catalog invariants additionally require typed writer
-- and whole-artifact validation; SQL CHECKs alone do not establish DTO coherence.
