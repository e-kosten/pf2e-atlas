# Integration and nightly dogfooding handoff

## Target and evidence

[PR25](https://github.com/e-kosten/pf2e-atlas/pull/25) merged from
`integration/source-modeling` into `main` at
`41865f60c187fafde4f50a7a3f3a29587aea8cc5`. Use the existing nightly workflow.
No separate development branch is needed. Nightly dogfooding is not a 1.0 release
or a declaration that all backlog work is complete.

At `e59f78d05a6764631299f4e65eb5d9ef5bd38113` (PR75), all eight checks in
[CI run 39206552603](https://github.com/e-kosten/pf2e-atlas/actions/runs/39206552603)
passed. This includes Rust, web UI, source contracts, release tools on three
platforms, Windows artifact publication, and the developer CLI without web assets.
It does not establish installed nightly acceptance for the eventual merge commit.

## Integration handoff

- [x] Maintainer accepted and merged PR25 after its eight checks passed.
- [ ] Check the current README and shipped CLI skill against the intended nightly
  experience. PR75 refreshed both; avoid carrying older installed skill copies
  forward without checking their installation status.
- [ ] Preserve the research and local workspace described in
  [the research handoff](../research/search-evaluation-handoff.md). Back up any
  workspace used for dogfooding before upgrading it. Nightly and stable binaries
  normally share data paths; a binary downgrade does not downgrade a database.
- [ ] Record accepted limitations from the current backlog. Incomplete condition
  automation, query spelling, parties/campaigns,
  and further presentation work remain follow-ups, not implied finished features.

## Installed nightly acceptance

[Nightly run 39208021368](https://github.com/e-kosten/pf2e-atlas/actions/runs/39208021368)
was dispatched on the merge commit. Dispatch is not publication or installed-build
acceptance; check its final result before installation.

- [ ] Let `nightly.yml` run on `main` (09:23 UTC daily), or explicitly dispatch it
  on `main`. It verifies and packages a pinned commit before publication. Scheduled
  runs may skip unchanged inputs. Check the workflow result and dated release,
  rather than assuming a merge produced a usable download.
- [ ] Install the nightly using the README instructions. Record the dated tag,
  `atlas --version`, platform, and workspace path. Verify downloaded checksums.
- [ ] On an isolated workspace, exercise setup, a name lookup, partial-name and
  conceptual searches, filters, JSON output, web startup, and offline reuse after
  source/model downloads. Check both selected semantic models if installed and
  Score-based/Rank-based fusion. Use fresh user queries as well as known cases.
- [ ] Create a list and encounter, restart the installed application, and confirm
  their persistence. Test an upgrade on a backup copy; verify state retention and
  unchanged embedding reuse. Do not run destructive rebuild experiments against
  the only copy of a workspace.
- [ ] Record failures with the installed tag and reproduction, separating release
  packaging/setup problems from search preferences and deferred product features.
  Replace local development previews with the installed nightly after acceptance.

## Follow-up boundaries

Search dogfooding keeps BGE Base and EmbeddingGemma available when built, with
Score-based fusion the default and Rank-based available for comparison. Neither
experiment established a universal winner. Exposed evaluation queries are no
longer fresh holdouts. See [search evaluation notes](../research/source-backed-search-evaluation.md).

The earlier `integration/record-refactor` / AO branches remain historical feature
inspiration; do not merge their implementations wholesale. Active work belongs in
the current architecture and backlog. This checklist does not authorize merging,
publishing, or deleting other contributors' checkouts.
