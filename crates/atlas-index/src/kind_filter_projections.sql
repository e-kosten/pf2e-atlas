-- Derived kind-specific filter data. Retained snapshots and prepared checks supply values on refresh.
CREATE TABLE background_projection(
    id INTEGER PRIMARY KEY,
    root_record_id INTEGER NOT NULL UNIQUE REFERENCES records ON DELETE CASCADE,
    restricted_boosts_state TEXT NOT NULL DEFAULT 'missing' CHECK(restricted_boosts_state IN ('value','missing','null','invalid','not_applicable')),
    has_free_boost_state TEXT NOT NULL DEFAULT 'missing' CHECK(has_free_boost_state IN ('value','missing','null','invalid','not_applicable')),
    has_free_boost INTEGER CHECK((has_free_boost_state = 'value' AND has_free_boost IS NOT NULL AND has_free_boost IN (0,1)) OR (has_free_boost_state <> 'value' AND has_free_boost IS NULL)),
    skill_training_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_training_state IN ('value','missing','null','invalid','not_applicable'))
) STRICT;

CREATE TABLE background_restricted_boosts(
    background_id INTEGER NOT NULL REFERENCES background_projection ON DELETE CASCADE,
    value TEXT NOT NULL,
    PRIMARY KEY(background_id, value)
) WITHOUT ROWID, STRICT;
CREATE INDEX background_restricted_boosts_by_value ON background_restricted_boosts(value, background_id);

CREATE TABLE background_skill_training(
    background_id INTEGER NOT NULL REFERENCES background_projection ON DELETE CASCADE,
    value TEXT NOT NULL,
    PRIMARY KEY(background_id, value)
) WITHOUT ROWID, STRICT;
CREATE INDEX background_skill_training_by_value ON background_skill_training(value, background_id);

CREATE TABLE ancestry_projection(
    id INTEGER PRIMARY KEY,
    root_record_id INTEGER NOT NULL UNIQUE REFERENCES records ON DELETE CASCADE,
    vision_state TEXT NOT NULL DEFAULT 'missing' CHECK(vision_state IN ('value','missing','null','invalid','not_applicable')),
    vision TEXT CHECK((vision_state = 'value' AND vision IS NOT NULL) OR (vision_state <> 'value' AND vision IS NULL)),
    land_speed_state TEXT NOT NULL DEFAULT 'missing' CHECK(land_speed_state IN ('value','missing','null','invalid','not_applicable')),
    land_speed ANY CHECK((land_speed_state = 'value' AND land_speed IS NOT NULL) OR (land_speed_state <> 'value' AND land_speed IS NULL)),
    starting_hp_state TEXT NOT NULL DEFAULT 'missing' CHECK(starting_hp_state IN ('value','missing','null','invalid','not_applicable')),
    starting_hp ANY CHECK((starting_hp_state = 'value' AND starting_hp IS NOT NULL) OR (starting_hp_state <> 'value' AND starting_hp IS NULL)),
    default_boosts_state TEXT NOT NULL DEFAULT 'missing' CHECK(default_boosts_state IN ('value','missing','null','invalid','not_applicable')),
    default_flaws_state TEXT NOT NULL DEFAULT 'missing' CHECK(default_flaws_state IN ('value','missing','null','invalid','not_applicable'))
) STRICT;

CREATE TABLE ancestry_default_boosts(
    ancestry_id INTEGER NOT NULL REFERENCES ancestry_projection ON DELETE CASCADE,
    value TEXT NOT NULL,
    PRIMARY KEY(ancestry_id, value)
) WITHOUT ROWID, STRICT;
CREATE INDEX ancestry_default_boosts_by_value ON ancestry_default_boosts(value, ancestry_id);

CREATE TABLE ancestry_default_flaws(
    ancestry_id INTEGER NOT NULL REFERENCES ancestry_projection ON DELETE CASCADE,
    value TEXT NOT NULL,
    PRIMARY KEY(ancestry_id, value)
) WITHOUT ROWID, STRICT;
CREATE INDEX ancestry_default_flaws_by_value ON ancestry_default_flaws(value, ancestry_id);

