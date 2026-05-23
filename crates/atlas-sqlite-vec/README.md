# atlas-sqlite-vec

`atlas-sqlite-vec` owns the unsafe sqlite-vec extension boundary.

This crate isolates sqlite-vec registration and capability probing so the rest of the runtime does not need to carry extension-loading details or unsafe FFI concerns.

## Owns

- sqlite-vec auto-extension registration.
- Low-level capability errors for extension loading.
- The narrow unsafe boundary required to make sqlite-vec available to SQLite connections.

## Should Not Own

- Domain, search, or ranking logic.
- Artifact metadata interpretation.
- Vector query SQL.
- Embedding model behavior.
- CLI presentation.

## Boundary Notes

`atlas-index` should call this crate when vector table operations require sqlite-vec capability. Keep all higher-level vector semantics in the crates that own artifact writing, index querying, or retrieval orchestration.
