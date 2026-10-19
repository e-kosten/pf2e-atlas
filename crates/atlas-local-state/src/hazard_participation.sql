UPDATE encounter_participants SET hazard_disabled = 1, defeated = 0
WHERE participant_kind = 'hazard' AND defeated = 1;

CREATE TRIGGER hazard_participation_insert BEFORE INSERT ON encounter_participants
WHEN NEW.participant_kind = 'hazard' AND NEW.defeated != 0
BEGIN SELECT RAISE(ABORT, 'Hazards use disabled participation'); END;
CREATE TRIGGER hazard_participation_update BEFORE UPDATE ON encounter_participants
WHEN NEW.participant_kind = 'hazard' AND NEW.defeated != 0
BEGIN SELECT RAISE(ABORT, 'Hazards use disabled participation'); END;
