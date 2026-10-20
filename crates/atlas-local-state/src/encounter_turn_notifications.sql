-- Turn receipts retain identity and frozen context, without an acknowledgment queue.
DROP INDEX encounter_outgoing_handled_request;
ALTER TABLE encounter_outgoing_turns DROP COLUMN handled_at_table;
ALTER TABLE encounter_outgoing_turns DROP COLUMN handled_request_id;
ALTER TABLE encounter_outgoing_turns DROP COLUMN handled_intent_json;
