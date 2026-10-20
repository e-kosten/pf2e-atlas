CREATE TABLE encounter_outgoing_turns (
    id INTEGER PRIMARY KEY,
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    boundary_key TEXT NOT NULL UNIQUE,
    request_id TEXT NOT NULL,
    intent_json TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    requires_review INTEGER NOT NULL DEFAULT 1 CHECK(requires_review IN (0,1)),
    handled_at_table INTEGER NOT NULL DEFAULT 0 CHECK(handled_at_table IN (0,1)),
    handled_request_id TEXT,
    handled_intent_json TEXT,
    UNIQUE(encounter_id, request_id)
);

CREATE UNIQUE INDEX encounter_outgoing_handled_request ON encounter_outgoing_turns(encounter_id,handled_request_id) WHERE handled_request_id IS NOT NULL;
