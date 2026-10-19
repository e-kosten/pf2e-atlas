# atlas-runtime

`atlas-runtime` resolves repo/global/custom source, model-cache and workspace paths. It owns explicit setup/fetch policy, composes the reference and saved-state migration ledger, and constructs bounded read/write capabilities over one persistent workspace.

Three read modes are deliberate: full retrieval opens compatible vectors and warms the explicit default search profile; lexical/detail mode needs neither embeddings nor the source checkout; stored-vector mode supports selected-profile Similar without loading a query model. Runtime composes one shared inference owner across independent SQL workers. Secondary profiles load lazily and remain retained; one native mutex serializes loading/inference with four threads, releasing before SQL ranking. Existing bounded retrieval admission remains. Normal product opening never recomputes source fingerprints.

Explicit setup compares the ordered content fingerprint of actual source inputs with the artifact context, including localization and manifest inputs. Repair preserves an existing indexing locale unless overridden; new artifacts default to English. Changing indexed locale requires rebuilding. Setup prepares the requested profiles plus the shared BGE tokenizer dependency. The embedding owner's checksum validator checks every required asset (five BGE files, six Gemma files, including external weights/config/special tokens), without inference or downloads during `--check`. Readiness distinguishes stored vectors, verified assets and current query usability; stored Similar needs no query assets.

`check_index_report` is a cheap executable schema/catalog/capability check returning reference statistics. `validate_index_report` requests full snapshot/projection/content/unit coherence validation. These evidence scopes are distinct. Startup applies supported schema migrations and query-projection backfills. Changes requiring source preparation are handled by setup; incompatible pre-release files have no format bridge. Setup delegates transactional publication to ingest/index and preserves saved state and the previous committed reference on build failure.

Search policy belongs to `atlas-search`, source loading/building to `atlas-ingest`, physical storage and validation to `atlas-index`, and model execution/cache contracts to `atlas-embedding`.

Omitted launch selection stays optional until runtime reads the manifest: one
stored profile selects itself; both select BGE Base. Explicit missing selections
error. Setup without profile flags preserves the readable stored set before asset
work; fresh setup uses Base. The same retained runtime serves readiness and query
workers. Requested source, locale and profile configuration persists separately
from active output; selecting an additional model retains existing profiles.
Readiness lists stored profiles only and retains checksum results behind
cheap required-file metadata witnesses, recovering after normal repair. Native
load always verifies bytes; retained sessions remain usable after disk changes.
Loaded membership comes from the inference owner's finite snapshot without
waiting for the native mutex.
