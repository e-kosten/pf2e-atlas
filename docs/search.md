# Search and related records

Start with a name or describe what you need in Search. Use filters for exact
constraints such as kind, level, rarity or traits. Leave the text empty to browse
everything matching your filters. Open a result to read it or add it to a list.

## Sort and explore

Open **Sort** to order results by name, level/rank or unit price. Add a second
attribute rule to order ties. Missing values appear last in either direction.
Semantic results still show a bounded set of Top matches. Sorting a saved list
is temporary and preserves its saved order.

**Random** and **Shuffle** help explore matching records. A retained seed keeps
the random order repeatable while paging. In the CLI, use `--sort random --seed 42`,
or omit `--seed` to generate one. Attribute sorting uses tokens such as
`--sort level_desc --sort unit_price_asc`.

The document header's **Explore** icon opens related-record previews.
**Search referring records** finds entries that link to the selected record;
**Search referenced records** finds entries it links to. These open the complete
filtered results in Search.

Use **Links** beside the search filters to choose another entry as a condition.
Each condition has a direction and a **Main entry only** / **Whole entry** scope.
Advanced filters combine link and ordinary conditions with AND, OR and NOT.
References establish links in the source, not that an item is worn or an effect
has been applied. See [CLI reference searches](cli-examples.md#follow-references)
for the equivalent commands and filter expressions.

## Search settings

Defaults are suitable for getting started. Open **Search settings** when you want
to compare how results are found. Browser preferences are remembered; explicit
search links retain their own settings.

**Score-based** ranking is the default for hybrid text search. **Rank-based** is
an alternative using the same content and model. Switching does not rebuild search
data. Ranking does not affect alphabetical browsing or Similar results.

**Search content: Core sections** focuses matches on a record type's main content
and its name and identity. Core includes creatures' descriptions and hazards'
descriptions, routines and intrinsic actions, with other selections for other
types. Records without Core prose use their stored name and identity. **All** is
the default. Core focuses existing content; keyword-only search still does not
search every description.

## Similar records

Record Similar defaults to **Passage matches**, comparing selected content with
other entries. **Content average** compares the selected content as a whole;
**Name and identity** compares the record's identity rather than its descriptive
passages. Trait Similar uses **Trait definition** by default.

**Sections to compare** selects All, Core or Custom source content. It is separate
from the result-side Search content choice. All and Core both include identity;
Custom lets you choose exact parts. Similar uses stored vectors and needs no model
weights at query time.

CLI examples:

```bash
atlas search "forest guardian" --search-content core
atlas similar "Lich" --sections core --search-content all
atlas similar "Brown Mold" --list-parts --sections core --json
```

## Optional search models

Fresh setup prepares **BGE Base** (`bge-base`). **EmbeddingGemma** (`embeddinggemma`)
is an alternative. To prepare both once, then select one:

```bash
atlas setup --search-profile bge-base --search-profile embeddinggemma
atlas web --search-profile bge-base
atlas search "forest guardian" --search-profile embeddinggemma
```

Preparing another model may download assets and run inference. Switching between
already prepared models does not rebuild data. Each request uses one profile;
without a flag, the sole stored profile is used, or BGE Base when both are stored.
Text semantic search needs the selected model's verified cache. Similar only needs
its stored vectors.

Rerunning setup without profile flags preserves the configured set. Explicit
flags add profiles to that set; setup repairs their pinned cached assets. For a
new workspace with keyword search and no models, use `atlas setup --no-embeddings`
and `atlas search QUERY --retrieval fts`. On an existing workspace,
`--no-embeddings` preserves configured models.

See [CLI examples](cli-examples.md#compare-search-profiles-and-ranking) for retrieval
and ranking comparisons, or [setup and maintenance](setup-and-maintenance.md) for
readiness and cache management.
