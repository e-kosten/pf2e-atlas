# Getting started with PF2e Atlas

[Install Atlas and run setup](../README.md#quick-start) before your first session.
Run setup again after upgrading Atlas. To open the browser:

```bash
atlas web --open
```

Keep that terminal running while you use the browser. Atlas prints its local
address if you need to open it yourself. Press Ctrl+C in the terminal when you
have finished; your saved lists and encounters remain available next time.

## Find something

In Search, enter a name such as “Treat Wounds” or a description such as “low level
healing spell.” Add a Kind filter for spells, creatures, equipment or another
type of entry. Add level, rarity or trait filters to narrow the results.
Leave the search text empty to browse everything matching your filters.

Open a result to read its details. Follow a linked name to preview another
record. **Similar** finds other options based on the selected record;
the header's **Explore** icon shows related records and source links.
See [Search and related records](search.md) for sorting, relationship searches
and optional search settings.

## Collect options in a list

Use **Add to list** from Search, a record, or a linked preview. Choose an existing
list or create one in the picker, and add a note if useful. Notes support multiple
lines. Open Lists to review your collection and edit entry notes. Creatures and
hazards use the combined **Add to list or encounter** picker. Lists and encounters
show the most recently changed first.

Lists are useful for session preparation, spell choices, equipment shortlists
and creatures you may want to use later. Setup updates preserve your collections.

## Prepare and run an encounter

Add a creature or hazard with **Add to list or encounter**, choosing an existing
encounter or creating one. Open Encounters to review the roster and add your PCs.
You can save a named party and use **Add Party** to add its members together;
a party member only needs a name. See [saved parties](#saved-parties) for later edits.

Use the encounter controls to set initiative, move through turns, adjust HP,
track conditions and spell resources, and roll dice.

Select a participant and choose **Add tracker** under Trackers for reminders such
as “Breath Weapon.” Enter a duration and choose whose turn counts it down, at its
start or end. Trackers stay visible as **Elapsed** at zero; use **Restart**
to begin the same countdown again. Editing the duration or timing restarts it;
renaming preserves its progress. New trackers default to the start of their owner's
turn. When a countdown finishes, turn reminders show it once for that boundary.
You can remove it there or close the reminder and keep it. End reminders show the
creature finishing its turn; start reminders show the creature beginning its turn.
When both apply, **Continue** moves from end reminders to start reminders.

Timed conditions also accept rounds, minutes, hours or days. With turn timing
configured, one minute counts down over ten normal rounds. For encounters using
longer turns, change **Seconds per round** in encounter settings; the default is 6.
Changing this setting preserves the remaining time of minute/hour/day timers.
Trackers entered in rounds continue to count turns.
Advancing turns updates the remaining count; selecting a turn manually does not.
An ended condition remains visible until you remove it. Existing durations without
turn timing remain reminders until you edit them to choose a timing participant
and boundary.

## Use commands or an agent

You can use Atlas from any directory after setup:

```bash
atlas search 'walk through walls' --kind spell --limit 5
atlas record resolve 'Treat Wounds' --pack-name actionspf2e
```

For looking up a specific entry, using filters, managing lists or reading JSON
output, see [CLI examples](cli-examples.md).

To give your AI agent instructions for using Atlas:

```bash
atlas agent skills install
```

Choose your agent and whether to install for one project or all projects. See
[agent skill installation](installation.md#agent-skill) for supported agents and
updating the skill after an Atlas upgrade.

## If setup or startup needs attention

Check what is missing before changing your data:

```bash
atlas setup --check
```

Follow the reported action, then rerun `atlas setup` if preparation is needed.
See [Setup and maintenance](setup-and-maintenance.md) for offline use, cache cleanup
and custom paths, or [Installation](installation.md) to update Atlas.

## Saved parties

Editing a party changes the names and levels used when you add it to a future
encounter. PCs already added to an encounter keep their existing state.
