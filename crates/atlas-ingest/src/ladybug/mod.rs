pub(crate) mod embeddings;
pub(crate) mod evidence;
pub(crate) mod output;
pub(crate) mod parquet;
pub(crate) mod schema;
pub(crate) mod writer;

pub(crate) use writer::write_artifact;
