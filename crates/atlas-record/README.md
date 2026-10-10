# atlas-record

`atlas-record` owns checked source records and borrowed views over the generated Foundry DTOs.

`SourceBackedRecord` retains the typed root and checked identity of its embedded documents. Borrowed queries distinguish values, missing fields, explicit nulls, invalid fields, and fields that do not apply. Root and embedded Items use the same views.

The source content interpreter prepares cleaned HTML with compact interaction and reference facts. Its transient parse nodes are private implementation details. Shared selection and canonical text functions provide attributable lexical and semantic input, and reconstruct selected passages from the same prepared content.

Reference markers carry their field-local ordinal and a compact digest binding the checked source locator/input hash to the visible occurrence facts. Shared validation requires exactly one marker per visible reference, including unresolved and blocked references; hidden references do not require markers. This checks coherent derived caches, not authentication against coordinated artifact changes or independent correctness of localized targets.

SQLite schema and artifact validation belong to `atlas-index`. Source loading and embedding execution belong to `atlas-ingest` and `atlas-embedding`. Product presentation and runtime overlays are consumer concerns; the source DTO remains their authority.
