# Exact source-leaf coverage v1

`schema.json` defines the machine-readable ledger consumed by
`atlas_ingest::parse_source_leaf_ledger`. A ledger owns one exact source
contract, document class, type discriminator, role, and parent context. Every
entry owns one normalized leaf. Lint authenticates that selector against the
313-entry `contracts/pf2e-type-registry.yaml` contract (SHA-256
`38da5a93e06f32e7c4374c968a02919a3b4f46f8e8a340ab9b92e6cd32f0ca1f`)
and accepts only the pinned PF2e source contract, commit, and signature.

`leaf_kind` distinguishes scalar, ordered array-member, and identity-retaining
map-member leaves. Object and collection containers are prefixes rather than
leaves. `[]` preserves member identity, order, and multiplicity. `*` is valid
only as a complete segment for a true map and must carry an explicit
`map_key_policy`. Recursive `**`, prefixes, implicit descendants, and
declaration-only consumer claims are invalid.

`prevalence.yaml` is the immutable full-pin leaf inventory bound by schema,
digest, source contract, commit, signature, and registry digest. Its
`record_count` must equal `corpus.count` on the exact authenticated registry
tuple; its `occurrence_count` is accepted only for the exact inventory entry
and leaf identity. Ledger-authored numeric counts cannot replace either source.

The raw source-path audit is discovery-only and cannot satisfy this contract.
Acceptance requires source-grounded actual-read receipts and parity at every
declared final-owner stage. Fixture record/path/digest identities are declared
in the ledger. The sealed accessor registry loads the source blob and
`static/system.json` from the pinned Git tree, derives document class from the
owning pack, discriminator from the raw record, and top-level/root context from
pack membership, then requires that raw selector to equal the ledger selector.
It also resolves the record key, runs the production parser and real selected
owner functions, and derives its mutation proof internally. There is
no caller-implemented accessor, receipt builder, caller byte payload, or
success flag. `source_prevalence` is full-pin metadata bound to the source and
registry identities; `fixture_prevalence` independently governs the focused
receipt set. Focused receipts reconcile fixture identity, collection identity,
authored order, and unit multiplicity without claiming to enumerate the corpus.
Creature coverage declarations and A2 fixtures are intentionally not part of
this A1 contract-engine slice.
