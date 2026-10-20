CREATE TABLE encounter_relationships (
    encounter_id INTEGER NOT NULL REFERENCES encounters(id) ON DELETE CASCADE,
    subject_id INTEGER NOT NULL REFERENCES encounter_participants(id) ON DELETE CASCADE,
    other_id INTEGER NOT NULL REFERENCES encounter_participants(id) ON DELETE CASCADE,
    property TEXT NOT NULL,
    layer TEXT NOT NULL,
    codec_version INTEGER NOT NULL,
    knowledge_json TEXT NOT NULL,
    PRIMARY KEY(subject_id, other_id, property, layer)
);
