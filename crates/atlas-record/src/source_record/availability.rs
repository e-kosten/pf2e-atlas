use atlas_foundry_model::{SourceFieldRejection, SourcePresence};
use serde::{Deserialize, Serialize};

/// A selected field view. Invalid ancestors propagate without copying raw data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFieldView<'a, T> {
    Value(T),
    Missing,
    Null,
    Invalid(&'a SourceFieldRejection),
    /// The source was admitted, but cannot supply this narrower query meaning.
    /// This does not change the source field or manufacture an admission error.
    ProjectionInvalid {
        source_path: &'static str,
        reason: &'static str,
    },
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldAvailability {
    Value,
    Missing,
    Null,
    /// Points into the source DTO/admission diagnostic, without duplicating it.
    Invalid {
        source_path: String,
    },
    NotApplicable,
}

impl<'a, T> From<&'a SourcePresence<T>> for SourceFieldView<'a, &'a T> {
    fn from(value: &'a SourcePresence<T>) -> Self {
        match value {
            SourcePresence::Value(value) => Self::Value(value),
            SourcePresence::Missing => Self::Missing,
            SourcePresence::Null => Self::Null,
            SourcePresence::Invalid(error) => Self::Invalid(error),
        }
    }
}

impl<'a, T> SourceFieldView<'a, T> {
    pub fn and_then<U>(
        self,
        next: impl FnOnce(T) -> SourceFieldView<'a, U>,
    ) -> SourceFieldView<'a, U> {
        match self {
            Self::Value(value) => next(value),
            Self::Missing => SourceFieldView::Missing,
            Self::Null => SourceFieldView::Null,
            Self::Invalid(error) => SourceFieldView::Invalid(error),
            Self::ProjectionInvalid {
                source_path,
                reason,
            } => SourceFieldView::ProjectionInvalid {
                source_path,
                reason,
            },
            Self::NotApplicable => SourceFieldView::NotApplicable,
        }
    }

    pub fn map<U>(self, next: impl FnOnce(T) -> U) -> SourceFieldView<'a, U> {
        self.and_then(|value| SourceFieldView::Value(next(value)))
    }

    pub fn value(self) -> Option<T> {
        if let Self::Value(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn availability(&self) -> FieldAvailability {
        match self {
            Self::Value(_) => FieldAvailability::Value,
            Self::Missing => FieldAvailability::Missing,
            Self::Null => FieldAvailability::Null,
            Self::Invalid(error) => FieldAvailability::Invalid {
                source_path: error.json_path.clone(),
            },
            Self::ProjectionInvalid { source_path, .. } => FieldAvailability::Invalid {
                source_path: (*source_path).to_owned(),
            },
            Self::NotApplicable => FieldAvailability::NotApplicable,
        }
    }
}