CREATE TABLE class_projection(
    id INTEGER PRIMARY KEY,
    root_record_id INTEGER NOT NULL UNIQUE REFERENCES records ON DELETE CASCADE,
    key_attributes_state TEXT NOT NULL DEFAULT 'missing' CHECK(key_attributes_state IN ('value','missing','null','invalid','not_applicable')),
    simple_weapons_state TEXT NOT NULL DEFAULT 'missing' CHECK(simple_weapons_state IN ('value','missing','null','invalid','not_applicable')),
    simple_weapons TEXT CHECK((simple_weapons_state = 'value' AND simple_weapons IS NOT NULL AND simple_weapons IN ('untrained','trained','expert','master','legendary')) OR (simple_weapons_state <> 'value' AND simple_weapons IS NULL)),
    martial_weapons_state TEXT NOT NULL DEFAULT 'missing' CHECK(martial_weapons_state IN ('value','missing','null','invalid','not_applicable')),
    martial_weapons TEXT CHECK((martial_weapons_state = 'value' AND martial_weapons IS NOT NULL AND martial_weapons IN ('untrained','trained','expert','master','legendary')) OR (martial_weapons_state <> 'value' AND martial_weapons IS NULL)),
    advanced_weapons_state TEXT NOT NULL DEFAULT 'missing' CHECK(advanced_weapons_state IN ('value','missing','null','invalid','not_applicable')),
    advanced_weapons TEXT CHECK((advanced_weapons_state = 'value' AND advanced_weapons IS NOT NULL AND advanced_weapons IN ('untrained','trained','expert','master','legendary')) OR (advanced_weapons_state <> 'value' AND advanced_weapons IS NULL)),
    unarmed_attacks_state TEXT NOT NULL DEFAULT 'missing' CHECK(unarmed_attacks_state IN ('value','missing','null','invalid','not_applicable')),
    unarmed_attacks TEXT CHECK((unarmed_attacks_state = 'value' AND unarmed_attacks IS NOT NULL AND unarmed_attacks IN ('untrained','trained','expert','master','legendary')) OR (unarmed_attacks_state <> 'value' AND unarmed_attacks IS NULL)),
    unarmored_state TEXT NOT NULL DEFAULT 'missing' CHECK(unarmored_state IN ('value','missing','null','invalid','not_applicable')),
    unarmored TEXT CHECK((unarmored_state = 'value' AND unarmored IS NOT NULL AND unarmored IN ('untrained','trained','expert','master','legendary')) OR (unarmored_state <> 'value' AND unarmored IS NULL)),
    light_armor_state TEXT NOT NULL DEFAULT 'missing' CHECK(light_armor_state IN ('value','missing','null','invalid','not_applicable')),
    light_armor TEXT CHECK((light_armor_state = 'value' AND light_armor IS NOT NULL AND light_armor IN ('untrained','trained','expert','master','legendary')) OR (light_armor_state <> 'value' AND light_armor IS NULL)),
    medium_armor_state TEXT NOT NULL DEFAULT 'missing' CHECK(medium_armor_state IN ('value','missing','null','invalid','not_applicable')),
    medium_armor TEXT CHECK((medium_armor_state = 'value' AND medium_armor IS NOT NULL AND medium_armor IN ('untrained','trained','expert','master','legendary')) OR (medium_armor_state <> 'value' AND medium_armor IS NULL)),
    heavy_armor_state TEXT NOT NULL DEFAULT 'missing' CHECK(heavy_armor_state IN ('value','missing','null','invalid','not_applicable')),
    heavy_armor TEXT CHECK((heavy_armor_state = 'value' AND heavy_armor IS NOT NULL AND heavy_armor IN ('untrained','trained','expert','master','legendary')) OR (heavy_armor_state <> 'value' AND heavy_armor IS NULL))
) STRICT;

