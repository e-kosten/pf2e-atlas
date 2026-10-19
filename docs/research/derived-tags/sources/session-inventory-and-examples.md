# Historical session inventory and examples

These generated queues include CLI test fixtures. Statuses and self-described
LLM sources do not establish human approval or semantic correctness. The extract
preserves every nonpending decision/record row and any noted row, representative
rationale templates, migration manifests, a translation example and original file
hashes. Generated pending rows, record-body copies and cursor state are omitted.

See [retention boundaries](../sources-and-cleanup.md) before deleting originals.

```json
{
  "captured": "2026-10-19",
  "scope": "Inventory and selected historical review state; NOT a backup of all original session payloads and NOT calibration truth",
  "migration_status_counts": {
    "needs_review": 70024,
    "complete": 5
  },
  "migration_nonempty_decision_row_occurrences": 2571,
  "migration_distinct_nonempty_decision_rows": 1243,
  "rationale_counts": {
    "Temporary manual CLI test fixture: pending review item for queue and session testing.": 5,
    "Temporary manual CLI test fixture: pending exclusion review for queue and TUI testing.": 5,
    "Temporary manual CLI test fixture: pending coastal-setting review for CLI testing.": 3,
    "Temporary manual CLI test fixture: pending role review to exercise mixed-family queue behavior.": 2,
    "Legacy seed migration currently supplies this tag; review whether to convert it into an explicit assignment.": 3091,
    "Legacy seed candidate should be reviewed to decide whether it remains a true exemplar.": 3091,
    "Review whether this creature remains a strong positive exemplar for \"urban_setting\".": 26
  },
  "nonpending_or_noted_rows": [
    {
      "row": {
        "recordKey": "pathfinder-monster-core:TGYELuImcTcuX0aH",
        "name": "Conspirator Dragon (Adult)",
        "category": "creature",
        "resolutionStatus": "complete",
        "decisions": [
          {
            "kind": "assignment",
            "family": "encounter_role",
            "tag": "combatant_npc",
            "mode": "include",
            "status": "rejected",
            "confidence": "low",
            "rationale": "Temporary manual CLI test fixture: pending role review to exercise mixed-family queue behavior.",
            "source": "llm"
          }
        ]
      },
      "occurrences": [
        {
          "session_id": "2026-04-16T23-00-30-781Z-review_queue-creature-combatant_npc",
          "line": 1
        }
      ]
    },
    {
      "row": {
        "recordKey": "agents-of-edgewatch-bestiary:BLRsSDFSMbZHcGDQ",
        "name": "Black Whale Guard",
        "category": "creature",
        "resolutionStatus": "complete",
        "decisions": [
          {
            "kind": "assignment",
            "family": "setting",
            "tag": "coastal_setting",
            "mode": "include",
            "status": "rejected",
            "confidence": "medium",
            "rationale": "Temporary manual CLI test fixture: pending coastal-setting review for CLI testing.",
            "source": "llm"
          }
        ]
      },
      "occurrences": [
        {
          "session_id": "2026-04-16T23-02-38-860Z-review_queue-creature-coastal_setting",
          "line": 1
        }
      ]
    },
    {
      "row": {
        "recordKey": "pathfinder-npc-core:MxcprNbX7hcpAU8p",
        "name": "Departmental Chair",
        "category": "creature",
        "resolutionStatus": "complete",
        "decisions": [
          {
            "kind": "assignment",
            "family": "setting",
            "tag": "urban_setting",
            "mode": "include",
            "status": "rejected",
            "confidence": "low",
            "rationale": "Temporary manual CLI test fixture: pending review item for queue and session testing.",
            "source": "llm"
          }
        ]
      },
      "occurrences": [
        {
          "session_id": "2026-04-16T23-06-41-878Z-review_queue-creature-urban_setting",
          "line": 1
        }
      ]
    },
    {
      "row": {
        "recordKey": "pathfinder-npc-core:OAxxUyACpMlX3q1X",
        "name": "False Priest",
        "category": "creature",
        "resolutionStatus": "complete",
        "decisions": [
          {
            "kind": "assignment",
            "family": "setting",
            "tag": "urban_setting",
            "mode": "exclude",
            "status": "approved",
            "confidence": "medium",
            "rationale": "Temporary manual CLI test fixture: pending exclusion review for queue and TUI testing.",
            "source": "llm"
          }
        ]
      },
      "occurrences": [
        {
          "session_id": "2026-04-16T23-06-41-878Z-review_queue-creature-urban_setting",
          "line": 2
        }
      ]
    },
    {
      "row": {
        "recordKey": "spells-srd:IT1aaqDBAISlHDUV",
        "name": "Achaekek's Clutch",
        "category": "spell",
        "resolutionStatus": "needs_review",
        "decisions": [
          {
            "kind": "assignment",
            "family": "impact",
            "tag": "persistent_damage",
            "mode": "include",
            "status": "approved",
            "confidence": "medium",
            "rationale": "Legacy seed migration currently supplies this tag; review whether to convert it into an explicit assignment.",
            "source": "llm"
          },
          {
            "kind": "exemplar",
            "tag": "persistent_damage",
            "polarity": "positive",
            "action": "keep",
            "status": "needs_review",
            "confidence": "medium",
            "rationale": "Legacy seed candidate should be reviewed to decide whether it remains a true exemplar.",
            "source": "llm",
            "currentPolarity": "none"
          }
        ]
      },
      "occurrences": [
        {
          "session_id": "2026-04-17T05-53-17-694Z-legacy_seed-all-batch",
          "line": 1
        }
      ]
    },
    {
      "row": {
        "recordKey": "season-of-ghosts-bestiary:V67VC975O8iC1Yq2",
        "name": "Animated Axe",
        "category": "creature",
        "resolutionStatus": "needs_review",
        "decisions": [
          {
            "kind": "assignment",
            "family": "bound_object",
            "tag": "animated_object",
            "mode": "include",
            "status": "approved",
            "confidence": "medium",
            "rationale": "Legacy seed migration currently supplies this tag; review whether to convert it into an explicit assignment.",
            "source": "llm"
          },
          {
            "kind": "exemplar",
            "tag": "animated_object",
            "polarity": "positive",
            "action": "keep",
            "status": "needs_review",
            "confidence": "medium",
            "rationale": "Legacy seed candidate should be reviewed to decide whether it remains a true exemplar.",
            "source": "llm",
            "currentPolarity": "none"
          }
        ]
      },
      "occurrences": [
        {
          "session_id": "2026-04-17T05-54-02-741Z-legacy_seed-all-batch",
          "line": 2
        }
      ]
    },
    {
      "row": {
        "recordKey": "pathfinder-monster-core:TGYELuImcTcuX0aH",
        "name": "Conspirator Dragon (Adult)",
        "category": "creature",
        "resolutionStatus": "complete",
        "decisions": [
          {
            "kind": "exemplar",
            "tag": "urban_setting",
            "polarity": "positive",
            "action": "keep",
            "status": "approved",
            "confidence": "medium",
            "rationale": "Review whether this creature remains a strong positive exemplar for \"urban_setting\".",
            "source": "llm",
            "currentPolarity": "positive"
          }
        ]
      },
      "occurrences": [
        {
          "session_id": "2026-04-17T09-09-45-582Z-exemplar_cleanup-all-batch",
          "line": 1
        }
      ]
    }
  ],
  "representative_rationale_rows": [
    {
      "row": {
        "recordKey": "pathfinder-npc-core:MxcprNbX7hcpAU8p",
        "name": "Departmental Chair",
        "category": "creature",
        "resolutionStatus": "needs_review",
        "decisions": [
          {
            "kind": "assignment",
            "family": "setting",
            "tag": "urban_setting",
            "mode": "include",
            "status": "needs_review",
            "confidence": "low",
            "rationale": "Temporary manual CLI test fixture: pending review item for queue and session testing.",
            "source": "llm"
          }
        ]
      },
      "occurrence": {
        "session_id": "2026-04-16T22-21-14-252Z-review_queue-creature-batch",
        "line": 1
      }
    },
    {
      "row": {
        "recordKey": "pathfinder-npc-core:OAxxUyACpMlX3q1X",
        "name": "False Priest",
        "category": "creature",
        "resolutionStatus": "needs_review",
        "decisions": [
          {
            "kind": "assignment",
            "family": "setting",
            "tag": "urban_setting",
            "mode": "exclude",
            "status": "needs_review",
            "confidence": "medium",
            "rationale": "Temporary manual CLI test fixture: pending exclusion review for queue and TUI testing.",
            "source": "llm"
          }
        ]
      },
      "occurrence": {
        "session_id": "2026-04-16T22-21-14-252Z-review_queue-creature-batch",
        "line": 2
      }
    },
    {
      "row": {
        "recordKey": "agents-of-edgewatch-bestiary:BLRsSDFSMbZHcGDQ",
        "name": "Black Whale Guard",
        "category": "creature",
        "resolutionStatus": "needs_review",
        "decisions": [
          {
            "kind": "assignment",
            "family": "setting",
            "tag": "coastal_setting",
            "mode": "include",
            "status": "needs_review",
            "confidence": "medium",
            "rationale": "Temporary manual CLI test fixture: pending coastal-setting review for CLI testing.",
            "source": "llm"
          }
        ]
      },
      "occurrence": {
        "session_id": "2026-04-16T22-53-44-459Z-review_queue-all-batch",
        "line": 1
      }
    },
    {
      "row": {
        "recordKey": "pathfinder-monster-core:TGYELuImcTcuX0aH",
        "name": "Conspirator Dragon (Adult)",
        "category": "creature",
        "resolutionStatus": "needs_review",
        "decisions": [
          {
            "kind": "assignment",
            "family": "encounter_role",
            "tag": "combatant_npc",
            "mode": "include",
            "status": "needs_review",
            "confidence": "low",
            "rationale": "Temporary manual CLI test fixture: pending role review to exercise mixed-family queue behavior.",
            "source": "llm"
          }
        ]
      },
      "occurrence": {
        "session_id": "2026-04-16T22-53-44-459Z-review_queue-all-batch",
        "line": 2
      }
    },
    {
      "row": {
        "recordKey": "spells-srd:IT1aaqDBAISlHDUV",
        "name": "Achaekek's Clutch",
        "category": "spell",
        "resolutionStatus": "needs_review",
        "decisions": [
          {
            "kind": "assignment",
            "family": "attrition",
            "tag": "persistent_damage",
            "mode": "include",
            "status": "needs_review",
            "confidence": "medium",
            "rationale": "Legacy seed migration currently supplies this tag; review whether to convert it into an explicit assignment.",
            "source": "llm"
          },
          {
            "kind": "exemplar",
            "tag": "persistent_damage",
            "polarity": "positive",
            "action": "keep",
            "status": "needs_review",
            "rationale": "Legacy seed candidate should be reviewed to decide whether it remains a true exemplar.",
            "source": "llm"
          }
        ]
      },
      "occurrence": {
        "session_id": "2026-04-16T23-07-11-223Z-legacy_seed-all-batch",
        "line": 1
      }
    },
    {
      "row": {
        "recordKey": "spells-srd:IT1aaqDBAISlHDUV",
        "name": "Achaekek's Clutch",
        "category": "spell",
        "resolutionStatus": "needs_review",
        "decisions": [
          {
            "kind": "assignment",
            "family": "attrition",
            "tag": "persistent_damage",
            "mode": "include",
            "status": "needs_review",
            "confidence": "medium",
            "rationale": "Legacy seed migration currently supplies this tag; review whether to convert it into an explicit assignment.",
            "source": "llm"
          },
          {
            "kind": "exemplar",
            "tag": "persistent_damage",
            "polarity": "positive",
            "action": "keep",
            "status": "needs_review",
            "rationale": "Legacy seed candidate should be reviewed to decide whether it remains a true exemplar.",
            "source": "llm"
          }
        ]
      },
      "occurrence": {
        "session_id": "2026-04-16T23-07-11-223Z-legacy_seed-all-batch",
        "line": 1
      }
    },
    {
      "row": {
        "recordKey": "pathfinder-monster-core:TGYELuImcTcuX0aH",
        "name": "Conspirator Dragon (Adult)",
        "category": "creature",
        "resolutionStatus": "needs_review",
        "decisions": [
          {
            "kind": "exemplar",
            "tag": "urban_setting",
            "polarity": "positive",
            "action": "keep",
            "status": "needs_review",
            "confidence": "medium",
            "rationale": "Review whether this creature remains a strong positive exemplar for \"urban_setting\".",
            "source": "llm",
            "currentPolarity": "positive"
          }
        ]
      },
      "occurrence": {
        "session_id": "2026-04-17T08-48-12-764Z-exemplar_cleanup-all-batch",
        "line": 1
      }
    }
  ],
  "migration_manifests": [
    {
      "session_id": "2026-04-16T22-21-14-252Z-review_queue-creature-batch",
      "manifest": {
        "id": "2026-04-16T22-21-14-252Z-review_queue-creature-batch",
        "mode": "review_queue",
        "category": "creature",
        "createdAt": "2026-04-16T22:21:14.253Z",
        "recordCount": 2
      }
    },
    {
      "session_id": "2026-04-16T22-49-41-352Z-review_queue-creature-urban_setting",
      "manifest": {
        "id": "2026-04-16T22-49-41-352Z-review_queue-creature-urban_setting",
        "mode": "review_queue",
        "category": "creature",
        "family": "setting",
        "tag": "urban_setting",
        "createdAt": "2026-04-16T22:49:41.353Z",
        "recordCount": 2
      }
    },
    {
      "session_id": "2026-04-16T22-53-44-459Z-review_queue-all-batch",
      "manifest": {
        "id": "2026-04-16T22-53-44-459Z-review_queue-all-batch",
        "mode": "review_queue",
        "createdAt": "2026-04-16T22:53:44.460Z",
        "recordCount": 4
      }
    },
    {
      "session_id": "2026-04-16T23-00-30-781Z-review_queue-creature-combatant_npc",
      "manifest": {
        "id": "2026-04-16T23-00-30-781Z-review_queue-creature-enforcer_npc",
        "mode": "review_queue",
        "category": "creature",
        "family": "encounter_role",
        "tag": "enforcer_npc",
        "createdAt": "2026-04-16T23:00:30.782Z",
        "recordCount": 1
      }
    },
    {
      "session_id": "2026-04-16T23-02-38-860Z-review_queue-creature-coastal_setting",
      "manifest": {
        "id": "2026-04-16T23-02-38-860Z-review_queue-creature-coastal_setting",
        "mode": "review_queue",
        "category": "creature",
        "family": "setting",
        "tag": "coastal_setting",
        "createdAt": "2026-04-16T23:02:38.861Z",
        "recordCount": 1
      }
    },
    {
      "session_id": "2026-04-16T23-03-09-273Z-review_queue-creature-urban_setting",
      "manifest": {
        "id": "2026-04-16T23-03-09-273Z-review_queue-creature-urban_setting",
        "mode": "review_queue",
        "category": "creature",
        "family": "setting",
        "tag": "urban_setting",
        "createdAt": "2026-04-16T23:03:09.273Z",
        "recordCount": 2
      }
    },
    {
      "session_id": "2026-04-16T23-03-11-477Z-review_queue-creature-coastal_setting",
      "manifest": {
        "id": "2026-04-16T23-03-11-477Z-review_queue-creature-coastal_setting",
        "mode": "review_queue",
        "category": "creature",
        "family": "setting",
        "tag": "coastal_setting",
        "createdAt": "2026-04-16T23:03:11.477Z",
        "recordCount": 1
      }
    },
    {
      "session_id": "2026-04-16T23-06-41-878Z-review_queue-creature-urban_setting",
      "manifest": {
        "id": "2026-04-16T23-06-41-878Z-review_queue-creature-urban_setting",
        "mode": "review_queue",
        "category": "creature",
        "family": "setting",
        "tag": "urban_setting",
        "createdAt": "2026-04-16T23:06:41.879Z",
        "recordCount": 2
      }
    },
    {
      "session_id": "2026-04-16T23-07-11-223Z-legacy_seed-all-batch",
      "manifest": {
        "id": "2026-04-16T23-07-11-223Z-legacy_seed-all-batch",
        "mode": "legacy_seed",
        "createdAt": "2026-04-16T23:07:11.223Z",
        "recordCount": 496
      }
    },
    {
      "session_id": "2026-04-17T05-53-17-694Z-legacy_seed-all-batch",
      "manifest": {
        "id": "2026-04-17T05-53-17-694Z-legacy_seed-all-batch",
        "mode": "legacy_seed",
        "createdAt": "2026-04-17T05:53:17.695Z",
        "recordCount": 496
      }
    },
    {
      "session_id": "2026-04-17T05-53-55-188Z-legacy_seed-affliction-batch",
      "manifest": {
        "id": "2026-04-17T05-53-55-188Z-legacy_seed-affliction-batch",
        "mode": "legacy_seed",
        "category": "affliction",
        "createdAt": "2026-04-17T05:53:55.188Z",
        "recordCount": 472
      }
    },
    {
      "session_id": "2026-04-17T05-54-02-741Z-legacy_seed-all-batch",
      "manifest": {
        "id": "2026-04-17T05-54-02-741Z-legacy_seed-all-batch",
        "mode": "legacy_seed",
        "createdAt": "2026-04-17T05:54:02.741Z",
        "recordCount": 50
      }
    },
    {
      "session_id": "2026-04-17T05-59-15-977Z-legacy_seed-all-batch",
      "manifest": {
        "id": "2026-04-17T05-59-15-977Z-legacy_seed-all-batch",
        "mode": "legacy_seed",
        "createdAt": "2026-04-17T05:59:15.977Z",
        "recordCount": 496
      }
    },
    {
      "session_id": "2026-04-17T08-00-38-085Z-legacy_seed-affliction-compulsion",
      "manifest": {
        "id": "2026-04-17T08-00-38-085Z-legacy_seed-affliction-compulsion",
        "mode": "legacy_seed",
        "category": "affliction",
        "tag": "compulsion",
        "createdAt": "2026-04-17T08:00:38.085Z",
        "recordCount": 472
      }
    },
    {
      "session_id": "2026-04-17T08-01-00-201Z-new_tagging-creature-batch",
      "manifest": {
        "id": "2026-04-17T08-01-00-201Z-new_tagging-creature-batch",
        "mode": "new_tagging",
        "category": "creature",
        "createdAt": "2026-04-17T08:01:00.201Z",
        "recordCount": 1653
      }
    },
    {
      "session_id": "2026-04-17T08-27-15-168Z-new_tagging-all-batch",
      "manifest": {
        "id": "2026-04-17T08-27-15-168Z-new_tagging-all-batch",
        "mode": "new_tagging",
        "createdAt": "2026-04-17T08:27:15.168Z",
        "recordCount": 18966
      }
    },
    {
      "session_id": "2026-04-17T08-35-58-568Z-new_tagging-affliction-batch",
      "manifest": {
        "id": "2026-04-17T08-35-58-568Z-new_tagging-affliction-batch",
        "mode": "new_tagging",
        "category": "affliction",
        "createdAt": "2026-04-17T08:35:58.569Z",
        "recordCount": 30
      }
    },
    {
      "session_id": "2026-04-17T08-36-14-597Z-new_tagging-all-batch",
      "manifest": {
        "id": "2026-04-17T08-36-14-597Z-new_tagging-all-batch",
        "mode": "new_tagging",
        "createdAt": "2026-04-17T08:36:14.598Z",
        "recordCount": 50
      }
    },
    {
      "session_id": "2026-04-17T08-37-10-672Z-new_tagging-all-batch",
      "manifest": {
        "id": "2026-04-17T08-37-10-672Z-new_tagging-all-batch",
        "mode": "new_tagging",
        "createdAt": "2026-04-17T08:37:10.672Z",
        "recordCount": 18966
      }
    },
    {
      "session_id": "2026-04-17T08-48-12-764Z-exemplar_cleanup-all-batch",
      "manifest": {
        "id": "2026-04-17T08-48-12-764Z-exemplar_cleanup-all-batch",
        "mode": "exemplar_cleanup",
        "createdAt": "2026-04-17T08:48:12.765Z",
        "recordCount": 2
      }
    },
    {
      "session_id": "2026-04-17T08-56-02-696Z-proposal_review-all-batch",
      "manifest": {
        "id": "2026-04-17T08-56-02-696Z-proposal_review-all-batch",
        "mode": "proposal_review",
        "createdAt": "2026-04-17T08:56:02.697Z",
        "recordCount": 26377
      }
    },
    {
      "session_id": "2026-04-17T08-57-15-299Z-exemplar_cleanup-affliction-contact_exposure",
      "manifest": {
        "id": "2026-04-17T08-57-15-299Z-exemplar_cleanup-affliction-contact_exposure",
        "mode": "exemplar_cleanup",
        "category": "affliction",
        "tag": "contact_exposure",
        "createdAt": "2026-04-17T08:57:15.299Z",
        "recordCount": 472
      }
    },
    {
      "session_id": "2026-04-17T09-02-53-025Z-proposal_review-all-batch",
      "manifest": {
        "id": "2026-04-17T09-02-53-025Z-proposal_review-all-batch",
        "mode": "proposal_review",
        "createdAt": "2026-04-17T09:02:53.026Z",
        "recordCount": 0
      }
    },
    {
      "session_id": "2026-04-17T09-03-12-088Z-exemplar_cleanup-all-batch",
      "manifest": {
        "id": "2026-04-17T09-03-12-088Z-exemplar_cleanup-all-batch",
        "mode": "exemplar_cleanup",
        "createdAt": "2026-04-17T09:03:12.088Z",
        "recordCount": 2
      }
    },
    {
      "session_id": "2026-04-17T09-09-45-582Z-exemplar_cleanup-all-batch",
      "manifest": {
        "id": "2026-04-17T09-09-45-582Z-exemplar_cleanup-all-batch",
        "mode": "exemplar_cleanup",
        "createdAt": "2026-04-17T09:09:45.583Z",
        "recordCount": 2
      }
    },
    {
      "session_id": "2026-04-17T09-10-02-517Z-exemplar_cleanup-all-batch",
      "manifest": {
        "id": "2026-04-17T09-10-02-517Z-exemplar_cleanup-all-batch",
        "mode": "exemplar_cleanup",
        "createdAt": "2026-04-17T09:10:02.517Z",
        "recordCount": 2
      }
    },
    {
      "session_id": "2026-04-17T09-14-06-713Z-exemplar_cleanup-all-batch",
      "manifest": {
        "id": "2026-04-17T09-14-06-713Z-exemplar_cleanup-all-batch",
        "mode": "exemplar_cleanup",
        "createdAt": "2026-04-17T09:14:06.714Z",
        "recordCount": 2
      }
    },
    {
      "session_id": "2026-04-17T09-17-48-126Z-exemplar_cleanup-all-batch",
      "manifest": {
        "id": "2026-04-17T09-17-48-126Z-exemplar_cleanup-all-batch",
        "mode": "exemplar_cleanup",
        "createdAt": "2026-04-17T09:17:48.127Z",
        "recordCount": 2
      }
    },
    {
      "session_id": "2026-04-17T09-34-54-005Z-exemplar_cleanup-all-batch",
      "manifest": {
        "id": "2026-04-17T09-34-54-005Z-exemplar_cleanup-all-batch",
        "mode": "exemplar_cleanup",
        "createdAt": "2026-04-17T09:34:54.006Z",
        "recordCount": 2
      }
    },
    {
      "session_id": "2026-04-17T09-46-34-237Z-exemplar_cleanup-all-batch",
      "manifest": {
        "id": "2026-04-17T09-46-34-237Z-exemplar_cleanup-all-batch",
        "mode": "exemplar_cleanup",
        "createdAt": "2026-04-17T09:46:34.238Z",
        "recordCount": 2
      }
    },
    {
      "session_id": "2026-04-17T10-19-15-069Z-exemplar_cleanup-all-batch",
      "manifest": {
        "id": "2026-04-17T10-19-15-069Z-exemplar_cleanup-all-batch",
        "mode": "exemplar_cleanup",
        "createdAt": "2026-04-17T10:19:15.069Z",
        "recordCount": 2
      }
    },
    {
      "session_id": "2026-04-17T10-19-28-744Z-legacy_seed-all-batch",
      "manifest": {
        "id": "2026-04-17T10-19-28-744Z-legacy_seed-all-batch",
        "mode": "legacy_seed",
        "createdAt": "2026-04-17T10:19:28.744Z",
        "recordCount": 496
      }
    },
    {
      "session_id": "2026-04-18T19-11-56-099Z-exemplar_cleanup-creature-urban_setting",
      "manifest": {
        "id": "2026-04-18T19-11-56-099Z-exemplar_cleanup-creature-urban_setting",
        "mode": "exemplar_cleanup",
        "category": "creature",
        "tag": "urban_setting",
        "createdAt": "2026-04-18T19:11:56.099Z",
        "recordCount": 2
      }
    },
    {
      "session_id": "2026-04-18T19-53-04-719Z-exemplar_cleanup-creature-batch",
      "manifest": {
        "id": "2026-04-18T19-53-04-719Z-exemplar_cleanup-creature-batch",
        "mode": "exemplar_cleanup",
        "category": "creature",
        "createdAt": "2026-04-18T19:53:04.719Z",
        "recordCount": 2
      }
    },
    {
      "session_id": "2026-04-18T20-00-14-131Z-exemplar_cleanup-all-batch",
      "manifest": {
        "id": "2026-04-18T20-00-14-131Z-exemplar_cleanup-all-batch",
        "mode": "exemplar_cleanup",
        "createdAt": "2026-04-18T20:00:14.131Z",
        "recordCount": 2
      }
    },
    {
      "session_id": "2026-04-23T15-11-31-627Z-proposal_review-all-batch",
      "manifest": {
        "id": "2026-04-23T15-11-31-627Z-proposal_review-all-batch",
        "mode": "proposal_review",
        "createdAt": "2026-04-23T15:11:31.627Z",
        "recordCount": 0
      }
    },
    {
      "session_id": "2026-04-23T15-12-55-752Z-exemplar_cleanup-all-batch",
      "manifest": {
        "id": "2026-04-23T15-12-55-752Z-exemplar_cleanup-all-batch",
        "mode": "exemplar_cleanup",
        "createdAt": "2026-04-23T15:12:55.752Z",
        "recordCount": 2
      }
    },
    {
      "session_id": "2026-05-04T07-44-41-808Z-legacy_seed-all-batch",
      "manifest": {
        "id": "2026-05-04T07-44-41-808Z-legacy_seed-all-batch",
        "mode": "legacy_seed",
        "createdAt": "2026-05-04T07:44:41.808Z",
        "recordCount": 496
      }
    }
  ],
  "translation_summary": {
    "sessions": 55,
    "row_snapshots": 2593,
    "imported_sessions": 0,
    "changed_override_rows": 0
  },
  "translation_format_example": {
    "selection": "One row from the identified session, not its complete rows collection",
    "manifest": {
      "id": "translation-2026-05-07T21-22-14-496Z",
      "createdAt": "2026-05-07T21:22:14.496Z",
      "rowCount": 244
    },
    "row_example": {
      "key": "creature:abaddon_setting",
      "base": {
        "currentCategory": "creature",
        "currentBrowseAxis": "setting",
        "currentFamily": "planar_setting",
        "currentTag": "abaddon_setting",
        "currentAssignmentMode": "hybrid",
        "translationStatus": "provisional",
        "canonicalConceptId": "abaddon_setting",
        "canonicalConceptLabel": "abaddon_setting",
        "schemaKind": "descriptive",
        "domainId": "",
        "operation": "",
        "primaryFacetKind": "setting",
        "primaryFacetValue": "planar",
        "secondaryFacets": [],
        "projectionAxis": "setting",
        "projectionFamily": "planar_setting",
        "publishTag": true
      },
      "currentOverride": {},
      "draftOverride": {}
    },
    "reviewState": {
      "currentIndex": 0,
      "categoryFilter": "all",
      "statusFilter": "all",
      "imported": false,
      "updatedAt": "2026-05-07T21:22:14.496Z"
    }
  },
  "original_files": [
    {
      "path": "scratch/migration-sessions/2026-04-16T22-21-14-252Z-review_queue-creature-batch/decisions.jsonl",
      "bytes": 776,
      "sha256": "b8a4f18f5bd4cbabba6eb3f4cc2a82cac449a9386e8e05eb569caa4442b0bf6a"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T22-21-14-252Z-review_queue-creature-batch/manifest.json",
      "bytes": 182,
      "sha256": "78570bbd881ac52033e5dc40872eb4deb03b81093aea91047ebd1c662bb0ca86"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T22-21-14-252Z-review_queue-creature-batch/records.jsonl",
      "bytes": 1919,
      "sha256": "de65ec3dfb84ce88238a66ef332bb9f8fefa10e0247f9e399a3f9f21851c4f42"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T22-21-14-252Z-review_queue-creature-batch/review-state.json",
      "bytes": 93,
      "sha256": "051c3941ad6ac1ebad6c9b3f7b0d6a8fae88cadf9174b810f6396cf35fc989c1"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T22-21-14-252Z-review_queue-creature-batch/summary.md",
      "bytes": 189,
      "sha256": "724154ccd091a6debdf2349365f4f0d36d71f53292128559a0bb390c05228a78"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T22-49-41-352Z-review_queue-creature-urban_setting/decisions.jsonl",
      "bytes": 776,
      "sha256": "b8a4f18f5bd4cbabba6eb3f4cc2a82cac449a9386e8e05eb569caa4442b0bf6a"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T22-49-41-352Z-review_queue-creature-urban_setting/manifest.json",
      "bytes": 239,
      "sha256": "be8c624b88f62dc807ae41074fd26a556f1510642f744396ba58133dbe3c8903"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T22-49-41-352Z-review_queue-creature-urban_setting/records.jsonl",
      "bytes": 1919,
      "sha256": "de65ec3dfb84ce88238a66ef332bb9f8fefa10e0247f9e399a3f9f21851c4f42"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T22-49-41-352Z-review_queue-creature-urban_setting/review-state.json",
      "bytes": 93,
      "sha256": "49bc1d3d488942defca269fe89445f5b2eac494c0c29e11eb022f3b83c7a4c9c"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T22-49-41-352Z-review_queue-creature-urban_setting/summary.md",
      "bytes": 247,
      "sha256": "8c177c505b08d950ae64d62312bb90c67969f3f3e687675fbe5e0c53208fab0e"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T22-53-44-459Z-review_queue-all-batch/decisions.jsonl",
      "bytes": 1593,
      "sha256": "367e4e37b94f2e7c304d1b51275d9d3f1eea0435f9769c8eb7bce585a47beefd"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T22-53-44-459Z-review_queue-all-batch/manifest.json",
      "bytes": 151,
      "sha256": "248936db48b9e404c7e985f1a89f57ce7b92d4b143bff4cb0d7e8a2ebfa93913"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T22-53-44-459Z-review_queue-all-batch/records.jsonl",
      "bytes": 4457,
      "sha256": "76f8795b0138504f06c32f894ab29e28209014303437264e8b69842b4d211c54"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T22-53-44-459Z-review_queue-all-batch/review-state.json",
      "bytes": 93,
      "sha256": "451e07b2dcacf9f5e1f33aa5b8cec225ddf277f6f58fec104640aaec4f710423"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T22-53-44-459Z-review_queue-all-batch/summary.md",
      "bytes": 223,
      "sha256": "d6d9332fe188f5df8929705280ed4076b606a75fd6b9ad7a721b38c8435d8a57"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-00-30-781Z-review_queue-creature-combatant_npc/decisions.jsonl",
      "bytes": 410,
      "sha256": "e917027047eab9d37d715a15a272bc0cdff6dcefdaddfa416bca8bc0db11fd55"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-00-30-781Z-review_queue-creature-combatant_npc/manifest.json",
      "bytes": 244,
      "sha256": "e1fe123dc26eaaa632f0d653b6f5e54c5768dc01f3fa0fb5cb9587e38adb82d8"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-00-30-781Z-review_queue-creature-combatant_npc/records.jsonl",
      "bytes": 1854,
      "sha256": "b52123df6595b663d2afdb822996e1aab952cc7f5914c0074b88d74404f04664"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-00-30-781Z-review_queue-creature-combatant_npc/review-state.json",
      "bytes": 94,
      "sha256": "141bcc4edc2822f5a6cf8cba00f6a9e1fc23eb7c51f245351acffe44bc9ae06a"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-00-30-781Z-review_queue-creature-combatant_npc/summary.md",
      "bytes": 232,
      "sha256": "b492739f55e1c49e53e4f2466c9856b255773583fa3cc7bf58cccd134318bff7"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-02-38-860Z-review_queue-creature-coastal_setting/decisions.jsonl",
      "bytes": 391,
      "sha256": "e178c791231c448a9944f3bb5eeaabbda2f2e9d4409cd1fdf3b449acccb4eac3"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-02-38-860Z-review_queue-creature-coastal_setting/manifest.json",
      "bytes": 243,
      "sha256": "9910df46499967dc0c41fa64de0250fc1878691809fc63c1a6d7fc5103bfacb6"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-02-38-860Z-review_queue-creature-coastal_setting/records.jsonl",
      "bytes": 684,
      "sha256": "1861daa68b9818a76d2e9a9cb404b6639ecc4fbab44c28a9df8927ad9f686b16"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-02-38-860Z-review_queue-creature-coastal_setting/review-state.json",
      "bytes": 93,
      "sha256": "47b7be282cf0a7b9a9001adf47eb570ef35612fe546fff0913b26de25fbba1c5"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-02-38-860Z-review_queue-creature-coastal_setting/summary.md",
      "bytes": 251,
      "sha256": "c75882a4f687745353bb3aa9fad52673d8a1396afb994b38e56b8276ecc3854f"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-03-09-273Z-review_queue-creature-urban_setting/decisions.jsonl",
      "bytes": 776,
      "sha256": "b8a4f18f5bd4cbabba6eb3f4cc2a82cac449a9386e8e05eb569caa4442b0bf6a"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-03-09-273Z-review_queue-creature-urban_setting/manifest.json",
      "bytes": 239,
      "sha256": "e7210b8113de37b2c8576c93a3c0e14c275688fcfd95a1e5494efa36415c2fee"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-03-09-273Z-review_queue-creature-urban_setting/records.jsonl",
      "bytes": 1919,
      "sha256": "de65ec3dfb84ce88238a66ef332bb9f8fefa10e0247f9e399a3f9f21851c4f42"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-03-09-273Z-review_queue-creature-urban_setting/review-state.json",
      "bytes": 93,
      "sha256": "fea82b2e15bd87050cbe0d93aafb427fc49a416a1b5d8107272f8b76d95d9ace"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-03-09-273Z-review_queue-creature-urban_setting/summary.md",
      "bytes": 247,
      "sha256": "151af65e1fdef800a9c9c8d167999ea17e3513a15f2bab3f008f33abfe5554c9"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-03-11-477Z-review_queue-creature-coastal_setting/decisions.jsonl",
      "bytes": 399,
      "sha256": "4766a12d32db4da102fa3eb7340517c52c81e68e4bde4252ebfba333ce74af2c"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-03-11-477Z-review_queue-creature-coastal_setting/manifest.json",
      "bytes": 243,
      "sha256": "0687599f9ad157b9d8a6a6c8c365e1fc25b809035f2fcb67efde81a442fee20b"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-03-11-477Z-review_queue-creature-coastal_setting/records.jsonl",
      "bytes": 684,
      "sha256": "1861daa68b9818a76d2e9a9cb404b6639ecc4fbab44c28a9df8927ad9f686b16"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-03-11-477Z-review_queue-creature-coastal_setting/review-state.json",
      "bytes": 93,
      "sha256": "c3532b7d030dbd8a4fedd1d4afd9be9bf25af8c8683e6f8446d29f43b8367918"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-03-11-477Z-review_queue-creature-coastal_setting/summary.md",
      "bytes": 251,
      "sha256": "34a930951fc5915672c12bcbd9b0a3f5af10662366fb5ad7411e3e0604a124e9"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-06-41-878Z-review_queue-creature-urban_setting/decisions.jsonl",
      "bytes": 760,
      "sha256": "8cbfe7896704e745fe7b73a0e865d1cbe57df31e6334b6afcb8a8ac8dd242694"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-06-41-878Z-review_queue-creature-urban_setting/manifest.json",
      "bytes": 239,
      "sha256": "26ea54db1fd15c04871cce17791972cdec8b5a9f3423a8c7775095bceddce628"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-06-41-878Z-review_queue-creature-urban_setting/records.jsonl",
      "bytes": 1919,
      "sha256": "de65ec3dfb84ce88238a66ef332bb9f8fefa10e0247f9e399a3f9f21851c4f42"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-06-41-878Z-review_queue-creature-urban_setting/review-state.json",
      "bytes": 94,
      "sha256": "3ce120124de07744c57419ebef3fc7a59aee7f7b58c23f4c2fc5e0c2b4d46a2e"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-06-41-878Z-review_queue-creature-urban_setting/summary.md",
      "bytes": 234,
      "sha256": "d1c13877e337a525b3c004eecf2477578b601cad16ffa2911c9986e58593d738"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-07-11-223Z-legacy_seed-all-batch/decisions.jsonl",
      "bytes": 370099,
      "sha256": "718fbb403ee7f7a826b9f480206d4d45f1e105205e382266ad870d6c0c2e4056"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-07-11-223Z-legacy_seed-all-batch/manifest.json",
      "bytes": 151,
      "sha256": "0c683c3fee522b902d909cc71b5f117f8b2da22299b565d45f362f8a2bb00c60"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-07-11-223Z-legacy_seed-all-batch/records.jsonl",
      "bytes": 615751,
      "sha256": "d68158b43b0ea50c50cb9ba95a03378908d9b9de00e92dbdbb838d1ef26c04ca"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-07-11-223Z-legacy_seed-all-batch/review-state.json",
      "bytes": 93,
      "sha256": "15b7d508c3c648f70795b769fc047b49174c4a2915f9dba686b4870c571bf495"
    },
    {
      "path": "scratch/migration-sessions/2026-04-16T23-07-11-223Z-legacy_seed-all-batch/summary.md",
      "bytes": 228,
      "sha256": "f78f3d9f01ed04bdc4d1ce0d67dd63ad05fe7e7df9f7dd1e88392ef9d1580553"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T05-53-17-694Z-legacy_seed-all-batch/decisions.jsonl",
      "bytes": 397927,
      "sha256": "316d9032bd58b252ed8b2632cc10131e028a48e5e8587984f5ce300f56063b6b"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T05-53-17-694Z-legacy_seed-all-batch/manifest.json",
      "bytes": 151,
      "sha256": "ebe2c5db61718e8a8836a0a374f1717233c24c0ebc1388fa287faf6caf1d3f6e"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T05-53-17-694Z-legacy_seed-all-batch/records.jsonl",
      "bytes": 615430,
      "sha256": "6c4c256d6f060025e9501af1cb7509199f280ba1e1ac03a0f902da377d68187d"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T05-53-17-694Z-legacy_seed-all-batch/review-state.json",
      "bytes": 94,
      "sha256": "ff66e45677a835a91b3974ad911e8d5ee5693e9e6d844b13055a11c6f8240580"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T05-53-17-694Z-legacy_seed-all-batch/summary.md",
      "bytes": 138,
      "sha256": "ac37b60a71c90c3aa65217b5ac212bf286880b99ba4dc0784b60379705202559"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T05-53-55-188Z-legacy_seed-affliction-batch/decisions.jsonl",
      "bytes": 67627,
      "sha256": "9b9912c0d081210644d8fc5a02727089c6d19a32800bf3ef6cf4b49c5672f6b5"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T05-53-55-188Z-legacy_seed-affliction-batch/manifest.json",
      "bytes": 186,
      "sha256": "033461460b982067696a018d9a80d969493897c32230e2903ea51f5be6b8c884"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T05-53-55-188Z-legacy_seed-affliction-batch/records.jsonl",
      "bytes": 292755,
      "sha256": "656bc6274e8ed788710aa79207cbb50a658ccb3b197cbd1bbbd54339f3ec56c8"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T05-53-55-188Z-legacy_seed-affliction-batch/review-state.json",
      "bytes": 93,
      "sha256": "0beab4994823bb51af82508ad3cecd778098e66c18a8b62985160317a6ac249d"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T05-53-55-188Z-legacy_seed-affliction-batch/summary.md",
      "bytes": 142,
      "sha256": "047e266f8f0101c82da4217fcc7c137d87dfb16ed7c8e9986bf93f9067c08f41"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T05-54-02-741Z-legacy_seed-all-batch/decisions.jsonl",
      "bytes": 58934,
      "sha256": "9f54b706e758e48c591535f03679203ed2cae936084e9819c7ed4fdca4d4d41e"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T05-54-02-741Z-legacy_seed-all-batch/manifest.json",
      "bytes": 150,
      "sha256": "e0160134d60f22422ef6a0f71dc467490eed7c86c72b68712e50ece8f0596d1a"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T05-54-02-741Z-legacy_seed-all-batch/records.jsonl",
      "bytes": 49275,
      "sha256": "c62b248e82f397e9815b9d73df3f9b354ac8fc285ce5aeacb649bbd2cba12183"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T05-54-02-741Z-legacy_seed-all-batch/review-state.json",
      "bytes": 94,
      "sha256": "25b0d8dd50a6007278e646dfe1cbdf90b22e4c093174a13a0c454cc9d3bd3c11"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T05-54-02-741Z-legacy_seed-all-batch/summary.md",
      "bytes": 137,
      "sha256": "cd80ca011420d533f5b7d46cdec2a8965abd4c5f41df205124fe419d4724760a"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T05-59-15-977Z-legacy_seed-all-batch/decisions.jsonl",
      "bytes": 397931,
      "sha256": "b883c131bb4e9120b004dd8614604305cb4e76d307bf4d13816fd88d1f8f0cd5"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T05-59-15-977Z-legacy_seed-all-batch/manifest.json",
      "bytes": 151,
      "sha256": "4fb0a5bf9bbec806ffa86c636647eef116e98fa0f079885396440d4ea60e0343"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T05-59-15-977Z-legacy_seed-all-batch/records.jsonl",
      "bytes": 615436,
      "sha256": "c4f685a49948f23e6f920363377373d59e17fd1e98f253f905c272ee6d964c7d"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T05-59-15-977Z-legacy_seed-all-batch/review-state.json",
      "bytes": 93,
      "sha256": "29a211881ba40ef57c4e4867349a3de46993cbe2ad157e745a58b8583c989471"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T05-59-15-977Z-legacy_seed-all-batch/summary.md",
      "bytes": 138,
      "sha256": "865428a63748459368ca2200b24db2c7f754764ee05b5b028dd4ccab9084251c"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-00-38-085Z-legacy_seed-affliction-compulsion/decisions.jsonl",
      "bytes": 67627,
      "sha256": "9b9912c0d081210644d8fc5a02727089c6d19a32800bf3ef6cf4b49c5672f6b5"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-00-38-085Z-legacy_seed-affliction-compulsion/manifest.json",
      "bytes": 214,
      "sha256": "acf8dc31e0650d1e3a6cb5c63725fcd1cb0692fe718587520612f534b571624a"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-00-38-085Z-legacy_seed-affliction-compulsion/records.jsonl",
      "bytes": 292755,
      "sha256": "656bc6274e8ed788710aa79207cbb50a658ccb3b197cbd1bbbd54339f3ec56c8"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-00-38-085Z-legacy_seed-affliction-compulsion/review-state.json",
      "bytes": 93,
      "sha256": "dc96bf0d904bbc338c160f5ef1c64f2fb28ba0e0a51b1f0c77749b5403944181"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-00-38-085Z-legacy_seed-affliction-compulsion/summary.md",
      "bytes": 147,
      "sha256": "8f08f02642f95b3c925b5146a0a7e162c499f6b4c55ea9b1611546539abc328d"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-01-00-201Z-new_tagging-creature-batch/decisions.jsonl",
      "bytes": 250144,
      "sha256": "3c232d9141618339e30a3aec87aa5a4b2ee4b72af939da22110eb40c10fe3e81"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-01-00-201Z-new_tagging-creature-batch/manifest.json",
      "bytes": 183,
      "sha256": "cc0cbd6c92a6f2dff6e5fc510d643f6ab23abdd285fbfe4c9846d52526f906ee"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-01-00-201Z-new_tagging-creature-batch/records.jsonl",
      "bytes": 1414685,
      "sha256": "8641192e8ab93efa9576508ac03806b1d21aebf7bcbf46b0c3f8b904377b669c"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-01-00-201Z-new_tagging-creature-batch/review-state.json",
      "bytes": 93,
      "sha256": "55152ad9b565e318441881378ad38dff8676f6a8795922284c9fd6fe92f95eeb"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-01-00-201Z-new_tagging-creature-batch/summary.md",
      "bytes": 140,
      "sha256": "03c1342cd8fd7c4b00792a63d15d4a62c961a4b13655d6045cce2ddc97d0c41c"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-27-15-168Z-new_tagging-all-batch/decisions.jsonl",
      "bytes": 2737795,
      "sha256": "cc0539a642339b75280716fff8cc7f39d7fcaa783405914fbb4424d04b29fc02"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-27-15-168Z-new_tagging-all-batch/manifest.json",
      "bytes": 153,
      "sha256": "8e80ad21fcaae012c9e797769796ad3b155c31b83240afe8c5b3c0fe51f85586"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-27-15-168Z-new_tagging-all-batch/records.jsonl",
      "bytes": 17322097,
      "sha256": "84a8a35f63783d47802517fd777f4a27199624ff9ed467e4377eecd8d2bf1b6d"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-27-15-168Z-new_tagging-all-batch/review-state.json",
      "bytes": 93,
      "sha256": "39da06b0af92e0db03321ea639b92efbcc91cd70db49a2b6fe599102daf1013f"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-27-15-168Z-new_tagging-all-batch/summary.md",
      "bytes": 135,
      "sha256": "87315fee46bc3c0e8da0e93586fa507f5c663601119668ef5b31302ec5b31429"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-35-58-568Z-new_tagging-affliction-batch/decisions.jsonl",
      "bytes": 4319,
      "sha256": "18d6b5220e5536d89818faa6c27c8c2f3f990dbaa02b4f6b32052dfc61ddecb0"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-35-58-568Z-new_tagging-affliction-batch/manifest.json",
      "bytes": 185,
      "sha256": "20bc3335a72dbd72931e89e30241bdc90b49bbc3406073d2822bcf2c5d120c6e"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-35-58-568Z-new_tagging-affliction-batch/records.jsonl",
      "bytes": 21083,
      "sha256": "c3e379e7f820ad780695a64ca3cf26e572cb1d438cf69eb52ae51a01362e2a23"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-35-58-568Z-new_tagging-affliction-batch/review-state.json",
      "bytes": 93,
      "sha256": "db952b02b29227e3ee0c347df82fbfe6768454e27f9e22a152fc5bf593544555"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-35-58-568Z-new_tagging-affliction-batch/summary.md",
      "bytes": 142,
      "sha256": "60608201a3871ff01487d57bd2131c0867b98d928d298998fcd2d23d07538940"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-36-14-597Z-new_tagging-all-batch/decisions.jsonl",
      "bytes": 8738,
      "sha256": "c7232a5643f0f71a6559d9071dad7c3f34f27736c04adbdbef20fbed93c57b1d"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-36-14-597Z-new_tagging-all-batch/manifest.json",
      "bytes": 150,
      "sha256": "88924abd2895c0c3cba81a8cdc0838ffe3d4cf53c7915af80e0c742524aa12b2"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-36-14-597Z-new_tagging-all-batch/records.jsonl",
      "bytes": 44615,
      "sha256": "5d16950f776fdbf9b17702f2a3130d6f124b6327d227552b5a276cdd7e93b008"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-36-14-597Z-new_tagging-all-batch/review-state.json",
      "bytes": 93,
      "sha256": "bf7af1aeb70f334b8a450a4cb6fdabbac882de39948d253181aa9c951aa4d748"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-36-14-597Z-new_tagging-all-batch/summary.md",
      "bytes": 135,
      "sha256": "bd4ce474dc5a6c7cba4e7b7bbe3404c16bb5179c8304141a1ddf820f55bea2d4"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-37-10-672Z-new_tagging-all-batch/decisions.jsonl",
      "bytes": 2737795,
      "sha256": "cc0539a642339b75280716fff8cc7f39d7fcaa783405914fbb4424d04b29fc02"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-37-10-672Z-new_tagging-all-batch/manifest.json",
      "bytes": 153,
      "sha256": "b40ef428064172037cfa80a94ca3c09ede69551918aec462821af59abf990d42"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-37-10-672Z-new_tagging-all-batch/records.jsonl",
      "bytes": 17322097,
      "sha256": "84a8a35f63783d47802517fd777f4a27199624ff9ed467e4377eecd8d2bf1b6d"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-37-10-672Z-new_tagging-all-batch/review-state.json",
      "bytes": 93,
      "sha256": "e231a93dc947d7c27799d2be0aecd28231dcb8327eb917bdaa42c1d6ea0b0177"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-37-10-672Z-new_tagging-all-batch/summary.md",
      "bytes": 135,
      "sha256": "0a01e49790a662b97a60cdb6b9f3f9d4801828968ad78754e2629ba83efc4804"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-48-12-764Z-exemplar_cleanup-all-batch/decisions.jsonl",
      "bytes": 857,
      "sha256": "a0d9ce581e7a3ebeafec2f04890dd6526a74af3806a4ba346cb723cad7248293"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-48-12-764Z-exemplar_cleanup-all-batch/manifest.json",
      "bytes": 159,
      "sha256": "d64d0a03846944412ec15e79abd2a0d7ba079ca137f30b1fedd1810fc1c7fa51"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-48-12-764Z-exemplar_cleanup-all-batch/records.jsonl",
      "bytes": 3924,
      "sha256": "84e361573bc080e60150a5ed8c3bc8adf1ff8ad334819c1ac44474484ca05c83"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-48-12-764Z-exemplar_cleanup-all-batch/review-state.json",
      "bytes": 93,
      "sha256": "83fc80718183f57b9c54888a34a6a20c33ef61a4a85673a4babe70b46aa3e61a"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-48-12-764Z-exemplar_cleanup-all-batch/summary.md",
      "bytes": 199,
      "sha256": "899958a329fe9a2fa7551ebff6c19a410e51a0143afab5305fdf294a07cf8dc2"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-56-02-696Z-proposal_review-all-batch/decisions.jsonl",
      "bytes": 3840723,
      "sha256": "614a537281bfd99084d51729895d24c6055fda95d1c16274882622b945e877ba"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-56-02-696Z-proposal_review-all-batch/manifest.json",
      "bytes": 161,
      "sha256": "7e4a356ef2bf9fba68cd846bdf05bd78ef6672826c7eed70377c29d3ab37ef14"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-56-02-696Z-proposal_review-all-batch/records.jsonl",
      "bytes": 24970337,
      "sha256": "1f1ddd1ccdf9f1f1f31878c0ee39265f6671a264c5588cad58e3fa6ec2d99a93"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-56-02-696Z-proposal_review-all-batch/review-state.json",
      "bytes": 93,
      "sha256": "3444a2ed6821c84cb8b827728232df7e8ad79100d830b575f500fc0f18cd52c2"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-56-02-696Z-proposal_review-all-batch/summary.md",
      "bytes": 195,
      "sha256": "d65bb6c71270c402ff338ce2ca2cf829262c4cc59cb61a75f30a9633f6ff2987"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-57-15-299Z-exemplar_cleanup-affliction-contact_exposure/decisions.jsonl",
      "bytes": 67627,
      "sha256": "9b9912c0d081210644d8fc5a02727089c6d19a32800bf3ef6cf4b49c5672f6b5"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-57-15-299Z-exemplar_cleanup-affliction-contact_exposure/manifest.json",
      "bytes": 236,
      "sha256": "22fc17ca424d64a63031ac508b246c76a0359ba6be2f0b8e35507a97974e7b35"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-57-15-299Z-exemplar_cleanup-affliction-contact_exposure/records.jsonl",
      "bytes": 292755,
      "sha256": "656bc6274e8ed788710aa79207cbb50a658ccb3b197cbd1bbbd54339f3ec56c8"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-57-15-299Z-exemplar_cleanup-affliction-contact_exposure/review-state.json",
      "bytes": 93,
      "sha256": "44d9f51632edbdc979945b9e0a77047c32f8b1e6d1082790ac590438b9b40b64"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T08-57-15-299Z-exemplar_cleanup-affliction-contact_exposure/summary.md",
      "bytes": 213,
      "sha256": "85f2755e72679603cfca66ecd646b7499bee7df6754803d59de931082acce2d4"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-02-53-025Z-proposal_review-all-batch/decisions.jsonl",
      "bytes": 0,
      "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-02-53-025Z-proposal_review-all-batch/manifest.json",
      "bytes": 157,
      "sha256": "ea9f84b8abbe189d05a3c5d8efcff2060564d27773f320753501f8bad07226b3"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-02-53-025Z-proposal_review-all-batch/records.jsonl",
      "bytes": 0,
      "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-02-53-025Z-proposal_review-all-batch/review-state.json",
      "bytes": 93,
      "sha256": "f18381176237cc662125c9192eb881f7248fcc3aab5febb90b08c72816ddf85c"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-02-53-025Z-proposal_review-all-batch/summary.md",
      "bytes": 191,
      "sha256": "2d99ba9733c73852fecb98e82257763d154031e448cfc88fde0cbbf5002737eb"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-03-12-088Z-exemplar_cleanup-all-batch/decisions.jsonl",
      "bytes": 857,
      "sha256": "a0d9ce581e7a3ebeafec2f04890dd6526a74af3806a4ba346cb723cad7248293"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-03-12-088Z-exemplar_cleanup-all-batch/manifest.json",
      "bytes": 159,
      "sha256": "34b9be793b46d51894c026f70c271840702b5f36b4e2f2a45046e8af17d683a5"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-03-12-088Z-exemplar_cleanup-all-batch/records.jsonl",
      "bytes": 3924,
      "sha256": "84e361573bc080e60150a5ed8c3bc8adf1ff8ad334819c1ac44474484ca05c83"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-03-12-088Z-exemplar_cleanup-all-batch/review-state.json",
      "bytes": 93,
      "sha256": "00473fa72036cfda74356ebd2e350fb4125f5512eca3a9b5e5631cf919aabe80"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-03-12-088Z-exemplar_cleanup-all-batch/summary.md",
      "bytes": 199,
      "sha256": "a034dc7c8efbcfb91f82698b8dc06314d58f25e798e88110d16ae6c70154ed93"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-09-45-582Z-exemplar_cleanup-all-batch/decisions.jsonl",
      "bytes": 849,
      "sha256": "75bd33da19018b7c77e463f61080c148948096dd4581363b7b437d7090e7574b"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-09-45-582Z-exemplar_cleanup-all-batch/manifest.json",
      "bytes": 159,
      "sha256": "a1ca23401b5a5535f8d5f54f48255168d99e4c78101306b09382f1dd73a63d31"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-09-45-582Z-exemplar_cleanup-all-batch/records.jsonl",
      "bytes": 3924,
      "sha256": "84e361573bc080e60150a5ed8c3bc8adf1ff8ad334819c1ac44474484ca05c83"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-09-45-582Z-exemplar_cleanup-all-batch/review-state.json",
      "bytes": 93,
      "sha256": "f80279694823d7a4c12911550bf1fcdbbf42510e210df08b67c3df7e020b9452"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-09-45-582Z-exemplar_cleanup-all-batch/summary.md",
      "bytes": 199,
      "sha256": "7474a716b75a59a43cf9f7e8fd2fbde2a48ecdfe7d309f0c785f4192daf19bb1"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-10-02-517Z-exemplar_cleanup-all-batch/decisions.jsonl",
      "bytes": 857,
      "sha256": "a0d9ce581e7a3ebeafec2f04890dd6526a74af3806a4ba346cb723cad7248293"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-10-02-517Z-exemplar_cleanup-all-batch/manifest.json",
      "bytes": 159,
      "sha256": "2d82e78a0c11d0c67a1f5447dffe1cc82d47b93497f59157ebfbbce56611029e"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-10-02-517Z-exemplar_cleanup-all-batch/records.jsonl",
      "bytes": 3924,
      "sha256": "84e361573bc080e60150a5ed8c3bc8adf1ff8ad334819c1ac44474484ca05c83"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-10-02-517Z-exemplar_cleanup-all-batch/review-state.json",
      "bytes": 93,
      "sha256": "6f1174f9dbb8d730796f4c10fbacbeebe1819b1d0eb2c8ee7c228d3188fbd674"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-10-02-517Z-exemplar_cleanup-all-batch/summary.md",
      "bytes": 199,
      "sha256": "a78dcc413c5cca6c256a3a85eb662e12f0e1dccabff75ffa92f45a49aede3d7e"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-14-06-713Z-exemplar_cleanup-all-batch/decisions.jsonl",
      "bytes": 857,
      "sha256": "a0d9ce581e7a3ebeafec2f04890dd6526a74af3806a4ba346cb723cad7248293"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-14-06-713Z-exemplar_cleanup-all-batch/manifest.json",
      "bytes": 159,
      "sha256": "ce43a1639c3a2b3fe03389e341589fb5809106835e815b8b89076c022f93103a"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-14-06-713Z-exemplar_cleanup-all-batch/records.jsonl",
      "bytes": 3924,
      "sha256": "84e361573bc080e60150a5ed8c3bc8adf1ff8ad334819c1ac44474484ca05c83"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-14-06-713Z-exemplar_cleanup-all-batch/review-state.json",
      "bytes": 93,
      "sha256": "3e1ea27d874297ce11de43552a40d9dbbeec6285b500b4d6ecd476fcb331e438"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-14-06-713Z-exemplar_cleanup-all-batch/summary.md",
      "bytes": 199,
      "sha256": "3f83d7a8a32655110cf1149f2772bd3c0f1d6d9645fa803bf5eb1cd8e6f89fe2"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-17-48-126Z-exemplar_cleanup-all-batch/decisions.jsonl",
      "bytes": 857,
      "sha256": "a0d9ce581e7a3ebeafec2f04890dd6526a74af3806a4ba346cb723cad7248293"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-17-48-126Z-exemplar_cleanup-all-batch/manifest.json",
      "bytes": 159,
      "sha256": "7c1c119260c46739f40f53280e3eadb46f326eff49e4e02c5a2251752f4cc54c"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-17-48-126Z-exemplar_cleanup-all-batch/records.jsonl",
      "bytes": 3924,
      "sha256": "84e361573bc080e60150a5ed8c3bc8adf1ff8ad334819c1ac44474484ca05c83"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-17-48-126Z-exemplar_cleanup-all-batch/review-state.json",
      "bytes": 93,
      "sha256": "63d4e0e0093d63cdb3d5f05d199403f2d394ce1c51c0a9f83f191c3a6485e602"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-17-48-126Z-exemplar_cleanup-all-batch/summary.md",
      "bytes": 199,
      "sha256": "dd2f16abf4617f8355ab648e0e59973fde694fd2172c71156188fd87bd84b4b6"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-34-54-005Z-exemplar_cleanup-all-batch/decisions.jsonl",
      "bytes": 857,
      "sha256": "a0d9ce581e7a3ebeafec2f04890dd6526a74af3806a4ba346cb723cad7248293"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-34-54-005Z-exemplar_cleanup-all-batch/manifest.json",
      "bytes": 159,
      "sha256": "56f93ff4746809964aa4574404cd743181a65af68fd6cf0583eb4aa9c130125c"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-34-54-005Z-exemplar_cleanup-all-batch/records.jsonl",
      "bytes": 5385,
      "sha256": "3ccf7e3139f6ac5f9d51179bdbd422e4ab01c4517f3a7cb70c9fc7ebdd31443a"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-34-54-005Z-exemplar_cleanup-all-batch/review-state.json",
      "bytes": 93,
      "sha256": "09f9ac032a13b934011070ca7e7fc8c69217b9557907c36056af04543ac61feb"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-34-54-005Z-exemplar_cleanup-all-batch/summary.md",
      "bytes": 199,
      "sha256": "dbd03713aefe540087c050da598e7a700ec527f52422ad2a073ca43c0f935741"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-46-34-237Z-exemplar_cleanup-all-batch/decisions.jsonl",
      "bytes": 857,
      "sha256": "a0d9ce581e7a3ebeafec2f04890dd6526a74af3806a4ba346cb723cad7248293"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-46-34-237Z-exemplar_cleanup-all-batch/manifest.json",
      "bytes": 159,
      "sha256": "918928f4320a9f36c5c0da5772e30cc19c8d5b2994701f8cb5f8af2c36933da9"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-46-34-237Z-exemplar_cleanup-all-batch/records.jsonl",
      "bytes": 5385,
      "sha256": "3ccf7e3139f6ac5f9d51179bdbd422e4ab01c4517f3a7cb70c9fc7ebdd31443a"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-46-34-237Z-exemplar_cleanup-all-batch/review-state.json",
      "bytes": 93,
      "sha256": "6ecd735192bd00689ae5e3875242016b43721c177417ee0fd161cd66bd46f3ef"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T09-46-34-237Z-exemplar_cleanup-all-batch/summary.md",
      "bytes": 199,
      "sha256": "72d3319fafeddbdf3d8bc19cb94a797fa5220e66c739a6701c4fc139669a0c8d"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T10-19-15-069Z-exemplar_cleanup-all-batch/decisions.jsonl",
      "bytes": 857,
      "sha256": "a0d9ce581e7a3ebeafec2f04890dd6526a74af3806a4ba346cb723cad7248293"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T10-19-15-069Z-exemplar_cleanup-all-batch/manifest.json",
      "bytes": 159,
      "sha256": "1876409e7cd72195829349e63780ecb779c43fedc520d8bca8959a6dec56f06b"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T10-19-15-069Z-exemplar_cleanup-all-batch/records.jsonl",
      "bytes": 5385,
      "sha256": "3ccf7e3139f6ac5f9d51179bdbd422e4ab01c4517f3a7cb70c9fc7ebdd31443a"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T10-19-15-069Z-exemplar_cleanup-all-batch/review-state.json",
      "bytes": 93,
      "sha256": "97757d16ed1daabdc91a2367293845ba10ec3e396fa5732b1f4501265a75b7a4"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T10-19-15-069Z-exemplar_cleanup-all-batch/summary.md",
      "bytes": 199,
      "sha256": "ebe9d002d2cca11418887caeef056751659e33fe2c743fffe89cc2347198f04a"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T10-19-28-744Z-legacy_seed-all-batch/decisions.jsonl",
      "bytes": 397597,
      "sha256": "c7531bddd485b8a41642cf302a56d5850243f0bdc813658e22670faa853ec644"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T10-19-28-744Z-legacy_seed-all-batch/manifest.json",
      "bytes": 151,
      "sha256": "01b598b757afce98ed0e30342100a9a23f1ab7ba56a45f5b9e768c1f6745adaf"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T10-19-28-744Z-legacy_seed-all-batch/records.jsonl",
      "bytes": 981965,
      "sha256": "60ed807b22734cc8103e9ec3a0f4c0a9b918477d9604b0dde435a018837c0e10"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T10-19-28-744Z-legacy_seed-all-batch/review-state.json",
      "bytes": 95,
      "sha256": "b29aabfaa4bb0ba9c85f31f9bb5c934a1fb9666f4f36386e1320b988f6cf21bd"
    },
    {
      "path": "scratch/migration-sessions/2026-04-17T10-19-28-744Z-legacy_seed-all-batch/summary.md",
      "bytes": 196,
      "sha256": "3d1ab588cfeb4aea6f3d350edce4f3057ae4bd0b9bb80d946a7113a47eb86cf8"
    },
    {
      "path": "scratch/migration-sessions/2026-04-18T19-11-56-099Z-exemplar_cleanup-creature-urban_setting/decisions.jsonl",
      "bytes": 857,
      "sha256": "a0d9ce581e7a3ebeafec2f04890dd6526a74af3806a4ba346cb723cad7248293"
    },
    {
      "path": "scratch/migration-sessions/2026-04-18T19-11-56-099Z-exemplar_cleanup-creature-urban_setting/manifest.json",
      "bytes": 224,
      "sha256": "bdc375e23a0bc9ab7e55eaf9d08bb9b52fa1dc88a2daf760dd793ecfdec96ee2"
    },
    {
      "path": "scratch/migration-sessions/2026-04-18T19-11-56-099Z-exemplar_cleanup-creature-urban_setting/records.jsonl",
      "bytes": 5385,
      "sha256": "3ccf7e3139f6ac5f9d51179bdbd422e4ab01c4517f3a7cb70c9fc7ebdd31443a"
    },
    {
      "path": "scratch/migration-sessions/2026-04-18T19-11-56-099Z-exemplar_cleanup-creature-urban_setting/review-state.json",
      "bytes": 93,
      "sha256": "dc8926e34b4171da8708a3f3b656731f4eacc32a50bba923e16f6e42cf84a867"
    },
    {
      "path": "scratch/migration-sessions/2026-04-18T19-11-56-099Z-exemplar_cleanup-creature-urban_setting/summary.md",
      "bytes": 212,
      "sha256": "c316da7333facf7aac6ae57509ba0f93874dbf6b8e03e09b4545602fb51f0044"
    },
    {
      "path": "scratch/migration-sessions/2026-04-18T19-53-04-719Z-exemplar_cleanup-creature-batch/decisions.jsonl",
      "bytes": 857,
      "sha256": "a0d9ce581e7a3ebeafec2f04890dd6526a74af3806a4ba346cb723cad7248293"
    },
    {
      "path": "scratch/migration-sessions/2026-04-18T19-53-04-719Z-exemplar_cleanup-creature-batch/manifest.json",
      "bytes": 190,
      "sha256": "b0c53d681fb48f97e5f1dcc9a056236b7e6d906e6039fdb795db6407ac205e44"
    },
    {
      "path": "scratch/migration-sessions/2026-04-18T19-53-04-719Z-exemplar_cleanup-creature-batch/records.jsonl",
      "bytes": 5385,
      "sha256": "3ccf7e3139f6ac5f9d51179bdbd422e4ab01c4517f3a7cb70c9fc7ebdd31443a"
    },
    {
      "path": "scratch/migration-sessions/2026-04-18T19-53-04-719Z-exemplar_cleanup-creature-batch/review-state.json",
      "bytes": 93,
      "sha256": "b524f604d214e5ce601431dbba701422196ad420fd7d1aea374e9424cfec781e"
    },
    {
      "path": "scratch/migration-sessions/2026-04-18T19-53-04-719Z-exemplar_cleanup-creature-batch/summary.md",
      "bytes": 204,
      "sha256": "38640ec522a471e533242bc19454277aaef45fdaa1b685becf6ef04dd558c01a"
    },
    {
      "path": "scratch/migration-sessions/2026-04-18T20-00-14-131Z-exemplar_cleanup-all-batch/decisions.jsonl",
      "bytes": 857,
      "sha256": "a0d9ce581e7a3ebeafec2f04890dd6526a74af3806a4ba346cb723cad7248293"
    },
    {
      "path": "scratch/migration-sessions/2026-04-18T20-00-14-131Z-exemplar_cleanup-all-batch/manifest.json",
      "bytes": 159,
      "sha256": "adcb5cfa89c05ba3ed1782aa04e31ca55991fa0fa8758d0b2f6047831795579f"
    },
    {
      "path": "scratch/migration-sessions/2026-04-18T20-00-14-131Z-exemplar_cleanup-all-batch/records.jsonl",
      "bytes": 5385,
      "sha256": "3ccf7e3139f6ac5f9d51179bdbd422e4ab01c4517f3a7cb70c9fc7ebdd31443a"
    },
    {
      "path": "scratch/migration-sessions/2026-04-18T20-00-14-131Z-exemplar_cleanup-all-batch/review-state.json",
      "bytes": 93,
      "sha256": "33cd721ffa8e148a71961da5a56234b11bce69a1d5087b0b2b291bbb8ca1a17f"
    },
    {
      "path": "scratch/migration-sessions/2026-04-18T20-00-14-131Z-exemplar_cleanup-all-batch/summary.md",
      "bytes": 199,
      "sha256": "3f0ea74d7fc11ab16f7ee8ab1d7ad0d7267902cf12207adccf850b0e1d308eb5"
    },
    {
      "path": "scratch/migration-sessions/2026-04-23T15-11-31-627Z-proposal_review-all-batch/decisions.jsonl",
      "bytes": 0,
      "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    },
    {
      "path": "scratch/migration-sessions/2026-04-23T15-11-31-627Z-proposal_review-all-batch/manifest.json",
      "bytes": 157,
      "sha256": "2f5ffb7a43a8afa8c9af6201b69a5df66c896993caef3677ece4491781c22081"
    },
    {
      "path": "scratch/migration-sessions/2026-04-23T15-11-31-627Z-proposal_review-all-batch/records.jsonl",
      "bytes": 0,
      "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    },
    {
      "path": "scratch/migration-sessions/2026-04-23T15-11-31-627Z-proposal_review-all-batch/review-state.json",
      "bytes": 93,
      "sha256": "bf0a973e37b296f66e87812c0c0d2f4502bdb7b84f37593953338dac4caf301f"
    },
    {
      "path": "scratch/migration-sessions/2026-04-23T15-11-31-627Z-proposal_review-all-batch/summary.md",
      "bytes": 191,
      "sha256": "43394d1aa704d1edfe32569a61b49e6fc402f323a67e46fe84abed5d64e0653c"
    },
    {
      "path": "scratch/migration-sessions/2026-04-23T15-12-55-752Z-exemplar_cleanup-all-batch/decisions.jsonl",
      "bytes": 857,
      "sha256": "a0d9ce581e7a3ebeafec2f04890dd6526a74af3806a4ba346cb723cad7248293"
    },
    {
      "path": "scratch/migration-sessions/2026-04-23T15-12-55-752Z-exemplar_cleanup-all-batch/manifest.json",
      "bytes": 159,
      "sha256": "02cf1dfda7cfefb2745539a15e39edb317b059200ee96a33b313f92fe5b650af"
    },
    {
      "path": "scratch/migration-sessions/2026-04-23T15-12-55-752Z-exemplar_cleanup-all-batch/records.jsonl",
      "bytes": 5421,
      "sha256": "3cddd75ad46d88c681f4a81154416c6a47fb937556f17bc2bdd3081b90c46bf0"
    },
    {
      "path": "scratch/migration-sessions/2026-04-23T15-12-55-752Z-exemplar_cleanup-all-batch/review-state.json",
      "bytes": 93,
      "sha256": "d4866911315ab6b61b02f9c8f4e572b10eb322e079bc7652b5e0542dde7d2118"
    },
    {
      "path": "scratch/migration-sessions/2026-04-23T15-12-55-752Z-exemplar_cleanup-all-batch/summary.md",
      "bytes": 199,
      "sha256": "28d390679a7c6bf514fdd1693490d4b9c96b70360fc20e81a0564eac05a1eb52"
    },
    {
      "path": "scratch/migration-sessions/2026-05-04T07-44-41-808Z-legacy_seed-all-batch/decisions.jsonl",
      "bytes": 397719,
      "sha256": "30345ca2c58385a4f8203b9953676e6d7b280f2ab983837a8eaeacc8fe23e03e"
    },
    {
      "path": "scratch/migration-sessions/2026-05-04T07-44-41-808Z-legacy_seed-all-batch/manifest.json",
      "bytes": 151,
      "sha256": "0401db1b95bddf66d7605d2988abe048a4e7544a49fd3c46561f412751bf928a"
    },
    {
      "path": "scratch/migration-sessions/2026-05-04T07-44-41-808Z-legacy_seed-all-batch/records.jsonl",
      "bytes": 994452,
      "sha256": "f6e7cc79ddcdfe7bcfebd2cc941f8170c35cc26f211730d35a4b18a670346145"
    },
    {
      "path": "scratch/migration-sessions/2026-05-04T07-44-41-808Z-legacy_seed-all-batch/review-state.json",
      "bytes": 94,
      "sha256": "1bad2241228e8a2d34881029b0ac41502c533a793de32c405ec9db3d86e72799"
    },
    {
      "path": "scratch/migration-sessions/2026-05-04T07-44-41-808Z-legacy_seed-all-batch/summary.md",
      "bytes": 196,
      "sha256": "6c8941eeab8120123fc6da42986a87efe44d1d6f0986d834dccae6be1b360c3e"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-07T21-22-14-496Z/session.json",
      "bytes": 204538,
      "sha256": "8546c4637200c3bdb2de56ca294453b50a2ed5c72e06ae5f79e5802d24779080"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-07T21-22-35-768Z/session.json",
      "bytes": 1102,
      "sha256": "f5942f13ed214421b03c2ce29eb824a6f61498f244bf3186691a3a0880d410c3"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-07T21-22-49-493Z/session.json",
      "bytes": 1102,
      "sha256": "0c08fab4d3af03d4907602fad7afadf0ba0dee0637b85b9162cd2cea70e8ea32"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-07T21-23-37-121Z/session.json",
      "bytes": 1102,
      "sha256": "5b87531385dea7243519964c4cec7f8ca6ff08fa0da060f76e1c03c9f061e077"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-07T21-24-30-558Z/session.json",
      "bytes": 1102,
      "sha256": "ada4a2d3f19bb61a8b5da80888a03958cf28a8826cc01f6acfba3e62abe0a97b"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-07T21-26-47-735Z/session.json",
      "bytes": 204538,
      "sha256": "84ee55be15bb6a0607cd953bd492389f132741147303801ea29c6fdb114aaba9"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-07T21-29-06-643Z/session.json",
      "bytes": 1102,
      "sha256": "a57388583ccd1b4bcf5a57f231f727e6d19d2950e85138ae2246ef8cb3dbd166"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-07T21-32-32-697Z/session.json",
      "bytes": 441521,
      "sha256": "272632d42e2b3ccb598089caa75278f1ae70c75fb9066c76c6a07a1b3e08833c"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-07T21-34-44-459Z/session.json",
      "bytes": 1110,
      "sha256": "c33dd8eb7ca09e3cd24f2b91a4f5b3f84b5c9050d4856907dd6b52b161f9f623"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-07T21-54-26-575Z/session.json",
      "bytes": 1110,
      "sha256": "8d8aa604eef79c393ac5b5c535f5735d0abb7897acb61b8bc9566342ff497e58"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-07T21-58-59-846Z/session.json",
      "bytes": 1110,
      "sha256": "ed21f456c8460cd44d695911bacd57e07419f68be0bef1714b34de0cf98e88ab"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-07T23-56-26-138Z/session.json",
      "bytes": 1110,
      "sha256": "c8ecf5a18fba33f66f4c0db039b739198413b65be77ea1c134ef713b2b34d2fa"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-08T00-09-41-374Z/session.json",
      "bytes": 1110,
      "sha256": "04fe06a24fb29ed9db995a18f01650807303303d58817421d58407697451e076"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-08T00-10-15-008Z/session.json",
      "bytes": 441521,
      "sha256": "46ba001c63ebcdd3c773274408b8440dcfbb2801357bdb0943936fbed52c41e1"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-08T00-31-53-272Z/session.json",
      "bytes": 1110,
      "sha256": "2320bf54817826a89073dd1c956f4d29ef55c32a03726d2c1f953356b8054c1c"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-08T00-51-33-400Z/session.json",
      "bytes": 1045,
      "sha256": "4bbfcaee4fabfdea8a87582a899ce00105e803e1446184263d09c4a226a4def7"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-08T00-52-28-565Z/session.json",
      "bytes": 1045,
      "sha256": "ff48c25394f1eaec6f3193910d12f62c9f84378086a6f9c672df178006160fdc"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-08T00-53-34-185Z/session.json",
      "bytes": 1045,
      "sha256": "c9d52fafcfec0e34ff3a7576d3fba61e7d5567daca8b6ce1cc0dacc22b8d5638"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-08T01-33-36-650Z/session.json",
      "bytes": 1045,
      "sha256": "6aafc8bf756a27bff457b78b95dbe569f98d98f5d2421670e3afb98066d59db3"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-08T01-34-41-522Z/session.json",
      "bytes": 1045,
      "sha256": "0b9cf9bab70177173171787a52d06e19758f17a2ecb4a6628aca46587ae9043a"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-08T01-35-13-892Z/session.json",
      "bytes": 1045,
      "sha256": "8635d28b5bebbb4ea9f1d877ffc5f811c4cbaf26aaa21418cf0b3cbd3de8662d"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-08T01-35-42-980Z/session.json",
      "bytes": 1045,
      "sha256": "511153ccf68df0239bb0c842aa21086d622d2f1c3b336720ce95db856908f815"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-08T01-57-44-848Z/session.json",
      "bytes": 1045,
      "sha256": "4d9360330de97b290fdac4110efcf6de849a9477b3238b9d1195e4e4aaeee5fe"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-08T01-59-16-891Z/session.json",
      "bytes": 1045,
      "sha256": "346a94db2d13b8cd8b574c003627dded95fe1aedfdcad8facbf687c2cbe199c8"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-08T01-59-36-159Z/session.json",
      "bytes": 1045,
      "sha256": "816a49e3cb67baa7c2d5d1c56d1c5562510093d9cb437376689c37550fbd1629"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-08T05-29-24-639Z/session.json",
      "bytes": 1045,
      "sha256": "5ae7d0b9b059bfaa116c094296bdf94a52b95c2ee7a7948f801b2ea5a81f2455"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-08T05-33-01-193Z/session.json",
      "bytes": 1045,
      "sha256": "1ab9fe44b46f9f7d5694e6ce86b31df79db3f5b9e8c50c1a1bb7797e36291501"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-08T05-34-55-657Z/session.json",
      "bytes": 1045,
      "sha256": "f5719becaea8716e75a7e061b288ee733547622711fbf1e8e505bcce499a7fb3"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-09T23-00-41-342Z/session.json",
      "bytes": 384691,
      "sha256": "5330f1e9ce1b553f80b186b01f22bea62dea86266c905f4b83c72fa4d57736bb"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-09T23-24-53-715Z/session.json",
      "bytes": 1045,
      "sha256": "ed7c253bcc6cae9d7059f654662db519f88f3b96638ba94741ef1a552e078448"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-09T23-31-03-949Z/session.json",
      "bytes": 1045,
      "sha256": "f57f02137b30fd30d6f6edf30ad9b5cf996c56525365bc3dee01c9c40b906080"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-09T23-38-18-561Z/session.json",
      "bytes": 1045,
      "sha256": "3cdefb0e797c7a6beef60da4f8ce199a84dd7197b7007134389ff9ffce9a7d02"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-09T23-38-42-571Z/session.json",
      "bytes": 1045,
      "sha256": "e867dde9410633f9e05f1a22109cfcbc596ead6509849962a8cb4cc1d5e03cc7"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-10T00-31-49-673Z/session.json",
      "bytes": 1045,
      "sha256": "0fb1edc916577cf51b909b2f180656886e91c5b0c5b3a94bd16de1880e6e4aa9"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-10T00-38-57-416Z/session.json",
      "bytes": 1045,
      "sha256": "9b1b4dca722a7073e1c93eb18fceb8d9d99e6eeed959aaf01c7d20f2e449a41a"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-10T00-53-50-841Z/session.json",
      "bytes": 1045,
      "sha256": "bfadf6ef3fbb15b52e828c16f8f51a7c654bf57714410c1a3821d4dacc1add5f"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-10T00-55-00-327Z/session.json",
      "bytes": 384691,
      "sha256": "da53bcba8abca08bc559b81d21aa31fcce697e75c2867e71b30a53fb34aa05f4"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-10T01-09-37-665Z/session.json",
      "bytes": 1002,
      "sha256": "00e421ebe31b66ab314dfc82a8433473d6f4858798c5d6b5f29aab2237c12e9f"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-10T01-10-38-600Z/session.json",
      "bytes": 1002,
      "sha256": "9dd527c0d77cf9f9966be606084d5a86c21b03718903a92d680c282a532acb0e"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-10T01-11-43-176Z/session.json",
      "bytes": 1002,
      "sha256": "92c52c8b0d090d54fbd201990871e0841e65d24f3fb17eb1cd6b59e92434e885"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-10T01-16-35-266Z/session.json",
      "bytes": 1002,
      "sha256": "700bcb7e11c7338c00f61c181454f727f006fd5c1f0098df3d65f28669c9b70c"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-10T01-29-34-676Z/session.json",
      "bytes": 1002,
      "sha256": "bd915000faf09eecb32182de161050412c63cfc186623610a52433c54fec9e8c"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-10T18-31-53-651Z/session.json",
      "bytes": 1002,
      "sha256": "e0ae3bf6a7ba562225feeacc02a217495f97046036b7faeff8b437086ff071bc"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-10T18-43-07-687Z/session.json",
      "bytes": 1002,
      "sha256": "1625d092d3f64fd914bf26bacfeddc57fd15d9f63d86f6b0c436f6af0c78c774"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-10T18-44-32-273Z/session.json",
      "bytes": 1002,
      "sha256": "231f72d1d46ba08976cecbe4c6aed77c11032d2b05d18d1b3859b46ff5b01123"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-10T18-45-03-616Z/session.json",
      "bytes": 1002,
      "sha256": "9bb1e91a3813020781ca8da9a3ba17d5484bca4c38686d509fd1a5a1302d5cc2"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-11T23-10-55-216Z/session.json",
      "bytes": 1002,
      "sha256": "7edb9104c8030063632158769da9852a8daa1aef6925a2c63d1dce19fee4f9bc"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-11T23-26-32-234Z/session.json",
      "bytes": 1002,
      "sha256": "13c99a1064394f186f9162d5167e0835f2f3fb3f9f88d2a813affb23e9336812"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-12T05-35-17-337Z/session.json",
      "bytes": 1002,
      "sha256": "22f8f25ecc8b9632a3643354fdcc42a8bc66e6f6fa20e377f86dad9211cbea0f"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-12T07-26-14-167Z/session.json",
      "bytes": 1002,
      "sha256": "cefae6b1fa8b387601c29a96dac35ad271ffa2275891db7a9367c8707462808e"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-12T07-35-40-800Z/session.json",
      "bytes": 1002,
      "sha256": "1fad6cdb1bab1dd4e688028d27418cf16228507d9ebf4bddd4e83c612b0e05e1"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-14T15-56-56-814Z/session.json",
      "bytes": 1002,
      "sha256": "a753434578e5c8cb5c20a81bff98b245b4c0321aebf0a7d62af1605726083176"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-14T16-21-03-368Z/session.json",
      "bytes": 1002,
      "sha256": "fb14f1bd90d8e9cbeef726d2b4c3e044d103e51ae537f4e11d64d25e516880b6"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-14T16-31-10-365Z/session.json",
      "bytes": 1002,
      "sha256": "c80058bf31d6364dd1d0a855102ba1e84bafae8505093ead2057bcf619a1e314"
    },
    {
      "path": "scratch/translation-sessions/translation-2026-05-19T03-28-02-358Z/session.json",
      "bytes": 1002,
      "sha256": "c183f389d38f2dd1ae6097003d6f329b1c429d30197e06e52d4af6748ef46f05"
    }
  ]
}
```
