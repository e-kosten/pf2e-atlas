use super::{
    SerializedSourceMember, SerializedSourceObject, SerializedSourceValue, SourcePresence,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RollTableSource {
    pub(crate) root: SerializedSourceObject,
    pub(crate) results: SourcePresence<Vec<SerializedSourceValue>>,
}

pub(crate) fn parse_roll_table_source(
    root: &SerializedSourceObject,
) -> Result<RollTableSource, String> {
    let results = match root.member("results") {
        SerializedSourceMember::Missing => SourcePresence::Missing,
        SerializedSourceMember::Null => SourcePresence::Null,
        SerializedSourceMember::Value(SerializedSourceValue::Array(values)) => {
            SourcePresence::Value(values.clone())
        }
        SerializedSourceMember::Value(value) => {
            return Err(format!(
                "roll table $.results must be an array, found {:?}",
                value.shape()
            ));
        }
        SerializedSourceMember::Duplicate(values) => {
            return Err(format!(
                "roll table $.results is duplicated: [{}]",
                values
                    .iter()
                    .map(|value| value.compact_json())
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }
    };
    Ok(RollTableSource {
        root: root.clone(),
        results,
    })
}
