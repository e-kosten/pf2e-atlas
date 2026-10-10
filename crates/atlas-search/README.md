# atlas-search

`AtlasRetrievalService` coordinates source-backed product retrieval through the checked `atlas-index` reader. Methods expose typed requests, `SourceRecordSummary` results, selected source detail, attributed passage witnesses, graph occurrences, and explicit candidate coverage.

All product operations exclude Macro tooling. Browse uses the executable filter catalog and stable name/key ordering. Names and verified aliases resolve strictly, with ambiguity preserved. Source details decode one root snapshot and load selected prepared HTML/control fields in bounded batches; product reads never scan the source checkout.

Lexical search groups the complete eligible posting relation by root. Semantic search filters before KNN, groups by maximum unit similarity, and expands fixed unit windows of 1024, 2048, and 4096 before paging. Hybrid combines one rank per root and lane using RRF with constant 60. Results carry at most three attributed witnesses and report bounded semantic coverage. Similar uses the seed's stored identity vector and excludes the seed before KNN; it requires no query embedder.

Verified remaster pairs suppress a legacy result only when the counterpart belongs to the same complete request candidate set. Exact access and legacy-only matches remain available. Suggested variants use constrained name conventions within one pack and family, with known physical compatibility and explicit ambiguity. They create no aliases, canonical records, or remaster evidence.

Filter discovery uses the same typed predicate/compiler and a separately frozen clause-removed candidate universe. Candidate values are restored before conditional pair preference. SQL, schema ownership, validation, and row loading remain in `atlas-index`; source admission remains in `atlas-ingest` and generated DTO ownership in `atlas-foundry-model`.
