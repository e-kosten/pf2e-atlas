//! Admission and delivery for jobs started by a bundled setup document.

use std::collections::HashMap;

use serde::Serialize;
use serde_json::Value;
use tauri::{Emitter, EventTarget, Manager, WebviewWindow};

use crate::{Desktop, bundled_origin, lock};

#[derive(Default)]
pub struct BootstrapJobs {
    generation: u64,
    documents: HashMap<String, Document>,
}

struct Document {
    generation: u64,
    active: Option<u64>,
    receipts: HashMap<u64, Action>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Launch,
    Prepare { local: bool },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Attempt {
    pub generation: u64,
    pub id: u64,
}

#[derive(Clone)]
pub struct BootstrapJob {
    pub window: WebviewWindow,
    pub generation: u64,
    pub id: u64,
}

#[derive(Serialize, Clone)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Update {
    Progress(Value),
    Navigate(String),
    Completed(Option<String>),
    Failed(Value),
    Retry,
}

#[derive(Serialize, Clone)]
struct Message {
    generation: u64,
    operation: u64,
    update: Update,
}

impl BootstrapJobs {
    pub fn commit(&mut self, label: &str, bundled: bool) {
        self.documents.remove(label);
        if bundled {
            self.generation += 1;
            self.documents.insert(
                label.to_owned(),
                Document {
                    generation: self.generation,
                    active: None,
                    receipts: HashMap::new(),
                },
            );
        }
    }

    pub fn generation(&self, label: &str) -> Option<u64> {
        self.documents.get(label).map(|doc| doc.generation)
    }

    pub fn admit(
        &mut self,
        label: &str,
        generation: u64,
        id: u64,
        action: Action,
    ) -> Result<bool, String> {
        let doc = self
            .documents
            .get_mut(label)
            .filter(|doc| doc.generation == generation)
            .ok_or("This startup request belongs to an earlier Atlas page.")?;
        if let Some(previous) = doc.receipts.get(&id) {
            return if *previous == action {
                Ok(false)
            } else {
                Err("This startup request ID already has a different action.".into())
            };
        }
        if doc.active.is_some() {
            return Err("Atlas is already working.".into());
        }
        doc.receipts.insert(id, action);
        doc.active = Some(id);
        Ok(true)
    }

    pub fn owns_attempt(&self, label: &str, attempt: Attempt) -> bool {
        self.documents.get(label).is_some_and(|doc| {
            doc.generation == attempt.generation && doc.active == Some(attempt.id)
        })
    }

    pub fn owns(&self, job: &BootstrapJob) -> bool {
        self.owns_attempt(job.window.label(), job.attempt())
    }

    fn complete(&mut self, label: &str, attempt: Attempt) -> bool {
        let Some(doc) = self
            .documents
            .get_mut(label)
            .filter(|doc| doc.generation == attempt.generation && doc.active == Some(attempt.id))
        else {
            return false;
        };
        doc.active = None;
        true
    }

    pub fn owns_prepare(&self, job: &BootstrapJob) -> bool {
        self.owns(job)
            && self.documents[job.window.label()]
                .receipts
                .get(&job.id)
                .is_some_and(|action| matches!(action, Action::Prepare { .. }))
    }
}

impl BootstrapJob {
    pub fn attempt(&self) -> Attempt {
        Attempt {
            generation: self.generation,
            id: self.id,
        }
    }

    pub fn update(&self, update: Update) {
        if self.is_bundled() && lock(&self.window.state::<Desktop>().bootstrap_jobs).owns(self) {
            self.emit(update);
        }
    }

    pub fn retire(&self) -> bool {
        lock(&self.window.state::<Desktop>().bootstrap_jobs)
            .complete(self.window.label(), self.attempt())
    }

    pub fn finish(&self, update: Update) {
        if self.retire() && self.is_bundled() {
            self.emit(update);
        }
    }

    pub fn is_bundled(&self) -> bool {
        self.window.url().is_ok_and(|url| bundled_origin(&url))
    }

    fn emit(&self, update: Update) {
        let _ = self.window.emit_to(
            EventTarget::webview_window(self.window.label()),
            "atlas-bootstrap-job",
            Message {
                generation: self.generation,
                operation: self.id,
                update,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admission_is_idempotent_even_after_the_active_slot_is_released() {
        let mut jobs = BootstrapJobs::default();
        let label = "atlas-primary";
        jobs.commit(label, true);
        let generation = jobs.generation(label).unwrap();
        let attempt = Attempt { generation, id: 1 };
        assert_eq!(jobs.admit(label, generation, 1, Action::Launch), Ok(true));
        assert_eq!(jobs.admit(label, generation, 1, Action::Launch), Ok(false));
        assert!(jobs.admit(label, generation, 2, Action::Launch).is_err());
        assert!(jobs.complete(label, attempt));
        assert_eq!(jobs.admit(label, generation, 1, Action::Launch), Ok(false));
        assert!(
            jobs.admit(label, generation, 1, Action::Prepare { local: false })
                .is_err()
        );
        assert_eq!(jobs.admit(label, generation, 2, Action::Launch), Ok(true));
        assert!(!jobs.complete(label, attempt));
    }

    #[test]
    fn a_new_document_invalidates_previous_admission_and_completion() {
        let mut jobs = BootstrapJobs::default();
        let label = "atlas-primary";
        jobs.commit(label, true);
        let generation = jobs.generation(label).unwrap();
        let attempt = Attempt { generation, id: 1 };
        assert_eq!(jobs.admit(label, generation, 1, Action::Launch), Ok(true));
        jobs.commit(label, true);
        assert!(jobs.admit(label, generation, 1, Action::Launch).is_err());
        assert!(!jobs.complete(label, attempt));
        let current = jobs.generation(label).unwrap();
        assert_eq!(jobs.admit(label, current, 1, Action::Launch), Ok(true));
    }
}
