# PF2E Derived Tag Sanity Checks

Use these commands after meaningful heuristic changes.

## Core Files
- `src/derived-tags.ts`
- `src/derived-tag-gap-evaluator.ts`
- `src/evaluate-derived-tag-gaps.ts`
- `tests/derived-tags.test.ts`
- `tests/pf2e-data.test.ts`
- `src/pf2e-data.ts` when output shape or SQL filtering is involved

## Validation Commands
```bash
npm test -- tests/derived-tags.test.ts tests/pf2e-data.test.ts
npm run build
npm test
npm run refresh-index
```

## Offline Gap Evaluator
Use the repo-owned evaluator when you are expanding coverage for a specific tag family or checking likely false negatives after a rebuild.

```bash
npm run evaluate-derived-tags -- --tag restraint_escape --category equipment --subcategory gear --limit 12 --exemplar-limit 8
```

```bash
npm run evaluate-derived-tags -- --tag anti_poison --category equipment --subcategory consumable --limit 12 --exemplar-limit 8
```

Interpretation guidance:
- treat the output as a review queue, not an auto-tagging system
- prefer candidates with concrete shared rule evidence over merely thematic similarity
- use borderline suggestions to design blockers and negative gates, not just positive anchors

## Corpus Counts
```bash
sqlite3 .cache/pf2e-index.sqlite "pragma busy_timeout=5000; \
select count(distinct tag) as derived_tag_count, count(*) as assignments \
from record_derived_tags rdt \
join records r on r.record_key = rdt.record_key \
where r.is_search_canonical = 1;"
```

```bash
sqlite3 .cache/pf2e-index.sqlite "pragma busy_timeout=5000; \
select tag, count(*) as c \
from record_derived_tags rdt \
join records r on r.record_key = rdt.record_key \
where r.is_search_canonical = 1 \
group by tag \
order by c desc, tag asc \
limit 25;"
```

## Named Record Spot Checks
```bash
sqlite3 .cache/pf2e-index.sqlite "pragma busy_timeout=5000; \
select r.name, group_concat(rdt.tag, ', ') \
from records r \
join record_derived_tags rdt on r.record_key = rdt.record_key \
where r.is_search_canonical = 1 \
  and r.name in ( \
    'Antidote (Lesser)', \
    'Accuser Agent', \
    'Abandoned Zealot', \
    'Adamantine Golem', \
    'Animated Armor', \
    'Agorron Guard' \
  ) \
group by r.record_key \
order by r.name;"
```

## Bad-Class Counters
```bash
sqlite3 .cache/pf2e-index.sqlite "pragma busy_timeout=5000; \
select 'scene_adjacent_constructs', count(*) \
from records r \
join record_derived_tags d on r.record_key = d.record_key \
where r.is_search_canonical = 1 \
  and d.tag = 'scene_adjacent' \
  and instr(lower(r.traits_json), 'construct') > 0; \
select 'offensive_with_healing_trait', count(*) \
from records r \
join record_derived_tags d on r.record_key = d.record_key \
where r.is_search_canonical = 1 \
  and d.tag = 'offensive' \
  and instr(lower(r.traits_json), 'healing') > 0; \
select 'arctic_with_officer_text', count(*) \
from records r \
join record_derived_tags d on r.record_key = d.record_key \
where r.is_search_canonical = 1 \
  and d.tag = 'arctic' \
  and lower(r.description_text) like '%officer%';"
```

## Random Untagged Samples
```bash
sqlite3 .cache/pf2e-index.sqlite "pragma busy_timeout=5000; \
select r.name, r.level, r.traits_json, substr(r.description_text, 1, 220) \
from records r \
left join record_derived_tags d on r.record_key = d.record_key \
where r.is_search_canonical = 1 \
  and r.category = 'creature' \
group by r.record_key \
having count(d.tag) = 0 \
order by random() \
limit 20;"
```

```bash
sqlite3 .cache/pf2e-index.sqlite "pragma busy_timeout=5000; \
select r.name, r.level, r.subcategory, r.traits_json, substr(r.description_text, 1, 220) \
from records r \
left join record_derived_tags d on r.record_key = d.record_key \
where r.is_search_canonical = 1 \
  and r.category = 'equipment' \
group by r.record_key \
having count(d.tag) = 0 \
order by random() \
limit 20;"
```

```bash
sqlite3 .cache/pf2e-index.sqlite "pragma busy_timeout=5000; \
select r.name, r.level, r.subcategory, r.traits_json, substr(r.description_text, 1, 220) \
from records r \
left join record_derived_tags d on r.record_key = d.record_key \
where r.is_search_canonical = 1 \
  and r.category = 'equipment' \
  and r.subcategory in ('gear', 'consumable') \
group by r.record_key \
having count(d.tag) = 0 \
order by random() \
limit 20;"
```

Use these samples to identify repeated missing concepts. If the same idea appears across multiple random samples, either broaden an existing rule or propose a new derived tag family. Do not add tags just to drive the untagged count to zero.

## Useful Follow-Up Checks
```bash
sqlite3 .cache/pf2e-index.sqlite "pragma busy_timeout=5000; \
select r.name, group_concat(rdt.tag, ', ') \
from records r \
join record_derived_tags rdt on r.record_key = rdt.record_key \
where r.is_search_canonical = 1 \
  and rdt.tag = 'anti_poison' \
group by r.record_key \
order by r.name \
limit 20;"
```

```bash
sqlite3 .cache/pf2e-index.sqlite "pragma busy_timeout=5000; \
select r.name, group_concat(rdt.tag, ', ') \
from records r \
join record_derived_tags rdt on r.record_key = rdt.record_key \
where r.is_search_canonical = 1 \
  and rdt.tag in ('nautical', 'aquatic_context', 'scene_adjacent', 'profession_npc') \
group by r.record_key \
order by r.name \
limit 50;"
```

## Evaluation Checklist
- Check whether the changed tags now cover the intended item or creature families.
- Check whether support vs offensive polarity still separates cleanly.
- Check whether scene-fit tags stay off obvious monsters and constructs.
- Check whether environment tags require enough evidence and avoid incidental flavor text.
- Check evaluator suggestions for likely false negatives before broadening a rule family.
- Review random untagged samples to see whether important practical concepts are still missing from the ontology.
- Compare corpus volume shifts before and after the change. Large drops or jumps need an explanation.
