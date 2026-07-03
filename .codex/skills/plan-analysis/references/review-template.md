# Plan Review Template

Use this as a starting shape for files under `scratch/plan-review/`.

```md
# <Plan Title>: Review

## Summary

<2-4 sentences on the overall quality of the plan and whether the main issues are architectural, boundary-related, validation-related, or mostly wording tightenings.>

## Review Scope

- Target plan: `<path>`
- Review date: `YYYY-MM-DD`
- Review goal: `<what the user asked to compare the plan against>`

## Docs And Code Owners Consulted

- `docs/architecture/overview.md`
- `docs/architecture/boundaries.md`
- `<focused docs>`
- `<important code owners>`

## Findings Checklist

- [ ] Finding 1 short label
- [ ] Finding 2 short label
- [ ] Finding 3 short label

## Detailed Findings

### 1. <Finding Title>

Why it matters:

- <reason 1>
- <reason 2>

Evidence:

- `<doc or file ref>`
- `<doc or file ref>`

Recommended tightening:

- <plan wording or boundary change>

Resolution:

- Leave blank until discussed, or add a short note such as:
- `Updated Slice E to require extension of the shared modal framework.`

### 2. <Finding Title>

...

## Open Questions Or Non-Issues

- <question or item that looked risky but is already covered by the plan>

## Recommended Plan Tightenings

- <short summary list of the plan changes you expect to make if the user agrees>
```
