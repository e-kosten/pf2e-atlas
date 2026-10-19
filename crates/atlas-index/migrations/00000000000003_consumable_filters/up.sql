ALTER TABLE physical_projection ADD COLUMN consumable_damage_kind_state TEXT NOT NULL DEFAULT 'missing'
 CHECK(consumable_damage_kind_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE physical_projection ADD COLUMN consumable_damage_kind TEXT
 CHECK((consumable_damage_kind_state='value' AND consumable_damage_kind IS NOT NULL) OR (consumable_damage_kind_state<>'value' AND consumable_damage_kind IS NULL));

ALTER TABLE physical_projection ADD COLUMN consumable_damage_type_state TEXT NOT NULL DEFAULT 'missing'
 CHECK(consumable_damage_type_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE physical_projection ADD COLUMN consumable_damage_type TEXT
 CHECK((consumable_damage_type_state='value' AND consumable_damage_type IS NOT NULL) OR (consumable_damage_type_state<>'value' AND consumable_damage_type IS NULL));
