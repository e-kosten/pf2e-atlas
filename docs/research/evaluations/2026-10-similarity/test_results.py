"""Synthetic displayed-audit regressions; no ranked results or source bodies."""
import copy
import importlib.util
import json
import shutil
import tempfile
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location('result_replay', Path(__file__).with_name('verify_results.py'))
r = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(r)


class DisplayedAuditOpinions(unittest.TestCase):
    def setUp(self):
        self.prompt = '1' * 64
        self.bindings = [{'audit_example_id': 'example', 'packet_sha256': '2' * 64,
                          'display_items': [{'display_item_id': 'a'}, {'display_item_id': 'b'}],
                          'cited_pointer_bindings': [{'pointer': '/displayed_evidence/0'}, {'pointer': '/displayed_evidence/1'}]}]
        def assessment(grade):
            return {'semantic_connection': grade, 'scope_fit': 'in_scope', 'ownership': 'intrinsic',
                    'uncertainty': 'low', 'rationale': 'Synthetic assessment.', 'source_pointers': ['/displayed_evidence/0']}
        self.opinions = [{'audit_example_id': 'example', 'judge_id': 'explanation-judge', 'judge_model': 'synthetic',
                          'judge_prompt_sha256': self.prompt, 'packet_sha256': '2' * 64,
                          'overall': assessment('supported'),
                          'items': [dict(assessment('unsupported'), display_item_id='a'), dict(assessment('unknown'), display_item_id='b')]}]

    def validate(self):
        return r.audit_opinion_counts(self.opinions, self.bindings, self.prompt)

    def test_keeps_candidate_assessment_separate_from_items_and_unknowns(self):
        self.assertEqual(self.validate(), {'overall_semantic_connection': {'supported': 1},
                                           'item_semantic_connection': {'unknown': 1, 'unsupported': 1}})

    def test_rejects_missing_or_duplicate_opinions(self):
        for rows in [[], self.opinions * 2]:
            with self.subTest(rows=len(rows)), self.assertRaisesRegex(ValueError, 'opinion census'):
                r.audit_opinion_counts(rows, self.bindings, self.prompt)

    def test_rejects_missing_duplicate_or_extra_item(self):
        original = copy.deepcopy(self.opinions[0]['items'])
        for rows in [original[:1], original[:1] * 2, original + [dict(original[0], display_item_id='extra')]]:
            self.opinions[0]['items'] = rows
            with self.subTest(items=[row['display_item_id'] for row in rows]), self.assertRaisesRegex(ValueError, 'item census'):
                self.validate()

    def test_rejects_prompt_or_packet_mismatch(self):
        for key in ['judge_prompt_sha256', 'packet_sha256']:
            original = self.opinions[0][key]
            self.opinions[0][key] = '3' * 64
            with self.subTest(key=key), self.assertRaisesRegex(ValueError, 'opinion digest'):
                self.validate()
            self.opinions[0][key] = original

    def test_rejects_unknown_enum_and_unbound_source(self):
        self.opinions[0]['overall']['semantic_connection'] = 'excellent'
        with self.assertRaisesRegex(ValueError, 'grade enum'):
            self.validate()
        self.opinions[0]['overall']['semantic_connection'] = 'partial'
        self.opinions[0]['items'][1]['source_pointers'] = ['/unassigned/source']
        with self.assertRaisesRegex(ValueError, 'source pointer'):
            self.validate()


class IntegratedArtifactReplay(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.dataset = Path(__file__).parent
        cls.results = cls.dataset/'results'
        cls.production = r.lines(cls.results/'production-scores.jsonl')
        cls.historical = r.load(cls.results/'provenance.json')['historical_artifact']

    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.results_copy = Path(self.temporary.name)
        shutil.copytree(self.results/'integration-parity', self.results_copy/'integration-parity')

    def validate(self):
        return r.verify_integration_parity(self.dataset, self.results_copy, self.production, self.historical)

    def amend_report(self, change):
        path = self.results_copy/'integration-parity/parity.json'
        report = json.loads(path.read_text())
        change(report)
        path.write_text(json.dumps(report))

    def test_rejects_equal_overlap_with_changed_order(self):
        def change(report):
            report['cells'][0]['new_top20'].reverse()
        self.amend_report(change)
        with self.assertRaisesRegex(ValueError, 'top-twenty parity'):
            self.validate()

    def test_rejects_unchanged_order_with_changed_exposed_score(self):
        def change(report):
            scores = report['cells'][0]['new_scores']
            scores[next(iter(scores))] += .001
        self.amend_report(change)
        with self.assertRaisesRegex(ValueError, 'exposed score parity'):
            self.validate()

    def test_rejects_trait_selector_stability_weakening_generation_guard(self):
        def change(report):
            row = next(row for row in report['negative_guard_probes'] if row['id'] == 'trait-old-generation')
            row['response']['status'] = 'ok'
        self.amend_report(change)
        with self.assertRaisesRegex(ValueError, 'stale guard response'):
            self.validate()


if __name__ == '__main__':
    unittest.main()
