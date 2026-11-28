//! Input directions shared by the platform history commands.

#[derive(Clone, Copy, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Back,
    Forward,
}
