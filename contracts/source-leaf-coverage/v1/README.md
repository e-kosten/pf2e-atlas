# Exact source-leaf coverage v1

`schema.json` defines the machine-readable ledger consumed by
`atlas_ingest::parse_source_leaf_ledger`. A ledger owns one exact source
contract, document class, type discriminator, role, and parent context. Every
entry owns one normalized leaf. Lint authenticates that selector against the
313-entry `contracts/pf2e-type-registry.yaml` contract (SHA-256
`38da5a93e06f32e7c4374c968a02919a3b4f46f8e8a340ab9b92e6cd32f0ca1f`)
and accepts only the pinned PF2e source contract, commit, and signature.

`[]` denotes an exact array-member shape and preserves identity, order, and
multiplicity. `*` is valid only as a complete segment for a true map and must
carry an explicit `map_key_policy`. Recursive `**`, prefixes, implicit
descendants, and declaration-only consumer claims are invalid.

The raw source-path audit is discovery-only and cannot satisfy this contract.
Acceptance requires source-grounded actual-read receipts and parity at every
declared final-owner stage. Fixture record/path/digest identities are declared
in the ledger. Receipts are emitted only by mutation-sensitive typed accessors,
and final-owner observations are emitted by typed stage accessors. Aggregate
receipts must reconcile declared record/occurrence prevalence, collection
identity, authored order, and unit multiplicity. Creature declarations and
fixtures are intentionally not part of this A1 contract-engine slice.
