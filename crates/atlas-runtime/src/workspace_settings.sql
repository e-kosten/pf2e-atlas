CREATE TABLE workspace_settings (
    id INTEGER PRIMARY KEY CHECK(id = 1),
    preferred_search_profile_id TEXT
) STRICT;
INSERT INTO workspace_settings(id, preferred_search_profile_id) VALUES(1, NULL);
DROP TABLE trait_references;
