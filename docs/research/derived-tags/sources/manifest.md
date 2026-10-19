# Historical snapshot manifest

Each entry binds original bytes to an inert Markdown snapshot. Paths are relative
to the repository root for sources and this research directory for archives.

```json
{
  "captured": "2026-10-19",
  "purpose": "Preserve unique authored documentation and vocabulary, not an approved corpus or runtime input",
  "extraction": "Read the single fenced payload; remove an added final newline only if source_final_newline is false. Check source_bytes and source_sha256.",
  "entries": [
    {
      "source": "scratch/derived-tag-ontology/derived-tag-concept-mapping-summary.md",
      "archive": "sources/node-concept-model/derived-tag-concept-mapping-summary.md",
      "source_sha256": "421c08815271df1bf42179212b871d1b6a809ee75eb426c7dde06f7581032a80",
      "source_bytes": 3970,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "local-only concept planning; no identical reachable Git blob found in audit"
    },
    {
      "source": "scratch/derived-tag-ontology/derived-tag-concept-mapping.csv",
      "archive": "sources/node-concept-model/derived-tag-concept-mapping.csv.md",
      "source_sha256": "a16aa6de10cbdcf24e7cfb1ea5c40aadbf15e4085c53fc5e319bf84a671622d9",
      "source_bytes": 120809,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "local-only concept planning; no identical reachable Git blob found in audit"
    },
    {
      "source": "scratch/derived-tag-ontology/derived-tag-concept-model.md",
      "archive": "sources/node-concept-model/derived-tag-concept-model.md",
      "source_sha256": "903f644f64815148436b643efae74c74122b1292292182c99afc954a3c969a5d",
      "source_bytes": 19071,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "local-only concept planning; no identical reachable Git blob found in audit"
    },
    {
      "source": "scratch/derived-tag-ontology/derived-tag-ontology-glossary.md",
      "archive": "sources/node-concept-model/derived-tag-ontology-glossary.md",
      "source_sha256": "d99f63918db4db1a98f2355bfd8bed54f3d4ac47807fc36e51966ae8713980ec",
      "source_bytes": 8277,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "local-only concept planning; no identical reachable Git blob found in audit"
    },
    {
      "source": "scratch/derived-tag-ontology/derived-tag-ontology-shape-primer.md",
      "archive": "sources/node-concept-model/derived-tag-ontology-shape-primer.md",
      "source_sha256": "bd9fe2588538d62b0cc9721a9b8d87e0db9c6c2849b1d242a6007146bebfe0f1",
      "source_bytes": 9286,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "local-only concept planning; no identical reachable Git blob found in audit"
    },
    {
      "source": "scratch/plans/2026-10-09-concept-labeling-and-ontology-design.md",
      "archive": "sources/2026-10-09-design-discussion.md",
      "source_sha256": "af722488d1ce1d40c326fc2a45cf38606f4483beea4e0e016745bde1d555b900",
      "source_bytes": 30584,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "local discussion plan, deferred; captured current 2026-10-13 revision"
    },
    {
      "source": ".worktrees/tag-corpus-loader/data/tags/catalog/character-build.yaml",
      "archive": "sources/rust-seed/catalog/character-build.yaml.md",
      "source_sha256": "52e43d92ba5a67789a16c8d0fa1a73af45f76e36a1e10f93905ee7eae439e196",
      "source_bytes": 10750,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working corpus at feat/tag-corpus-seed aa04bf804039f7d46411a3cf2296ebc36aa22d0c plus uncommitted edits"
    },
    {
      "source": ".worktrees/tag-corpus-loader/data/tags/catalog/combat-effect.yaml",
      "archive": "sources/rust-seed/catalog/combat-effect.yaml.md",
      "source_sha256": "e658594a9345d0ea2c08862f2504ebaf1bfd391945b1874b61740ba0daf3e44c",
      "source_bytes": 6914,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working corpus at feat/tag-corpus-seed aa04bf804039f7d46411a3cf2296ebc36aa22d0c plus uncommitted edits"
    },
    {
      "source": ".worktrees/tag-corpus-loader/data/tags/catalog/encounter-role.yaml",
      "archive": "sources/rust-seed/catalog/encounter-role.yaml.md",
      "source_sha256": "a3acff6afb2d36be6fb73d6ca967cbbff17f820fe3589b91c65bcdc37b6fa733",
      "source_bytes": 4730,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working corpus at feat/tag-corpus-seed aa04bf804039f7d46411a3cf2296ebc36aa22d0c plus uncommitted edits"
    },
    {
      "source": ".worktrees/tag-corpus-loader/data/tags/catalog/exploration.yaml",
      "archive": "sources/rust-seed/catalog/exploration.yaml.md",
      "source_sha256": "4d52455ab84dd778099f98097549c5f7166ac288e8128c0714e5224ac1605133",
      "source_bytes": 7526,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working corpus at feat/tag-corpus-seed aa04bf804039f7d46411a3cf2296ebc36aa22d0c plus uncommitted edits"
    },
    {
      "source": ".worktrees/tag-corpus-loader/data/tags/catalog/problem-response/access.yaml",
      "archive": "sources/rust-seed/catalog/problem-response/access.yaml.md",
      "source_sha256": "d3d4e539ba05cd4ca5626a871bebf36a6141106a1c0d7e7501e173496a01abfc",
      "source_bytes": 4292,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working corpus at feat/tag-corpus-seed aa04bf804039f7d46411a3cf2296ebc36aa22d0c plus uncommitted edits"
    },
    {
      "source": ".worktrees/tag-corpus-loader/data/tags/catalog/problem-response/communication.yaml",
      "archive": "sources/rust-seed/catalog/problem-response/communication.yaml.md",
      "source_sha256": "aebe3aa1314aa97fd79ad7fb370e69aa29fbecfbd4140e6b8697dd50e48cbcc5",
      "source_bytes": 2818,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working corpus at feat/tag-corpus-seed aa04bf804039f7d46411a3cf2296ebc36aa22d0c plus uncommitted edits"
    },
    {
      "source": ".worktrees/tag-corpus-loader/data/tags/catalog/problem-response/countermeasure.yaml",
      "archive": "sources/rust-seed/catalog/problem-response/countermeasure.yaml.md",
      "source_sha256": "effba43034c1a538ea08f01d87b209294bb85a80c89817c67a3c4f6ecca09974",
      "source_bytes": 9221,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working corpus at feat/tag-corpus-seed aa04bf804039f7d46411a3cf2296ebc36aa22d0c plus uncommitted edits"
    },
    {
      "source": ".worktrees/tag-corpus-loader/data/tags/catalog/problem-response/investigation.yaml",
      "archive": "sources/rust-seed/catalog/problem-response/investigation.yaml.md",
      "source_sha256": "db81842ce0a97a1233d04bb67b3d2e7c6210700fa1a48bd00f7bce0ebe2c0d45",
      "source_bytes": 3479,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working corpus at feat/tag-corpus-seed aa04bf804039f7d46411a3cf2296ebc36aa22d0c plus uncommitted edits"
    },
    {
      "source": ".worktrees/tag-corpus-loader/data/tags/catalog/problem-response/restoration.yaml",
      "archive": "sources/rust-seed/catalog/problem-response/restoration.yaml.md",
      "source_sha256": "27aed22f2832a4e8947ffabdcce258ea81ac3f56e9ac92532052608236adf960",
      "source_bytes": 1527,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working corpus at feat/tag-corpus-seed aa04bf804039f7d46411a3cf2296ebc36aa22d0c plus uncommitted edits"
    },
    {
      "source": ".worktrees/tag-corpus-loader/data/tags/catalog/problem-response/risk-mitigation.yaml",
      "archive": "sources/rust-seed/catalog/problem-response/risk-mitigation.yaml.md",
      "source_sha256": "c6c095ea4d286c7385acad6fad727b91d8ca7f05163b8e02119f02af7bd4bbbb",
      "source_bytes": 3543,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working corpus at feat/tag-corpus-seed aa04bf804039f7d46411a3cf2296ebc36aa22d0c plus uncommitted edits"
    },
    {
      "source": ".worktrees/tag-corpus-loader/data/tags/catalog/scene-challenge.yaml",
      "archive": "sources/rust-seed/catalog/scene-challenge.yaml.md",
      "source_sha256": "f2ce961bbb4889b57f15fecf23a981bd5cf1abf174184ef120fe4d8524e529ea",
      "source_bytes": 6335,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working corpus at feat/tag-corpus-seed aa04bf804039f7d46411a3cf2296ebc36aa22d0c plus uncommitted edits"
    },
    {
      "source": ".worktrees/tag-corpus-loader/data/tags/catalog/setting.yaml",
      "archive": "sources/rust-seed/catalog/setting.yaml.md",
      "source_sha256": "a72630df03d68c64c79f63a58d2b4fed0f36b93b656205a0bb2c5d8c0ba1655f",
      "source_bytes": 5483,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working corpus at feat/tag-corpus-seed aa04bf804039f7d46411a3cf2296ebc36aa22d0c plus uncommitted edits"
    },
    {
      "source": ".worktrees/tag-corpus-loader/data/tags/catalog/status-effect.yaml",
      "archive": "sources/rust-seed/catalog/status-effect.yaml.md",
      "source_sha256": "df056f26388196d32f4c9f22b22bf1ae47a856b638d1c31d185881581a49754b",
      "source_bytes": 4438,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working corpus at feat/tag-corpus-seed aa04bf804039f7d46411a3cf2296ebc36aa22d0c plus uncommitted edits"
    },
    {
      "source": ".worktrees/tag-corpus-loader/data/tags/catalog/theme.yaml",
      "archive": "sources/rust-seed/catalog/theme.yaml.md",
      "source_sha256": "17f8bf84ee688918cef3ad1c163d7a0d69f8949ee7bc2ff65b50ae161411cea8",
      "source_bytes": 2771,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working corpus at feat/tag-corpus-seed aa04bf804039f7d46411a3cf2296ebc36aa22d0c plus uncommitted edits"
    },
    {
      "source": ".worktrees/tag-corpus-loader/data/tags/registry/character-build.yaml",
      "archive": "sources/rust-seed/registry/character-build.yaml.md",
      "source_sha256": "707df64afa2acd17419fd4eb4b5262bdb1753b302b76b5eb3f7fd431af8851f8",
      "source_bytes": 2112,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working corpus at feat/tag-corpus-seed aa04bf804039f7d46411a3cf2296ebc36aa22d0c plus uncommitted edits"
    },
    {
      "source": ".worktrees/tag-corpus-loader/data/tags/registry/combat-effect.yaml",
      "archive": "sources/rust-seed/registry/combat-effect.yaml.md",
      "source_sha256": "cbbb2022354e6e16c241af1d4d44ff39bb522bbe0966cfaf6eb699909a262bb5",
      "source_bytes": 1660,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working corpus at feat/tag-corpus-seed aa04bf804039f7d46411a3cf2296ebc36aa22d0c plus uncommitted edits"
    },
    {
      "source": ".worktrees/tag-corpus-loader/data/tags/registry/encounter-role.yaml",
      "archive": "sources/rust-seed/registry/encounter-role.yaml.md",
      "source_sha256": "9c43f5c69395f6ca30928cba573befa91cab6f405567d458c49cbdefd96d2108",
      "source_bytes": 1157,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working corpus at feat/tag-corpus-seed aa04bf804039f7d46411a3cf2296ebc36aa22d0c plus uncommitted edits"
    },
    {
      "source": ".worktrees/tag-corpus-loader/data/tags/registry/exploration.yaml",
      "archive": "sources/rust-seed/registry/exploration.yaml.md",
      "source_sha256": "bce22457d2584ae77ddc080f09673e957dd6d1b1ebacf3c21636cc5fd1210038",
      "source_bytes": 1455,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working corpus at feat/tag-corpus-seed aa04bf804039f7d46411a3cf2296ebc36aa22d0c plus uncommitted edits"
    },
    {
      "source": ".worktrees/tag-corpus-loader/data/tags/registry/problem-response.yaml",
      "archive": "sources/rust-seed/registry/problem-response.yaml.md",
      "source_sha256": "cf9c18389968bbd7b03842b0d96f23eb8d73e07189f4188846e9bebb53855609",
      "source_bytes": 1751,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working corpus at feat/tag-corpus-seed aa04bf804039f7d46411a3cf2296ebc36aa22d0c plus uncommitted edits"
    },
    {
      "source": ".worktrees/tag-corpus-loader/data/tags/registry/scene-challenge.yaml",
      "archive": "sources/rust-seed/registry/scene-challenge.yaml.md",
      "source_sha256": "1d84ec6504b795b3f2493fa02195bf70c5f0b4b5b30ff3df85303b29c862fd87",
      "source_bytes": 2181,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working corpus at feat/tag-corpus-seed aa04bf804039f7d46411a3cf2296ebc36aa22d0c plus uncommitted edits"
    },
    {
      "source": ".worktrees/tag-corpus-loader/data/tags/registry/setting.yaml",
      "archive": "sources/rust-seed/registry/setting.yaml.md",
      "source_sha256": "c208df6623b2a326cecc1b94ad399015e97b1e1273425d34442e83b5d961467f",
      "source_bytes": 1972,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working corpus at feat/tag-corpus-seed aa04bf804039f7d46411a3cf2296ebc36aa22d0c plus uncommitted edits"
    },
    {
      "source": ".worktrees/tag-corpus-loader/data/tags/registry/status-effect.yaml",
      "archive": "sources/rust-seed/registry/status-effect.yaml.md",
      "source_sha256": "dbc63efdd047ebcc21244c7e467512829d71ddde7e9145a037343a8926bae2e6",
      "source_bytes": 2010,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working corpus at feat/tag-corpus-seed aa04bf804039f7d46411a3cf2296ebc36aa22d0c plus uncommitted edits"
    },
    {
      "source": ".worktrees/tag-corpus-loader/data/tags/registry/theme.yaml",
      "archive": "sources/rust-seed/registry/theme.yaml.md",
      "source_sha256": "5e37cb9bcb188be10909c49988d7ecfde598f7915dd2c2e6f278d083900863d7",
      "source_bytes": 1767,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working corpus at feat/tag-corpus-seed aa04bf804039f7d46411a3cf2296ebc36aa22d0c plus uncommitted edits"
    },
    {
      "source": ".worktrees/tag-corpus-loader/data/tags/registry/usage-pattern.yaml",
      "archive": "sources/rust-seed/registry/usage-pattern.yaml.md",
      "source_sha256": "9aa83167c3038744c62fc48826bde48e1602c2ce34438d883951fcf4513e4a8e",
      "source_bytes": 1605,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working corpus at feat/tag-corpus-seed aa04bf804039f7d46411a3cf2296ebc36aa22d0c plus uncommitted edits"
    },
    {
      "source": ".worktrees/tag-corpus-loader/docs/architecture/tag-ontology.md",
      "archive": "sources/rust-seed/tag-ontology.md",
      "source_sha256": "6386c5a0ad4dcb463b722c84b4870474e65d0a4d27fde248fc97af35d86bf2ac",
      "source_bytes": 18095,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "working ontology proposal including uncommitted edits"
    },
    {
      "source": "scratch/plans/archive/2026-04/2026-04-26-ontology-derived-tag-mode-semantics-plan.md",
      "archive": "sources/node-browser-plans/2026-04-26-ontology-derived-tag-mode-semantics-plan.md",
      "source_sha256": "2e6c9821aea0c76c401fa10f95d3c13622aac680e967ab56c352962ca636d767",
      "source_bytes": 11712,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "historical local archived plan; execution status not inferred"
    },
    {
      "source": "scratch/plans/archive/2026-04/ontology-browser-search-semantics-cold-open-plan-2026-04-20.md",
      "archive": "sources/node-browser-plans/ontology-browser-search-semantics-cold-open-plan-2026-04-20.md",
      "source_sha256": "46e6b3b7ac209a8fda55e9363a9dfb4c43be4922c7fea1b24d9631e7c73d1f33",
      "source_bytes": 7331,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "historical local archived plan; execution status not inferred"
    },
    {
      "source": "scratch/plans/archive/2026-05/2026-05-06-ontology-child-resolution-contract-plan.md",
      "archive": "sources/node-browser-plans/2026-05-06-ontology-child-resolution-contract-plan.md",
      "source_sha256": "e531dbc19b52b59194c9ea996be18c0cee61b2a8bfcea366ab64ae0e1fd01389",
      "source_bytes": 9628,
      "source_final_newline": true,
      "encoding": "single fenced UTF-8 payload",
      "provenance": "historical local archived plan; execution status not inferred"
    }
  ]
}
```
