# Reference content inputs

The bundled supplement is a normalized, selected input for explicit setup and
direct reference builds. `manifest.json` binds `payload.json`, `notices.json`,
the pristine registered ORC PDF and the Community Use Policy PDF by checksum.
Normal setup and check mode consume these bundled bytes directly through ingest.

The payload contains 24 Player Core typical-speaker fields and four Player Core
weapon specialization paragraphs from the pinned Pf2eTools source. Wildsong uses
the existing Foundry class-feature record instead. The original complete language
and group datasets, regional lore and legacy variants are not distributed here.
Known display tags were flattened without adding executable rules or inferred
links. Duplicate Crossbow entries were accepted only after exact payload equality.

Use the source-contract package's reference-supplement candidate command for
updates. Review text, mapping, removal, source and notice changes before replacing
the approved files. Changing an extraction or permission recipe is also a content
input update. The source repository's software license does not license Paizo's
underlying prose; the required ORC, CUP and contributor notices are retained
separately. The PDFs preserve the original license/policy documents alongside
source-free notice text.

`source-provenance.json` is a separate, finite reviewed policy for the Foundry
standalone definitions. It contains publication/permission evidence keyed by exact
definition identity, without another copy of the definition bodies. It includes the
current Foundry-declared ORC selection for Metal, Toil and Wood, the wholly OGL
CRB/Dark Archive apocryphal source components, and an unknown original page for
Naga. It does not rename tokens or select a different content locale.

`base-notices.json` supplies the registered license, original publication credits,
bounded Monster Core 2 creator attribution and CUP notice independently of whether
external speaker/group enrichment is enabled. Ingest also captures the selected
Foundry source's actual OGL/ORC notice files. Their presence, absence and exact
bytes participate in reference freshness separately from record provenance.
Missing source notices disable unsupported new standalone prose while preserving
ordinary admitted records and checked mappings to their existing content.

The fixed metadata and selected notices are preparation inputs. Published readers
use the canonical reference store and require neither these source files nor the
original Foundry checkout. Exports/distribution carrying prose must retain the
applicable notices; later app/CLI presentation exposes that stored attribution.

Supplemental entry provenance names the original `data/languages.json` or
`data/groups.json` file and its raw SHA-256. The prepared markup hash identifies
the normalized selected field. The stored `reference-subset-manifest` provenance
notice retains the extraction/mapping recipes and payload checksum alongside the
content for source-free attribution.
