# Schema 42 release fixture

The compressed workspace was produced by the schema 42 CLI built from
`ad9dc05a3` with `atlas setup --no-embeddings --offline`. Its uncompressed SHA-256
is `cf8894eb2715d0a5b37598199c63a8c30405f6634731695a7010f6ae207e30b2`.
The baseline ledger entry matches the published schema 42 release.

Use a disposable copy of the `core-record-types` ingest fixture. Assign each
document a zero-padded 16-character ID in sorted path order, and add an empty
JSON object at `static/lang/en.json`. Run the old executable with explicit
`--source`, `--index` and `--embedding-cache-path` pointing to that disposable
directory. Rewrite the requested source root to `/fixture/source` to remove the
local path. Checkpoint the WAL, switch to DELETE journaling, vacuum and gzip the
database. Tests add synthetic saved work and vectors when needed.
