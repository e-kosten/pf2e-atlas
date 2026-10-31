# Search and related records

Search for a particular entry or describe an idea you want to explore. Atlas
combines matches on names and keywords with matches on meaning.

## Search by idea

Try “a guardian for an enchanted forest,” “magic for a haunted-house adventure,”
or “ways to get through locked doors.” Open a result to read its details and see
which parts matched your search. Follow linked names to explore related entries.

When you know the name, enter it directly—for example, “Treat Wounds.” You can
also leave the text empty to browse entries using filters alone.

## Narrow the results

Use **Kind** to look for creatures, spells, equipment or another type of entry.
Add level, rarity or trait filters for requirements that matter to your game.
For example, combine a creature idea with a level range suited to your party.
Filters keep those requirements explicit while you experiment with the wording.

## Find similar options

Open **Similar** from an entry you like to find alternatives. In **Search settings**,
choose **Sections to compare** to use all of that entry's content, its main sections
(Core), or sections you select yourself (Custom).

The default **Passage matches** method looks for matches to individual passages.
**Content average** compares the selected content as a whole. **Name and identity**
uses the entry's name and basic details rather than its descriptions. Starting
from a trait uses **Trait definition** by default.

**Sections to compare** chooses content from the entry you started with.
**Search content** chooses which parts of the other entries can match it.
All and Core both include the name and basic details; Custom lets you select
individual parts.

## Sort results

Open **Sort** to order results by name, level/rank or unit price. Add a second
sorting rule to decide the order when the first values are equal. Missing values
appear last whether sorting upwards or downwards.

When results are labelled **Top matches**, sorting orders that selected set of
matches. It does not expand the search to every entry in the database. Sorting a
saved list changes the current view while keeping its saved order.

Use **Random** to explore results in a random order and **Shuffle** to change that
order. The order stays consistent as you move between pages.

## Follow references

The header's **Explore** icon opens previews of related entries.
**Search referring records** finds entries that link to the selected record;
**Search referenced records** finds entries it links to. Both open filtered
results in Search.

Use **Links** beside the search filters to choose another entry as a condition.
**Whole entry** includes references in abilities and other contents;
**Main entry only** checks references made by the main entry itself. Advanced
filters combine conditions with AND, OR and NOT.

These searches find references between entries. A creature linking to a piece
of equipment, for example, does not by itself establish that the creature wears it.
See [CLI reference searches](cli-examples.md#follow-references) for commands.

## Adjust search settings

Open **Search settings** to compare ways of finding results. Atlas remembers your
browser preferences; opening a shared search link uses the settings in that link.

**Score-based** ranking is the default. **Rank-based** combines the same keyword
and meaning-based matches differently. If useful matches appear too far down the
list, try both and compare the results. This choice affects text search, while
browsing and Similar have their own ordering.

**Search content: Core sections** focuses on each entry type's main sections, such
as a creature's description or a hazard's description and routine. **All** includes
all sections prepared for searching and is the default. Neither setting makes
keyword search cover every description; meaning-based search covers selected
passages from descriptions and other prose.

## Try another search model

Setup prepares **BGE Base** by default. You can also try **EmbeddingGemma** to
compare its results. Preparing another model downloads its files and processes
your reference content, which may take a while:

```bash
atlas setup --search-profile bge-base --search-profile embeddinggemma
```

Once both are ready, select a model in **Search settings**. Keep using the default
if you do not want to compare models. Setup remembers the models you have prepared
for future updates.

See [CLI examples](cli-examples.md#compare-search-profiles-and-ranking) for model
selection, keyword-only search, Core sections and sorting commands, or
[setup and maintenance](setup-and-maintenance.md) for missing model files.
