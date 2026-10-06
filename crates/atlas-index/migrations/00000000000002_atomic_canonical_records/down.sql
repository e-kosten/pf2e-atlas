-- Generated Atlas artifacts are rebuilt, never migrated in place. This down migration is
-- intentionally unsupported because contract v2 has no compatibility or backfill path.
SELECT RAISE(ABORT, 'pf2e-atlas-artifact/v2 must be rebuilt; downgrade is unsupported');
