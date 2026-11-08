use super::*;
use rusqlite::{Connection, params};

impl EncountersWrite<'_> {
    pub fn import_document(
        &self,
        mut document: EncounterDocument,
        slug: Option<String>,
    ) -> LocalStateResult<Encounter> {
        let mut identities = document.validate()?;
        let db = self.connection;
        crate::atomic_action::run(db, || {
            let root = &document.snapshot.encounter;
            let slug = slug.unwrap_or_else(|| root.slug.clone());
            let key = storage::insert_encounter(
                db,
                NewEncounter {
                    slug: slug.clone(),
                    name: root.name.clone(),
                    description: root.description.clone(),
                    note: root.note.clone(),
                },
            )?;
            let id = db.last_insert_rowid();
            identities.allocate(db, key, slug)?;
            identities.visit(&mut document);
            write_document(db, id, &document)?;
            storage::get(db, &document.snapshot.encounter.encounter_key)?
                .ok_or_else(|| invalid("restored encounter could not be read"))
        })
    }
}

fn write_document(db: &Connection, id: i64, document: &EncounterDocument) -> LocalStateResult<()> {
    for participant in &document.snapshot.participants {
        write_participant(db, id, participant)?;
    }
    let root = &document.snapshot.encounter;
    db.execute(
        "UPDATE encounters SET status=?2,round_number=?3,current_turn_participant_key=?4,
         created_at=?5,updated_at=?6,revision=?7,difficulty_party_level=?8,
         difficulty_party_size=?9,round_duration_seconds=?10 WHERE id=?1",
        params![
            id,
            root.status.as_str(),
            root.round_number,
            root.current_turn_participant_key,
            root.created_at,
            root.updated_at,
            root.revision,
            root.difficulty_reference.party_level,
            root.difficulty_reference.party_size,
            root.round_duration_seconds
        ],
    )?;
    for row in &document.baselines {
        db.execute(
            "INSERT INTO encounter_participant_baselines(participant_key,payload_json)
            VALUES(?1,?2)",
            params![row.participant_key, receipt_codec::encode(&row.gameplay)?],
        )?;
    }
    for row in &document.pending {
        db.execute("INSERT INTO encounter_pending_consequences(encounter_id,consequence_key,participant_key,payload_json)
            VALUES(?1,?2,?3,?4)", params![id, row.consequence_key,
            snapshot::consequence_participant(&row.consequence), receipt_codec::encode(&row.consequence)?])?;
    }
    for row in &document.applications {
        db.execute("INSERT INTO encounter_condition_applications(encounter_id,participant_key,application_id,grant_slot,condition_id)
            VALUES(?1,?2,?3,?4,?5)", params![id, row.participant_key,
            row.application_id, row.grant_slot, row.condition_id])?;
    }
    for row in &document.receipts {
        let (family, boundary, payload) = match &row.outcome {
            ReceiptOutcome::Turn { boundary_key } => ("turn", Some(boundary_key), None),
            ReceiptOutcome::Event { receipt } => {
                ("event", None, Some(receipt_codec::encode(receipt)?))
            }
        };
        db.execute("INSERT INTO encounter_command_receipts(encounter_id,family,request_id,intent_json,boundary_key,receipt_json)
            VALUES(?1,?2,?3,?4,?5,?6)", params![id, family, row.request_id,
            serde_json::to_string(&row.intent)?, boundary, payload])?;
    }
    for row in &document.claims {
        let claim = &row.claim;
        db.execute("INSERT INTO encounter_boundary_claims(encounter_id,participant_key,round_number,kind,scope_key,request_id)
            VALUES(?1,?2,?3,?4,?5,?6)", params![id, claim.participant_key,
            claim.round_number, claim.kind.as_str(), claim.scope_key, row.request_id])?;
    }
    if let Some(row) = &document.outgoing {
        db.execute("INSERT INTO encounter_outgoing_turns(encounter_id,request_id,payload_json,requires_review)
            VALUES(?1,?2,?3,?4)", params![id, row.request_id, receipt_codec::encode(&row.turn)?, row.requires_review])?;
    }
    Ok(())
}

