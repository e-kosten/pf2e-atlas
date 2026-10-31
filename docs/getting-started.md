# Getting started with PF2e Atlas

[Install Atlas and run setup](../README.md#quick-start) once. To start a session:

```bash
atlas web --open
```

Keep that terminal running while you use the browser. Atlas prints its local
address if you need to open it yourself. Press Ctrl+C in the terminal when you
have finished; your saved lists and encounters remain available next time.

## Find something

In Search, enter a name such as “Treat Wounds” or a description such as “low level
healing spell.” Add a Kind filter for spells, creatures, equipment or another
record type. Add level, rarity or trait filters when those constraints matter.
Leave the search text empty to browse everything matching your filters.

Open a result to read its details. Follow a linked name to preview another
record. **Similar** finds other options based on the selected record;
the header's **Explore** icon shows related records and source links.
See [Search and related records](search.md) for sorting, relationship searches
and optional search settings.

## Collect options in a list

Use **Add to list** from Search, a record, or a linked preview. Choose an existing
list or create one in the picker. Open Lists to review your collection and add
notes to individual entries. Creatures and hazards use the combined
**Add to list or encounter** picker.

Lists are useful for session preparation, spell choices, equipment shortlists
and creatures you may want to use later. Setup updates preserve your collections.

## Prepare and run an encounter

Add a creature or hazard with **Add to list or encounter**, choosing an existing
encounter or creating one. Open Encounters to review the roster and add your PCs.
You can save a named party and use **Add Party** to add its members together;
a party member only needs a name.

Use the encounter controls to set initiative, move through turns, adjust HP,
track conditions and spell resources, and roll dice. Check a participant's rules
and any displayed restrictions when deciding an action at the table. Party edits
apply to future additions; PCs already in an encounter keep their own state.

## Use commands or an agent

You can use Atlas from any directory after setup:

```bash
atlas search 'walk through walls' --kind spell --limit 5
atlas record resolve 'Treat Wounds' --pack-name actionspf2e
atlas roll '2d20kh1+12'
```

The last command rolls twice and keeps the higher die. Rolls work even without
reference setup. For exact record reading, filters, lists and JSON, use the
[CLI examples](cli-examples.md).

To give a local agent the matching Atlas skill:

```bash
atlas agent skills install
```

Choose your agent and scope. See [agent skill installation](installation.md#agent-skill)
for supported targets and updating the skill after a CLI upgrade.

## If setup or startup needs attention

Check what is missing before changing your data:

```bash
atlas setup --check
```

Follow the reported action, then rerun `atlas setup` if preparation is needed.
See [Setup and maintenance](setup-and-maintenance.md) for offline use, cache cleanup
and custom paths, or [Installation](installation.md) to update the executable.
