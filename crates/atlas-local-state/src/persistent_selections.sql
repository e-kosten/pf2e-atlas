CREATE TABLE encounter_persistent_selections (
    participant_id INTEGER NOT NULL REFERENCES encounter_participants(id) ON DELETE CASCADE,
    episode_id TEXT NOT NULL,
    selected_source_id INTEGER NOT NULL,
    membership_json TEXT NOT NULL,
    PRIMARY KEY (participant_id, episode_id)
);
