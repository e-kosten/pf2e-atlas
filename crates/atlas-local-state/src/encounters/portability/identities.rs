use super::*;
use rusqlite::Connection;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub(super) struct Identities {
    participants: BTreeMap<String, String>,
    conditions: BTreeMap<i64, i64>,
    boundaries: BTreeMap<String, String>,
    turn_keys: BTreeMap<String, String>,
    requests: BTreeMap<String, String>,
    derived: BTreeMap<String, String>,
    encounter_identity: Option<(String, String)>,
}

impl EncounterDocument {
    pub(super) fn validate(&mut self) -> LocalStateResult<Identities> {
        if self.version != VERSION {
            return Err(invalid("unsupported encounter file version"));
        }
        let root = &self.snapshot.encounter;
        crate::slug::validate_slug(&root.slug)?;
        let participants = validate_snapshot(&self.snapshot)?;
        let live = |key: &str| {
            if participants.contains(key) {
                Ok(())
            } else {
                Err(invalid(format!("unknown live participant {key}")))
            }
        };
        if let Some(key) = &root.current_turn_participant_key {
            live(key)?;
        }
        let mut unique = BTreeSet::new();
        for row in &self.baselines {
            live(&row.participant_key)?;
            require_unique(&mut unique, &row.participant_key, "baseline")?;
            unique_conditions(&row.gameplay.conditions)?;
        }
        let mut pending = BTreeSet::new();
        for row in &self.pending {
            live(snapshot::consequence_participant(&row.consequence))?;
            require_unique(&mut pending, &row.consequence_key, "pending consequence")?;
        }
        let mut applications = BTreeSet::new();
        for row in &self.applications {
            live(&row.participant_key)?;
            require_unique(
                &mut applications,
                (&row.participant_key, &row.application_id, &row.grant_slot),
                "consumed application",
            )?;
        }
        let mut claims = BTreeSet::new();
        for row in &self.claims {
            live(&row.claim.participant_key)?;
            require_unique(
                &mut claims,
                (
                    &row.claim.participant_key,
                    row.claim.round_number,
                    row.claim.kind.as_str(),
                    &row.claim.scope_key,
                ),
                "boundary claim",
            )?;
            if row.claim.round_number < 0 {
                return Err(invalid("invalid boundary claim round"));
            }
        }
        let mut turn_requests = BTreeMap::new();
        let mut boundaries = BTreeSet::new();
        let mut event_requests = BTreeSet::new();
        for row in &self.receipts {
            if row.request_id.is_empty()
                || row.request_id.len() > 128
                || serde_json::to_vec(&row.intent)?.len() > 64 * 1024
            {
                return Err(invalid(format!(
                    "receipt {} exceeds its request bounds",
                    row.request_id
                )));
            }
            match &row.outcome {
                ReceiptOutcome::Turn { boundary_key } => {
                    require_unique(&mut boundaries, boundary_key, "turn boundary")?;
                    if turn_requests
                        .insert(&row.request_id, boundary_key)
                        .is_some()
                    {
                        return Err(invalid("duplicate turn request"));
                    }
                }
                ReceiptOutcome::Event { receipt } => {
                    require_unique(&mut event_requests, &row.request_id, "event request")?;
                    if receipt.encounter_key != root.encounter_key
                        || receipt.request_id != row.request_id
                    {
                        return Err(invalid("event receipt identity disagrees with its row"));
                    }
                }
            }
        }
        for row in &self.claims {
            if !event_requests.contains(&row.request_id) {
                return Err(invalid(format!(
                    "boundary claim has no event receipt {}",
                    row.request_id
                )));
            }
        }
        if let Some(outgoing) = &self.outgoing {
            let frozen = validate_snapshot(&outgoing.turn.snapshot)?;
            if outgoing
                .turn
                .next_participant_key
                .as_deref()
                .is_some_and(|key| !frozen.contains(key))
                || (outgoing.requires_review && outgoing.turn.next_participant_key.is_none())
            {
                return Err(invalid("outgoing review has no valid next participant"));
            }
            if outgoing.turn.snapshot.encounter.encounter_key != root.encounter_key
                || turn_requests.get(&outgoing.request_id) != Some(&&outgoing.turn.boundary_key)
            {
                return Err(invalid("outgoing snapshot has no matching turn receipt"));
            }
        }
        let record_keys: BTreeSet<_> = self.record_keys()?.into_iter().collect();
        let mut sources = BTreeSet::new();
        for source in &self.record_sources {
            if !record_keys.contains(&source.record_key) {
                return Err(invalid(
                    "reference fingerprint has no matching participant record",
                ));
            }
            require_unique(&mut sources, &source.record_key, "reference fingerprint")?;
        }
        let mut ids = Identities::default();
        ids.visit(self);
        if ids.conditions.keys().any(|id| *id <= 0) {
            return Err(invalid("condition IDs must be positive"));
        }
        if ids.participants.keys().any(String::is_empty)
            || ids.boundaries.keys().any(String::is_empty)
        {
            return Err(invalid("participant and boundary keys must be nonempty"));
        }
        Ok(ids)
    }
}

