# Atlas CLI

`atlas` is the product CLI for local PF2e reference data. Start with `atlas setup`
or `atlas setup --no-embeddings`; inspect readiness using `atlas setup --check`.

```sh
atlas search 'Ghoul Fever' --retrieval fts --json
atlas search --kind creature --trait undead --where 'actor.hp.maximum >= 80'
atlas filters fields --json
atlas filters values --field traits --kind spell --json
atlas record resolve 'Treat Wounds' --json
atlas record get actionspf2e:1kGNdIIhuglAjIp9
atlas graph uses 'Frightened' --json
atlas similar 'Dirge of Doom' --kind spell --json
atlas lists create research --name Research
atlas web
```

Filters use the maintained CEL grammar's supported catalog subset. Common flags
compile to the same shared predicate. `filters fields` exposes types, operators,
applicability, units and closed choices; `filters values` exposes samples and
availability counts. Unsupported syntax fails before execution.

Search returns root summaries and up to three source-attributed witnesses.
Semantic counts describe bounded candidate windows. Names resolve strictly with
verified alias evidence. Embedded definitions retain their parent keys and checked
owner/field/passage addresses. `record get --owners JSON --field PATH --passage
JSON` focuses a returned witness. Terminal prose is formatted from sanitized HTML
at read time with html2text; no terminal document is stored in SQLite.

Use `index build --locale LOCALE` to set the preparation/search locale (English by
default), or `setup --locale LOCALE` to change it through setup. Setup preserves an
existing artifact locale when no override is given. Changing locale requires a
rebuild. `index check` is a readiness check; `index validate` checks all typed
snapshots and derived caches/projections. Both accept `--no-embeddings` for lexical
artifacts. Old artifacts require rebuilding.

`atlas-dev` is a separate unpublished developer CLI for typed source loading,
analysis, raw path/value discovery and checked artifact/source inspection. It has
no web UI/Node build dependency and does not invoke TypeScript tooling. Private
TypeScript source-contract generation lives under `dev-tools/source-contracts`.
