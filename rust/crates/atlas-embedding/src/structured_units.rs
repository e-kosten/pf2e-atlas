mod markup;
mod splitter;

pub(crate) use markup::strip_markup_for_embedding_units;
pub(crate) use splitter::{StructuredEmbeddingUnit, extract_structured_embedding_units};
