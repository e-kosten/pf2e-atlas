use super::{
    SerializedSourceMember, SerializedSourceObject, SerializedSourceValue, SourcePresence,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct JournalSource {
    pub(crate) root: SerializedSourceObject,
    pub(crate) pages: SourcePresence<Vec<SerializedSourceValue>>,
}

pub(crate) fn parse_journal_source(root: &SerializedSourceObject) -> Result<JournalSource, String> {
    let pages = match root.member("pages") {
        SerializedSourceMember::Missing => SourcePresence::Missing,
        SerializedSourceMember::Null => SourcePresence::Null,
        SerializedSourceMember::Value(SerializedSourceValue::Array(values)) => {
            SourcePresence::Value(values.clone())
        }
        SerializedSourceMember::Value(value) => {
            return Err(format!(
                "journal $.pages must be an array, found {:?}",
                value.shape()
            ));
        }
        SerializedSourceMember::Duplicate(values) => {
            return Err(format!(
                "journal $.pages is duplicated: [{}]",
                values
                    .iter()
                    .map(|value| value.compact_json())
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }
    };
    Ok(JournalSource {
        root: root.clone(),
        pages,
    })
}
