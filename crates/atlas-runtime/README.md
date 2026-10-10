# atlas-runtime

`atlas-runtime` resolves repo/global/custom source, model-cache, artifact, and separate local-state paths. It owns explicit setup/fetch policy and constructs the source-backed retrieval service.

Three read modes are deliberate: full retrieval opens compatible vectors and the pinned query model; lexical/detail mode needs neither embeddings nor the source checkout; stored-vector mode supports Similar without loading a query model. Normal product opening never recomputes source fingerprints.

Explicit setup compares the ordered content fingerprint of actual source inputs with the artifact context, including localization and manifest inputs. Repair preserves an existing indexing locale unless overridden; new artifacts default to English. Changing indexed locale requires rebuilding. Model readiness uses the embedding owner's checksum validator for all four pinned assets, without loading inference or downloading during `--check`.

`check_index_report` is a cheap executable schema/catalog/capability check returning artifact statistics. `validate_index_report` requests full snapshot/projection/content/unit coherence validation. These evidence scopes are distinct. Incompatible older artifacts require rebuilding; there is no migration adapter. Setup delegates atomic publication to ingest/index and preserves the previous artifact on build failure.

Search policy belongs to `atlas-search`, source loading/building to `atlas-ingest`, physical storage and validation to `atlas-index`, and model execution/cache contracts to `atlas-embedding`.
