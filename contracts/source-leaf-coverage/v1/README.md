# Exact source-leaf coverage v1

`schema.json` defines the machine-readable ledger consumed by
`atlas_ingest::parse_source_leaf_ledger`. A ledger owns one exact source
contract, document class, type discriminator, role, and parent context. Every
entry owns one normalized leaf. Lint authenticates that selector against the
313-entry `contracts/pf2e-type-registry.yaml` contract (SHA-256
`e62b1d6797a72dd97297b5af05170477871712623fdb89497e6ab2100331d618`)
and accepts only the pinned PF2e source contract, commit, and signature.

`leaf_kind` distinguishes scalar, ordered array-member, identity-retaining
map-member, and nested map-plus-array member leaves. Object and collection
containers are prefixes rather than leaves. `[]` preserves member identity,
order, and multiplicity. `*` is valid only as a complete segment for a true map
and must carry an explicit `map_key_policy`. Recursive `**`, prefixes, implicit
descendants, and declaration-only consumer claims are invalid.

`prevalence.yaml` is the immutable full-pin leaf inventory bound by schema,
digest, source contract, commit, signature, and registry digest. Its
`record_count` must equal `corpus.count` on the exact authenticated registry
tuple; its `occurrence_count` is accepted only for the exact inventory entry
and leaf identity. Sparse and conditional leaves may have fewer occurrences
than records in the tuple. Ledger-authored numeric counts cannot replace either
source.

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
`actor-npc.yaml` is the A2 adversarial pilot. It binds the six exact authored
ability modifier leaves and the true skills map's exact `base` members to
pinned source excerpts. Its promoted declarations intentionally fail on the
current parser/canonical/persistence/public-owner mismatches; B1 owns changing
those failures to passing product values after independent A1+A2 review.

The three hazard ledgers reconcile the exact populated root/common/rule leaf
inventory plus the embedded action and melee schema boundaries at H1-B/C. They
distinguish promoted gameplay/content data, exact source-editor
provenance, typed-unsupported drift, and declared zero-occurrence contexts.
Their sealed grouped receipts execute the source DTO, canonical, and
post-projection owners against byte-identical pinned records. Each receipt
selects the exact typed field, collection member, unsupported fact, or
provenance fact named by its normalized path and binds that accessor's changed
value to the internal mutation; a changed whole record or retained raw source
blob is not accepted as owner evidence. Provenance receipts additionally prove
that the same mutation leaves typed presentation and mechanic projections
unchanged. The four ledger identity sets must exactly partition every hazard
entry in the authenticated prevalence inventory, so sentinel declarations or
counts cannot stand in for an undispositioned path. Optional-leaf
occurrence counts may be below the registry tuple count; the tuple count still
authenticates the complete owning record family. Later H1 slices extend these
already-reconciled ingest leaves to artifact and public owners; they do not
defer discovery of source-to-canonical disposition until final integration.

The four H5 consumable ledgers own the standalone, NPC, Character, and Hazard
contexts independently. Their accepted prevalence covers every observed leaf
outside `system.spell`, whose complete child snapshot remains owned by the H2
ledger. The H5 context inventories preserve intrinsic definition fields,
read-only source state, provenance, rich content, and exact unsupported rule
objects without using an actor occurrence as a fallback definition. Character
occurrences reach canonical persistence and hydration but deliberately have no
public presentation until H9. The pinned family census authenticates each
context's exact normalized path/count/shape digest and checks representative
rare typed values. Focused mutation receipts and SQLite round trips must then
establish the independent source, canonical, post-projection, and hydration
observers.
`item-spell.yaml` and `consumable-spell-child.yaml` extend the same contract for
the H2 source boundary. They bind every accepted normalized top-level spell and
consumable-child leaf or exact keyed/conditional pattern to the ordered spell
DTO, canonical definition/child, and post-projection owners. Ledger lint also
rejects an accepted prevalence entry that has no declaration in its exact
owning ledger.
Root and child images are codec-retained typed provenance with no fetch,
embedding, display, or ranking behavior. Spell licenses reuse the validated
`PublicationLicense` canonical type but remain outside mechanics and ordinary
FTS or presentation. Local consumable-child rarity reuses the validated
`Rarity` type, and child publication, rarity, and slug stay typed and independent
of target resolution; slug is nonsemantic provenance and never identity. Only
the single-child Foundry editor sort is bound to the
containing record's value-bearing raw provenance. The consumable
selector is a child `Item|spell` under the exact `Item|consumable` relationship
`Consumable.system.spell`; its fixture record key therefore identifies the
authenticated parent while the child `_id` remains an independently receipted
local identity. Artifact and public-surface owners remain deferred to their
later H2 slices rather than being claimed by the ingest receipt.
