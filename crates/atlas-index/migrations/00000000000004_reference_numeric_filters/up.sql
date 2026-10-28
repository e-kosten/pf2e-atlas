ALTER TABLE actor_projection ADD COLUMN stealth_state TEXT NOT NULL DEFAULT 'missing' CHECK(stealth_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN stealth ANY CHECK((stealth_state='value' AND stealth IS NOT NULL AND typeof(stealth) IN ('integer','real')) OR (stealth_state<>'value' AND stealth IS NULL));

ALTER TABLE actor_projection ADD COLUMN ability_str_state TEXT NOT NULL DEFAULT 'missing' CHECK(ability_str_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN ability_str ANY CHECK((ability_str_state='value' AND ability_str IS NOT NULL AND typeof(ability_str) IN ('integer','real')) OR (ability_str_state<>'value' AND ability_str IS NULL));

ALTER TABLE actor_projection ADD COLUMN ability_dex_state TEXT NOT NULL DEFAULT 'missing' CHECK(ability_dex_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN ability_dex ANY CHECK((ability_dex_state='value' AND ability_dex IS NOT NULL AND typeof(ability_dex) IN ('integer','real')) OR (ability_dex_state<>'value' AND ability_dex IS NULL));

ALTER TABLE actor_projection ADD COLUMN ability_con_state TEXT NOT NULL DEFAULT 'missing' CHECK(ability_con_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN ability_con ANY CHECK((ability_con_state='value' AND ability_con IS NOT NULL AND typeof(ability_con) IN ('integer','real')) OR (ability_con_state<>'value' AND ability_con IS NULL));

ALTER TABLE actor_projection ADD COLUMN ability_int_state TEXT NOT NULL DEFAULT 'missing' CHECK(ability_int_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN ability_int ANY CHECK((ability_int_state='value' AND ability_int IS NOT NULL AND typeof(ability_int) IN ('integer','real')) OR (ability_int_state<>'value' AND ability_int IS NULL));

ALTER TABLE actor_projection ADD COLUMN ability_wis_state TEXT NOT NULL DEFAULT 'missing' CHECK(ability_wis_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN ability_wis ANY CHECK((ability_wis_state='value' AND ability_wis IS NOT NULL AND typeof(ability_wis) IN ('integer','real')) OR (ability_wis_state<>'value' AND ability_wis IS NULL));

ALTER TABLE actor_projection ADD COLUMN ability_cha_state TEXT NOT NULL DEFAULT 'missing' CHECK(ability_cha_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN ability_cha ANY CHECK((ability_cha_state='value' AND ability_cha IS NOT NULL AND typeof(ability_cha) IN ('integer','real')) OR (ability_cha_state<>'value' AND ability_cha IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_acrobatics_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_acrobatics_modifier_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_acrobatics_modifier ANY CHECK((skill_acrobatics_modifier_state='value' AND skill_acrobatics_modifier IS NOT NULL AND typeof(skill_acrobatics_modifier) IN ('integer','real')) OR (skill_acrobatics_modifier_state<>'value' AND skill_acrobatics_modifier IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_acrobatics_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_acrobatics_rank_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_acrobatics_rank ANY CHECK((skill_acrobatics_rank_state='value' AND skill_acrobatics_rank IS NOT NULL AND typeof(skill_acrobatics_rank) IN ('integer','real')) OR (skill_acrobatics_rank_state<>'value' AND skill_acrobatics_rank IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_arcana_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_arcana_modifier_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_arcana_modifier ANY CHECK((skill_arcana_modifier_state='value' AND skill_arcana_modifier IS NOT NULL AND typeof(skill_arcana_modifier) IN ('integer','real')) OR (skill_arcana_modifier_state<>'value' AND skill_arcana_modifier IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_arcana_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_arcana_rank_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_arcana_rank ANY CHECK((skill_arcana_rank_state='value' AND skill_arcana_rank IS NOT NULL AND typeof(skill_arcana_rank) IN ('integer','real')) OR (skill_arcana_rank_state<>'value' AND skill_arcana_rank IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_athletics_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_athletics_modifier_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_athletics_modifier ANY CHECK((skill_athletics_modifier_state='value' AND skill_athletics_modifier IS NOT NULL AND typeof(skill_athletics_modifier) IN ('integer','real')) OR (skill_athletics_modifier_state<>'value' AND skill_athletics_modifier IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_athletics_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_athletics_rank_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_athletics_rank ANY CHECK((skill_athletics_rank_state='value' AND skill_athletics_rank IS NOT NULL AND typeof(skill_athletics_rank) IN ('integer','real')) OR (skill_athletics_rank_state<>'value' AND skill_athletics_rank IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_computers_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_computers_modifier_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_computers_modifier ANY CHECK((skill_computers_modifier_state='value' AND skill_computers_modifier IS NOT NULL AND typeof(skill_computers_modifier) IN ('integer','real')) OR (skill_computers_modifier_state<>'value' AND skill_computers_modifier IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_computers_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_computers_rank_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_computers_rank ANY CHECK((skill_computers_rank_state='value' AND skill_computers_rank IS NOT NULL AND typeof(skill_computers_rank) IN ('integer','real')) OR (skill_computers_rank_state<>'value' AND skill_computers_rank IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_crafting_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_crafting_modifier_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_crafting_modifier ANY CHECK((skill_crafting_modifier_state='value' AND skill_crafting_modifier IS NOT NULL AND typeof(skill_crafting_modifier) IN ('integer','real')) OR (skill_crafting_modifier_state<>'value' AND skill_crafting_modifier IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_crafting_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_crafting_rank_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_crafting_rank ANY CHECK((skill_crafting_rank_state='value' AND skill_crafting_rank IS NOT NULL AND typeof(skill_crafting_rank) IN ('integer','real')) OR (skill_crafting_rank_state<>'value' AND skill_crafting_rank IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_deception_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_deception_modifier_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_deception_modifier ANY CHECK((skill_deception_modifier_state='value' AND skill_deception_modifier IS NOT NULL AND typeof(skill_deception_modifier) IN ('integer','real')) OR (skill_deception_modifier_state<>'value' AND skill_deception_modifier IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_deception_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_deception_rank_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_deception_rank ANY CHECK((skill_deception_rank_state='value' AND skill_deception_rank IS NOT NULL AND typeof(skill_deception_rank) IN ('integer','real')) OR (skill_deception_rank_state<>'value' AND skill_deception_rank IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_diplomacy_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_diplomacy_modifier_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_diplomacy_modifier ANY CHECK((skill_diplomacy_modifier_state='value' AND skill_diplomacy_modifier IS NOT NULL AND typeof(skill_diplomacy_modifier) IN ('integer','real')) OR (skill_diplomacy_modifier_state<>'value' AND skill_diplomacy_modifier IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_diplomacy_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_diplomacy_rank_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_diplomacy_rank ANY CHECK((skill_diplomacy_rank_state='value' AND skill_diplomacy_rank IS NOT NULL AND typeof(skill_diplomacy_rank) IN ('integer','real')) OR (skill_diplomacy_rank_state<>'value' AND skill_diplomacy_rank IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_intimidation_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_intimidation_modifier_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_intimidation_modifier ANY CHECK((skill_intimidation_modifier_state='value' AND skill_intimidation_modifier IS NOT NULL AND typeof(skill_intimidation_modifier) IN ('integer','real')) OR (skill_intimidation_modifier_state<>'value' AND skill_intimidation_modifier IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_intimidation_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_intimidation_rank_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_intimidation_rank ANY CHECK((skill_intimidation_rank_state='value' AND skill_intimidation_rank IS NOT NULL AND typeof(skill_intimidation_rank) IN ('integer','real')) OR (skill_intimidation_rank_state<>'value' AND skill_intimidation_rank IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_medicine_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_medicine_modifier_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_medicine_modifier ANY CHECK((skill_medicine_modifier_state='value' AND skill_medicine_modifier IS NOT NULL AND typeof(skill_medicine_modifier) IN ('integer','real')) OR (skill_medicine_modifier_state<>'value' AND skill_medicine_modifier IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_medicine_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_medicine_rank_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_medicine_rank ANY CHECK((skill_medicine_rank_state='value' AND skill_medicine_rank IS NOT NULL AND typeof(skill_medicine_rank) IN ('integer','real')) OR (skill_medicine_rank_state<>'value' AND skill_medicine_rank IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_nature_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_nature_modifier_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_nature_modifier ANY CHECK((skill_nature_modifier_state='value' AND skill_nature_modifier IS NOT NULL AND typeof(skill_nature_modifier) IN ('integer','real')) OR (skill_nature_modifier_state<>'value' AND skill_nature_modifier IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_nature_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_nature_rank_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_nature_rank ANY CHECK((skill_nature_rank_state='value' AND skill_nature_rank IS NOT NULL AND typeof(skill_nature_rank) IN ('integer','real')) OR (skill_nature_rank_state<>'value' AND skill_nature_rank IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_occultism_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_occultism_modifier_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_occultism_modifier ANY CHECK((skill_occultism_modifier_state='value' AND skill_occultism_modifier IS NOT NULL AND typeof(skill_occultism_modifier) IN ('integer','real')) OR (skill_occultism_modifier_state<>'value' AND skill_occultism_modifier IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_occultism_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_occultism_rank_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_occultism_rank ANY CHECK((skill_occultism_rank_state='value' AND skill_occultism_rank IS NOT NULL AND typeof(skill_occultism_rank) IN ('integer','real')) OR (skill_occultism_rank_state<>'value' AND skill_occultism_rank IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_performance_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_performance_modifier_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_performance_modifier ANY CHECK((skill_performance_modifier_state='value' AND skill_performance_modifier IS NOT NULL AND typeof(skill_performance_modifier) IN ('integer','real')) OR (skill_performance_modifier_state<>'value' AND skill_performance_modifier IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_performance_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_performance_rank_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_performance_rank ANY CHECK((skill_performance_rank_state='value' AND skill_performance_rank IS NOT NULL AND typeof(skill_performance_rank) IN ('integer','real')) OR (skill_performance_rank_state<>'value' AND skill_performance_rank IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_piloting_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_piloting_modifier_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_piloting_modifier ANY CHECK((skill_piloting_modifier_state='value' AND skill_piloting_modifier IS NOT NULL AND typeof(skill_piloting_modifier) IN ('integer','real')) OR (skill_piloting_modifier_state<>'value' AND skill_piloting_modifier IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_piloting_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_piloting_rank_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_piloting_rank ANY CHECK((skill_piloting_rank_state='value' AND skill_piloting_rank IS NOT NULL AND typeof(skill_piloting_rank) IN ('integer','real')) OR (skill_piloting_rank_state<>'value' AND skill_piloting_rank IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_religion_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_religion_modifier_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_religion_modifier ANY CHECK((skill_religion_modifier_state='value' AND skill_religion_modifier IS NOT NULL AND typeof(skill_religion_modifier) IN ('integer','real')) OR (skill_religion_modifier_state<>'value' AND skill_religion_modifier IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_religion_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_religion_rank_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_religion_rank ANY CHECK((skill_religion_rank_state='value' AND skill_religion_rank IS NOT NULL AND typeof(skill_religion_rank) IN ('integer','real')) OR (skill_religion_rank_state<>'value' AND skill_religion_rank IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_society_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_society_modifier_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_society_modifier ANY CHECK((skill_society_modifier_state='value' AND skill_society_modifier IS NOT NULL AND typeof(skill_society_modifier) IN ('integer','real')) OR (skill_society_modifier_state<>'value' AND skill_society_modifier IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_society_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_society_rank_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_society_rank ANY CHECK((skill_society_rank_state='value' AND skill_society_rank IS NOT NULL AND typeof(skill_society_rank) IN ('integer','real')) OR (skill_society_rank_state<>'value' AND skill_society_rank IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_stealth_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_stealth_modifier_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_stealth_modifier ANY CHECK((skill_stealth_modifier_state='value' AND skill_stealth_modifier IS NOT NULL AND typeof(skill_stealth_modifier) IN ('integer','real')) OR (skill_stealth_modifier_state<>'value' AND skill_stealth_modifier IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_stealth_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_stealth_rank_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_stealth_rank ANY CHECK((skill_stealth_rank_state='value' AND skill_stealth_rank IS NOT NULL AND typeof(skill_stealth_rank) IN ('integer','real')) OR (skill_stealth_rank_state<>'value' AND skill_stealth_rank IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_survival_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_survival_modifier_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_survival_modifier ANY CHECK((skill_survival_modifier_state='value' AND skill_survival_modifier IS NOT NULL AND typeof(skill_survival_modifier) IN ('integer','real')) OR (skill_survival_modifier_state<>'value' AND skill_survival_modifier IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_survival_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_survival_rank_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_survival_rank ANY CHECK((skill_survival_rank_state='value' AND skill_survival_rank IS NOT NULL AND typeof(skill_survival_rank) IN ('integer','real')) OR (skill_survival_rank_state<>'value' AND skill_survival_rank IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_thievery_modifier_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_thievery_modifier_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_thievery_modifier ANY CHECK((skill_thievery_modifier_state='value' AND skill_thievery_modifier IS NOT NULL AND typeof(skill_thievery_modifier) IN ('integer','real')) OR (skill_thievery_modifier_state<>'value' AND skill_thievery_modifier IS NULL));

ALTER TABLE actor_projection ADD COLUMN skill_thievery_rank_state TEXT NOT NULL DEFAULT 'missing' CHECK(skill_thievery_rank_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE actor_projection ADD COLUMN skill_thievery_rank ANY CHECK((skill_thievery_rank_state='value' AND skill_thievery_rank IS NOT NULL AND typeof(skill_thievery_rank) IN ('integer','real')) OR (skill_thievery_rank_state<>'value' AND skill_thievery_rank IS NULL));

ALTER TABLE weapon_projection ADD COLUMN damage_dice_state TEXT NOT NULL DEFAULT 'missing' CHECK(damage_dice_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE weapon_projection ADD COLUMN damage_dice ANY CHECK((damage_dice_state='value' AND damage_dice IS NOT NULL AND typeof(damage_dice) IN ('integer','real')) OR (damage_dice_state<>'value' AND damage_dice IS NULL));

ALTER TABLE weapon_projection ADD COLUMN damage_die_faces_state TEXT NOT NULL DEFAULT 'missing' CHECK(damage_die_faces_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE weapon_projection ADD COLUMN damage_die_faces ANY CHECK((damage_die_faces_state='value' AND damage_die_faces IS NOT NULL AND typeof(damage_die_faces) IN ('integer','real')) OR (damage_die_faces_state<>'value' AND damage_die_faces IS NULL));

ALTER TABLE armor_projection ADD COLUMN strength_state TEXT NOT NULL DEFAULT 'missing' CHECK(strength_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE armor_projection ADD COLUMN strength ANY CHECK((strength_state='value' AND strength IS NOT NULL AND typeof(strength) IN ('integer','real')) OR (strength_state<>'value' AND strength IS NULL));

ALTER TABLE armor_projection ADD COLUMN check_penalty_state TEXT NOT NULL DEFAULT 'missing' CHECK(check_penalty_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE armor_projection ADD COLUMN check_penalty ANY CHECK((check_penalty_state='value' AND check_penalty IS NOT NULL AND typeof(check_penalty) IN ('integer','real')) OR (check_penalty_state<>'value' AND check_penalty IS NULL));

ALTER TABLE armor_projection ADD COLUMN speed_penalty_state TEXT NOT NULL DEFAULT 'missing' CHECK(speed_penalty_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE armor_projection ADD COLUMN speed_penalty ANY CHECK((speed_penalty_state='value' AND speed_penalty IS NOT NULL AND typeof(speed_penalty) IN ('integer','real')) OR (speed_penalty_state<>'value' AND speed_penalty IS NULL));

ALTER TABLE shield_projection ADD COLUMN ac_bonus_state TEXT NOT NULL DEFAULT 'missing' CHECK(ac_bonus_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE shield_projection ADD COLUMN ac_bonus ANY CHECK((ac_bonus_state='value' AND ac_bonus IS NOT NULL AND typeof(ac_bonus) IN ('integer','real')) OR (ac_bonus_state<>'value' AND ac_bonus IS NULL));

ALTER TABLE shield_projection ADD COLUMN broken_threshold_state TEXT NOT NULL DEFAULT 'missing' CHECK(broken_threshold_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE shield_projection ADD COLUMN broken_threshold ANY CHECK((broken_threshold_state='value' AND broken_threshold IS NOT NULL AND typeof(broken_threshold) IN ('integer','real')) OR (broken_threshold_state<>'value' AND broken_threshold IS NULL));