CREATE TABLE class_key_attributes(
    class_id INTEGER NOT NULL REFERENCES class_projection ON DELETE CASCADE,
    value TEXT NOT NULL,
    PRIMARY KEY(class_id, value)
) WITHOUT ROWID, STRICT;
CREATE INDEX class_key_attributes_by_value ON class_key_attributes(value, class_id);

ALTER TABLE deity_projection ADD COLUMN category_state TEXT NOT NULL DEFAULT 'missing' CHECK(category_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE deity_projection ADD COLUMN category TEXT CHECK((category_state = 'value' AND category IS NOT NULL) OR (category_state <> 'value' AND category IS NULL));
ALTER TABLE deity_projection ADD COLUMN favored_weapons_state TEXT NOT NULL DEFAULT 'missing' CHECK(favored_weapons_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE deity_projection ADD COLUMN divine_skills_state TEXT NOT NULL DEFAULT 'missing' CHECK(divine_skills_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE deity_projection ADD COLUMN sanctification_requirement_state TEXT NOT NULL DEFAULT 'missing' CHECK(sanctification_requirement_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE deity_projection ADD COLUMN sanctification_requirement TEXT CHECK((sanctification_requirement_state = 'value' AND sanctification_requirement IS NOT NULL) OR (sanctification_requirement_state <> 'value' AND sanctification_requirement IS NULL));
ALTER TABLE deity_projection ADD COLUMN sanctification_options_state TEXT NOT NULL DEFAULT 'missing' CHECK(sanctification_options_state IN ('value','missing','null','invalid','not_applicable'));

CREATE TABLE deity_favored_weapons(
    deity_id INTEGER NOT NULL REFERENCES deity_projection ON DELETE CASCADE,
    value TEXT NOT NULL,
    PRIMARY KEY(deity_id, value)
) WITHOUT ROWID, STRICT;
CREATE INDEX deity_favored_weapons_by_value ON deity_favored_weapons(value, deity_id);

CREATE TABLE deity_divine_skills(
    deity_id INTEGER NOT NULL REFERENCES deity_projection ON DELETE CASCADE,
    value TEXT NOT NULL,
    PRIMARY KEY(deity_id, value)
) WITHOUT ROWID, STRICT;
CREATE INDEX deity_divine_skills_by_value ON deity_divine_skills(value, deity_id);

CREATE TABLE deity_sanctification_options(
    deity_id INTEGER NOT NULL REFERENCES deity_projection ON DELETE CASCADE,
    value TEXT NOT NULL,
    PRIMARY KEY(deity_id, value)
) WITHOUT ROWID, STRICT;
CREATE INDEX deity_sanctification_options_by_value ON deity_sanctification_options(value, deity_id);

ALTER TABLE actor_projection ADD COLUMN disable_skills_state TEXT NOT NULL DEFAULT 'missing' CHECK(disable_skills_state IN ('value','missing','null','invalid','not_applicable'));

CREATE TABLE hazard_disable_skills(
    record_id INTEGER NOT NULL REFERENCES actor_projection(record_id) ON DELETE CASCADE,
    value TEXT NOT NULL,
    PRIMARY KEY(record_id, value)
) WITHOUT ROWID, STRICT;
CREATE INDEX hazard_disable_skills_by_value ON hazard_disable_skills(value, record_id);

ALTER TABLE spell_projection ADD COLUMN cast_range_state TEXT NOT NULL DEFAULT 'missing' CHECK(cast_range_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE spell_projection ADD COLUMN cast_range INTEGER CHECK((cast_range_state = 'value' AND cast_range IS NOT NULL AND cast_range > 0) OR (cast_range_state <> 'value' AND cast_range IS NULL));
ALTER TABLE spell_projection ADD COLUMN touch_range_state TEXT NOT NULL DEFAULT 'missing' CHECK(touch_range_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE spell_projection ADD COLUMN touch_range INTEGER CHECK((touch_range_state = 'value' AND touch_range IS NOT NULL AND touch_range IN (0,1)) OR (touch_range_state <> 'value' AND touch_range IS NULL));
