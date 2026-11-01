-- Unknown historical pending values retain their bytes until a complete target
-- is promoted by the explicit update owner. This migration performs no cleanup.
CREATE TABLE native_update_targets(
 job_key TEXT PRIMARY KEY REFERENCES reference_update_jobs(job_key),
 phase TEXT NOT NULL CHECK(phase IN ('candidate','active','retired')),
 registered_count INTEGER NOT NULL DEFAULT 0 CHECK(registered_count>=0),
 member_count INTEGER NOT NULL DEFAULT 0 CHECK(member_count>=0),
 membership_digest BLOB CHECK(length(membership_digest)=32)
) STRICT;
CREATE UNIQUE INDEX native_update_one_candidate ON native_update_targets(phase) WHERE phase='candidate';
CREATE UNIQUE INDEX native_update_one_active ON native_update_targets(phase) WHERE phase='active';
CREATE TABLE native_target_inputs(
 job_key TEXT NOT NULL REFERENCES native_update_targets(job_key),
 input_id INTEGER NOT NULL REFERENCES embedding_inputs(input_id),
 selected_hash BLOB CHECK(length(selected_hash)=32),
 PRIMARY KEY(job_key,input_id)
) WITHOUT ROWID,STRICT;
CREATE INDEX native_target_input_lookup ON native_target_inputs(input_id);
