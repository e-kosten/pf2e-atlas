# Similar Search Entry and Source Changes

Status: deferred
Priority: later
Owner: unassigned

## Problem and disposition

Consider choosing or replacing a Similar source directly from Search, particularly
when the desired source is excluded by the current result filters. The user
deferred this work after reviewing the working prototype in
[closed PR #136](https://github.com/e-kosten/pf2e-atlas/pull/136). No UI change is
selected for implementation.

The existing record-to-Similar route remains available. The prototype's permanent
"Similar to…" and "Change similar record…" toolbar buttons gave a secondary
capability too much prominence. Adding a button for every search workflow would
clutter the toolbar; the prominence of the dormant Links control is also open to
review.

## Findings to preserve

- From a fresh Search view, reaching full Similar results through a record takes
  four clicks plus the name-search submission: select the record, Explore,
  Similar, then Search similar records. The record opens within Search's reader.
- The prototype chooser takes three clicks plus submission: open the chooser,
  select a source, then Find similar. Putting its entry behind a Search options
  menu returns the count to four clicks. Counts exclude typing and input focus.
- The existing five-result Similar preview is already available after three
  clicks. Its full-search link can be followed without waiting for that preview.
- A local Leather Armor / BGE Base comparison estimated about 1.5 seconds of
  accumulated loading for the existing full route versus 1.1 seconds for the
  prototype, using a warm model and fresh UI caches. This sums individual loading
  steps and excludes human reading and automation pauses; it is not a general
  performance benchmark. Skipping the intermediate preview wait largely removes
  the difference. Both flows load the selected source's full record detail.
- The existing Similar handoff already retains filters. The chooser's distinct
  benefit is finding a source independently of those filters, rather than faster
  retrieval or filter preservation itself.

## Constraints for reconsideration

- Keep applied Similar and record-link criteria prominent and understandable,
  including when entered through another part of the app. Secondary creation
  controls need not have the same prominence as active criteria.
- Consider making the applied "Similar to" label itself open the source chooser;
  the user preferred this direction. Preserve the source name's document-reading
  affordance. This remains a candidate, not approved implementation scope.
- Reconcile with [PR #134](https://github.com/e-kosten/pf2e-atlas/pull/134)'s search
  settings and section-comparison UX before choosing an entry point. A broader
  Search options menu was discussed, but no menu layout was selected.
- Revisit when repeated source changes or choosing sources outside active filters
  establish enough value to justify another entry point. Preserve the existing
  workflow while this item is deferred.

## Related

- [Web filter UX expansion](rust-web-filter-ux-expansion.md)
- [Relationship-scoped search](../../architecture/decisions/0057-relationship-scoped-search.md)