fn validate_snapshot(snapshot: &EncounterWithParticipants) -> LocalStateResult<BTreeSet<&str>> {
    let root = &snapshot.encounter;
    if root.encounter_key.is_empty()
        || !(0..i64::MAX).contains(&root.revision)
        || !(0..i64::MAX).contains(&root.round_number)
        || !(1..=86400).contains(&root.round_duration_seconds)
        || root
            .difficulty_reference
            .party_level
            .is_some_and(|n| !(1..=20).contains(&n))
        || root
            .difficulty_reference
            .party_size
            .is_some_and(|n| !(1..=100).contains(&n))
    {
        return Err(invalid("invalid encounter clock, revision or difficulty"));
    }
    let mut participants = BTreeSet::new();
    let mut positions = BTreeSet::new();
    let mut initiative = BTreeSet::new();
    let mut conditions = BTreeSet::new();
    for p in &snapshot.participants {
        require_unique(&mut participants, p.participant_key.as_str(), "participant")?;
        require_unique(&mut positions, p.position, "roster position")?;
        require_unique(
            &mut initiative,
            (p.initiative, p.initiative_order),
            "initiative order",
        )?;
        if !(0..i64::MAX).contains(&p.revision)
            || !(1..i64::MAX).contains(&p.position)
            || !(1..i64::MAX).contains(&p.initiative_order)
            || p.base_death_threshold <= 0
            || p.temporary_hp < 0
            || (p.defeated && p.participant_kind == ParticipantKind::Hazard)
            || (p.hazard_disabled && p.participant_kind != ParticipantKind::Hazard)
            || p.turn_resources
                .as_ref()
                .is_some_and(|stock| stock.participant_key != p.participant_key)
        {
            return Err(invalid(format!(
                "invalid saved participant {}",
                p.participant_key
            )));
        }
        for condition in &p.conditions {
            require_unique(&mut conditions, condition.condition_id, "live condition")?;
        }
        unique_conditions(&p.conditions)?;
        let mut spell_resources = BTreeSet::new();
        for resource in &p.spell_resources {
            require_unique(&mut spell_resources, &resource.key, "spell resource")?;
            if !(0..=resource.capacity).contains(&resource.remaining)
                || resource
                    .undo_remaining
                    .is_some_and(|n| !(0..=resource.capacity).contains(&n))
            {
                return Err(invalid("invalid spell resource capacity"));
            }
        }
        let mut spells = BTreeSet::new();
        for spell in &p.expended_spells {
            require_unique(&mut spells, spell, "expended spell")?;
        }
        let mut trackers = BTreeSet::new();
        for tracker in &p.manual_trackers {
            require_unique(&mut trackers, &tracker.tracker_key, "tracker")?;
        }
        let mut abilities = BTreeSet::new();
        for ability in &p.ability_states {
            if ability
                .rule_ordinal
                .is_some_and(|ordinal| i64::try_from(ordinal).is_err())
            {
                return Err(invalid("invalid ability rule ordinal"));
            }
            require_unique(
                &mut abilities,
                (serde_json::to_string(&ability.owner)?, ability.rule_ordinal),
                "ability state",
            )?;
        }
        let mut episodes = BTreeSet::new();
        for episode in &p.persistent_precedence {
            require_unique(
                &mut episodes,
                &episode.episode_id,
                "persistent damage episode",
            )?;
        }
    }
    if root
        .current_turn_participant_key
        .as_deref()
        .is_some_and(|key| !participants.contains(key))
    {
        return Err(invalid("current turn points outside its roster"));
    }
    Ok(participants)
}

fn unique_conditions(conditions: &[EncounterParticipantCondition]) -> LocalStateResult<()> {
    let mut ids = BTreeSet::new();
    for condition in conditions {
        require_unique(&mut ids, condition.condition_id, "condition")?;
        if condition
            .duration
            .as_ref()
            .is_some_and(|d| d.amount < 0 || d.remaining_seconds.is_some_and(|n| n < 0))
        {
            return Err(invalid("negative condition duration"));
        }
    }
    Ok(())
}

fn require_unique<T: Ord>(seen: &mut BTreeSet<T>, key: T, label: &str) -> LocalStateResult<()> {
    if !seen.insert(key) {
        return Err(invalid(format!("duplicate {label}")));
    }
    Ok(())
}

