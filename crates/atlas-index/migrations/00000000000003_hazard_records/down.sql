-- Generated Atlas artifacts are rebuilt, never migrated in place. Artifact contract v6 has no
-- compatibility or backfill path, so downgrade is intentionally unsupported.
SELECT RAISE(ABORT, 'pf2e-atlas-artifact/v6 must be rebuilt; downgrade is unsupported');
