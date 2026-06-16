#![deny(unsafe_code)]

mod error;
mod model;
mod store;

pub const LOCAL_STATE_SCHEMA_VERSION: &str = "1";
pub const LOCAL_STATE_CONTRACT_VERSION: &str = "pf2e-atlas-local-state/v1";

pub use error::{LocalStateError, LocalStateResult};
pub use model::{
    AddSavedListItemOutcome, HydratedSavedListItem, NewSavedList, NewSavedListItem, SavedList,
    SavedListItem, SavedListItemSnapshot, SavedListItemStatus, SavedListWithItems,
    hydrate_saved_list_item,
};
pub use store::LocalStateStore;
