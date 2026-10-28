ALTER TABLE physical_projection ADD COLUMN base_type_state TEXT NOT NULL DEFAULT 'missing' CHECK(base_type_state IN ('value','missing','null','invalid','not_applicable'));
ALTER TABLE physical_projection ADD COLUMN base_type TEXT CHECK((base_type_state='value' AND base_type IS NOT NULL) OR (base_type_state<>'value' AND base_type IS NULL));
