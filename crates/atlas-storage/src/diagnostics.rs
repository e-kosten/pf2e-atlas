//! Explicit observation of workspace connection opens on the calling thread.
use crate::{StorageError, StorageResult};
use std::cell::Cell;

thread_local! {
    static ACTIVE: Cell<Option<u64>> = const { Cell::new(None) };
}
struct Scope;
impl Drop for Scope {
    fn drop(&mut self) {
        ACTIVE.set(None);
    }
}

/// Count workspace connections opened in one explicit scope. Nested scopes are
/// rejected. Unwinding clears observation; the operation retains its own normal
/// transaction/error behavior. This does not initialize, lock or mutate storage.
pub fn count_workspace_connections<T>(operation: impl FnOnce() -> T) -> StorageResult<(T, u64)> {
    if ACTIVE.get().is_some() {
        return Err(StorageError::Unsupported(
            "Workspace connection diagnostic scopes cannot nest".into(),
        ));
    }
    ACTIVE.set(Some(0));
    let _scope = Scope;
    let result = operation();
    let connections = ACTIVE.replace(None).ok_or_else(|| {
        StorageError::Unsupported("Workspace connection diagnostic scope disappeared".into())
    })?;
    Ok((result, connections))
}

pub(crate) fn record_workspace_connection() {
    if let Some(connections) = ACTIVE.get() {
        ACTIVE.set(Some(connections.saturating_add(1)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_counts_are_scoped_and_nested_scopes_are_rejected() {
        record_workspace_connection();
        let (nested, connections) = count_workspace_connections(|| {
            record_workspace_connection();
            count_workspace_connections(|| ())
        })
        .unwrap();
        assert!(nested.is_err());
        assert_eq!(connections, 1);
        assert_eq!(count_workspace_connections(|| ()).unwrap().1, 0);
    }

    #[test]
    fn unwind_does_not_leave_a_diagnostic_scope_active() {
        let result = std::panic::catch_unwind(|| {
            count_workspace_connections(|| panic!("diagnostic operation failed"))
        });
        assert!(result.is_err());
        assert!(count_workspace_connections(|| ()).is_ok());
    }
}