impl Identities {
    pub(super) fn condition_floor(&self) -> i64 {
        self.conditions.last_key_value().map_or(0, |(&id, _)| id)
    }

    pub(super) fn check_budget(&self, remaining: usize) -> LocalStateResult<()> {
        if self.conditions.len()
            + self.participants.len()
            + self.boundaries.len()
            + self.turn_keys.len()
            > remaining
        {
            return Err(invalid(
                "encounter exceeds 100,000 entries including identity slots",
            ));
        }
        Ok(())
    }

    pub(super) fn allocate(
        &mut self,
        db: &Connection,
        key: String,
        slug: String,
    ) -> LocalStateResult<()> {
        self.encounter_identity = Some((key, slug));
        for (old, new) in &mut self.participants {
            *new = storage::new_participant_key();
            if *old == *new {
                return Err(invalid("participant key allocation collision"));
            }
        }
        for (old, new) in &mut self.boundaries {
            *new = format!("turn_{:016x}", rand::random::<u64>());
            self.requests
                .insert(format!("start:{old}"), format!("start:{new}"));
        }
        for new in self.turn_keys.values_mut() {
            *new = format!("turn:imported:{:016x}", rand::random::<u64>());
        }
        for (old, new) in &self.requests {
            self.derived.insert(old.clone(), new.clone());
            for suffix in [
                "hp-lifecycle",
                "confused-damage",
                "initiative-placement",
                "drained-hp",
            ] {
                self.derived
                    .insert(format!("{old}:{suffix}"), format!("{new}:{suffix}"));
            }
        }
        if !self.conditions.is_empty() {
            let floor: i64 = db.query_row(
                "SELECT MAX(COALESCE((SELECT MAX(id) FROM encounter_participant_conditions),0),
                  COALESCE((SELECT MAX(seq) FROM sqlite_sequence WHERE name='encounter_participant_conditions'),0))",
                [], |row| row.get(0))?;
            let count = i64::try_from(self.conditions.len())
                .map_err(|_| invalid("too many condition IDs"))?;
            let high = floor
                .checked_add(count)
                .filter(|n| *n < i64::MAX)
                .ok_or_else(|| invalid("target has exhausted condition IDs"))?;
            for (id, new) in ((floor + 1)..=high).zip(self.conditions.values_mut()) {
                *new = id;
            }
            db.execute("INSERT INTO sqlite_sequence(name,seq) SELECT 'encounter_participant_conditions',0
                WHERE NOT EXISTS (SELECT 1 FROM sqlite_sequence WHERE name='encounter_participant_conditions')", [])?;
            db.execute(
                "UPDATE sqlite_sequence SET seq=?1 WHERE name='encounter_participant_conditions'",
                [high],
            )?;
        }
        Ok(())
    }

    pub(super) fn visit(&mut self, document: &mut EncounterDocument) {
        self.snapshot(&mut document.snapshot);
        for row in &mut document.baselines {
            self.participant(&mut row.participant_key);
            for condition in &mut row.gameplay.conditions {
                self.condition(condition);
            }
            for tracker in &mut row.gameplay.manual_trackers {
                self.duration(&mut tracker.duration);
            }
            self.precedence(&mut row.gameplay.persistent_precedence);
        }
        for pending in &mut document.pending {
            self.pending(pending);
        }
        for row in &mut document.applications {
            self.participant(&mut row.participant_key);
            rewrite(&mut row.application_id, &self.derived);
            if let Some(id) = &mut row.condition_id {
                self.condition_id(id);
            }
        }
        for row in &mut document.claims {
            self.participant(&mut row.claim.participant_key);
            rewrite(&mut row.request_id, &self.requests);
        }
        for row in &mut document.receipts {
            rewrite(&mut row.request_id, &self.requests);
            match &mut row.outcome {
                ReceiptOutcome::Turn { boundary_key } => track(&mut self.boundaries, boundary_key),
                ReceiptOutcome::Event { receipt } => self.event(receipt),
            }
        }
        if let Some(row) = &mut document.outgoing {
            rewrite(&mut row.request_id, &self.requests);
            track(&mut self.boundaries, &mut row.turn.boundary_key);
            self.snapshot(&mut row.turn.snapshot);
            self.optional_participant(&mut row.turn.next_participant_key);
        }
    }