fn write_participant(
    db: &Connection,
    encounter_id: i64,
    p: &EncounterParticipant,
) -> LocalStateResult<()> {
    if let Some(key) = &p.record_key {
        crate::references::retain(
            db,
            key,
            p.record_title_snapshot
                .as_deref()
                .unwrap_or(&p.display_name),
        )?;
    }
    db.execute("INSERT INTO encounter_participants(encounter_id,participant_key,record_key,
        participant_kind,participant_variant,position,display_name,record_title_snapshot,record_kind_snapshot,
        side,initiative,initiative_order,max_hp,current_hp,temporary_hp,defeated,hidden,note,created_at,updated_at,
        hp_origin,variant_origin,revision,hazard_disabled,track_dying,mortality,level_override,
        base_death_threshold,turn_resources_json)
        VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,
        ?20,?21,?22,?23,?24,?25,?26,?27,?28,?29)",
        params![encounter_id,p.participant_key,p.record_key,p.participant_kind.as_str(),
        p.participant_variant.as_str(),p.position,p.display_name,p.record_title_snapshot,p.record_kind_snapshot,
        p.side.as_str(),p.initiative,p.initiative_order,p.max_hp,p.current_hp,p.temporary_hp,p.defeated,
        p.hidden,p.note,p.created_at,p.updated_at,p.hp_origin.as_str(),p.variant_origin.as_str(),p.revision,
        p.hazard_disabled,p.track_dying,match p.mortality { MortalityState::Alive => "alive", MortalityState::Dead => "dead" },p.level_override,p.base_death_threshold,
        p.turn_resources.as_ref().map(receipt_codec::encode).transpose()?])?;
    let id = db.last_insert_rowid();
    for condition in &p.conditions {
        write_condition(db, id, condition)?;
    }
    for resource in &p.spell_resources {
        db.execute("INSERT INTO encounter_spell_resources(participant_id,resource_key,capacity,remaining,undo_remaining)
            VALUES(?1,?2,?3,?4,?5)", params![id,resource.key,resource.capacity,resource.remaining,resource.undo_remaining])?;
    }
    for spell in &p.expended_spells {
        db.execute(
            "INSERT INTO encounter_expended_spells(participant_id,spell_id) VALUES(?1,?2)",
            params![id, spell],
        )?;
    }
    for ability in &p.ability_states {
        let ordinal = ability
            .rule_ordinal
            .map(i64::try_from)
            .transpose()
            .map_err(|_| invalid("invalid ability rule ordinal"))?
            .unwrap_or(-1);
        db.execute("INSERT INTO encounter_ability_states(participant_key,owner_key,rule_ordinal,payload_json)
            VALUES(?1,?2,?3,?4)", params![p.participant_key,serde_json::to_string(&ability.owner)?,
            ordinal,receipt_codec::encode(ability)?])?;
    }
    for tracker in &p.manual_trackers {
        db.execute(
            "INSERT INTO encounter_manual_trackers(participant_key,tracker_key,payload_json)
            VALUES(?1,?2,?3)",
            params![
                p.participant_key,
                tracker.tracker_key,
                receipt_codec::encode(tracker)?
            ],
        )?;
    }
    for episode in &p.persistent_precedence {
        persistent_precedence::write(db, &p.participant_key, episode)?;
    }
    Ok(())
}

fn write_condition(
    db: &Connection,
    participant_id: i64,
    c: &EncounterParticipantCondition,
) -> LocalStateResult<()> {
    let duration = c.duration.as_ref();
    let timing = duration.and_then(|d| d.timing.as_ref());
    db.execute("INSERT INTO encounter_participant_conditions(id,participant_id,condition_key,name,value,
        source_participant_key,duration_amount,duration_unit,duration_anchor_key,duration_event,note,
        scope_json,extra_action_json,immunity_override,created_at,updated_at,parameters_json,origin_json,
        duration_remaining_seconds) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
        params![c.condition_id,participant_id,c.condition_key,c.name,c.value,c.source_participant_key,
        duration.map(|d| d.amount),duration.map(|d| d.unit.as_str()),timing.map(|t| t.participant_key.as_str()),
        timing.map(|t| t.event.as_str()),c.note,serde_json::to_string(&c.scope)?,
        serde_json::to_string(&c.extra_action)?,c.immunity_override,c.created_at,c.updated_at,
        serde_json::to_string(&c.parameters)?,c.origin.as_ref().map(serde_json::to_string).transpose()?,
        duration.and_then(|d| d.remaining_seconds)])?;
    Ok(())
}
