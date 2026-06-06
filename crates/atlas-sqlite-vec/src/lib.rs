#![deny(unsafe_op_in_unsafe_fn)]

mod registration;

pub use registration::{SqliteVecRegistrationError, register_sqlite_vec_auto_extension};
