-- Generated Atlas artifacts are rebuilt, never migrated in place. Artifact contract v9 has no
-- compatibility or backfill path, so downgrade is intentionally unsupported.
SELECT RAISE(ABORT, 'pf2e-atlas-artifact/v9 must be rebuilt; downgrade is unsupported');
