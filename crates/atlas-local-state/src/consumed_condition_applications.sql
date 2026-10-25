-- Applications consumed before insertion retain identity without a saved source.
ALTER TABLE encounter_condition_applications RENAME TO encounter_condition_applications_previous;
CREATE TABLE encounter_condition_applications (
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    participant_key TEXT NOT NULL REFERENCES encounter_participants(participant_key) ON DELETE CASCADE,
    application_id TEXT NOT NULL,
    grant_slot TEXT NOT NULL,
    condition_id INTEGER,
    PRIMARY KEY(encounter_id, participant_key, application_id, grant_slot)
);
INSERT INTO encounter_condition_applications (
    encounter_id, participant_key, application_id, grant_slot, condition_id
)
SELECT encounter_id, participant_key, application_id, grant_slot, condition_id
FROM encounter_condition_applications_previous;
DROP TABLE encounter_condition_applications_previous;
