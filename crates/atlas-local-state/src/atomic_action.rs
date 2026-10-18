//! Savepoint scope for one domain action inside the caller's workspace transaction.
use rusqlite::Connection;

use crate::{LocalStateError, LocalStateResult};

// SQLite resolves duplicate savepoint names from the innermost scope, allowing
// composed actions to nest without competing with the caller's transaction.
const SAVEPOINT: &str = "SAVEPOINT atlas_local_state_action";
const RELEASE: &str = "RELEASE atlas_local_state_action";
const ROLLBACK: &str = "ROLLBACK TO atlas_local_state_action; RELEASE atlas_local_state_action";

pub(crate) fn run<T>(
    connection: &Connection,
    action: impl FnOnce() -> LocalStateResult<T>,
) -> LocalStateResult<T> {
    if connection.is_autocommit() {
        return Err(LocalStateError::ActionRequiresTransaction);
    }
    connection.execute_batch(SAVEPOINT)?;
    let mut scope = ActionScope {
        connection,
        released: false,
    };
    let result = action()?;
    connection.execute_batch(RELEASE)?;
    scope.released = true;
    Ok(result)
}

struct ActionScope<'a> {
    connection: &'a Connection,
    released: bool,
}

impl Drop for ActionScope<'_> {
    fn drop(&mut self) {
        if !self.released && self.connection.execute_batch(ROLLBACK).is_err() {
            // If SQLite cannot restore the savepoint, prevent committing a
            // possibly partial action. The caller's eventual COMMIT must fail.
            let _ = self.connection.execute_batch("ROLLBACK");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unwind_rolls_back_action_and_keeps_prior_outer_work() -> LocalStateResult<()> {
        let mut connection = Connection::open_in_memory()?;
        connection.execute_batch("CREATE TABLE facts(value INTEGER)")?;
        let tx = connection.transaction()?;
        tx.execute("INSERT INTO facts VALUES(1)", [])?;
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _: LocalStateResult<()> = run(&tx, || {
                tx.execute("INSERT INTO facts VALUES(2)", [])?;
                panic!("evaluation interrupted");
            });
        }));
        assert!(panic.is_err());
        tx.commit()?;
        assert_eq!(
            connection.query_row("SELECT sum(value) FROM facts", [], |r| r.get::<_, i64>(0))?,
            1
        );
        Ok(())
    }
}
