# Exact source-leaf coverage v1

`schema.json` defines the machine-readable ledger consumed by
`atlas_ingest::parse_source_leaf_ledger`. A ledger owns one exact source
contract, document class, type discriminator, role, and parent context. Every
entry owns one normalized leaf.

`[]` denotes an exact array-member shape and preserves identity, order, and
multiplicity. `*` is valid only as a complete segment for a true map and must
carry an explicit `map_key_policy`. Recursive `**`, prefixes, implicit
descendants, and declaration-only consumer claims are invalid.

The raw source-path audit is discovery-only and cannot satisfy this contract.
Acceptance requires source-grounded actual-read receipts and parity at every
declared final-owner stage. Creature declarations and fixtures are intentionally
not part of this A1 contract-engine slice.
