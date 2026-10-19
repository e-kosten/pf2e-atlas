# Annotation research and evaluation questions

The October 9 discussion investigated controlled vocabularies, LLM annotation and
independent review. Primary publication pages were checked again on 2026-10-19.
These findings concern adjacent tasks; none establishes Atlas labeling quality,
an optimal reviewer count, or current Luna/Sol performance.

| Primary source | Relevant finding | Boundary on its use here |
| --- | --- | --- |
| [W3C SKOS Primer](https://www.w3.org/TR/skos-primer/) | Separates concepts, preferred/alternative labels, scope notes, examples, semantic relations and collections. | Useful vocabulary practices do not select RDF, OWL or a complete formal ontology for Atlas. |
| [Fonseca and Cohen, 2024: Can Large Language Models Follow Concept Annotation Guidelines?](https://aclanthology.org/2024.findings-acl.478/) | Studies explicit concept guidelines and model adherence in scientific/financial annotation. | Definitions and examples need testing; the task/models differ from PF2e multilabel assignment. |
| [Verga et al., 2024: Replacing Judges with Juries](https://arxiv.org/abs/2404.18796) | A panel of diverse judges improved evaluation in studied settings. | The three-model panel and task-dependent aggregation do not prove three is universally optimal. |
| [Nahum et al.: Are LLMs Better than Reported?](https://arxiv.org/abs/2410.18889) | Model ensembles helped detect label errors and improve factual-consistency/summarization evaluation. | Existing “gold” labels can be wrong; this is not an Atlas assignment benchmark. |
| [Kim et al., ICML 2025: Correlated Errors in Large Language Models](https://proceedings.mlr.press/v267/kim25e.html) | Studies correlated errors and resulting ensemble benefits/limits. | Separate requests or providers do not guarantee independent mistakes. |
| [Kohli, 2026 preprint: Nine Judges, Two Effective Votes](https://arxiv.org/abs/2605.29800) | Reports strong error correlation and diminishing panel benefit in its studied evaluation tasks. | Its effective-vote estimates and panel-size findings are task/model-specific, not universal reviewer limits. |
| [Elumar et al.: Cost-aware LLM-based Online Dataset Annotation](https://arxiv.org/abs/2505.15101) | Studies adaptive, cost-aware use of annotators. | Routing reviewer effort is distinct from pruning required record/concept coverage. |

The original discussion also consulted OpenAI's model-selection and evaluation
guidance, linked in the [discussion snapshot](sources/2026-10-09-design-discussion.md).
Its transferable recommendation was task-specific comparison calibrated against
human judgments. Historical model names are experiment hypotheses, not an enduring
model recommendation or API contract.

## A pilot if labeling is justified

Agents prepare representative and deliberately difficult examples; the human
approves verdicts, evidence and suitability. Include different families, lengths,
rare concepts, conditional effects, misleading mentions, weak/strong negatives,
empty positive sets and surprising family combinations. Model-selected easy
examples alone cannot establish quality. Ambiguous or rejected candidates do not
automatically become negative gold labels.

Separate teaching examples, reusable development evaluations and an untouched
final evaluation. Keep related record variants in the same split. Once failures
inform definitions, prompts or policy, those cases become development evidence;
reserve fresh cases for an independent estimate of generalization.

The proposed comparison was two-, three- and five-reviewer configurations plus
a stronger single-model control on identical cases. Preserve independent first
judgments before sharing evidence. Measure reviewer count, model diversity and
acceptance/reconsideration policy separately. A tentative hypothesis was two
Luna assessments with Sol for disputes and independent audits of agreements,
negatives and empty sets; compare routine stronger-model participation on hard
concepts. Human review remains necessary for ambiguous meanings and calibration.

Measure per-concept precision/recall, shared omissions, accepted-case errors,
source/evidence correctness, additional human-confirmed errors found, escalation
and human time, latency and cost. Include random negative/strong-negative audits
and applicability challenges. Coverage manifests prove that requests received
dispositions, not that models noticed every applicable concept.

Unanimity can increase escalation and still accept shared mistakes. Majority
voting can discard correct minority positives. Keep positive nominations until
they receive an evidence-based disposition. Bound reconsideration, retain original
votes and distinguish agreement after discussion from independent corroboration.
Definition problems should return to ontology review rather than being forced
into consensus.

Choose a policy from the measured marginal value of additional reviewers. The
research preserved here does not supply a universal two-versus-three answer.
