# Schema 41 bridge fixture

The CLI from `v0.1.0-nightly.20261109050726.g787c00b3b26e` (787c00b3b26e5d73a39dc69708f72e95e686c402) created this database
from a synthetic source with an empty actions pack. The
`atlas-cli-only-aarch64-apple-darwin.tar.xz` archive SHA-256 is
`5c0e8fd2a17d965ace0734b5f63b79fff9fc2514d587a44017f39620b5d05734`, as pinned in
[schema-41-predecessor.json](../../../../scripts/validation/schema-41-predecessor.json).

The source contained an empty `packs/actions/` directory, a `static/lang/en.json`
file with `{}`, and this `module.json`:

```json
{"packs":[{"name":"actions","label":"Actions","type":"Item","path":"packs/actions"}]}
```

```sh
atlas index build --no-embeddings --path-mode repo --source "$fixture_source" --output "$fixture_db"
sqlite3 "$fixture_db" 'PRAGMA wal_checkpoint(TRUNCATE);'
gzip -n -c "$fixture_db" > schema-41.sqlite.gz
```

The fixture retains the generated reference metadata and contains zero records,
embedding models, and saved lists, parties, or encounters. Its SQLite
application ID is `1096043603` (`0x41544c53`). The ordered ledger
fingerprint is `063c58315d7841c8dff56ed43ddc6a9465d92e1ff9924e59f250879d9815dd85`.

- SQLite bytes SHA-256 `84adbe9e2c8c46f916b7a5f05945708d992c1d697d10e2c9aa66df38d9a918be`
- Checked-in gzip SHA-256 `6c49745026dee6683befde312a8a671694212c0ac590c5a6e0f50eafbb90451c`

Each test expands the snapshot into its own disposable directory. Workspace IDs
and applied timestamps vary when regenerating a capture from the same source.
