use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(crate) type Coordinates = BTreeMap<String, String>;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Axis {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) alternatives_label: Option<String>,
    pub(crate) options: Vec<AxisOption>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AxisOption {
    pub(crate) id: String,
    pub(crate) label: String,
}
