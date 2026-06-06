pub(crate) mod discovery;
pub(crate) mod graph;
pub(crate) mod records;
pub(crate) mod search;
pub(crate) mod sql;
pub(crate) mod validation;

/// Composite read capability for product retrieval orchestration.
///
/// Focused read traits own individual index capabilities. This bundle is the
/// index-owned contract for consumers that legitimately need the full retrieval
/// read surface.
pub trait RetrievalReadIndex:
    search::RecordReadIndex
    + search::IdentityReadIndex
    + search::FilterReadIndex
    + search::FtsReadIndex
    + search::VectorReadIndex
    + graph::product::ReferenceReadIndex
    + graph::product::VariantReadIndex
    + graph::product::RemasterReadIndex
    + discovery::DiscoveryReadIndex
{
}

impl<T> RetrievalReadIndex for T where
    T: search::RecordReadIndex
        + search::IdentityReadIndex
        + search::FilterReadIndex
        + search::FtsReadIndex
        + search::VectorReadIndex
        + graph::product::ReferenceReadIndex
        + graph::product::VariantReadIndex
        + graph::product::RemasterReadIndex
        + discovery::DiscoveryReadIndex
{
}
