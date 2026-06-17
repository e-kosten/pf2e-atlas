#![deny(unsafe_code)]

mod error;
mod saved_lists;
mod schema;
mod store;

pub const LOCAL_STATE_SCHEMA_VERSION: &str = "2";
pub const LOCAL_STATE_CONTRACT_VERSION: &str = "pf2e-atlas-local-state/v1";

pub use error::{LocalStateError, LocalStateResult};
pub use saved_lists::{
    AddSavedListItemOutcome, HydratedSavedListItem, NewSavedList, ResolvedSavedListItem, SavedList,
    SavedListItem, SavedListItemSnapshot, SavedListItemStatus, SavedListWithItems, SavedLists,
    UpdateSavedList, hydrate_saved_list_item,
};
pub use store::LocalStateStore;