    fn snapshot(&mut self, snapshot: &mut EncounterWithParticipants) {
        if let Some((key, slug)) = &self.encounter_identity {
            snapshot.encounter.encounter_key.clone_from(key);
            snapshot.encounter.slug.clone_from(slug);
        }
        self.optional_participant(&mut snapshot.encounter.current_turn_participant_key);
        for p in &mut snapshot.participants {
            self.participant(&mut p.participant_key);
            for condition in &mut p.conditions {
                self.condition(condition);
            }
            for tracker in &mut p.manual_trackers {
                self.duration(&mut tracker.duration);
            }
            self.precedence(&mut p.persistent_precedence);
            if let Some(stock) = &mut p.turn_resources {
                self.participant(&mut stock.participant_key);
                track(&mut self.turn_keys, &mut stock.turn_key);
                for effect in &mut stock.effects {
                    if let Some(id) = &mut effect.condition_id {
                        self.condition_id(id);
                    }
                }
            }
        }
    }

    fn event(&mut self, receipt: &mut EncounterEventReceipt) {
        if let Some((key, _)) = &self.encounter_identity {
            receipt.encounter_key.clone_from(key);
        }
        rewrite(&mut receipt.request_id, &self.requests);
        for id in &mut receipt.added_condition_ids {
            self.condition_id(id);
        }
        for pending in &mut receipt.pending {
            self.pending(pending);
        }
        for outcome in &mut receipt.outcomes {
            match outcome {
                ConditionEventOutcome::LifecycleCheck {
                    participant_key, ..
                } => self.participant(participant_key),
                ConditionEventOutcome::PersistentDamage {
                    participant_key,
                    groups,
                    ..
                } => {
                    self.participant(participant_key);
                    for group in groups {
                        self.condition_id(&mut group.source_condition_id);
                    }
                }
                ConditionEventOutcome::PersistentRecovery { .. } => {}
            }
        }
    }

    fn pending(&mut self, pending: &mut SavedPendingConsequence) {
        rewrite(&mut pending.consequence_key, &self.derived);
        match &mut pending.consequence {
            PendingConsequence::ConfusedDamageRecovery {
                participant_key,
                condition_ids,
                ..
            } => {
                self.participant(participant_key);
                for id in condition_ids {
                    self.condition_id(id);
                }
            }
            PendingConsequence::BrokenHpThreshold { participant_key }
            | PendingConsequence::DrainedHpLoss {
                participant_key, ..
            } => self.participant(participant_key),
            PendingConsequence::InitiativePlacement {
                participant_key,
                context,
                ..
            }
            | PendingConsequence::HpLifecycle {
                participant_key,
                context,
                ..
            } => {
                self.participant(participant_key);
                self.optional_participant(&mut context.responsible_participant_key);
            }
        }
    }

    fn condition(&mut self, condition: &mut EncounterParticipantCondition) {
        self.condition_id(&mut condition.condition_id);
        self.optional_participant(&mut condition.source_participant_key);
        if let ConditionScope::SelectedParticipants { participant_keys } = &mut condition.scope {
            for key in participant_keys {
                self.participant(key);
            }
        }
        match &mut condition.parameters {
            ConditionParameters::Fleeing { subject }
            | ConditionParameters::Fascinated { subject } => {
                if let Some(ConditionSubject::Participant { participant_key }) = subject {
                    self.participant(participant_key);
                }
            }
            ConditionParameters::Attitude
            | ConditionParameters::Frightened { .. }
            | ConditionParameters::Broken { .. }
            | ConditionParameters::None
            | ConditionParameters::PersistentDamageReminder { .. }
            | ConditionParameters::PersistentDamage { .. }
            | ConditionParameters::Sickened { .. }
            | ConditionParameters::Unconscious { .. } => {}
        }
        if let Some(origin) = &mut condition.origin {
            self.optional_participant(&mut origin.participant_key);
            rewrite(&mut origin.application_id, &self.derived);
        }
        if let Some(duration) = &mut condition.duration {
            self.duration(duration);
        }
    }

    fn precedence(&mut self, values: &mut [PersistentSourcePrecedence]) {
        for value in values {
            for id in &mut value.source_order {
                self.condition_id(id);
            }
        }
    }

    fn duration(&mut self, duration: &mut ConditionDuration) {
        if let Some(timing) = &mut duration.timing {
            self.participant(&mut timing.participant_key);
        }
    }

    fn optional_participant(&mut self, key: &mut Option<String>) {
        if let Some(key) = key {
            self.participant(key);
        }
    }

    fn participant(&mut self, key: &mut String) {
        track(&mut self.participants, key);
    }

    fn condition_id(&mut self, id: &mut i64) {
        *id = *self.conditions.entry(*id).or_insert(*id);
    }
}

fn track(keys: &mut BTreeMap<String, String>, key: &mut String) {
    let mapped = keys.entry(key.clone()).or_insert_with(|| key.clone());
    key.clone_from(mapped);
}

fn rewrite(key: &mut String, keys: &BTreeMap<String, String>) {
    if let Some(mapped) = keys.get(key) {
        key.clone_from(mapped);
    }
}
