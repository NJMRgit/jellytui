use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by [`JellyfinClient`](crate::JellyfinClient).
///
/// Match on the variant rather than the [`Display`](std::fmt::Display) string —
/// the wording may change between releases.
#[derive(Debug, Error)]
pub enum Error {
    /// The operation requires a logged-in session, but the client has no
    /// stored token or user id. Call
    /// [`authenticate`](crate::JellyfinClient::authenticate) first.
    #[error("not authenticated")]
    Unauthenticated,

    /// The server rejected the request with HTTP 401. The stored token is
    /// likely invalid or expired; re-authenticate.
    #[error("unauthorized: 401")]
    Unauthorized,

    /// The server returned a non-success, non-401 HTTP status.
    #[error("{operation} failed: HTTP {status} - {body}")]
    Server {
        operation: &'static str,
        status: u16,
        body: String,
    },

    /// A transport-layer or response-parsing error from the underlying
    /// HTTP client.
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}
