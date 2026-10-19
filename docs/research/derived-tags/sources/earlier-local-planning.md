# Earlier local tag-planning excerpts

Tag-specific passages from a mixed product scratch document. Unrelated search,
UI and refactor notes are outside this workstream. The personal-plans backup
duplicates these ideas. Historical requests do not authorize implementation.
[Source map](../sources-and-cleanup.md) · [Manifest](manifest.md)

## Original lines 5–6

````markdown
# Figure out the tag/assignemtns folder structure
How to model the assignments folder. While there isn't much in it right now, it will eventually map the assignments for the 25k+ live records coming out of the foundry data, so the total size of it will be very large and require a smart / logical structure. The first thing that comes to mind is to use a similar pack based structure to the source foundry data, but I'm open to recommendations and discussion on the matter
````

## Original lines 31–34

````markdown
# Category relevance script
The derived tagging exercise will require a lot of review and keeping the relevant tag set in the agents memory. This should be supported by a script. Ideally, the script would do something like take a family (or maybe a list of entities) and spit out the relevant tags. The script should complain if the list provided doesn't all share the same ta set. The benefit then of this is that it can force the agent to do tagging in batches while keeping the full tag set in memory. Can then do an orchestrator/sub-agent path so that in one job I can run throuhg like 1000 things without worrying that a single agent can effectively keep that much context in mind.

The con of this is that the agents might not proose new missing categories. Something to think about, but it coul dbe a worthwhile tradeoff
````
