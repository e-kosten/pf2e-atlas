use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use atlas_app_model::AppErrorCode;

use crate::{AppServiceError, AppServiceResult};

/// Cooperative abandonment checkpoints for replaceable reads.
///
/// This does not interrupt running SQL or native inference. Durable mutations
/// use an explicit uncancellable context, independently of their HTTP caller.
#[derive(Debug, Clone)]
pub struct ReadExecutionContext {
    abandoned: Option<Arc<AtomicBool>>,
}

impl ReadExecutionContext {
    /// Creates a read context unaffected by caller abandonment.
    pub fn uncancellable() -> Self {
        Self { abandoned: None }
    }

    /// Creates a context whose clones observe the same abandonment signal.
    pub fn tracked() -> Self {
        Self {
            abandoned: Some(Arc::new(AtomicBool::new(false))),
        }
    }

    /// Marks tracked work abandoned; already-entered native work may finish.
    pub fn abandon(&self) {
        if let Some(abandoned) = &self.abandoned {
            abandoned.store(true, Ordering::Release);
        }
    }

    /// Checks whether a safe checkpoint should skip further read work.
    pub fn check(&self) -> AppServiceResult<()> {
        if self
            .abandoned
            .as_ref()
            .is_some_and(|abandoned| abandoned.load(Ordering::Acquire))
        {
            return Err(AppServiceError::new(
                AppErrorCode::OperationCancelled,
                "read caller no longer awaits this operation",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracked_clones_share_abandonment_while_uncancellable_stays_live() {
        let tracked = ReadExecutionContext::tracked();
        let clone = tracked.clone();
        assert!(clone.check().is_ok());
        tracked.abandon();
        assert_eq!(
            clone.check().unwrap_err().into_app_error().code,
            AppErrorCode::OperationCancelled
        );
        let uncancellable = ReadExecutionContext::uncancellable();
        uncancellable.abandon();
        assert!(uncancellable.check().is_ok());
    }
}
