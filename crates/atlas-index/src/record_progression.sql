CREATE TABLE record_progression (
    record_id INTEGER PRIMARY KEY REFERENCES records(record_id) ON DELETE CASCADE,
    requirements_json TEXT CHECK(requirements_json IS NULL OR json_type(requirements_json)='array'),
    ancestry_access_json TEXT CHECK(ancestry_access_json IS NULL OR json_type(ancestry_access_json)='array'),
    CHECK(requirements_json IS NOT NULL OR ancestry_access_json IS NOT NULL)
) STRICT;
