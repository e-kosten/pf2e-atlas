-- Preserve the visible key order for trackers saved before positions existed.
CREATE TABLE encounter_manual_trackers_ordered (
    participant_key TEXT NOT NULL REFERENCES encounter_participants(participant_key) ON DELETE CASCADE,
    tracker_key TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    position INTEGER NOT NULL CHECK(position > 0),
    PRIMARY KEY(participant_key, tracker_key),
    UNIQUE(participant_key, position)
);

INSERT INTO encounter_manual_trackers_ordered (participant_key, tracker_key, payload_json, position)
SELECT participant_key, tracker_key, payload_json,
       row_number() OVER (PARTITION BY participant_key ORDER BY tracker_key)
FROM encounter_manual_trackers;

DROP TABLE encounter_manual_trackers;
ALTER TABLE encounter_manual_trackers_ordered RENAME TO encounter_manual_trackers;
