#[cfg(feature = "error_reporting")]
use peace::miette;

/// Error while managing tar extraction.
#[cfg_attr(feature = "error_reporting", derive(peace::miette::Diagnostic))]
#[derive(Debug, thiserror::Error)]
pub enum TarXError {
    // === Framework errors === //
    /// A `peace` runtime error occurred.
    #[error("A `peace` runtime error occurred.")]
    PeaceRtError(
        #[cfg_attr(feature = "error_reporting", diagnostic_source)]
        #[source]
        #[from]
        peace::rt_model::Error,
    ),
}