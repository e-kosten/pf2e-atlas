#![deny(unsafe_op_in_unsafe_fn)]

use std::os::raw::c_char;

use rusqlite::ffi::{SQLITE_OK, sqlite3, sqlite3_api_routines, sqlite3_auto_extension};
use sqlite_vec::sqlite3_vec_init;
use thiserror::Error;

type SqliteExtensionEntrypoint =
    unsafe extern "C" fn(*mut sqlite3, *mut *mut c_char, *const sqlite3_api_routines) -> i32;

#[derive(Debug, Error)]
pub enum SqliteVecRegistrationError {
    #[error("sqlite3_auto_extension(sqlite3_vec_init) failed with code {0}")]
    AutoExtensionFailed(i32),
}

pub fn register_sqlite_vec_auto_extension() -> Result<(), SqliteVecRegistrationError> {
    let entrypoint: SqliteExtensionEntrypoint = unsafe {
        std::mem::transmute::<*const (), SqliteExtensionEntrypoint>(sqlite3_vec_init as *const ())
    };
    let result = unsafe { sqlite3_auto_extension(Some(entrypoint)) };
    if result == SQLITE_OK {
        Ok(())
    } else {
        Err(SqliteVecRegistrationError::AutoExtensionFailed(result))
    }
}
